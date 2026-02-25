use std::io::Write;
use termcolor::{Color, ColorChoice as TermColorChoice, ColorSpec, StandardStream, WriteColor};

#[allow(dead_code)]
pub enum LogLevel {
    Info,
    Debug,
    Trace,
}

/// A text segment with optional color and bold styling.
#[allow(dead_code)]
pub struct Span {
    pub text: String,
    pub color: Option<Color>,
    pub bold: bool,
}

#[allow(dead_code)]
impl Span {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: None,
            bold: false,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

/// Log a simple single-color message.
#[allow(dead_code)]
pub fn log(level: LogLevel, _verbose: u8, message: &str) -> std::io::Result<()> {
    let (prefix, color, bold) = match level {
        LogLevel::Info => ("[INFO]", Color::Green, true),
        LogLevel::Debug => ("[DEBUG]", Color::Blue, false),
        LogLevel::Trace => ("[TRACE]", Color::Magenta, false),
    };

    let mut prefix_span = Span::new(format!("{} ", prefix)).color(color);
    if bold {
        prefix_span = prefix_span.bold();
    }

    log_rich(&[prefix_span, Span::new(message)])
}

/// Log a line composed of multiple colored spans.
#[allow(dead_code)]
pub fn log_rich(spans: &[Span]) -> std::io::Result<()> {
    let mut stdout = StandardStream::stdout(TermColorChoice::Auto);

    for span in spans {
        let mut spec = ColorSpec::new();
        if let Some(c) = span.color {
            spec.set_fg(Some(c));
        }
        spec.set_bold(span.bold);
        stdout.set_color(&spec)?;
        write!(&mut stdout, "{}", span.text)?;
    }

    stdout.reset()?;
    writeln!(&mut stdout)?;
    Ok(())
}
