use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

use serde_json::Value;

// ! Only keys quoted under these prefixes get embedded, so the catalogue never drifts from what the code actually uses.
const PREFIXES: [&str; 4] = ["tray_", "backup_", "startup_", "exit_"];

fn main() {
    tauri_build::build();

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let folder = root.join("../messages");
    let sources = root.join("src");
    println!("cargo:rerun-if-changed={}", folder.display());
    println!("cargo:rerun-if-changed={}", sources.display());

    let mut named = BTreeSet::new();
    named_keys(&sources, &mut named);

    let mut files: Vec<PathBuf> = fs::read_dir(&folder)
        .expect("the messages folder sits next to src-tauri")
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();

    let mut out = String::from("pub(crate) static CATALOG: &[(&str, &[(&str, &str)])] = &[\n");

    for path in files {
        println!("cargo:rerun-if-changed={}", path.display());
        let tag = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("a language file named by its tag");
        let raw = fs::read_to_string(&path).expect("a readable language file");
        let messages: serde_json::Map<String, Value> =
            serde_json::from_str(&raw).expect("a language file in JSON");

        if tag == "en" {
            if let Some(key) = named.iter().find(|key| !messages.contains_key(*key)) {
                panic!("the native catalogue names {key}, which en.json lacks");
            }
        }

        writeln!(out, "    ({tag:?}, &[").unwrap();

        for key in &named {
            if let Some(value) = messages.get(key) {
                let text = value.as_str().unwrap_or_else(|| {
                    panic!("{tag}.json: {key} must be a plain text for the native catalogue")
                });
                writeln!(out, "        ({key:?}, {text:?}),").unwrap();
            }
        }

        out.push_str("    ]),\n");
    }

    out.push_str("];\n");

    let target = Path::new(&env::var("OUT_DIR").expect("set by cargo")).join("catalog.rs");
    fs::write(target, out).expect("OUT_DIR is writable");
}

fn named_keys(dir: &Path, keys: &mut BTreeSet<String>) {
    for entry in fs::read_dir(dir).expect("a readable source folder") {
        let path = entry.expect("a readable source entry").path();

        if path.is_dir() {
            named_keys(&path, keys);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = fs::read_to_string(&path).expect("a readable source file");

            keys.extend(quoted_keys(&source).map(str::to_owned));
        }
    }
}

fn quoted_keys(source: &str) -> impl Iterator<Item = &str> {
    source.match_indices('"').filter_map(move |(at, _)| {
        let rest = &source[at + 1..];
        let end =
            rest.find(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'))?;
        let key = &rest[..end];

        (rest[end..].starts_with('"') && PREFIXES.iter().any(|prefix| key.starts_with(prefix)))
            .then_some(key)
    })
}
