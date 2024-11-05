use jetbrains_web_app::configuration;
use jetbrains_web_app::startup::Appliaction;
use jetbrains_web_app::telemetry::initialize_tracing;

#[tokio::main]
async fn main() {
    initialize_tracing().expect("Failed to initialize application tracing.");
    let configuration = configuration::get_configuration().expect("Failed to read configuration.");

    let application = Appliaction::build(&configuration)
        .await
        .expect("Failed to build application.");

    application.run_until_stopped().await;
}
