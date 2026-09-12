use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "Aplicación TUI para gestionar TODOs", long_about = None)]
pub struct Cli {
    /// Directorio a escanear en busca de comentarios TODO
    #[arg(short, long, value_name = "DIR")]
    pub scan: Option<PathBuf>,

    /// Archivo JSON de tareas a cargar o guardar
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}
