FROM rust:bookworm as builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/Daily-Newspaper /usr/local/bin/daily-newspaper
COPY feeds.json .
CMD ["daily-newspaper"]
