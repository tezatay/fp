use arboard::Clipboard;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::process;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug)]
enum AppError {
    NoFileProvided,
    InvalidArgument(String),
    FileNotFound(String),
    FileReadError(String, std::io::Error),
    NotUtf8(String),
    ClipboardError(arboard::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NoFileProvided => {
                write!(f, "File is not specified. Use --help for more info")
            }
            AppError::InvalidArgument(arg) => {
                write!(f, "Unknown argument: '{arg}'")
            }
            AppError::FileNotFound(path) => {
                write!(f, "File wasn't found: '{path}'")
            }
            AppError::FileReadError(path, e) => {
                write!(f, "Unable to read the file '{path}': {e}")
            }
            AppError::NotUtf8(path) => {
                write!(f, "File '{path}' contains non-UTF-8 data")
            }
            AppError::ClipboardError(e) => {
                write!(f, "Clipboard error: {e}")
            }
        }
    }
}

impl From<arboard::Error> for AppError {
    fn from(e: arboard::Error) -> Self {
        AppError::ClipboardError(e)
    }
}

enum Action {
    CopyFile(String),
    ShowHelp,
    ShowVersion,
}

fn parse_args(args: &[String]) -> Result<Action, AppError> {
    if args.len() < 2 {
        return Err(AppError::NoFileProvided);
    }

    let mut file_path: Option<String> = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Action::ShowHelp),
            "--version" | "-v" => return Ok(Action::ShowVersion),
            s if s.starts_with('-') => {
                return Err(AppError::InvalidArgument(arg.clone()));
            }
            _ => {
                file_path = Some(arg.trim().to_string());
            }
        }
    }

    file_path
        .map(Action::CopyFile)
        .ok_or(AppError::NoFileProvided)
}

fn print_help() {
    println!(
        "FP -- copies file into clipboard\n\
         \n\
         Usage:\n\
             fp [OPTION] <FILE>\n\
         \n\
         Arguments:\n\
             <FILE>    File Path (UTF-8 only)\n\
         \n\
         Options:\n\
             -h, --help       Show this text\n\
             -v, --version    Print version"
    );
}

fn print_version() {
    println!("FP version {VERSION}");
}

fn read_file(path: &Path) -> Result<String, AppError> {
    let path_str = path.display().to_string();

    if !path.exists() {
        return Err(AppError::FileNotFound(path_str));
    }

    let bytes = fs::read(path).map_err(|e| AppError::FileReadError(path_str.clone(), e))?;

    String::from_utf8(bytes).map_err(|_| AppError::NotUtf8(path_str))
}

fn copy_to_clipboard(text: String) -> Result<(), AppError> {
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text(text)?;

    #[cfg(target_os = "linux")]
    wait_for_clipboard_on_linux();

    Ok(())
}

#[cfg(target_os = "linux")]
fn wait_for_clipboard_on_linux() {
    use std::time::Duration;

    eprintln!("[Linux] Clipboard is active. Press Enter for exit");

    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);

    std::thread::sleep(Duration::from_millis(100));
}

fn run() -> Result<(), AppError> {
    let args: Vec<String> = env::args().collect();

    match parse_args(&args)? {
        Action::ShowHelp => print_help(),
        Action::ShowVersion => print_version(),
        Action::CopyFile(path_str) => {
            let path = Path::new(&path_str);
            let content = read_file(path)?;

            if content.is_empty() {
                eprintln!("Warning: File is empty, clipboard wasn't changed");
                return Ok(());
            }

            copy_to_clipboard(content)?;
            println!("Successful: '{}'", path.display());
        }
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
