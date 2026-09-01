use crate::bench::BenchmarkResult;
use colored::*;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, ContentArrangement, Table};

pub fn print_banner() {
    let banner = r#"
  ____       _            ____                      _   ____  ____  
 |  _ \ _ __(_)_   _____ / ___| _ __   ___  ___  __| | |  _ \/ ___| 
 | | | | '__| \ \ / / _ \\___ \| '_ \ / _ \/ _ \/ _` | | |_) \___ \ 
 | |_| | |  | |\ V /  __/ ___) | |_) |  __/  __/ (_| | |  _ < ___) |
 |____/|_|  |_| \_/ \___||____/| .__/ \___|\___|\__,_| |_| \_\____/ 
                               |_|                                  
    "#;
    println!("{}", banner.bold().cyan());
    println!(
        "   {}",
        "⚡ Cross-Platform Drive Read/Write Performance Benchmark"
            .italic()
            .bright_white()
    );
    println!();
}

pub fn print_results_table(target_path: &str, size_mb: usize, results: &[BenchmarkResult]) {
    println!("\n{}", "═".repeat(70).bright_black());
    println!(
        " 📊 {} for: {}",
        "BENCHMARK RESULTS".bold().yellow(),
        target_path.bold().bright_green()
    );
    println!(
        "    Sample Size: {} MB | Cache Bypass: Enabled",
        size_mb.to_string().cyan()
    );
    println!("{}\n", "═".repeat(70).bright_black());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Test Profile").fg(Color::Yellow),
            Cell::new("Block Size").fg(Color::Yellow),
            Cell::new("Throughput (MB/s)").fg(Color::Green),
            Cell::new("Throughput (GB/s)").fg(Color::Cyan),
            Cell::new("IOPS").fg(Color::Magenta),
            Cell::new("Avg Latency").fg(Color::Blue),
        ]);

    for r in results {
        let speed_gb_s = r.speed_mb_s / 1024.0;
        let speed_color = if r.speed_mb_s > 1000.0 {
            Color::Green
        } else if r.speed_mb_s > 200.0 {
            Color::Cyan
        } else {
            Color::Yellow
        };

        table.add_row(vec![
            Cell::new(&r.test_name).fg(Color::White),
            Cell::new(format!("{} KB", r.block_size_kb)),
            Cell::new(format!("{:.2} MB/s", r.speed_mb_s)).fg(speed_color),
            Cell::new(format!("{:.2} GB/s", speed_gb_s)),
            Cell::new(format!("{:.0} IOPS", r.iops)).fg(Color::Magenta),
            Cell::new(format!("{:.2} ms", r.avg_latency_ms)).fg(Color::Blue),
        ]);
    }

    println!("{table}");
    println!(
        "\n{} {}",
        "✨ All tests completed successfully.".bold().green(),
        "Temporary benchmark files cleaned up."
            .dimmed()
            .italic()
    );
}
