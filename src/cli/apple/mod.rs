cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        use clap::Args;

        #[derive(Args, Debug)]
        pub struct AppleCommand {
            #[arg(
                short,
                long,
                action = clap::ArgAction::SetTrue,
                help = "Use verbose output"
            )]
            verbose: bool,

            #[arg(help = "Dotfile name")]
            pub name: String,
        }
    }
}
