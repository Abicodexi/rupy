use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "rupy-cli")]
#[command(about = "Rupy engine CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run {
        /// Cargo build profile to use (e.g. dev-no-debug-assertions, release-with-debug-assertions)
        #[arg(short, long)]
        profile: Option<String>,
        /// Package name to run (e.g. app)
        #[arg(short, long, default_value = "app")]
        target: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { profile, target } => {
            let build_status = Command::new("cargo")
                .args([
                    "build",
                    "--profile",
                    &profile.clone().unwrap_or_else(|| "dev".to_string()),
                    "-p",
                    &target,
                ])
                .status()
                .expect("Failed to build package");

            if !build_status.success() {
                std::process::exit(1);
            }

            let binary_path = format!(
                "./target/{}/{}",
                profile.clone().unwrap_or_else(|| "debug".to_string()),
                target
            );

            let run_status = Command::new(binary_path)
                .status()
                .expect("Failed to run binary");

            if !run_status.success() {
                std::process::exit(1);
            }
        }
    }
}
