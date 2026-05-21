use arboard::Clipboard;
use clap::Parser;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;
use thiserror::Error;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(
    name = "fp",
    version,
    about = "Copy file or stdin content into clipboard",
    long_about = None
)]
struct Cli {
    /// File to copy into clipboard
    file: Option<PathBuf>,

    /// Read content from stdin
    #[arg(short, long)]
    stdin: bool,

    /// Trim trailing whitespace
    #[arg(short, long)]
    trim: bool,

    /// Print extra information
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Error)]
enum AppError {
    #[error("No input provided. Use file path or --stdin")]
    NoInput,

    #[error("Failed to read file '{path}': {source}")]
    FileReadError {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("Failed to read stdin: {0}")]
    StdinReadError(io::Error),

    #[error("Clipboard error: {0}")]
    ClipboardError(#[from] arboard::Error),
}

fn read_file(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|e| AppError::FileReadError {
        path: path.display().to_string(),
        source: e,
    })
}

fn read_stdin() -> Result<String, AppError> {
    let mut buffer = String::new();

    io::stdin()
        .read_to_string(&mut buffer)
        .map_err(AppError::StdinReadError)?;

    Ok(buffer)
}

fn copy_to_clipboard(content: &str) -> Result<(), AppError> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(content)?;

    #[cfg(target_os = "linux")]
    keep_clipboard_alive();

    Ok(())
}

#[cfg(target_os = "linux")]
fn keep_clipboard_alive() {
    eprintln!("Clipboard active. Press Enter to exit...");

    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
}

fn load_content(cli: &Cli) -> Result<String, AppError> {
    if cli.stdin {
        return read_stdin();
    }

    if let Some(path) = &cli.file {
        return read_file(path);
    }

    Err(AppError::NoInput)
}

fn print_verbose_info(cli: &Cli, content: &str) {
    println!("Version      : {VERSION}");
    println!("Content size : {} bytes", content.len());
    println!("Trim enabled : {}", cli.trim);

    if let Some(path) = &cli.file {
        println!("Source file  : {}", path.display());
    } else {
        println!("Source       : stdin");
    }

    println!();
}

fn run() -> Result<(), AppError> {
    let cli = Cli::parse();

    let mut content = load_content(&cli)?;

    if cli.trim {
        content = content.trim().to_string();
    }

    if content.is_empty() {
        println!("Warning: input is empty");
        return Ok(());
    }

    if cli.verbose {
        print_verbose_info(&cli, &content);
    }

    copy_to_clipboard(&content)?;

    match &cli.file {
        Some(path) => {
            println!("Copied '{}' to clipboard", path.display());
        }
        None => {
            println!("Copied stdin content to clipboard");
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
