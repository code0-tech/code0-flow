pub mod errors;
use std::error::Error;

use opentelemetry::{KeyValue, global, trace::TracerProvider as _};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    Resource,
    logs::SdkLoggerProvider,
    metrics::{PeriodicReader, SdkMeterProvider},
    propagation::TraceContextPropagator,
    trace::SdkTracerProvider,
};
use serde::{Deserialize, Serialize};
use tracing_subscriber::{
    EnvFilter, Layer, filter::filter_fn, layer::SubscriberExt, util::SubscriberInitExt,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct OpenTelemetry {
    pub enabled: bool,
    pub service_name: String,
    pub logs_endpoint: Option<String>,
    pub metrics_endpoint: Option<String>,
    pub traces_endpoint: Option<String>,
}

impl OpenTelemetry {
    pub fn logs_endpoint(&self) -> Option<&str> {
        non_empty_url(&self.logs_endpoint)
    }

    pub fn metrics_endpoint(&self) -> Option<&str> {
        non_empty_url(&self.metrics_endpoint)
    }

    pub fn traces_endpoint(&self) -> Option<&str> {
        non_empty_url(&self.traces_endpoint)
    }

    pub fn has_enabled_exporter(&self) -> bool {
        self.logs_endpoint().is_some()
            || self.metrics_endpoint().is_some()
            || self.traces_endpoint().is_some()
    }
}

impl Default for OpenTelemetry {
    fn default() -> Self {
        Self {
            enabled: false,
            service_name: env!("CARGO_PKG_NAME").into(),
            logs_endpoint: None,
            metrics_endpoint: None,
            traces_endpoint: None,
        }
    }
}

pub struct TelemetrySettings<'a> {
    pub environment: &'a str,
    pub default_log_level: &'a str,
    pub service_version: &'a str,
    pub instrumentation_name: &'a str,
    pub initialize_metrics: Option<fn()>,
}

pub struct Telemetry {
    logger_provider: Option<SdkLoggerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    tracer_provider: Option<SdkTracerProvider>,
}

impl Telemetry {
    pub fn initialize(
        config: &OpenTelemetry,
        settings: TelemetrySettings<'_>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(settings.default_log_level))?;
        let fmt_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_target(false)
            .with_filter(filter_fn(|metadata| {
                metadata.target() != errors::EXCEPTION_TARGET
            }));

        if !config.enabled || !config.has_enabled_exporter() {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .init();
            return Ok(Self {
                logger_provider: None,
                meter_provider: None,
                tracer_provider: None,
            });
        }

        let resource = Resource::builder()
            .with_service_name(config.service_name.clone())
            .with_attributes([
                KeyValue::new("service.version", settings.service_version.to_owned()),
                KeyValue::new(
                    "deployment.environment.name",
                    settings.environment.to_owned(),
                ),
            ])
            .build();

        let tracer_provider = if let Some(endpoint) = config.traces_endpoint() {
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint.to_owned())
                .build()?;
            let provider = SdkTracerProvider::builder()
                .with_resource(resource.clone())
                .with_batch_exporter(exporter)
                .build();
            global::set_text_map_propagator(TraceContextPropagator::new());
            Some(provider)
        } else {
            None
        };

        let logger_provider = if let Some(endpoint) = config.logs_endpoint() {
            let exporter = opentelemetry_otlp::LogExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint.to_owned())
                .build()?;
            Some(
                SdkLoggerProvider::builder()
                    .with_resource(resource.clone())
                    .with_batch_exporter(exporter)
                    .build(),
            )
        } else {
            None
        };

        let meter_provider = if let Some(endpoint) = config.metrics_endpoint() {
            let exporter = opentelemetry_otlp::MetricExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint.to_owned())
                .build()?;
            let reader = PeriodicReader::builder(exporter).build();
            let provider = SdkMeterProvider::builder()
                .with_resource(resource)
                .with_reader(reader)
                .build();
            global::set_meter_provider(provider.clone());
            if let Some(initialize_metrics) = settings.initialize_metrics {
                initialize_metrics();
            }
            Some(provider)
        } else {
            None
        };

        if config.logs_endpoint().is_some() || config.traces_endpoint().is_some() {
            errors::enable_backtraces();
        }

        let trace_layer = tracer_provider.as_ref().map(|provider| {
            tracing_opentelemetry::layer()
                .with_tracer(provider.tracer(settings.instrumentation_name.to_owned()))
                .with_error_records_to_exceptions(true)
                .with_error_events_to_status(true)
                .with_filter(filter_fn(|metadata| {
                    metadata.target() != errors::SUMMARY_TARGET
                }))
        });
        let log_layer = logger_provider.as_ref().map(|provider| {
            OpenTelemetryTracingBridge::new(provider).with_filter(filter_fn(|metadata| {
                metadata.target() != errors::SUMMARY_TARGET
            }))
        });

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(trace_layer)
            .with(log_layer)
            .init();

        Ok(Self {
            logger_provider,
            meter_provider,
            tracer_provider,
        })
    }

    pub fn shutdown(self) {
        if let Some(provider) = self.logger_provider {
            let _ = provider.shutdown();
        }
        if let Some(provider) = self.meter_provider {
            let _ = provider.shutdown();
        }
        if let Some(provider) = self.tracer_provider {
            let _ = provider.shutdown();
        }
    }
}

fn non_empty_url(url: &Option<String>) -> Option<&str> {
    url.as_deref().filter(|value| !value.trim().is_empty())
}
