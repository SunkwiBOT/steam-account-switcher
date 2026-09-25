use std::path::{Path, PathBuf};

fn main() {
    tauri_build::build();
    watch_interface();
}

/// The interface is embedded by `tauri::generate_context!`, which cargo knows
/// nothing about: without these instructions a rebuilt `ui/dist` would be
/// ignored and the binary would keep serving the previous interface.
fn watch_interface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/dist");
    println!("cargo:rerun-if-changed={}", root.display());

    let mut pending: Vec<PathBuf> = vec![root];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else {
                println!("cargo:rerun-if-changed={}", path.display());
            }
        }
    }
}
