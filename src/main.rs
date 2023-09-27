mod mvdup;
use crate::mvdup::cli::{update, Cli, Commands};
use clap::Parser;
use mvdup::cli::find;

fn main() {
    let cli = Cli::parse();

    // Handle default command
    if let None = cli.command {
        return mvdup::cli::mvdup(cli);
    }

    // Handle optional commands
    match cli {
        Commands::Update { path, verify } => update(path, verify),
        Commands::Grep { path, target } => {
            let path = path.unwrap_or(".".to_string());
            find(path, target)
        },
        Commands::Init {path, password} => {

        },
    }
}
