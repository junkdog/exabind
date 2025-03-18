use clap::Parser;
use std::path::PathBuf;

/// Exabind - A keyboard shortcut visualization tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to KDE global shortcuts file (typically ~/.config/kglobalshortcutsrc)
    #[arg(short, long)]
    pub shortcuts_file: Option<PathBuf>,
}

pub fn parse_args() -> Result<PathBuf, String> {
    let args = Args::parse();

    // use provided path or fall back to default
    let shortcuts_path = args
        .shortcuts_file
        .unwrap_or(PathBuf::from("~/.config/kglobalshortcutsrc"));

    // expand tilde if present
    let expanded_path = if shortcuts_path.to_string_lossy().starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            let path_str = shortcuts_path
                .to_string_lossy()
                .replace('~', &home.to_string_lossy());
            PathBuf::from(path_str)
        } else {
            return Err("Could not determine home directory".to_string());
        }
    } else {
        shortcuts_path
    };

    // verify file exists
    if !expanded_path.exists() {
        return Err(format!(
            "Shortcuts file not found at: {}\nProvide path with --shortcuts-file or place file at default location",
            expanded_path.display()
        ));
    }

    Ok(expanded_path)
}
