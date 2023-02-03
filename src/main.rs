mod mvdup;
use crate::mvdup::cli::{update, Cli, Commands};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    println!("{:?}", cli.command);

    match cli.command {
        None => mvdup::cli::mvdup(cli),
        Some(Commands::Update { path, verify }) => update(path, verify),
    }
}
