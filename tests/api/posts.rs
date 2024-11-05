use crate::helpers::{get_image_asset, spawn_app};
use jetbrains_web_app::domain::BlogPost;
use reqwest::multipart;

const JETBRAINS_PNG_LOGO_URL: &str = "https://w7.pngwing.com/pngs/101/125/png-transparent-intellij-idea-integrated-development-environment-computer-software-source-code-jetbrains-php-logo-angle-text-logo-thumbnail.png";
const JETBRAINS_JPG_LOGO_URL: &str =
    "https://upload.wikimedia.org/wikipedia/commons/e/e6/JetBrains_logo.jpg";

#[tokio::test]
async fn create_post_success_redirects_home_and_exists_in_database() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let image_name = "jetbrains-logo.png";

    let image = get_image_asset(&image_name);

    let form = multipart::Form::new()
        .text("text", "This is a sample post text.")
        .text("username", "valid_user")
        .text("user_avatar_url", JETBRAINS_PNG_LOGO_URL)
        .part("image", multipart::Part::bytes(image).file_name(image_name));

    let response = client
        .post(&format!("{}/posts", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(response.status().as_u16(), 200);

    let db_pool = &app.db_pool;
    let post = sqlx::query_as!(
        BlogPost,
        r#"
        SELECT *
        FROM blog_posts
        WHERE username = $1
        "#,
        "valid_user"
    )
    .fetch_one(db_pool)
    .await
    .expect("Failed to fetch post from database.");

    assert_eq!(post.text, "This is a sample post text.");
    assert_eq!(post.username, "valid_user");
    assert!(post.image_path.is_some());
    assert!(post.user_avatar_path.is_some());
}

#[tokio::test]
async fn create_post_missing_text_returns_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let form = multipart::Form::new().text("username", "valid_user");

    let response = client
        .post(&format!("{}/posts", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_post_invalid_username_length_returns_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let form = multipart::Form::new()
        .text("text", "This is a sample post text.")
        .text("username", "a");

    let response = client
        .post(&format!("{}/posts", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
    let error_message = response
        .text()
        .await
        .expect("Failed to read response text.");
    assert!(error_message.contains("Username must be between 2 and 50 characters"));
}

#[tokio::test]
async fn create_post_invalid_text_length_returns_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let form = multipart::Form::new()
        .text("text", "Short")
        .text("username", "valid_user");

    let response = client
        .post(&format!("{}/posts", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
    let error_message = response
        .text()
        .await
        .expect("Failed to read response text.");
    assert!(error_message.contains("Text must be between 10 and 10,000 characters"));
}

#[tokio::test]
async fn create_post_invalid_image_type_returns_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let image_name = "jetbrains-logo-wrong-format.jpg";

    let image = get_image_asset(&image_name);

    let form = multipart::Form::new()
        .text("text", "This is a sample post text.")
        .text("username", "valid_user")
        .part("image", multipart::Part::bytes(image).file_name(image_name));

    let response = client
        .post(&format!("{}/posts", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn create_post_invalid_avatar_type_returns_400() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let image_name = "jetbrains-logo.png";

    let image = get_image_asset(&image_name);

    let form = multipart::Form::new()
        .text("text", "This is a sample post text.")
        .text("username", "valid_user")
        .text("user_avatar_url", JETBRAINS_JPG_LOGO_URL)
        .part("image", multipart::Part::bytes(image).file_name(image_name));

    let response = client
        .post(&format!("{}/posts", &app.address))
        .multipart(form)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 400);
}
