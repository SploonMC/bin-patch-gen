//! Module containing utilities.

use chrono::Local;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

pub mod dir;

pub struct TimeFormatter;
impl FormatTime for TimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let date = Local::now();
        write!(w, "{}", date.format("%H:%M:%S"))
    }
}