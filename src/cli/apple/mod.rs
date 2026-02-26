cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        use clap::Args;

        #[derive(Args, Debug)]
        pub struct AppleCommand {
            #[arg(help = "Dotfile name")]
            pub name: String,
        }
    }
}
