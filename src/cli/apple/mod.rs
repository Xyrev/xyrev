cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        use clap::Args;

        #[derive(Args, Debug)]
        pub struct AppleCommand {
            #[arg(
                short,
                long,
                action = clap::ArgAction::Count,
                help = "Use verbose output (-vv for very verbose)"
            )]
            verbose: u8,

            #[arg(help = "Dotfile name")]
            pub name: String,
        }
    }
}
