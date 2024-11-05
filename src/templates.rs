use askama_axum::Template;

use crate::domain::BlogPost;

#[derive(Template)]
#[template(path = "home.html")]
pub struct HomeTemplate {
    pub posts: Vec<BlogPost>,
    pub upload_path: String,
}
