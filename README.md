# Ollama Model Installer

A lightweight, asynchronous command‑line utility written in Rust that downloads a LLM model file (usually from HuggingFace)
saves it into a directory of your choice and installs it in `ollama`.

It streams the response directly to disk and provides a **progress bar** (or a spinner when the size is unknown) so you can monitor the download in real time.

---

## ✨ Features
- **Simple CLI**: `ollama-model-installer <URL> [directory] [--filename <NAME>]`
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
git clone https://github.com/eranif/ollama-model-installer.git
cd ollama-model-installer

# Build the release binary
cargo build --release
```
The compiled binary will be located at `target/release/ollama-model-installer`.

### Install with Cargo (recommended)
```bash
cargo install --git https://github.com/eranif/ollama-model-installer.git ollama-model-installer
```
This command fetches the latest version from the repository and installs the binary into `~/.cargo/bin`.

---

## 🚀 Usage

```bash
# Provide a custom filename for the saved file
ollama-model-installer https://example.com/my-model-q4.gguf -d ./downloads --filename my-model.gguf
```

### Options
| Flag | Description |
|------|-------------|
| `-f, --filename <FILENAME>` | Name of the file to write inside `folder`. If omitted the program will derive a name from the URL |
| `-d`, `--directory` `<NAME>` | Destination folder (will be created if it does not exist, default: `.`) |
| `--model-name <NAME>` | The Model name. This name will appear when you run `ollama ls` |
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
If you plan to embed `ollama-model-installer` as a library, the crate exposes the following public API:
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
