# ⚡ DriveSpeed RS (`drivespeedrs`)

A blazing-fast, cross-platform CLI tool written in Rust to benchmark the true physical read and write speeds of internal SSDs/HDDs and external USB drives with hardware-accurate cache bypassing and interactive terminal UI.

```text
  ____       _            ____                      _   ____  ____  
 |  _ \ _ __(_)_   _____ / ___| _ __   ___  ___  __| | |  _ \/ ___| 
 | | | | '__| \ \ / / _ \\___ \| '_ \ / _ \/ _ \/ _` | | |_) \___ \ 
 | |_| | |  | |\ V /  __/ ___) | |_) |  __/  __/ (_| | |  _ < ___) |
 |____/|_|  |_| \_/ \___||____/| .__/ \___|\___|\__,_| |_| \_\____/ 
                               |_|                                  
```

---

## 🚀 One-Line Installation via `curl`

Install the pre-compiled binary instantly:

```bash
curl -fsSL https://raw.githubusercontent.com/khokharsnehil45/drivespeedrs/main/install.sh | bash
```

Or install via `cargo`:

```bash
cargo install --path .
# or once published to crates.io:
cargo install drivespeedrs
```

---

## 🎮 Usage

Simply run:

```bash
drivespeedrs
```

This launches the interactive drive selector.

### Options & Flags

```bash
# List all detected storage drives with filesystem and free space
drivespeedrs --list

# Benchmark a specific path / drive with custom sample size
drivespeedrs --path /run/media/kevin/ExtremeSSD --size-mb 1024

# Benchmark your current directory with a 256MB sample
drivespeedrs --size-mb 256
```

---

## 🛠️ Features

* **Cache Invalidation (`POSIX_FADV_DONTNEED`):** Drops RAM page cache to measure actual physical hardware read speeds.
* **Anti-Cheat Random Byte Buffers:** Prevents hardware compression controllers from spoofing transfer rates with zeroes.
* **Sequential & Random 4K IOPS:** Measures both large-file streaming throughput and database/OS random I/O latency.
* **Auto-Cleanup:** Safely purges temporary benchmark test files on completion.

---

## 📄 License

MIT License.
