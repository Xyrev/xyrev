use clap::Args;
use std::process::Command;

#[derive(Args, Debug)]
pub struct CopilotCommand {
    /// Install method (auto, brew, npm, winget, script)
    #[arg(short, long, default_value = "auto")]
    pub method: String,

    /// Install prerelease version
    #[arg(long)]
    pub prerelease: bool,

    /// Specific version to install (e.g., v0.0.369)
    #[arg(long)]
    pub version: Option<String>,

    /// Custom installation prefix (for script method)
    #[arg(long)]
    pub prefix: Option<String>,
}

impl CopilotCommand {
    pub fn run(&self, verbose: bool) {
        // If Copilot CLI is already installed, invoke it directly
        if self.command_exists("copilot") {
            if verbose {
                println!("[VERBOSE] GitHub Copilot CLI is already installed, invoking directly...");
            }
            let status = Command::new("copilot").status().unwrap_or_else(|e| {
                eprintln!("✗ Failed to run copilot: {}", e);
                std::process::exit(1);
            });
            std::process::exit(status.code().unwrap_or(1));
        }

        if verbose {
            println!("[VERBOSE] Installing GitHub Copilot CLI...");
        }

        match self.install(verbose) {
            Ok(_) => {
                println!("✓ GitHub Copilot CLI installed successfully!");
                println!("\nTo get started, run:");
                println!("  copilot");
            }
            Err(e) => {
                eprintln!("✗ Installation failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    fn install(&self, verbose: bool) -> Result<(), String> {
        let method = self.detect_install_method()?;

        if verbose {
            println!("[VERBOSE] Using installation method: {}", method);
        }

        match method.as_str() {
            "brew" => self.install_with_brew(verbose),
            "npm" => self.install_with_npm(verbose),
            "winget" => self.install_with_winget(verbose),
            "script" => self.install_with_script(verbose),
            _ => Err(format!("Unknown installation method: {}", method)),
        }
    }

    fn detect_install_method(&self) -> Result<String, String> {
        // If user specified a method, use it
        if self.method != "auto" {
            return Ok(self.method.clone());
        }

        // Auto-detect based on platform and available tools
        let os = std::env::consts::OS;

        match os {
            "windows" => {
                if self.command_exists("winget") {
                    Ok("winget".to_string())
                } else if self.command_exists("npm") {
                    Ok("npm".to_string())
                } else {
                    Err(
                        "No suitable package manager found. Please install winget or npm."
                            .to_string(),
                    )
                }
            }
            "macos" | "linux" => {
                if self.command_exists("brew") {
                    Ok("brew".to_string())
                } else if self.command_exists("npm") {
                    Ok("npm".to_string())
                } else {
                    Ok("script".to_string())
                }
            }
            _ => Err(format!("Unsupported operating system: {}", os)),
        }
    }

    fn command_exists(&self, cmd: &str) -> bool {
        Command::new(cmd).arg("--version").output().is_ok()
    }

    fn install_with_brew(&self, verbose: bool) -> Result<(), String> {
        println!("📦 Installing with Homebrew...");

        let package = if self.prerelease {
            "copilot-cli@prerelease"
        } else {
            "copilot-cli"
        };

        if verbose {
            println!("[DEBUG] Running: brew install {}", package);
        }

        let output = Command::new("brew")
            .arg("install")
            .arg(package)
            .output()
            .map_err(|e| format!("Failed to run brew: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Homebrew installation failed: {}", stderr))
        }
    }

    fn install_with_npm(&self, verbose: bool) -> Result<(), String> {
        println!("📦 Installing with npm...");

        let package = if let Some(version) = &self.version {
            format!("@github/copilot@{}", version)
        } else if self.prerelease {
            "@github/copilot@prerelease".to_string()
        } else {
            "@github/copilot".to_string()
        };

        if verbose {
            println!("[DEBUG] Running: npm install -g {}", package);
        }

        let output = Command::new("npm")
            .arg("install")
            .arg("-g")
            .arg(&package)
            .output()
            .map_err(|e| format!("Failed to run npm: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("npm installation failed: {}", stderr))
        }
    }

    fn install_with_winget(&self, verbose: bool) -> Result<(), String> {
        println!("📦 Installing with WinGet...");

        let package = if self.prerelease {
            "GitHub.Copilot.Prerelease"
        } else {
            "GitHub.Copilot"
        };

        if verbose {
            println!("[DEBUG] Running: winget install {}", package);
        }

        let output = Command::new("winget")
            .arg("install")
            .arg(package)
            .output()
            .map_err(|e| format!("Failed to run winget: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("WinGet installation failed: {}", stderr))
        }
    }

    fn install_with_script(&self, verbose: bool) -> Result<(), String> {
        println!("📦 Installing with install script...");

        let install_url = "https://gh.io/copilot-install";

        // Build environment variables
        let mut env_vars = Vec::new();

        if let Some(version) = &self.version {
            env_vars.push(format!("VERSION={}", version));
        }

        if let Some(prefix) = &self.prefix {
            env_vars.push(format!("PREFIX={}", prefix));
        }

        // Detect curl or wget
        let (download_cmd, download_args) = if self.command_exists("curl") {
            ("curl", vec!["-fsSL", install_url])
        } else if self.command_exists("wget") {
            ("wget", vec!["-qO-", install_url])
        } else {
            return Err("Neither curl nor wget found. Please install one of them.".to_string());
        };

        if verbose {
            println!(
                "[DEBUG] Download command: {} {}",
                download_cmd,
                download_args.join(" ")
            );
            if !env_vars.is_empty() {
                println!("[DEBUG] Environment: {}", env_vars.join(" "));
            }
        }

        // Download and execute script
        let download_output = Command::new(download_cmd)
            .args(&download_args)
            .output()
            .map_err(|e| format!("Failed to download install script: {}", e))?;

        if !download_output.status.success() {
            return Err("Failed to download install script".to_string());
        }

        let script = String::from_utf8_lossy(&download_output.stdout);

        // Execute script with bash
        let mut bash_cmd = Command::new("bash");

        for env_var in &env_vars {
            let parts: Vec<&str> = env_var.splitn(2, '=').collect();
            if parts.len() == 2 {
                bash_cmd.env(parts[0], parts[1]);
            }
        }

        let output = bash_cmd
            .arg("-c")
            .arg(script.as_ref())
            .output()
            .map_err(|e| format!("Failed to execute install script: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Install script failed: {}", stderr))
        }
    }
}
