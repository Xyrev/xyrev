use std::io::Write;
use termcolor::{Color, ColorChoice as TermColorChoice, ColorSpec, StandardStream, WriteColor};

#[allow(dead_code)]
pub enum LogLevel {
    Info,
    Debug,
    Trace,
}

#[allow(dead_code)]
pub fn log(level: LogLevel, _verbose: u8, message: &str) -> std::io::Result<()> {
    let mut stdout = StandardStream::stdout(TermColorChoice::Auto);

    match level {
        LogLevel::Info => {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)).set_bold(true))?;
            writeln!(&mut stdout, "[INFO] {}", message)?;
        }
        LogLevel::Debug => {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            writeln!(&mut stdout, "[DEBUG] {}", message)?;
        }
        LogLevel::Trace => {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)))?;
            writeln!(&mut stdout, "[TRACE] {}", message)?;
        }
    }

    stdout.reset()?;
    Ok(())
}
