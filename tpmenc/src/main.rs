mod crypto;
mod tpm;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tpmenc")]
#[command(about = "TPM-sealed file encryption — machine-bound, no keys stored")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Seal (encrypt) a file with TPM. Output: <file>.sealed
    Seal {
        input: PathBuf,
        /// Delete the original after sealing
        #[arg(short, long)]
        shred: bool,
    },
    /// Unseal (decrypt) a .sealed file. Output: original filename
    Unseal { input: PathBuf },
    /// Preview (decrypt and display) a .sealed file without writing to disk
    Preview { input: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Seal { input, shred } => {
            let out = tpm::seal_file(&input)?;
            println!("sealed -> {}", out.display());
            if shred {
                std::fs::remove_file(&input)?;
                println!("removed {}", input.display());
            }
        }
        Commands::Unseal { input } => {
            let out = tpm::unseal_file(&input)?;
            println!("unsealed -> {}", out.display());
        }
        Commands::Preview { input } => {
            let plaintext = tpm::preview_file(&input)?;
            let stdout = std::io::stdout();
            stdout.lock().write_all(&plaintext)?;
        }
    }
    Ok(())
}
