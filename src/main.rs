mod cli;
mod summarize;

use summarize::summarize;

fn main() {
    use std::process::exit;

    if let Err(e) = run() {
        eprintln!("fatal: {}", e);
        exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    use cli::{Cli, Commands};

    let cli = Cli::try_parse()?;

    match cli.command {
        Commands::Summarize { src } => summarize(&src).map_err(|e| e.into()),
    }
}

// fn validate_directories(src: &Path, dst: &Path) -> Result<()> {
//     if !src.is_dir() {
//         bail!("{} is not a directory", src.display());
//     }
//     if !dst.is_dir() {
//         bail!("{} is not a directory", dst.display());
//     }
//     if dst.starts_with(src) {
//         bail!(
//             "<src> must not contain <dst> ({} contains {})",
//             dst.display(),
//             src.display()
//         );
//     }
//     Ok(())
// }
