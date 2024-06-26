use std::{collections::HashMap, str::FromStr};

use opentelemetry_otlp::{HttpExporterBuilder, TonicExporterBuilder, WithExportConfig};
use tracing::Level;
use tracing_appender::rolling::RollingFileAppender;
use utils::config::Config;

pub enum Tracer {
    Stdout {
        level: Level,
        ansi: bool,
    },
    Log {
        level: Level,
        appender: RollingFileAppender,
        ansi: bool,
    },
    Journal {
        level: Level,
    },
    Otel {
        level: Level,
        tracer: OtelTracer,
    },
}

pub enum OtelTracer {
    Gprc(TonicExporterBuilder),
    Http(HttpExporterBuilder),
}

pub struct Tracers {
    pub tracers: Vec<Tracer>,
}

impl Tracers {
    pub fn parse(config: &mut Config) -> Self {
        let mut tracers = Vec::new();

        for tracer_id in config
            .sub_keys("tracer", ".type")
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
        {
            let id = tracer_id.as_str();
            let level = Level::from_str(config.value(("tracer", id, "level")).unwrap_or("info"))
                .map_err(|err| {
                    config.new_parse_error(
                        ("tracer", id, "level"),
                        format!("Invalid log level: {err}"),
                    )
                })
                .unwrap_or(Level::INFO);
            match config
                .value(("tracer", id, "type"))
                .unwrap_or_default()
                .to_string()
                .as_str()
            {
                "log" => {
                    if let Some(path) = config
                        .value_require_(("tracer", id, "path"))
                        .map(|s| s.to_string())
                    {
                        let prefix = config.value(("tracer", id, "prefix")).unwrap_or("stalwart");
                        let appender =
                            match config.value(("tracer", id, "rotate")).unwrap_or("daily") {
                                "daily" => tracing_appender::rolling::daily(path, prefix),
                                "hourly" => tracing_appender::rolling::hourly(path, prefix),
                                "minutely" => tracing_appender::rolling::minutely(path, prefix),
                                "never" => tracing_appender::rolling::never(path, prefix),
                                rotate => {
                                    let appender = tracing_appender::rolling::daily(path, prefix);
                                    let err = format!("Invalid rotate value: {rotate}");
                                    config.new_parse_error(("tracer", id, "rotate"), err);
                                    appender
                                }
                            };
                        tracers.push(Tracer::Log {
                            level,
                            appender,
                            ansi: config
                                .property_or_default_(("tracer", id, "ansi"), "true")
                                .unwrap_or(true),
                        });
                    }
                }
                "stdout" => {
                    tracers.push(Tracer::Stdout {
                        level,
                        ansi: config
                            .property_or_default_(("tracer", id, "ansi"), "true")
                            .unwrap_or(true),
                    });
                }
                "otel" | "open-telemetry" => {
                    match config
                        .value_require_(("tracer", id, "transport"))
                        .unwrap_or_default()
                    {
                        "gprc" => {
                            let mut exporter = opentelemetry_otlp::new_exporter().tonic();
                            if let Some(endpoint) = config.value(("tracer", id, "endpoint")) {
                                exporter = exporter.with_endpoint(endpoint);
                            }
                            tracers.push(Tracer::Otel {
                                level,
                                tracer: OtelTracer::Gprc(exporter),
                            });
                        }
                        "http" => {
                            if let Some(endpoint) = config
                                .value_require_(("tracer", id, "endpoint"))
                                .map(|s| s.to_string())
                            {
                                let mut headers = HashMap::new();
                                let mut err = None;
                                for (_, value) in config.values(("tracer", id, "headers")) {
                                    if let Some((key, value)) = value.split_once(':') {
                                        headers.insert(
                                            key.trim().to_string(),
                                            value.trim().to_string(),
                                        );
                                    } else {
                                        err = format!("Invalid open-telemetry header {value:?}")
                                            .into();
                                        break;
                                    }
                                }
                                if let Some(err) = err {
                                    config.new_parse_error(("tracer", id, "headers"), err);
                                }

                                let mut exporter = opentelemetry_otlp::new_exporter()
                                    .http()
                                    .with_endpoint(endpoint);
                                if !headers.is_empty() {
                                    exporter = exporter.with_headers(headers);
                                }

                                tracers.push(Tracer::Otel {
                                    level,
                                    tracer: OtelTracer::Http(exporter),
                                });
                            }
                        }
                        "" => {}
                        transport => {
                            let err = format!("Invalid transport: {transport}");
                            config.new_parse_error(("tracer", id, "transport"), err);
                        }
                    }
                }
                "journal" => {
                    tracers.push(Tracer::Journal { level });
                }
                unknown => {
                    config.new_parse_error(
                        ("tracer", id, "type"),
                        format!("Unknown tracer type: {unknown}"),
                    );
                }
            }
        }

        Tracers { tracers }
    }
}
