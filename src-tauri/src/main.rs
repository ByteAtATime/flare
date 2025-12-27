// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use flare_lib::dmenu::{Cli, Commands, DmenuSession};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Dmenu {
            case_insensitive,
            prompt,
            ..
        }) => {
            // dmenu mode: read items from stdin and launch minimal UI
            match DmenuSession::from_stdin(case_insensitive, prompt) {
                Ok(session) => {
                    flare_lib::run_dmenu(session);
                }
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            // Normal launcher mode
            flare_lib::run();
        }
    }
}
