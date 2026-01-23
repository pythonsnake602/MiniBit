use opentelemetry::global;
use opentelemetry::global::BoxedTracer;
use serde::{Deserialize, Serialize};
use tracing::{dispatcher, span, Id, Span};
use tracing::span::EnteredSpan;
use valence::prelude::*;
use valence::registry::Registry;

#[derive(Default, Clone, Copy, Deserialize, Serialize)]
#[serde(default)]
pub struct TelemetryConfig {
    pub tracing: bool,
}

pub struct TelemetryPlugin {
    pub name: String,
    pub config: TelemetryConfig,
}

#[derive(Resource)]
struct SpanIdResource(Option<Span>, Option<Id>);

impl Plugin for TelemetryPlugin {
    fn build(&self, app: &mut App) {
        if self.config.tracing {
            app.insert_resource(SpanIdResource(None, None))
                .add_systems(First, record_tick_start)
                .add_systems(Last, exit_tick);
        }
    }
}

fn record_tick_start(
    mut span_id: ResMut<SpanIdResource>,
) {
    let span = span!(tracing::Level::INFO, "tick");
    if let Some(id) = span.id() {
        span_id.1 = span.id();
        span_id.0 = Some(span);
        dispatcher::get_default(|dispatch| dispatch.enter(&id))
    }
}

fn exit_tick(
    mut span_id: ResMut<SpanIdResource>,
) {
    if let Some(id) = &span_id.1 {
        dispatcher::get_default(|dispatch| dispatch.exit(id));
        span_id.0 = None;
        span_id.1 = None;
    }
}