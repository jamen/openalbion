use derive_more::{Display, Error};
use log::{Level, LevelFilter, Log, Metadata, Record};
use nu_ansi_term::Color;

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        let level = record.level();

        let prefix = match level {
            Level::Error => Color::Red.paint("error"),
            Level::Warn => Color::Yellow.paint("warning"),
            Level::Info => Color::Blue.paint("info"),
            Level::Debug => Color::DarkGray.paint("debug"),
            Level::Trace => Color::Green.paint("trace"),
        };

        let message = record.args();
        let output = format!("{}: {}", prefix, message);

        if level == Level::Error {
            eprintln!("{}", output)
        } else {
            println!("{}", output)
        }
    }

    fn flush(&self) {}
}

#[derive(Error, Debug, Display)]
#[display("Failed to initialize logger")]
pub struct SimpleLoggerInitError;

pub fn init(max_level: LevelFilter) -> Result<(), SimpleLoggerInitError> {
    log::set_logger(&SimpleLogger).or(Err(SimpleLoggerInitError))?;
    log::set_max_level(max_level);
    Ok(())
}
