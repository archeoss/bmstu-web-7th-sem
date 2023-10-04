use cli::{Config, LoggerConfig};
use core::fmt;
use error_stack::Context;
use file_rotate::{suffix::AppendTimestamp, ContentLimit, FileRotate};
use std::fmt::Display;
use tower_http::cors::CorsLayer;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{filter::LevelFilter, prelude::*};

#[allow(clippy::module_name_repetitions)]
pub trait ConfigExt {
    /// Return either very permissive [`CORS`](`CorsLayer`) configuration
    /// or empty one based on `cors_allow_all` field
    fn get_cors_configuration(&self) -> CorsLayer;
}

pub trait LoggerExt {
    /// Initialize logger.
    ///
    /// Returns [`WorkerGuard`]s for off-thread writers.
    /// Should not be dropped.
    ///
    /// # Errors
    ///
    /// Function returns error if `init_file_rotate` fails
    fn init_logger(&self) -> Result<Vec<WorkerGuard>, LoggerError>;

    /// Returns [`std:io::Write`] object that rotates files on write
    ///
    /// # Errors
    ///
    /// Function returns error if `log_file` is not specified
    fn init_file_rotate(&self) -> Result<FileRotate<AppendTimestamp>, LoggerError>;
}

impl ConfigExt for Config {
    fn get_cors_configuration(&self) -> CorsLayer {
        self.cors_allow_all
            .then_some(CorsLayer::very_permissive())
            .unwrap_or_default()
    }
}

impl LoggerExt for LoggerConfig {
    fn init_logger(&self) -> Result<Vec<WorkerGuard>, LoggerError> {
        let mut guards = Vec::with_capacity(2);
        let file_writer = self.file.as_ref().map_or_else(
            || Ok(tracing_appender::non_blocking(std::io::sink()).0),
            |config| {
                Ok(if config.enabled {
                    let (writer, guard) = tracing_appender::non_blocking(self.init_file_rotate()?);
                    guards.push(guard);
                    writer
                } else {
                    tracing_appender::non_blocking(std::io::sink()).0
                })
            },
        )?;
        let std_out_writer = self.stdout.as_ref().map_or_else(
            || tracing_appender::non_blocking(std::io::sink()).0,
            |config| {
                if config.enabled {
                    let (writer, guard) = tracing_appender::non_blocking(std::io::stdout());
                    guards.push(guard);
                    writer
                } else {
                    tracing_appender::non_blocking(std::io::sink()).0
                }
            },
        );

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_writer)
                    .with_filter(LevelFilter::from_level(self.trace_level)),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std_out_writer)
                    .with_filter(LevelFilter::from_level(self.trace_level)),
            )
            .init();

        Ok(guards)
    }

    fn init_file_rotate(&self) -> Result<FileRotate<AppendTimestamp>, LoggerError> {
        let config = self.file.as_ref().ok_or(LoggerError::EmptyConfig)?;
        Ok(FileRotate::new(
            config.log_file.as_ref().ok_or(LoggerError::NoFileName)?,
            AppendTimestamp::default(file_rotate::suffix::FileLimit::MaxFiles(config.log_amount)),
            ContentLimit::BytesSurpassed(config.log_size),
            file_rotate::compression::Compression::OnRotate(1),
            None,
        ))
    }
}

#[derive(Debug)]
pub enum LoggerError {
    EmptyConfig,
    NoFileName,
}

impl Display for LoggerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyConfig => "Empty logger configuration",
            Self::NoFileName => "No filename specified",
        })
    }
}

impl Context for LoggerError {}
