use chrono::Utc;
use colored::Colorize;

pub fn timestamp() -> String {
    Utc::now().to_rfc3339()
}

pub fn title(msg: &str) {
    println!(
        "{} {} {}",
        timestamp().truecolor(128, 128, 128),
        " TITLE ".on_blue().white().bold(),
        msg.bright_blue().bold()
    );
}

pub fn success(msg: &str) {
    println!(
        "{} {} {}",
        timestamp().truecolor(128, 128, 128),
        " SUCCESS ".on_green().black().bold(),
        msg.bright_green()
    );
}

pub fn warning(msg: &str) {
    println!(
        "{} {} {}",
        timestamp().truecolor(128, 128, 128),
        " WARNING ".on_yellow().black().bold(),
        msg.yellow()
    );
}

pub fn info(msg: &str) {
    println!(
        "{} {} {}",
        timestamp().truecolor(128, 128, 128),
        " INFO ".on_cyan().black().bold(),
        msg.cyan()
    );
}

pub fn error_msg(msg: &str, err: Option<&(dyn std::error::Error + Send + Sync)>) {
    let full = match err {
        Some(e) => format!("{}: {}", msg, e),
        None => msg.to_string(),
    };
    println!(
        "{} {} {}",
        timestamp().truecolor(128, 128, 128),
        " ERROR ".on_red().white().bold(),
        full.bright_red().bold()
    );
}

pub fn debug(msg: &str) {
    if std::env::var("DEBUG").as_deref() == Ok("true") {
        println!(
            "{} {} {}",
            timestamp().truecolor(128, 128, 128),
            " DEBUG ".on_magenta().white().bold(),
            msg.magenta()
        );
    }
}

pub struct Logger;

impl Logger {
    pub fn title(msg: &str) {
        title(msg);
    }
    pub fn success(msg: &str) {
        success(msg);
    }
    pub fn warning(msg: &str) {
        warning(msg);
    }
    pub fn info(msg: &str) {
        info(msg);
    }
    pub fn error(msg: &str, err: Option<&(dyn std::error::Error + Send + Sync)>) {
        error_msg(msg, err);
    }
    pub fn debug(msg: &str) {
        debug(msg);
    }
}

pub fn logger() -> Logger {
    Logger
}
