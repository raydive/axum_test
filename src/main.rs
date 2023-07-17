use std::net::SocketAddr;

use axum::{handler::Handler, routing::get, Router};

use tower_http::trace::TraceLayer;

mod otel;

#[tokio::main]
async fn main() {
    // initialize tracing
    let tracer = otel::init_tracer();
    otel::setup_subscriber(tracer);

    let traced_root = root.layer(
        TraceLayer::new_for_http()
            .make_span_with(otel::make_span)
            .on_request(otel::on_request),
    );

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(traced_root));

    let addr = SocketAddr::from(([127, 0, 0, 1], 19000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    opentelemetry::global::shutdown_tracer_provider();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
