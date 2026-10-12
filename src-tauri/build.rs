use std::path::{Path, PathBuf};

fn main() {
    tauri_build::build();
    watch_interface();
}

/// ! `tauri::generate_context!` embeds `ui/dist` without cargo knowing: without
/// these rerun-if-changed lines a rebuilt interface is ignored and the binary
/// keeps serving the old one.
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
