use std::fmt::{Error, Write};
use std::path::Path;

use crossterm::style::Colorize;
use tracing::subscriber::Subscriber;
use tracing::{info, Event, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_log::{LogTracer, NormalizeEvent};
use tracing_subscriber::fmt::time::{ChronoLocal, FormatTime};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

pub fn init_logging(log_file: &str) -> WorkerGuard {
    let default_level = Level::INFO;
    let log_file = Path::new(log_file);
    let log_dir = log_file.parent().unwrap();
    let file_name = log_file.file_name().unwrap();

    let file_appender = rolling::never(log_dir, file_name);
    let (log_writer, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_max_level(match std::env::var("RUST_LOG") {
            Ok(level) => match level.as_str() {
                "info" | "INFO" => Level::INFO,
                "warn" | "WARN" => Level::WARN,
                "error" | "ERROR" => Level::ERROR,
                "debug" | "DEBUG" => Level::DEBUG,
                "trace" | "TRACE" => Level::TRACE,
                _ => default_level,
            },
            _ => default_level,
        })
        .with_writer(log_writer)
        .event_format(SimpleFmt)
        .try_init()
        .unwrap();
    info!("Starting barnine...");

    match LogTracer::init() {
        Ok(_) => (),
        Err(err) => {
            info!("{}", err.to_string())
        },
    }

    guard
}

struct SimpleFmt;

impl<S, N> FormatEvent<S, N> for SimpleFmt
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        writer: &mut dyn Write,
        event: &Event<'_>,
    ) -> Result<(), Error> {
        let time_format = "%b %d %I:%M:%S%.6f %p";
        let mut time_now = String::new();
        ChronoLocal::with_format(time_format.into()).format_time(&mut time_now)?;

        // Get line numbers from log crate events
        let normalized_meta = event.normalized_metadata();
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());

        let message = format!(
            "{}{} {}{}{} ",
            time_now.grey(),
            meta.level().to_string().blue(),
            meta.file().unwrap_or("").to_string().yellow(),
            String::from(":").yellow(),
            meta.line().unwrap_or(0).to_string().yellow(),
        );
        write!(writer, "{}", message).unwrap();
        ctx.format_fields(writer, event)?;
        writeln!(writer)
    }
}
