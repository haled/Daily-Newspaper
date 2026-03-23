use askama::Template;
use crate::models::Article;

#[derive(Template)]
#[template(path = "newspaper.html")]
pub struct NewspaperTemplate {
    pub articles: Vec<Article>,
    pub date: String,
}
