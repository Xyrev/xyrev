use crate::cli::init::InitCommand;
use crate::common::constants::ROOT_DIR;
use std::{env, fs};

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
