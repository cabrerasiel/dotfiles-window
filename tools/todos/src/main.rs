pub mod app;
pub mod cli;
pub mod handler;
pub mod models;
pub mod scanner;
pub mod ui;

use app::App;
use clap::Parser;
use cli::Cli;
use scanner::scan_directory;
use std::fs;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    let todos = if let Some(scan_dir) = &cli.scan {
        scan_directory(scan_dir)
    } else if let Some(file_path) = &cli.file {
        if file_path.exists() {
            let content = fs::read_to_string(file_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        App::default_todos()
    };

    let mut app = App::with_todos(todos);
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
