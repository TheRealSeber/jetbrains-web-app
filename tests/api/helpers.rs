use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use jetbrains_web_app::{
    configuration::{get_configuration, DatabaseSettings},
    startup::Appliaction,
    telemetry::initialize_tracing,
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tracing::error;

static TRACING: Lazy<()> = Lazy::new(|| {
    // Lower level tracing results in mess output as this is not yet fixed
    // https://github.com/launchbadge/sqlx/pull/3548
    let _ = initialize_tracing("error");
});

pub struct TestApp {
    pub address: String,
    pub upload_path: PathBuf,
    pub db_pool: PgPool,
}

impl Drop for TestApp {
    fn drop(&mut self) {
        if self.upload_path.exists() {
            if let Err(e) = fs::remove_dir_all(&self.upload_path) {
                error!(
                    "Failed to clean up upload path {}: {}",
                    self.upload_path.display(),
                    e
                );
            }
        }
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        c.database.db_name = uuid::Uuid::new_v4().to_string();
        c.application.port = 0;
        c.application.upload_path = create_temp_image_dir();
        c
    };

    let db_pool = configure_database_for_tests(&configuration.database).await;

    let application = Appliaction::build(&configuration.clone())
        .await
        .expect("Failed to build application");

    let application_port = application.port();
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address: format!("http://localhost:{}", application_port),
        upload_path: configuration.application.upload_path,
        db_pool,
    }
}

async fn configure_database_for_tests(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.db_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations");

    connection_pool
}

pub fn get_image_asset(name: &str) -> Vec<u8> {
    let mut image_data = Vec::new();
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests\\resources")
        .join(name);

    let mut file = File::open(d).expect("Failed to open image file");
    file.read_to_end(&mut image_data)
        .expect("Failed to read image file");
    image_data
}

pub fn create_temp_image_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());
    fs::create_dir(&dir).expect("Failed to create temp dir");
    dir
}
