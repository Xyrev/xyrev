use clap::Args;

#[derive(Args, Debug)]
pub struct InitCommand {
    #[arg(help = "Agent name")]
    pub name: String,
}

impl InitCommand {
    pub fn run(&self, verbose: bool) {
        if verbose {
            println!("[VERBOSE] Initializing agent: {}", self.name);
        }

        println!("Initializing agent: {}", self.name);
        // TODO: Implement agent initialization logic
    }
}
