mod transpiler;

use clap::{Parser, Subcommand};
use anyhow::Result;
use std::process::Command;

#[derive(Parser)]
#[command(name = "build-tool")]
#[command(about = "TOML-to-Makefile transpiler for multi-language projects")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, default_value = "build.toml")]
    config: String,

    #[arg(short, long, global = true, default_value = "languages.toml")]
    registry: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Transpile build.toml to Makefile
    Transpile {
        #[arg(short, long, default_value = "Makefile")]
        output: String,
    },

    /// Transpile and run make
    Build {
        #[arg(default_value = "all")]
        target: String,
    },

    /// Clean and build a single target
    Make {
        target: String,
    },

    /// Run make clean
    Clean,

    /// Show available targets from build.toml
    Info,

    /// Add a new target to build.toml (interactive)
    Add {
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Transpile { output } => {
            eprintln!("Transpiling {} → {}...", cli.config, output);
            transpiler::transpile(&cli.config, &cli.registry, &output)?;
            println!("✓ Generated {}", output);
        }

        Commands::Build { target } => {
            eprintln!("Transpiling {} → Makefile...", cli.config);
            transpiler::transpile(&cli.config, &cli.registry, "Makefile")?;

            eprintln!("Building target: {}", target);
            let status = Command::new("make")
                .arg(&target)
                .status()?;

            std::process::exit(status.code().unwrap_or(1));
        }

        Commands::Clean => {
            eprintln!("Transpiling {} → Makefile...", cli.config);
            transpiler::transpile(&cli.config, &cli.registry, "Makefile")?;

            eprintln!("Cleaning...");
            let status = Command::new("make")
                .arg("clean")
                .status()?;

            std::process::exit(status.code().unwrap_or(1));
        }

        Commands::Info => {
            let config = transpiler::load_build_config(&cli.config)?;

            println!("Targets in {}:", cli.config);
            for (name, target) in &config.targets {
                println!("  {} ({})", name, target.target_type);
                if let Some(source) = &target.source {
                    println!("    source: {}", source);
                }
                println!("    output: {}", target.output);
                if let Some(flags) = &target.flags {
                    println!("    flags: {}", flags);
                }
                if let Some(deps) = &target.depends_on {
                    println!("    depends_on: {}", deps.join(", "));
                }
                println!();
            }
        }

        Commands::Make { target } => {
            let config = transpiler::load_build_config(&cli.config)?;

            // Validate target exists
            if !config.targets.contains_key(&target) {
                anyhow::bail!("Target '{}' not found in {}", target, cli.config);
            }

            let target_info = &config.targets[&target];

            eprintln!("Transpiling {} → Makefile...", cli.config);
            transpiler::transpile(&cli.config, &cli.registry, "Makefile")?;

            // Clean the target's output
            eprintln!("Cleaning target: {}", target);
            if std::path::Path::new(&target_info.output).exists() {
                std::fs::remove_file(&target_info.output)
                    .map_err(|e| anyhow::anyhow!("Failed to remove {}: {}", target_info.output, e))?;
                eprintln!("✓ Removed {}", target_info.output);
            }

            // Build the target
            eprintln!("Building target: {}", target);
            let status = Command::new("make")
                .arg(&target)
                .status()?;

            std::process::exit(status.code().unwrap_or(1));
        }

        Commands::Add { name } => {
            println!("Adding target '{}' to {}...", name, cli.config);
            println!("This is a stub. Edit {} manually for now.", cli.config);
        }
    }

    Ok(())
}
