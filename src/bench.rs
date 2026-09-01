use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub test_name: String,
    pub block_size_kb: usize,
    pub total_bytes: usize,
    pub speed_mb_s: f64,
    pub iops: f64,
    pub avg_latency_ms: f64,
}

pub struct BenchmarkRunner {
    pub target_dir: PathBuf,
    pub test_file_path: PathBuf,
    pub total_size_bytes: usize,
}

impl BenchmarkRunner {
    pub fn new(target_dir: &Path, size_mb: usize) -> Self {
        let test_file_path = target_dir.join(".drivespeed_test_tmp.bin");
        Self {
            target_dir: target_dir.to_path_buf(),
            test_file_path,
            total_size_bytes: size_mb * 1024 * 1024,
        }
    }

    pub fn run_all_tests(&self) -> Result<Vec<BenchmarkResult>> {
        let mut results = Vec::new();

        println!(
            "\n{}",
            "🚀 Starting Benchmark Sequence (POSIX_FADV_DONTNEED Cache Invalidation)..."
                .bold()
                .cyan()
        );

        // 1. Sequential Write (1 MB blocks)
        let seq_write = self.bench_seq_write(1024 * 1024)?;
        results.push(seq_write);

        // 2. Sequential Read (1 MB blocks)
        let seq_read = self.bench_seq_read(1024 * 1024)?;
        results.push(seq_read);

        // 3. Random 4K Write
        let rand_write = self.bench_rand_write(4 * 1024, (self.total_size_bytes / 8).min(64 * 1024 * 1024))?;
        results.push(rand_write);

        // 4. Random 4K Read
        let rand_read = self.bench_rand_read(4 * 1024, (self.total_size_bytes / 8).min(64 * 1024 * 1024))?;
        results.push(rand_read);

        // Cleanup
        self.cleanup();

        Ok(results)
    }

