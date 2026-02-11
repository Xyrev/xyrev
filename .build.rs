use clap::CommandFactory;
use clap_complete::{generate_to, Shell};
use clap_mangen::Man;
use std::fs;
use std::io::Error;
use std::path::PathBuf;

#[path = "src/cli/mod.rs"]
mod cli;

use cli::Cli;

fn main() -> Result<(), Error> {
    // 创建 CLI 实例
    let mut cmd = Cli::command();
    
    // 生成 completions
    generate_completions(&mut cmd)?;
    
    // 生成 man page
    generate_man_page(&mut cmd)?;
    
    println!("cargo:rerun-if-changed=src/cli/");
    println!("cargo:rerun-if-changed=src/main.rs");
    
    Ok(())
}

fn generate_completions(cmd: &mut clap::Command) -> Result<(), Error> {
    let completions_dir = PathBuf::from("completions");
    fs::create_dir_all(&completions_dir)?;
    
    let bin_name = "xyrev";
    
    // 生成各种 shell 的 completion 文件
    generate_to(Shell::Bash, cmd, bin_name, &completions_dir)?;
    generate_to(Shell::Fish, cmd, bin_name, &completions_dir)?;
    generate_to(Shell::Zsh, cmd, bin_name, &completions_dir)?;
    generate_to(Shell::PowerShell, cmd, bin_name, &completions_dir)?;
    generate_to(Shell::Elvish, cmd, bin_name, &completions_dir)?;

    Ok(())
}

fn generate_man_page(cmd: &mut clap::Command) -> Result<(), Error> {
    let man_dir = PathBuf::from("man-page");
    fs::create_dir_all(&man_dir)?;
    
    let man = Man::new(cmd.clone());
    let mut buffer = Vec::new();
    man.render(&mut buffer)?;
    
    let man_path = man_dir.join("xyrev.1");
    fs::write(&man_path, buffer)?;
    
    Ok(())
}
