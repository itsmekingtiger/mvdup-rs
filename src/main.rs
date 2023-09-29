use clap::Parser;

use mvdup::cli::handle_find;

use crate::mvdup::cli::{Cli, Commands, handle_add, handle_init, handle_open_test, handle_update};

mod mvdup;

fn main() {
    let cli = Cli::parse();


    match cli.command {
        Commands::Add { source, destination, take } => handle_add(source, destination, take),
        Commands::Update { path, verify } => handle_update(path, verify),
        Commands::Grep { path, target } => {
            let path = path.unwrap_or(".".to_string());
            handle_find(path, target)
        }
        Commands::Init { path, encrypt } => handle_init(path, encrypt),
        Commands::OpenTest { path } => handle_open_test(path),
    }
}
