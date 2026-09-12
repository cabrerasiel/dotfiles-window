pub mod app;
pub mod cli;
pub mod handler;
pub mod models;
pub mod scanner;
pub mod ui;

use app::App;
use clap::Parser;
use cli::Cli;
use models::todos::Section;
use scanner::scan_directory;
use std::fs;
use uuid::Uuid;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();

    // Sin --scan, las tareas se guardan por defecto en la carpeta del usuario ($HOME/.todos.json)
    // salvo que se indique un --file explícito. En modo --scan solo se guarda si se pasó --file,
    // para no sobrescribir la base de datos local del usuario con resultados de un escaneo.
    let save_path = if cli.scan.is_some() {
        cli.file.clone()
    } else {
        Some(cli.file.clone().unwrap_or_else(App::default_save_path))
    };

    let sections = if let Some(scan_dir) = &cli.scan {
        let todos = scan_directory(scan_dir);
        if todos.is_empty() {
            Vec::new()
        } else {
            vec![Section {
                id: Uuid::new_v4(),
                name: format!("Escaneo: {}", scan_dir.display()),
                todos,
            }]
        }
    } else if let Some(file_path) = &cli.file {
        if file_path.exists() {
            let content = fs::read_to_string(file_path)?;
            App::parse_sections(&content)
        } else {
            Vec::new()
        }
    } else if save_path.as_ref().is_some_and(|p| p.exists()) {
        let content = fs::read_to_string(save_path.as_ref().unwrap())?;
        App::parse_sections(&content)
    } else {
        App::default_sections()
    };

    let mut app = App::with_sections(sections);
    app.file_path = save_path;
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
