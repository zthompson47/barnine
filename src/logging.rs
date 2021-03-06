use std::env;
use std::fmt::{Error, Write};
use std::path::Path;

use crossterm::style::Colorize;
use tracing::subscriber::Subscriber;
use tracing::{info, Event, Level};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_log::NormalizeEvent;
use tracing_subscriber::fmt::time::{ChronoLocal, FormatTime};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;

pub fn init_logging(app_name: &str) -> WorkerGuard {
    let default_level = Level::INFO;
    let log_dir = get_log_dir(app_name);
    let mut file_name = app_name.to_string();
    file_name.push_str(".log");

    let file_appender = rolling::never(log_dir, file_name);
    let (log_writer, guard) = tracing_appender::non_blocking(file_appender);

    match tracing_subscriber::fmt()
        .with_max_level(match env::var("RUST_LOG") {
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
    {
        Ok(_) => info!("Starting barnine..."),
        Err(e) => eprintln!("{}", e.to_string()),
    }

    guard
}

fn get_log_dir(app_name: &str) -> Box<Path> {
    // Look for APPNAME_DEV_DIR environment variable to override default
    let mut dev_dir = app_name.to_uppercase();
    dev_dir.push_str("_DEV_DIR");

    let result = match env::var(dev_dir) {
        Ok(dir) => Path::new(&dir).to_path_buf(),
        Err(_) => match env::var("XDG_CACHE_DIR") {
            Ok(dir) => Path::new(&dir).join(app_name),
            Err(_) => match env::var("HOME") {
                Ok(dir) => Path::new(&dir).join(".cache").join(app_name),
                Err(_) => Path::new("/tmp").join(app_name),
            },
        },
    };

    result.into_boxed_path()
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
        // Create timestamp
        let time_format = "%b %d %I:%M:%S%.6f %p";
        let mut time_now = String::new();
        ChronoLocal::with_format(time_format.into()).format_time(&mut time_now)?;

        // Get line numbers from log crate events
        let normalized_meta = event.normalized_metadata();
        let meta = normalized_meta.as_ref().unwrap_or_else(|| event.metadata());

        // Write formatted log record
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

#[cfg(test)]
mod tests {
    use std::{env, fs, io::prelude::*, io::BufReader, path::Path};

    use log;
    use tempfile::tempdir;
    use tracing;

    use super::{get_log_dir, init_logging};

    #[test]
    fn generate_log_records() {
        // Use tempdir for log files
        let dir = tempdir().unwrap().into_path();
        env::set_var("XDG_CACHE_DIR", &dir);
        let _guard = init_logging("barnine-test");

        // Try both logging crates
        log::info!("test log INFO");
        tracing::debug!("test tracing DEBUG");

        // Confirm log file created
        let dir = dir.join("barnine-test");
        assert!(dir.is_dir());
        let log_file = dir.join("barnine-test.log");
        assert!(log_file.is_file());

        // Drop logging guard to flush logs.
        // TODO test it??  failure below was intermittent
        //   hmm still dropping records after this drop..
        drop(_guard);

        // Check log records
        let file = fs::File::open(&log_file).unwrap();
        let buf_reader = BufReader::new(file);
        let log_records: Vec<String> = buf_reader.lines().map(|x| x.unwrap()).collect();
        assert!(log_records.len() >= 2);
        let idx = log_records.len() - 2;

        if !log_records[idx].contains("test log INFO") {
            // TODO - Log records are occasionally dropped... ??
            println!("{}", fs::read_to_string(&log_file).unwrap());
        }

        assert!(log_records[idx].contains("test log INFO"));
        assert!(log_records[idx + 1].ends_with("test tracing DEBUG"));
    }

    #[test]
    fn change_log_dir_location() {
        env::remove_var("XDG_CACHE_DIR");
        env::remove_var("HOME");
        assert_eq!(Path::new("/tmp/test"), &*get_log_dir("test"));

        env::set_var("XDG_CACHE_DIR", "/foo");
        assert_eq!(Path::new("/foo/test"), &*get_log_dir("test"));

        env::remove_var("XDG_CACHE_DIR");
        env::set_var("HOME", "/bar");
        assert_eq!(Path::new("/bar/.cache/test"), &*get_log_dir("test"));
    }
}
