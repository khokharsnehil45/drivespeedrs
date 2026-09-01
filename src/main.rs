mod bench;
mod disks;
mod report;

use anyhow::{bail, Result};
use bench::BenchmarkRunner;
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Select};
use disks::detect_disks;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "drivespeed",
    about = "⚡ Fast, beautiful disk read/write speed benchmark utility written in Rust"
)]
struct Cli {
    /// Target path/directory to test (e.g. /mnt/usbdrive or /home/kevin). If not provided, an interactive menu will appear.
    #[arg(short, long)]
    path: Option<PathBuf>,

    /// Size of the benchmark test in Megabytes (default: 512 MB).
    #[arg(short, long, default_value_t = 512)]
    size_mb: usize,

    /// List detected drives and exit.
    #[arg(short, long)]
    list: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    report::print_banner();

    let detected_disks = detect_disks();

    if cli.list {
        println!("{}", "💾 Detected Storage Drives:".bold().yellow());
        for (i, d) in detected_disks.iter().enumerate() {
            let type_str = if d.is_removable {
                "USB / Removable".magenta()
            } else {
                "Internal Drive".cyan()
            };
            println!(
                "  [{}] {} ({})",
                i + 1,
                d.mount_point.bold().green(),
                type_str
            );
            println!(
                "      Name: {} | FS: {} | Free: {:.1} GB / {:.1} GB",
                d.name, d.file_system, d.available_space_gb, d.total_space_gb
            );
        }
        return Ok(());
    }

    let target_dir: PathBuf = match cli.path {
        Some(p) => {
            if !p.exists() || !p.is_dir() {
                bail!("The specified path '{:?}' does not exist or is not a directory!", p);
            }
            p
        }
        None => {
            if detected_disks.is_empty() {
                println!("{}", "No drives auto-detected. Using current directory.".yellow());
                std::env::current_dir()?
            } else {
                let mut options: Vec<String> = detected_disks
                    .iter()
                    .map(|d| {
                        let tag = if d.is_removable { "[USB/Ext]" } else { "[Internal]" };
                        format!(
                            "{:<10} {:<25} ({:.1} GB free / {:.1} GB) [{}]",
                            tag, d.mount_point, d.available_space_gb, d.total_space_gb, d.file_system
                        )
                    })
                    .collect();
                options.push("📁 Current Working Directory (.)".to_string());

                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select Drive / Partition to benchmark")
                    .default(0)
                    .items(&options)
                    .interact()?;

                if selection < detected_disks.len() {
                    PathBuf::from(&detected_disks[selection].mount_point)
                } else {
                    std::env::current_dir()?
                }
            }
        }
    };

    println!(
        "\n{} {}",
        "🎯 Target Directory:".bold().yellow(),
        target_dir.display().to_string().bold().green()
    );
    println!(
        "{} {} MB",
        "📦 Benchmark Sample Size:".bold().yellow(),
        cli.size_mb.to_string().cyan()
    );

    let runner = BenchmarkRunner::new(&target_dir, cli.size_mb);
    let results = runner.run_all_tests()?;

    report::print_results_table(&target_dir.to_string_lossy(), cli.size_mb, &results);

    Ok(())
}
