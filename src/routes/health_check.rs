#[tracing::instrument]
pub async fn handle_get() -> axum::response::Response {
    return axum::response::IntoResponse::into_response((
        axum::http::StatusCode::OK,
        "Hello World!",
    ));
}
