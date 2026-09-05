# Publish `/newspaper/today/index.html` to Firebase Hosting (Rust service)

This document specifies how a **Rust** scheduled task (running in Google Cloud) should
publish a single generated `index.html` to Firebase Hosting so it is served at
`/newspaper/today/`. The file is intentionally **not** managed by the Zola site's CI/CD
(see "Coordination with the site repo" below).

## Fixed values

| Item | Value |
| --- | --- |
| GCP project | `valiant-azimuth-296116` |
| Firebase Hosting site ID | `valiant-azimuth-296116` (default site = project ID) |
| Target path | `/newspaper/today/index.html` |
| Served URL | `https://www.darrenehale.com/newspaper/today/` (also `https://valiant-azimuth-296116.web.app/newspaper/today/`) |
| API base | `https://firebasehosting.googleapis.com/v1beta1` |
| Upload base | `https://upload-firebasehosting.googleapis.com` |
| OAuth scope | `https://www.googleapis.com/auth/cloud-platform` (or `.../auth/firebase.hosting`) |

## IAM prerequisite

The task's runtime service account must be granted Hosting Admin on the project. Run once
(owner/admin needed):

```bash
gcloud projects add-iam-policy-binding valiant-azimuth-296116 \
  --member="serviceAccount:<TASK_SA_EMAIL>" \
  --role="roles/firebasehosting.admin"
```

The service authenticates with **Application Default Credentials** (its own service
account on the Google Cloud runtime). No keys to store; do not add a `x-goog-user-project`
header (that is only needed for user OAuth credentials, not service accounts).

## Current deployment

- **Cloud Run Job:** `daily-newspaper-job` (region `us-central1`), built and deployed by
  `.github/workflows/deploy.yml` on every push to `main` (the workflow also fires one
  immediate execution after each deploy).
- **Schedule:** Cloud Scheduler job `daily-newspaper-schedule` (region `us-central1`),
  cron `0 5 * * *`, timezone `America/Chicago`, HTTP target
  `https://us-central1-run.googleapis.com/apis/run.googleapis.com/v1/namespaces/valiant-azimuth-296116/jobs/daily-newspaper-job:run`.
- **Env vars on the job:** `GCS_BUCKET=valiant-azimuth-296116-newspaper` and
  `FIREBASE_SITE_ID=valiant-azimuth-296116`. The Firebase publish runs whenever
  `FIREBASE_SITE_ID` is set.
- **Runtime service account:**
  `deploymentactionaccount@valiant-azimuth-296116.iam.gserviceaccount.com`, granted
  `roles/firebasehosting.admin`, `roles/iam.serviceAccountUser`, and
  `roles/storage.objectAdmin` on the `valiant-azimuth-296116-newspaper` bucket.
- **GCS is the durable history store:** `history.json` is downloaded from and re-uploaded
  to `gs://valiant-azimuth-296116-newspaper/newspaper/history.json` on every run, so the
  headline-dedup log survives between executions (see `gcs.rs`). If the download fails
  (e.g. object missing on first run), the app falls back to the local `history.json` and
  starts with an empty log. The `index.html` GCS upload is best-effort; a failure is
  logged and the run continues to the Firebase publish.

## Deploy algorithm (single-file "replace one file")

All requests send `Authorization: Bearer <token>` and `Content-Type: application/json`
unless noted. A request that fails with a transient status (429, 5xx) should be retried
with exponential backoff.

### 1. Get the current live version

```
GET /v1beta1/sites/valiant-azimuth-296116/channels/live/releases?pageSize=1
```

From `releases[0].version.name` (e.g. `sites/valiant-azimuth-296116/versions/<SRC_ID>`)
take the version ID as `<SRC_ID>`.

### 2. Clone the live version, excluding the file being replaced

```
POST /v1beta1/sites/valiant-azimuth-296116/versions:clone
```

```json
{
  "sourceVersion": "sites/valiant-azimuth-296116/versions/<SRC_ID>",
  "exclude": {
    "regexes": ["/newspaper/today/index.html"]
  },
  "finalize": false
}
```

- `exclude` is a **path filter** object whose `regexes` array lists the paths to
  exclude from the clone — not a bare array.
