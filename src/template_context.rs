use askama::Template;
use crate::models::Article;

pub struct Section {
    pub name: String,
    pub articles: Vec<Article>,
}

#[derive(Template)]
#[template(path = "newspaper.html")]
pub struct NewspaperTemplate {
    pub sections: Vec<Section>,
    pub date: String,
}
