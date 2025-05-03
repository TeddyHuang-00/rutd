use std::process::ExitCode;

use clap::CommandFactory;
use clap_complete::CompleteEnv;
use rutd_cli::{app, cli::Cli};

fn main() -> ExitCode {
    // Check if we're being called for completion generation
    // Pass the factory function `cli_factory`
    CompleteEnv::with_factory(Cli::command).complete();

    // Call the main application function
    app()
}
