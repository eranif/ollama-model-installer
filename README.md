# Ollama Puller

A lightweight, asynchronous command‑line utility written in Rust that downloads a file from a given URL and saves it into a directory of your choice. It streams the response directly to disk and provides a **progress bar** (or a spinner when the size is unknown) so you can monitor the download in real time.

---

## ✨ Features
- **Simple CLI**: `ollama-puller <URL> [directory] [--filename <NAME>]`
- **Automatic directory creation** if the target folder does not exist.
- **Smart filename handling** – derives a sensible name from the URL when none is provided.
- **Live progress indicator** using the `indicatif` crate (percentage, elapsed time, ETA, etc.).
- **Fully asynchronous** with `tokio` and `reqwest` streaming for optimal performance.
- **Zero‑dependency binary** – the built binary contains everything needed to run.

---

## 📋 Prerequisites
- **Rust toolchain** (stable 1.70+). Install via [rustup](https://rustup.rs/).
- An active internet connection for downloading files.

---

## 🛠️ Installation
You can either **build from source** or **install the binary via Cargo**.

### Build from source
```bash
# Clone the repository
git clone <repo-url>
cd ollama-puller

# Build the release binary
cargo build --release
```
The compiled binary will be located at `target/release/ollama-puller`.

### Install with Cargo (recommended)
```bash
cargo install --git <repo-url> ollama-puller
```
This command fetches the latest version from the repository and installs the binary into `~/.cargo/bin`.

---

## 🚀 Usage
```bash
# Basic download – saved in the current directory
ollama-puller https://example.com/file.txt

# Specify a target directory (created automatically if missing)
ollama-puller https://example.com/file.txt ./downloads

# Provide a custom filename for the saved file
ollama-puller https://example.com/file.txt ./downloads --filename my_file.txt
```

### Options
| Flag | Description |
|------|-------------|
| `--filename <NAME>` | Override the filename derived from the URL. |
| `-h`, `--help` | Show help message and exit. |
| `-V`, `--version` | Print version information. |

When the server supplies a `Content‑Length` header, a deterministic progress bar is displayed. If the length is unknown, a spinner is shown instead.

---

## 📊 Example Output
```
[00:00:02] [#########################---------------] 12.3 MB/25.0 MB (12s)
Downloaded 'https://example.com/file.txt' → './downloads/file.txt'
```

---

## 📦 Crate Features (for developers)
If you plan to embed `ollama-puller` as a library, the crate exposes the following public API:
- `download(url: &str, target_dir: &Path, filename: Option<&str>) -> Result<PathBuf>` – performs the download and returns the path to the saved file.
- Customizable progress callbacks via the `indicatif` progress bar.

---

## 📄 License
This project is licensed under the **MIT License** – see the `LICENSE` file for details.

---

## 🙏 Acknowledgments
- **`reqwest`** – async HTTP client.
- **`tokio`** – asynchronous runtime.
- **`indicatif`** – beautiful progress bars.
- **`clap`** – ergonomic command‑line argument parsing.
- All contributors and the open‑source community.
