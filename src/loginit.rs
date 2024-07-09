use colored::{ColoredString, Colorize};
use env_logger::{Builder, Target};
use log::Level;
use std::io::Write;
use std::sync::Once;
static INIT: Once = Once::new();

fn colored_level(level: Level) -> ColoredString {
    match level {
        Level::Trace => "TRACE".magenta(),
        Level::Debug => "DEBUG".blue(),
        Level::Info => "INFO".green(),
        Level::Warn => "WARN".yellow(),
        Level::Error => "ERROR".red(),
    }
}

pub fn init_from_env(level: Option<log::LevelFilter>) {
    INIT.call_once(|| {
        let mut builder = Builder::from_default_env();
        builder.target(Target::Stderr);
        if let Some(level) = level {
            builder.filter_level(level);
        }
        builder.filter_module("cranelift_codegen", log::LevelFilter::Warn)
            .filter_module("wasmtime", log::LevelFilter::Warn)
            .filter_module("wasmtime_cranelift", log::LevelFilter::Warn)
            .filter_module("h2", log::LevelFilter::Warn)
            .filter_module("hyper", log::LevelFilter::Warn)
            .filter_module("rustls",log::LevelFilter::Warn);
        // builder
        //     .try_init()
        //     .expect("env_logger::init should not be called after logger initialized")
        builder
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{}] {} [{}:{}] {}",
                    colored_level(record.level()),
                    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.target().cyan(),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();
    });
}

// #[cfg(any(not(test), debug_assertions))]
pub fn init_level(level: &str) {
    let level = match level {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Error,
    };
    init_from_env(Some(level));
}
