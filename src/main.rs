mod build_logs;
mod cli;
mod config;
mod deno_logs;
mod installer;
mod kernel_logs;
mod log_generator;
mod messages;
mod stages;
mod ui;

use clap::Parser;
use cli::Cli;
use colored::*;
use installer::Installer;
use rand::seq::SliceRandom;
use std::io;

fn main() {
    let cli = Cli::parse();
    let compact = cli.compact;

    if let Err(e) = run_installer(cli) {
        handle_error(e, compact);
    }
}

fn run_installer(cli: Cli) -> io::Result<()> {
    let mut stages = cli.get_stages();
    let compact = cli.compact;

    let mut rng = rand::thread_rng();
    stages.shuffle(&mut rng);

    let mut installer = Installer::new(stages, compact);
    installer.run()
}

fn handle_error(e: io::Error, compact: bool) {
    if e.kind() == io::ErrorKind::Interrupted {
        let separator = if compact {
            "═".repeat(38)
        } else {
            "═".repeat(39)
        };
        println!("\n\n{}", separator.bright_cyan());
        println!("{}", "Installation cancelled by user.".bright_white());
        println!(
            "{}",
            "Thank you for using Universal System Installer!".bright_white()
        );
        println!("{}", separator.bright_cyan());
    } else {
        eprintln!("\n{} {:?}", "Error:".bright_red(), e);
        std::process::exit(1);
    }
}
