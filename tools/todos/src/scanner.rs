use std::fs;
use std::path::Path;
use crate::models::todos::Todo;

pub fn scan_directory(dir: &Path) -> Vec<Todo> {
    let mut todos = Vec::new();
    scan_dir_recursive(dir, dir, &mut todos);
    todos
}

fn scan_dir_recursive(root: &Path, current: &Path, todos: &mut Vec<Todo>) {
    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        // Ignorar carpetas/archivos ocultos o pesados
        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "build" {
            continue;
        }

        if path.is_dir() {
            scan_dir_recursive(root, &path, todos);
        } else if path.is_file() {
            scan_file(root, &path, todos);
        }
    }
}

fn scan_file(root: &Path, path: &Path, todos: &mut Vec<Todo>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return, // Ignorar archivos binarios o no-UTF8
    };

    let rel_path = path.strip_prefix(root).unwrap_or(path);

    for (line_num, line) in content.lines().enumerate() {
        if let Some(idx) = line.find("TODO") {
            let todo_str = line[idx..].trim();
            // Limpiar prefijos comunes como // TODO: o # TODO:
            let cleaned = todo_str
                .trim_start_matches("TODO")
                .trim_start_matches(':')
                .trim();

            let text = if cleaned.is_empty() {
                todo_str.to_string()
            } else {
                cleaned.to_string()
            };

            let formatted_text = format!("{}:{} - {}", rel_path.display(), line_num + 1, text);
            todos.push(Todo::new(formatted_text));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_file_extracts_todos() {
        let temp_dir = std::env::temp_dir().join("todos_test_scan");
        let _ = fs::create_dir_all(&temp_dir);
        let file_path = temp_dir.join("sample.rs");
        fs::write(
            &file_path,
            "// TODO: Refactor this function\nfn foo() {}\n// TODO Fix bug\n",
        )
        .unwrap();

        let todos = scan_directory(&temp_dir);
        assert_eq!(todos.len(), 2);
        assert!(todos[0].text.contains("Refactor this function"));
        assert!(todos[1].text.contains("Fix bug"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
