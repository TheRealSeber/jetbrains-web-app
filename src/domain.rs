use chrono::{DateTime, Utc};
use image::ImageFormat;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub const MAX_TEXT_LENGTH: u64 = 10000;
pub const MIN_TEXT_LENGTH: u64 = 10;
pub const ALLOWED_IMAGE_TYPE: ImageFormat = ImageFormat::Png;

pub static USERNAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-zA-Z0-9_-]{2,50}$").unwrap());

#[derive(Debug, Serialize, Deserialize)]
pub struct BlogPost {
    pub id: Uuid,
    pub text: String,
    pub published_at: DateTime<Utc>,
    pub image_path: Option<String>,
    pub username: String,
    pub user_avatar_path: Option<String>,
}

#[tracing::instrument(name = "Saving post to database", skip(tx))]
pub async fn save_post(
    tx: &mut Transaction<'_, Postgres>,
    text: &str,
    username: &str,
    image_path: Option<&str>,
    avatar_path: Option<&str>,
) -> Result<(), sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO blog_posts (
            id,
            text,
            username,
            image_path,
            user_avatar_path
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
        id,
        text,
        username,
        image_path,
        avatar_path,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[tracing::instrument(name = "Getting all posts from database", skip(pool))]
pub async fn get_all_posts(pool: &sqlx::PgPool) -> Result<Vec<BlogPost>, sqlx::Error> {
    let posts = sqlx::query_as!(
        BlogPost,
        r#"
        SELECT 
            id,
            text,
            published_at,
            image_path,
            username,
            user_avatar_path
        FROM blog_posts
        ORDER BY published_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(posts)
}
