use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use flash_protocol::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::timeout;

mod serial;
mod commands;

use serial::SerialConnection;
use commands::FlashCommands;

#[derive(Parser)]
#[command(name = "flash-programmer")]
#[command(about = "STM32G4 Flash Programmer Tool")]
#[command(version = "0.1.0")]
struct Cli {
    /// Serial port to connect to
    #[arg(short, long, default_value = "/dev/ttyACM0")]
    port: String,

    /// Baud rate (ignored for USB CDC, but kept for compatibility)
    #[arg(short, long, default_value = "115200")]
    baud: u32,

    /// Connection timeout in seconds
    #[arg(short, long, default_value = "10")]
    timeout: u64,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get flash information
    Info,
    /// Erase flash sectors
    Erase {
        /// Start address (hex)
        #[arg(short, long, value_parser = parse_hex)]
        address: u32,
        /// Size to erase in bytes (hex)
        #[arg(short, long, value_parser = parse_hex)]
        size: u32,
    },
    /// Write file to flash
    Write {
        /// Input file path
        #[arg(short, long)]
        file: PathBuf,
        /// Start address (hex)
        #[arg(short, long, value_parser = parse_hex, default_value = "0")]
        address: u32,
        /// Erase before writing
        #[arg(short, long)]
        erase: bool,
        /// Verify after writing
        #[arg(short, long)]
        verify: bool,
    },
    /// Read flash to file
    Read {
        /// Output file path
        #[arg(short, long)]
        file: PathBuf,
        /// Start address (hex)
        #[arg(short, long, value_parser = parse_hex, default_value = "0")]
        address: u32,
        /// Size to read in bytes (hex)
        #[arg(short, long, value_parser = parse_hex)]
        size: u32,
    },
    /// Verify file against flash
    Verify {
        /// File to verify
        #[arg(short, long)]
        file: PathBuf,
        /// Start address (hex)
        #[arg(short, long, value_parser = parse_hex, default_value = "0")]
        address: u32,
    },
}

fn parse_hex(s: &str) -> Result<u32, std::num::ParseIntError> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u32::from_str_radix(&s[2..], 16)
    } else {
        s.parse()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("STM32G4 Flash Programmer Tool v0.1.0");
    println!("Connecting to {}...", cli.port);

    // Connect to device
    let mut connection = timeout(
        Duration::from_secs(cli.timeout),
        SerialConnection::new(&cli.port, cli.baud)
    )
    .await
    .context("Connection timeout")?
    .context("Failed to connect to device")?;

    println!("Connected successfully!");

    // Create flash commands handler
    let mut flash_commands = FlashCommands::new(&mut connection);

    // Execute command
    match cli.command {
        Commands::Info => {
            println!("Getting flash information...");
            let info = flash_commands.get_info().await?;
            println!("Flash Information:");
            println!("  JEDEC ID: 0x{:06X}", info.jedec_id);
            println!("  Total Size: {} MB ({} bytes)", 
                     info.total_size / (1024 * 1024), info.total_size);
            println!("  Page Size: {} bytes", info.page_size);
            println!("  Sector Size: {} KB ({} bytes)", 
                     info.sector_size / 1024, info.sector_size);
        }

        Commands::Erase { address, size } => {
            println!("Erasing flash at 0x{:08X}, size: {} bytes...", address, size);
            
            let pb = ProgressBar::new(1);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap());
            pb.set_message("Erasing...");

            flash_commands.erase(address, size).await?;
            
            pb.finish_with_message("Erase completed!");
            println!("Flash erased successfully!");
        }

        Commands::Write { file, address, erase, verify } => {
            println!("Reading file: {:?}", file);
            let data = fs::read(&file).await
                .with_context(|| format!("Failed to read file: {:?}", file))?;
            
            println!("File size: {} bytes", data.len());
            
            if erase {
                println!("Erasing flash at 0x{:08X}, size: {} bytes...", address, data.len());
                flash_commands.erase(address, data.len() as u32).await?;
                println!("Erase completed!");
            }

            println!("Writing to flash at 0x{:08X}...", address);
            let pb = ProgressBar::new(data.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap());

            if verify {
                // Use write with automatic verification for maximum data integrity
                flash_commands.write_and_verify_with_progress(address, &data, &pb).await?;
                pb.finish_with_message("Write and verification completed!");
                println!("✅ Data written and verified successfully!");
            } else {
                // Use high-speed write only
                flash_commands.write_with_progress(address, &data, &pb).await?;
                pb.finish_with_message("Write completed!");
                println!("✅ Data written successfully!");
                println!("⚠️  Warning: Data was not verified. Use --verify flag to ensure data integrity.");
            }
        }

        Commands::Read { file, address, size } => {
            println!("Reading {} bytes from flash at 0x{:08X}...", size, address);
            
            let pb = ProgressBar::new(size as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap());

            let data = flash_commands.read_with_progress(address, size, &pb).await?;
            
            pb.finish_with_message("Read completed!");

            println!("Writing to file: {:?}", file);
            fs::write(&file, &data).await
                .with_context(|| format!("Failed to write file: {:?}", file))?;
            
            println!("File saved successfully!");
        }

        Commands::Verify { file, address } => {
            println!("Reading file: {:?}", file);
            let data = fs::read(&file).await
                .with_context(|| format!("Failed to read file: {:?}", file))?;
            
            println!("Verifying {} bytes at 0x{:08X}...", data.len(), address);
            
            let pb = ProgressBar::new(data.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.yellow/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap());

            flash_commands.verify_with_progress(address, &data, &pb).await?;
            
            pb.finish_with_message("Verification completed!");
            println!("Verification successful!");
        }
    }

    println!("Operation completed successfully!");
    Ok(())
}