- The response is a long-running **Operation**, not the version itself. Poll it
  until it finishes:
  - `GET /v1beta1/projects/<N>/operations/<BASE64>` (the operation's full `name`)
    until `"done": true`.
  - If `error` is present, the clone failed.
  - On success, `response.name` is the **full version resource name**
    `sites/valiant-azimuth-296116/versions/<NEW_ID>`. Steps 4-7 use that full name
    as the `<VERSION>`.

The new version inherits every other file from the live site.

### 3. Compress and hash the generated file

- Gzip the `index.html` bytes (no mtime, deterministic — e.g. `flate2` with mtime 0).
- Compute the lowercase hex SHA-256 of the **gzipped** bytes.

### 4. Populate files

```
POST /v1beta1/sites/valiant-azimuth-296116/versions/<NEW_ID>:populateFiles
```

(`<NEW_ID>` is the trailing ID of the full `<VERSION>` name returned in step 2.)

```json
{
  "files": {
    "/newspaper/today/index.html": "<sha256-hex of gzipped bytes>"
  }
}
```

Response:

```json
{
  "uploadRequiredHashes": ["..."],
  "uploadUrl": "https://upload-firebasehosting.googleapis.com/upload/sites/valiant-azimuth-296116/versions/<NEW_ID>/files"
}
```

If `uploadRequiredHashes` is empty (hash already stored), skip step 5.

### 5. Upload the bytes

For each hash in `uploadRequiredHashes`:

```
PUT {uploadUrl}/{hash}
```

- `Content-Type: application/octet-stream`
- Body: the **gzipped** bytes (must match the hashed bytes exactly)
- Expect `200 OK`.

### 6. Finalize the version

```
PATCH /v1beta1/sites/valiant-azimuth-296116/versions/<NEW_ID>?update_mask=status
```

```json
{ "status": "FINALIZED" }
```

### 7. Release it (go live)

```
POST /v1beta1/sites/valiant-azimuth-296116/releases?versionName=sites/valiant-azimuth-296116/versions/<NEW_ID>
```

Empty body, but the request must carry an explicit `Content-Length: 0` (a bare
body-less POST returns `411 Length Required`). Expect `200 OK` with a `type: "DEPLOY"`
release.

## Rust implementation notes

- HTTP: `reqwest` (rustls). JSON: `serde` / `serde_json`.
- Auth: use the `google-cloud-auth` crate (Application Default Credentials) to obtain a
  bearer token for the scope above; refresh before expiry (tokens live ~1 hour).
- Gzip: `flate2` with a `GzBuilder` mtime of 0 so identical input yields an identical
  hash across runs.
- Hashing: `sha2` (`Sha256`).
- Structure: one function `publish(html: &[u8], site_id: &str) -> Result<()>`
  implementing steps 1-7 (in `src/firebase.rs`), called from `main.rs` when the
  `FIREBASE_SITE_ID` env var is set. The daily schedule is handled by Cloud Scheduler
  (see "Current deployment").

## Verification

After a successful run, check the live URL responds with your HTML:

```bash
curl -sI https://valiant-azimuth-296116.web.app/newspaper/today/
curl -s  https://valiant-azimuth-296116.web.app/newspaper/today/ | head
```

For manual debugging you can reproduce the flow with `curl` using a temporary token:

```bash
TOKEN=$(gcloud auth print-access-token)
curl -s -H "Authorization: Bearer $TOKEN" -H "x-goog-user-project: valiant-azimuth-296116" \
  "https://firebasehosting.googleapis.com/v1beta1/sites/valiant-azimuth-296116/channels/live/releases?pageSize=1"
```

(Note the `x-goog-user-project` header is only needed when testing with user credentials.)

## Coordination with the site repo

The site repo (Zola + CI/CD) has declared that it does **not** manage this route:

- `static/newspaper/today/` was removed from the repo.
- `firebase.json` contains `"**/newspaper/today/**"` in `hosting.ignore`, so normal CI
  deploys neither upload nor delete anything under `/newspaper/today/`.

This service owns `/newspaper/today/index.html` exclusively. A full CI deploy will not
overwrite it; a missing file after deploys means only this task can have written it.
