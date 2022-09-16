use std::io::Write;
use std::thread;

use chrono::prelude::*;
use env_logger::fmt::Formatter;
use env_logger::Builder;
use log::{LevelFilter, Record};

/// setup_logger
///
/// Setup a logger for message processing by consumers or producers
///
/// # Arguments
///
/// * `log_thread` - flag for logging the processing thread
/// * `rust_log` - string containing the logging level for the function caller
///
/// # Examples
///
/// ```rust
/// use rust_with_kafka_tls::example_utils::setup_logger;
/// setup_logger(true, Some("rdkafka=trace"));
/// ```
///
pub fn setup_logger(log_thread: bool, rust_log: Option<&str>) {
    let output_format = move |formatter: &mut Formatter, record: &Record| {
        let thread_name = if log_thread {
            format!("(t: {}) ", thread::current().name().unwrap_or("unknown"))
        } else {
            "".to_string()
        };

        let local_time: DateTime<Local> = Local::now();
        let time_str = local_time.format("%H:%M:%S%.3f").to_string();
        writeln!(
            formatter,
            "{} {}{} - {} - {}\n",
            time_str,
            thread_name,
            record.level(),
            record.target(),
            record.args()
        )
    };

    let mut builder = Builder::new();
    builder
        .format(output_format)
        .filter(None, LevelFilter::Info);

    rust_log.map(|conf| builder.parse_filters(conf));

    builder.init();
}

#[allow(dead_code)]
fn main() {
    println!("This is not an example");
}
