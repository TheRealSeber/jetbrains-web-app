use axum::{
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use serde::Serialize;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum CreatePostError {
    #[error("Invalid file type. Supported types is only PNG")]
    InvalidFileType,

    #[error("File too large. Maximum size is 5MB")]
    FileTooLarge,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to download avatar: {0}")]
    AvatarDownloadError(String),

    #[error("Invalid input: {0}")]
    ValidationError(String),

    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    #[error("Internal server error")]
    InternalError,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status_code: u16,
    message: String,
}

impl CreatePostError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidFileType
            | Self::FileTooLarge
            | Self::ValidationError(_)
            | Self::AvatarDownloadError(_) => StatusCode::BAD_REQUEST,
            Self::DatabaseError(_)
            | Self::IoError(_)
            | Self::ImageError(_)
            | Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for CreatePostError {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse {
            status_code: self.status_code().as_u16(),
            message: self.to_string(),
        };
        tracing::error!("{:?}", error_response);
        (self.status_code(), Json(error_response)).into_response()
    }
}
