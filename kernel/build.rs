use chrono::Utc;
use chrono::{Datelike, Timelike};

fn main() {
    let now = Utc::now();

    let formatted = format!(
        "{:02}/{:02}/{:04} {:02}:{:02}:{:02}",
        now.day(),
        now.month(),
        now.year(),
        now.hour(),
        now.minute(),
        now.second()
    );

    println!("cargo:rustc-env=BUILD_TIME={}", formatted);
}