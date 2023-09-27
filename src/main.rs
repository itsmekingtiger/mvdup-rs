mod mvdup;

use crate::mvdup::cli::{update, Cli, Commands, init, open_test, mvdup};
use clap::Parser;
use mvdup::cli::find;

fn main() {
    let cli = Cli::parse();


    match cli.command {
        Commands::Add { source, take } => mvdup(source, take),
        Commands::Update { path, verify } => update(path, verify),
        Commands::Grep { path, target } => {
            let path = path.unwrap_or(".".to_string());
            find(path, target)
        }
        Commands::Init { path, encrypt } => init(path, encrypt),
        Commands::OpenTest { path } => open_test(path),
    }
}
