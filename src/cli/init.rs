use crate::common::constants::ROOT_DIR;
use clap::Args;
use std::env;
use std::fs;

#[derive(Args, Debug)]
#[command(
    about = "Create a new configuration folder or reinitialize the existing one",
    after_help = "For example:\n  stool dotfile init https://github.com/YOUR_NAME/dotfiles"
)]
pub struct InitCommand {
    #[arg(
        short,
        long,
        help = "Force reinitialize configuration (overwrite existing)"
    )]
    pub force: bool,

    #[arg(
        short = 'C',
        long,
        help = "Clean and reinitialize (remove all existing data)"
    )]
    pub clean: bool,
}

impl InitCommand {
    pub fn run(&self, verbose: bool) {
        if verbose {
            println!("[VERBOSE] Initializing with verbose mode enabled");
        }

        if let Ok(home_dir) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
            let _stool_dir = std::path::Path::new(&home_dir).join(ROOT_DIR);
        } else {
            println!("无法获取用户主目录");
        }
    }

    #[allow(dead_code)]
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("===> 正在初始化项目...");

        // 获取用户家目录
        let home_dir = env::var("HOME").or_else(|_| env::var("USERPROFILE"))?;

        let stool_dir = std::path::Path::new(&home_dir).join(ROOT_DIR);

        // 处理 --clean 选项
        if self.clean & stool_dir.exists() {
            fs::remove_dir_all(&stool_dir)?;
        }

        if stool_dir.exists() {
            if self.force {
                fs::remove_dir_all(&stool_dir)?;
                fs::create_dir(&stool_dir)?;
            } else if !self.clean {
                return Ok(());
            }
        } else {
            fs::create_dir(&stool_dir)?;
        }

        Ok(())
    }
}
