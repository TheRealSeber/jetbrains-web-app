use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    domain::{
        save_post, ALLOWED_IMAGE_TYPE, MAX_FILE_SIZE, MAX_TEXT_LENGTH, MIN_TEXT_LENGTH, USERNAME_RE,
    },
    startup::AppState,
};
use axum::{
    extract::{Multipart, State},
    response::{IntoResponse, Redirect},
};
use hyper::header;
use reqwest::Client;
use tracing::warn;
use uuid::Uuid;
use validator::Validate;

use super::errors::CreatePostError;

#[derive(Debug, Validate)]
struct NewPostData {
    #[validate(length(
        min = "MIN_TEXT_LENGTH",
        max = "MAX_TEXT_LENGTH",
        message = "Text must be between 10 and 10,000 characters"
    ))]
    text: String,

    #[validate(length(
        min = 2,
        max = 50,
        message = "Username must be between 2 and 50 characters"
    ))]
    #[validate(regex(path = *USERNAME_RE, message = "Username contains invalid characters"))]
    username: String,

    #[validate(url)]
    #[validate(custom(function = "Self::validate_image_url"))]
    user_avatar_url: Option<String>,
    image_data: Option<Vec<u8>>,
}

impl NewPostData {
    fn validate_image_url(url: &str) -> Result<(), validator::ValidationError> {
        if !url.ends_with(".png") {
            return Err(validator::ValidationError::new(
                "Avatar URL must point to a PNG image",
            ));
        }
        Ok(())
    }
}

struct CleanupGuard {
    paths: Vec<PathBuf>,
}

impl CleanupGuard {
    fn new() -> Self {
        Self { paths: Vec::new() }
    }

    fn add(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    fn dismiss(mut self) {
        self.paths.clear();
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        for path in self.paths.iter() {
            if let Err(e) = std::fs::remove_file(path) {
                warn!("Failed to clean up file {}: {}", path.display(), e);
            }
        }
    }
}

#[tracing::instrument(name = "Creating a new post", skip(state, multipart))]
pub async fn create_post(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, CreatePostError> {
    let mut cleanup_guard = CleanupGuard::new();
    let (text, username, user_avatar_url, image_data) =
        process_multipart_fields(&mut multipart).await?;

    let post_data = NewPostData {
        text: text
            .ok_or_else(|| CreatePostError::ValidationError("Text is required".to_string()))?,
        username: username
            .ok_or_else(|| CreatePostError::ValidationError("Username is required".to_string()))?,
        user_avatar_url,
        image_data,
    };

    post_data
        .validate()
        .map_err(|e| CreatePostError::ValidationError(e.to_string()))?;

    let mut tx = state
        .connection_pool
        .begin()
        .await
        .map_err(|e| CreatePostError::DatabaseError(e))?;

    let image_path = if let Some(image_data) = post_data.image_data {
        let file_name = format!("{}.png", Uuid::new_v4());
        let file_path = state.upload_path.join(&file_name);

        save_image(&image_data, &file_path).await?;
        cleanup_guard.add(file_path);

        Some(file_name)
    } else {
        None
    };

    let avatar_path = if let Some(url) = post_data.user_avatar_url {
        let file_name = format!("avatar_{}.png", Uuid::new_v4());
        let file_path = state.upload_path.join(&file_name);

        download_and_save_avatar(&state.http_client, &url, &file_path).await?;
        cleanup_guard.add(file_path.clone());

        Some(file_name)
    } else {
        None
    };

    save_post(
        &mut tx,
        &post_data.text,
        &post_data.username,
        image_path.as_deref(),
        avatar_path.as_deref(),
    )
    .await
    .map_err(|e| CreatePostError::DatabaseError(e))?;

    tx.commit()
        .await
        .map_err(|e| CreatePostError::DatabaseError(e))?;

    cleanup_guard.dismiss();

    Ok(Redirect::to("/home"))
}

#[tracing::instrument(name = "Saving image to disk", skip(data))]
async fn save_image(data: &[u8], path: &Path) -> Result<(), CreatePostError> {
    let img = image::load_from_memory(data)?;

    let mut buffer = BufWriter::new(File::create(path).map_err(|e| CreatePostError::IoError(e))?);
    img.write_to(&mut buffer, image::ImageFormat::Png)?;

    Ok(())
}

#[tracing::instrument(name = "Downloading and saving avatar")]
async fn download_and_save_avatar(
    client: &Client,
    url: &str,
    path: &Path,
) -> Result<(), CreatePostError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| CreatePostError::AvatarDownloadError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(CreatePostError::AvatarDownloadError(format!(
            "Failed to download avatar: HTTP {}",
            response.status()
        )));
    }

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    tracing::trace!("Avatar content type: {}", content_type);

    if ALLOWED_IMAGE_TYPE != content_type {
        return Err(CreatePostError::AvatarDownloadError(
            "Invalid avatar image type".to_string(),
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| CreatePostError::AvatarDownloadError(e.to_string()))?;

    if bytes.len() > MAX_FILE_SIZE {
        return Err(CreatePostError::FileTooLarge);
    }

    save_image(&bytes, path).await?;
    Ok(())
}

async fn process_multipart_fields(
    multipart: &mut Multipart,
) -> Result<
    (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<Vec<u8>>,
    ),
    CreatePostError,
> {
    let mut text = None;
    let mut username = None;
    let mut user_avatar_url = None;
    let mut image_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| CreatePostError::InternalError)?
    {
        let name = field
            .name()
            .ok_or_else(|| CreatePostError::ValidationError("Missing field name".to_string()))?
            .to_string();

        match name.as_str() {
            "text" => {
                text = Some(field.text().await.map_err(|e| {
                    CreatePostError::ValidationError(format!("Invalid text field: {}", e))
                })?);
                if text.clone().unwrap().is_empty() {
                    return Err(CreatePostError::ValidationError(
                        "Text is required".to_string(),
                    ));
                }
            }
            "username" => {
                username = Some(field.text().await.map_err(|e| {
                    CreatePostError::ValidationError(format!("Invalid username field: {}", e))
                })?);
                if username.clone().unwrap().is_empty() {
                    return Err(CreatePostError::ValidationError(
                        "Username is required".to_string(),
                    ));
                }
            }
            "user_avatar_url" => {
                let url = field.text().await.map_err(|e| {
                    CreatePostError::ValidationError(format!(
                        "Invalid user_avatar_url field: {}",
                        e
                    ))
                })?;
                if !url.is_empty() {
                    user_avatar_url = Some(url);
                }
            }
            "image" => {
                if field.file_name().is_none() || field.file_name().unwrap().is_empty() {
                    continue;
                }
                if let Some(content_type) = field.content_type() {
                    if ALLOWED_IMAGE_TYPE != content_type {
                        return Err(CreatePostError::InvalidFileType);
                    }
                }

                tracing::info!("Field {:?}", field);

                let data = field.bytes().await.map_err(|e| {
                    CreatePostError::ValidationError(format!("Failed to read image data: {}", e))
                })?;

                if data.len() > MAX_FILE_SIZE {
                    return Err(CreatePostError::FileTooLarge);
                }

                if !data.is_empty() {
                    image_data = Some(data.to_vec());
                }
            }
            _ => {
                warn!("Unknown field received: {}", name);
            }
        }
    }

    Ok((text, username, user_avatar_url, image_data))
}
