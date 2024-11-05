use std::sync::Arc;

use askama_axum::{IntoResponse, Response};
use axum::extract::State;
use hyper::StatusCode;

use crate::{domain::get_all_posts, startup::AppState, templates::HomeTemplate};

#[tracing::instrument(skip(state))]
pub async fn home(State(state): State<Arc<AppState>>) -> Result<Response, StatusCode> {
    let posts = get_all_posts(&state.connection_pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let template = HomeTemplate {
        posts,
        upload_path: state.upload_path.to_str().unwrap().to_string(),
    };

    Ok(template.into_response())
}
