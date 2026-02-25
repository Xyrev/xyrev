mod cli;
mod common;

use clap::Parser;
use cli::Cli;
use common::log::{log_rich, Span};
use termcolor::Color;

fn main() {
    let cli = Cli::parse();

    // let log_level = match cli.verbose {
    //     0 => log::LogLevel::Info,
    //     1 => log::LogLevel::Debug,
    //     _ => log::LogLevel::Trace,
    // };

    infolog!("asdasdad");

    log_rich(&[
        Span::new("==> ").color(Color::Green).bold(),
        Span::new("编译完成，耗时 ").bold(),
        Span::new("1.23s").color(Color::Cyan),
    ])
    .unwrap();
    // log(log_level, cli.verbose, "欢迎使用 Xyrev！").expect("日志输出失败");

    match cli.command {
        cli::Commands::Init(cmd) => cmd.run(cli.verbose),
        cli::Commands::Agent(cmd) => cmd.run(cli.verbose),
        #[cfg(target_os = "macos")]
        cli::Commands::Apple(cmd) => println!("Apple {}", cmd.name),
        cli::Commands::Dotfile(cmd) => println!("Dotfile {}", cmd.name),
    }
}
