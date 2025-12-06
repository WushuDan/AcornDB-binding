use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "acorn", about = "AcornDB Rust CLI (scaffold)")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Verify the CLI wiring without performing work.
    Check,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Check) => println!("acorn-cli stub: command wiring is in place."),
        None => println!("acorn-cli stub: commands will arrive in later phases."),
    }
}
