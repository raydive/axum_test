use hyper::Request;
use opentelemetry::{
    global,
    sdk::{export::trace::stdout, trace::Tracer},
};
use opentelemetry_http::HeaderExtractor;
use tracing::{subscriber, Span};
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};
use tracing_subscriber::layer::SubscriberExt;

pub fn init_tracer() -> Tracer {
    let tracer = stdout::new_pipeline().install_simple();

    tracer
}

pub fn setup_subscriber(tracer: Tracer) {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
    );

    let layer = OpenTelemetryLayer::new(tracer).with_location(true);

    let subscriber = tracing_subscriber::Registry::default().with(layer);
    subscriber::set_global_default(subscriber).expect("setting tracing default failed");
}

pub fn make_span<B>(req: &Request<B>) -> Span {
    let ctx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    });
    let span = tracing::info_span!(
        "request",
        method = %req.method(),
        path = %req.uri().path(),
        version = ?req.version(),
        host = ?req.uri().host(),
        scheme = ?req.uri().scheme_str(),
        query = ?req.uri().query(),
        request_id = tracing::field::Empty,
    );

    span.set_parent(ctx);
    span
}

pub fn on_request<B>(req: &Request<B>, span: &Span) {
    span.record(
        "request_id",
        &tracing::field::debug(
            req.headers()
                .get("x-request-id")
                .unwrap_or(&"none".parse().unwrap()),
        ),
    );

    // this field will not be shown in the logs
    span.record("not_shown", &tracing::field::display("not shown"));
}
