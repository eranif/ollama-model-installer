//! A tiny commandâ€‘line program that downloads a URL and stores the
//! response body into a file inside a given folder.
//!
//! Example usage:
//!     cargo run -- https://www.rust-lang.org/ ./downloads rust_homepage.html
//!
//! The above will fetch the Rust homepage and write it to
//! `./downloads/rust_homepage.html` (creating the folder if necessary).

use ansi_term::Colour::{Green, Red, Yellow};
use std::process::Command;
use std::time::Duration;
use std::{
    fs::{self, OpenOptions},
    io::{self, stderr, stdout, Write},
    path::{Path, PathBuf},
};

use clap::Parser;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::env;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Simple downloader
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The URL to download (must be a valid http/https URL)
    url: String,

    /// Destination folder (will be created if it does not exist)
    #[arg(short, long, default_value = ".")]
    directory: PathBuf,

    #[arg(short, long)]
    model_name: String,

    /// Name of the file to write inside `folder`. If omitted the
    /// program will derive a name from the URL.
    #[arg(short, long)]
    filename: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // -------------------------------------------------------------
    // Parse CLI arguments
    // -------------------------------------------------------------
    let args = Args::parse();

    // -------------------------------------------------------------
    // Resolve the final output path
    // -------------------------------------------------------------
    // Ensure the output directory exists (creates recursively)
    fs::create_dir_all(&args.directory)?;
    let file_name = match args.filename {
        Some(name) => name,
        None => derive_filename_from_url(&args.url)?,
    };
    let out_path = args.directory.join(file_name);

    // -------------------------------------------------------------
    // Perform the HTTP GET request (streaming)
    // -------------------------------------------------------------
    let response = reqwest::get(&args.url).await?;
    if !response.status().is_success() {
        return Err(format!("Failed to download: HTTP {}", response.status()).into());
    }

    // -------------------------------------------------------------
    // Set up the progress bar (if Contentâ€‘Length is known)
    // -------------------------------------------------------------
    let total_size = response.content_length();
    let pb = match total_size {
        Some(len) => ProgressBar::new(len),
        None => ProgressBar::new_spinner(),
    };
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    if total_size.is_none() {
        pb.enable_steady_tick(Duration::from_millis(100));
    }

    // -------------------------------------------------------------
    // Write the response body to disk while streaming
    // -------------------------------------------------------------
    let mut file = File::create(&out_path).await?;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let data = chunk?;
        file.write_all(&data).await?;
        pb.inc(data.len() as u64);
    }
    file.flush().await?;
    let bin_file = out_path.display().to_string();
    pb.finish_with_message("download complete");
    info(
        stdout(),
        &format!("Downloaded '{}' => '{}'", args.url, bin_file),
    )?;

    // Create a ModelFile.
    let model_file = Path::new(&args.directory).join("ModelFile");
    write_to_file(&model_file, format!("FROM {}", bin_file))?;
    info(
        stdout(),
        &format!("Successfully create file '{}'", model_file.display()),
    )?;

    // Install the model in Ollama server.
    let Some(ollama_exec) = which("ollama") else {
        warning(stderr(), "Could not find 'ollama' executable in PATH")?;
        return Ok(());
    };
    info(stdout(), &format!("Installing file {}...", bin_file))?;

    match Command::new(ollama_exec.display().to_string())
        .arg("-f")
        .arg(model_file.display().to_string())
        .output()
    {
        Ok(o) if o.status.success() => info(
            stdout(),
            String::from_utf8_lossy(&o.stdout).to_string().as_str(),
        )?,
        Ok(o) => error(
            stderr(),
            format!(
                "error (code {:?}): {}",
                o.status.code(),
                String::from_utf8_lossy(&o.stderr)
            )
            .as_str(),
        )?,
        Err(e) => error(stderr(), format!("failed to spawn: {}", e).as_str())?,
    }
    Ok(())
}

// -------------------------------------------------------------
// Helper utilities
// -------------------------------------------------------------

/// Derives a filename from a URL.
fn derive_filename_from_url(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let parsed = url::Url::parse(url)?;
    let segments: Vec<&str> = parsed.path_segments().map_or(Vec::new(), |s| s.collect());
    if let Some(last) = segments.iter().rev().find(|s| !s.is_empty()) {
        if last.contains('.') {
            return Ok(last.to_string());
        }
    }
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    Ok(format!("download_{}.bin", timestamp))
}

/// Writes `content` to `target_path`.
///
/// * `target_path` - The full path (including file name) where the data should be stored.
/// * `content` - The data to write - anything that can be turned into a byte slice (`&[u8]`).
///
/// Returns the absolute path of the written file on success, or an `io::Error` on failure.
///
/// # Example
///
/// ```no_compile
/// # fn main() -> std::io::Result<()> {
/// let path = "output/example.txt";
/// write_to_file(path, "Hello, world!")?;
/// # Ok(())
/// # }
/// ```
pub fn write_to_file<P, C>(target_path: P, content: C) -> io::Result<PathBuf>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let path = target_path.as_ref();

    // -----------------------------------------------------------------
    // 1️⃣ Ensure the directory hierarchy exists
    // -----------------------------------------------------------------
    if let Some(parent) = path.parent() {
        // `create_dir_all` is idempotent – it does nothing if the directory already exists.
        fs::create_dir_all(parent)?;
    }

    // -----------------------------------------------------------------
    // 2️⃣ Open the file for writing (create it if missing, truncate otherwise)
    // -----------------------------------------------------------------
    let mut file = OpenOptions::new()
        .write(true)
        .create(true) // create if it does not exist
        .truncate(true) // replace any existing content
        .open(path)?;

    // -----------------------------------------------------------------
    // 3️⃣ Write the data
    // -----------------------------------------------------------------
    file.write_all(content.as_ref())?;

    // -----------------------------------------------------------------
    // 4️⃣ Flush to ensure everything is persisted on disk
    // -----------------------------------------------------------------
    file.flush()?;

    // Return the canonical (absolute) path for convenience
    path.canonicalize()
}

#[cfg(target_os = "windows")]
const PATH_SEP: &str = ";";

#[cfg(not(target_os = "windows"))]
const PATH_SEP: &str = ":";

/// Searches for the given command in the directories specified by the `PATH`
/// environment variable and returns its absolute path if found.
///
/// # Arguments
///
/// * `cmd` – The name of the command to look for.
///
/// # Returns
///
/// `Some(PathBuf)` containing the full path to the command if it exists in one of
/// the `PATH` directories, otherwise `None`.
///
/// # Behavior
///
/// The function reads the `PATH` variable, splits it on the platform‑specific
/// separator (`PATH_SEP`), and iterates over each directory. For each directory it
/// constructs a candidate path by joining the directory with `cmd`. If the
/// candidate is a regular file, that path is returned. If no such file is found,
/// the function returns `None`.
fn which(cmd: &str) -> Option<PathBuf> {
    env::var_os("PATH")?
        .to_string_lossy()
        .split(PATH_SEP)
        .find_map(|dir| {
            let candidate = Path::new(dir).join(cmd);
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        })
}

pub fn error<W: Write>(mut w: W, msg: &str) -> io::Result<()> {
    writeln!(w, "{}", Red.paint(msg))
}

pub fn warning<W: Write>(mut w: W, msg: &str) -> io::Result<()> {
    writeln!(w, "{}", Yellow.paint(msg))
}

pub fn info<W: Write>(mut w: W, msg: &str) -> io::Result<()> {
    writeln!(w, "{}", Green.paint(msg))
}
