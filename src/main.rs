use axum::{
    routing::get,
    Router, handler::Handler,
};
use opentelemetry::{sdk::export::trace::stdout, sdk::trace::Tracer};
use tower_http::trace::TraceLayer;
use tracing::{Span, field::Empty, subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
async fn main() {
    // initialize tracing
    let tracer = init_tracer();
    setup_subscriber(tracer);

    let traced_root = root.layer(
        TraceLayer::new_for_http()
            .make_span_with(|_: &_| tracing::info_span!("root", request_id = Empty) )
            .on_request(|_: &_, span: &Span| {
                span.record("request_id", &tracing::field::display("12345"));

                // this field will not be shown in the logs
                span.record("not_shown", &tracing::field::display("not shown"));
            })
    );

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(traced_root));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:9000").await.unwrap();
    let addr = listener.local_addr().unwrap();
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

fn init_tracer() -> Tracer {
    let tracer = stdout::new_pipeline().install_simple();

    tracer
}

fn setup_subscriber(tracer: Tracer) {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
    );

    let layer = OpenTelemetryLayer::new(tracer)
        .with_location(true);

    let subscriber = tracing_subscriber::Registry::default().with(layer);
    subscriber::set_global_default(subscriber).expect("setting tracing default failed");
}