    fn create_progress_bar(msg: &'static str, total_bytes: u64) -> ProgressBar {
        let pb = ProgressBar::new(total_bytes);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  {prefix:.bold.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, ETA: {eta})")
                .unwrap()
                .progress_chars("█▉▊▋▌▍▎▏ "),
        );
        pb.set_prefix(msg);
        pb
    }

    fn evict_cache_and_sync(file: &File, len: u64) {
        let fd = file.as_raw_fd();
        unsafe {
            // Drop pages from OS cache so subsequent reads hit the physical disk
            libc::posix_fadvise(fd, 0, len as libc::off_t, libc::POSIX_FADV_DONTNEED);
        }
    }

    fn bench_seq_write(&self, block_size: usize) -> Result<BenchmarkResult> {
        let pb = Self::create_progress_bar("Sequential Write (1MB)", self.total_size_bytes as u64);

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.test_file_path)
            .with_context(|| format!("Failed to create test file at {:?}", self.test_file_path))?;

        let mut rng = rand::rng();
        let mut buffer = vec![0u8; block_size];
        rng.fill(&mut buffer[..]);

        let num_blocks = self.total_size_bytes / block_size;
        let start = Instant::now();

        for _ in 0..num_blocks {
            file.write_all(&buffer)?;
            pb.inc(block_size as u64);
        }
        file.sync_all()?;

        let elapsed = start.elapsed();
        pb.finish_with_message("Done");

        // Evict write pages from Linux RAM cache
        Self::evict_cache_and_sync(&file, self.total_size_bytes as u64);

        let duration_secs = elapsed.as_secs_f64();
        let speed_mb_s = (self.total_size_bytes as f64 / (1024.0 * 1024.0)) / duration_secs;
        let iops = num_blocks as f64 / duration_secs;
        let avg_latency_ms = (duration_secs * 1000.0) / num_blocks as f64;

        Ok(BenchmarkResult {
            test_name: "Sequential Write (1M)".to_string(),
            block_size_kb: block_size / 1024,
            total_bytes: self.total_size_bytes,
            speed_mb_s,
            iops,
            avg_latency_ms,
        })
    }

    fn bench_seq_read(&self, block_size: usize) -> Result<BenchmarkResult> {
        let pb = Self::create_progress_bar("Sequential Read  (1MB)", self.total_size_bytes as u64);

        let mut file = File::open(&self.test_file_path)
            .with_context(|| format!("Failed to open test file at {:?}", self.test_file_path))?;

        // Evict any existing OS cache before reading
        Self::evict_cache_and_sync(&file, self.total_size_bytes as u64);

        let mut buffer = vec![0u8; block_size];
        let num_blocks = self.total_size_bytes / block_size;
        let start = Instant::now();

        for _ in 0..num_blocks {
            file.read_exact(&mut buffer)?;
            pb.inc(block_size as u64);
        }

        let elapsed = start.elapsed();
        pb.finish_with_message("Done");

        let duration_secs = elapsed.as_secs_f64();
        let speed_mb_s = (self.total_size_bytes as f64 / (1024.0 * 1024.0)) / duration_secs;
        let iops = num_blocks as f64 / duration_secs;
        let avg_latency_ms = (duration_secs * 1000.0) / num_blocks as f64;

        Ok(BenchmarkResult {
            test_name: "Sequential Read  (1M)".to_string(),
            block_size_kb: block_size / 1024,
            total_bytes: self.total_size_bytes,
            speed_mb_s,
            iops,
            avg_latency_ms,
        })
    }

    fn bench_rand_write(&self, block_size: usize, test_bytes: usize) -> Result<BenchmarkResult> {
        let pb = Self::create_progress_bar("Random 4K Write       ", test_bytes as u64);

        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.test_file_path)?;

        let mut rng = rand::rng();
        let mut buffer = vec![0u8; block_size];
        rng.fill(&mut buffer[..]);

        let max_offset_blocks = (self.total_size_bytes - block_size) / block_size;
        let num_ops = test_bytes / block_size;

        let start = Instant::now();
        for _ in 0..num_ops {
            let block_idx = rng.random_range(0..=max_offset_blocks);
            file.seek(SeekFrom::Start((block_idx * block_size) as u64))?;
            file.write_all(&buffer)?;
            pb.inc(block_size as u64);
        }
        file.sync_all()?;

        let elapsed = start.elapsed();
        pb.finish_with_message("Done");

        // Evict written cache
        Self::evict_cache_and_sync(&file, self.total_size_bytes as u64);

        let duration_secs = elapsed.as_secs_f64();
        let speed_mb_s = (test_bytes as f64 / (1024.0 * 1024.0)) / duration_secs;
        let iops = num_ops as f64 / duration_secs;
        let avg_latency_ms = (duration_secs * 1000.0) / num_ops as f64;

        Ok(BenchmarkResult {
            test_name: "Random 4K Write       ".to_string(),
            block_size_kb: block_size / 1024,
            total_bytes: test_bytes,
            speed_mb_s,
            iops,
            avg_latency_ms,
        })
    }

    fn bench_rand_read(&self, block_size: usize, test_bytes: usize) -> Result<BenchmarkResult> {
        let pb = Self::create_progress_bar("Random 4K Read        ", test_bytes as u64);

        let mut file = File::open(&self.test_file_path)?;

        // Invalidate cache before random reads
        Self::evict_cache_and_sync(&file, self.total_size_bytes as u64);

        let mut rng = rand::rng();
        let mut buffer = vec![0u8; block_size];
        let max_offset_blocks = (self.total_size_bytes - block_size) / block_size;
        let num_ops = test_bytes / block_size;

        let start = Instant::now();
        for _ in 0..num_ops {
            let block_idx = rng.random_range(0..=max_offset_blocks);
            file.seek(SeekFrom::Start((block_idx * block_size) as u64))?;
            file.read_exact(&mut buffer)?;
            pb.inc(block_size as u64);
        }

        let elapsed = start.elapsed();
        pb.finish_with_message("Done");

        let duration_secs = elapsed.as_secs_f64();
        let speed_mb_s = (test_bytes as f64 / (1024.0 * 1024.0)) / duration_secs;
        let iops = num_ops as f64 / duration_secs;
        let avg_latency_ms = (duration_secs * 1000.0) / num_ops as f64;

        Ok(BenchmarkResult {
            test_name: "Random 4K Read         ".to_string(),
            block_size_kb: block_size / 1024,
            total_bytes: test_bytes,
            speed_mb_s,
            iops,
            avg_latency_ms,
        })
    }

    pub fn cleanup(&self) {
        if self.test_file_path.exists() {
            let _ = std::fs::remove_file(&self.test_file_path);
        }
    }
}

impl Drop for BenchmarkRunner {
    fn drop(&mut self) {
        self.cleanup();
    }
}
