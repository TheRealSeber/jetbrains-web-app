use crate::configuration::Settings;
use crate::routes::health_check::handle_get;
use crate::routes::home::home;
use crate::routes::posts::create_post;
use crate::telemetry::{
    trace_layer_make_span_with, trace_layer_on_request, trace_layer_on_response,
};
use axum::routing::{get, post};
use axum::{serve::Serve, Router};
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub struct Appliaction {
    port: u16,
    server: Serve<Router, Router>,
}

#[derive(Clone)]
pub struct AppState {
    pub connection_pool: PgPool,
    pub upload_path: std::path::PathBuf,
    pub http_client: Client,
}

impl Appliaction {
    pub async fn build(configuration: &Settings) -> Result<Self, std::io::Error> {
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );

        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr().unwrap().port();
        let connection_pool = get_connection_pool(configuration);
        let http_client = Client::new();

        let app_state = AppState {
            connection_pool,
            upload_path: configuration.application.upload_path.clone(),
            http_client,
        };

        let server = run(listener, app_state)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) {
        self.server.await.expect("Server failed");
    }
}

fn get_connection_pool(settings: &Settings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(settings.database.with_db())
}

pub fn run(
    listener: TcpListener,
    app_state: AppState,
) -> Result<Serve<Router, Router>, std::io::Error> {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(trace_layer_make_span_with)
        .on_request(trace_layer_on_request)
        .on_response(trace_layer_on_response);

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::serve(
        listener,
        Router::new()
            .route("/health_check", get(handle_get))
            .route("/home", get(home))
            .route("/posts", post(create_post))
            .nest_service("/uploads", ServeDir::new(&app_state.upload_path.clone()))
            .with_state(app_state.into())
            .layer(trace_layer),
    );

    Ok(server)
}
