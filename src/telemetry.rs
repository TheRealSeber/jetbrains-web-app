use axum::response::Response;
use hyper::Request;
use std::{error::Error, io::IsTerminal, time::Duration};
use tracing::{Span, Subscriber};
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};

pub fn fmt_layer_json<S>() -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    tracing_subscriber::fmt::Layer::new()
        .with_ansi(std::io::stderr().is_terminal())
        .with_writer(std::io::stderr)
        .json()
}

pub fn trace_layer_make_span_with(request: &Request<axum::body::Body>) -> Span {
    let request_id = uuid::Uuid::new_v4();
    tracing::error_span!("request",
        uri = %request.uri(),
        method = %request.method(),
        request_id = %request_id,
        status = tracing::field::Empty,
        latency = tracing::field::Empty,
    )
}

pub fn trace_layer_on_request(_request: &Request<axum::body::Body>, _span: &Span) {
    tracing::trace!("START")
}

pub fn trace_layer_on_response(
    response: &Response<axum::body::Body>,
    latency: Duration,
    span: &Span,
) {
    span.record(
        "latency",
        tracing::field::display(format!("{}μs", latency.as_micros())),
    );
    span.record("status", tracing::field::display(response.status()));
    tracing::trace!("END");
}

pub fn initialize_tracing(env_filter: &str) -> Result<(), Box<dyn Error>> {
    let filter = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new(env_filter))?;

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer_json())
        .with(tracing_error::ErrorLayer::default())
        .init();

    Ok(())
}
