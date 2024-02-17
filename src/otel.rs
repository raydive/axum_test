use hyper::Request;
use opentelemetry::{global, trace::TracerProvider as _};

use opentelemetry_http::HeaderExtractor;
use opentelemetry_sdk::trace::{Tracer, TracerProvider};
use tracing::{subscriber, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_tracer() -> Tracer {
    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    provider.tracer("otel/sample")
}

pub fn setup_subscriber(tracer: Tracer) {
    global::set_text_map_propagator(opentelemetry_sdk::propagation::TraceContextPropagator::new());

    let layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let log = tracing_subscriber::fmt::layer();

    let subscriber = tracing_subscriber::Registry::default().with(layer).with(log);
    subscriber::set_global_default(subscriber).expect("setting tracing default failed");
}

pub fn make_span<B>(req: &Request<B>) -> Span {
    let ctx = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(req.headers()))
    });
    let span = tracing::info_span!(
        "a request",
        method = %req.method(),
        path = %req.uri().path(),
        version = ?req.version(),
        host = ?req.uri().host(),
        scheme = ?req.uri().scheme_str(),
        query = ?req.uri().query(),
        request_id = tracing::field::Empty,
    );
    span.set_attribute("att", "you show me!");

    span.set_parent(ctx);
    span
}

pub fn on_request<B>(req: &Request<B>, span: &Span) {
    span.record(
        "request_id",
        &tracing::field::debug(
            req.headers()
                .get("x-request-id")
                .unwrap_or(&"who are you?".parse().unwrap()),
        ),
    );

    // this field will not be shown in the logs
    span.record("not_shown", &tracing::field::display("not shown"));
}
