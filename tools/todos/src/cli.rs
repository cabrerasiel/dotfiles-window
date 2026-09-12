use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Aplicación TUI para gestionar TODOs",
    long_about = "Aplicación TUI para gestionar TODOs.\n\n\
        Sin argumentos, las tareas se cargan y guardan (tecla `s`) en la base de datos \
        local del usuario: $HOME/.todos.json."
)]
pub struct Cli {
    /// Directorio a escanear en busca de comentarios TODO
    #[arg(short, long, value_name = "DIR")]
    pub scan: Option<PathBuf>,

    /// Archivo JSON de tareas a cargar o guardar (por defecto: $HOME/.todos.json)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}
