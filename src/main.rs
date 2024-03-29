mod mvdup;
use crate::mvdup::cli::{update, Cli, Commands};
use clap::Parser;
use mvdup::cli::find;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => mvdup::cli::mvdup(cli),
        Some(Commands::Update { path, verify }) => update(path, verify),
        Some(Commands::Grep { path, target }) => {
            let path = path.unwrap_or(".".to_string());
            find(path, target)
        }
    }
}
