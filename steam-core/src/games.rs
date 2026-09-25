//! Resolves Steam app ids to game names from the local libraries.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::accounts::read_vdf;
use crate::model::{expand_home, SteamInstallation};

/// Reads `libraryfolders.vdf` and `appmanifest_*.acf` to name detected games.
///
/// Nothing is downloaded: unknown games simply keep their app id.
#[derive(Debug)]
pub struct SteamGameCatalog {
    installation: SteamInstallation,
    cache: Mutex<HashMap<String, String>>,
}

impl SteamGameCatalog {
    pub fn new(installation: &SteamInstallation) -> Self {
        Self {
            installation: installation.clone(),
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Steam library roots, including the installation itself.
    pub fn library_roots(&self) -> Vec<PathBuf> {
        let mut roots = vec![self.installation.root.clone()];

        let library_file = self.installation.root.join("steamapps/libraryfolders.vdf");
        let Ok(data) = read_vdf(&library_file) else {
            return roots;
        };

        let Some(folders) = data.get_object("libraryfolders") else {
            return roots;
        };

        for (_, value) in folders.iter() {
            let path = match value {
                crate::vdf::VdfValue::String(text) => Some(text.to_string()),
                crate::vdf::VdfValue::Object(folder) => folder.get_str("path").map(str::to_string),
            };

            if let Some(path) = path {
                let path = expand_home(PathBuf::from(path).as_path());
                if !roots.contains(&path) {
                    roots.push(path);
                }
            }
        }

        roots
    }

    /// Game name for `app_id`, empty when Steam has no manifest for it.
    pub fn name_for(&self, app_id: &str) -> String {
        if let Ok(cache) = self.cache.lock() {
            if let Some(name) = cache.get(app_id) {
                return name.clone();
            }
        }

        let name = self.read_name(app_id).unwrap_or_default();

        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(app_id.to_string(), name.clone());
        }

        name
    }

    /// Game name, falling back to `App <id>` so the UI never shows an empty row.
    pub fn display_name_for(&self, app_id: &str) -> String {
        let name = self.name_for(app_id);
        if name.is_empty() {
            format!("App {app_id}")
        } else {
            name
        }
    }

    fn read_name(&self, app_id: &str) -> Option<String> {
        for root in self.library_roots() {
            let manifest = root.join(format!("steamapps/appmanifest_{app_id}.acf"));
            if !manifest.is_file() {
                continue;
            }

            let data = read_vdf(&manifest).ok()?;
            let name = data
                .get_object("AppState")
                .and_then(|state| state.get_str("name"))
                .unwrap_or_default()
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InstallKind;

    #[test]
    fn resolves_names_from_a_library_manifest() {
        let directory =
            std::env::temp_dir().join(format!("steam-core-games-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        let steamapps = directory.join("steamapps");
        std::fs::create_dir_all(&steamapps).expect("steamapps");

        std::fs::write(
            directory.join("steamapps/libraryfolders.vdf"),
            "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"{ROOT}\"\n\t}\n}\n"
                .replace("{ROOT}", &directory.to_string_lossy()),
        )
        .expect("libraryfolders");
        std::fs::write(
            steamapps.join("appmanifest_570.acf"),
            "\"AppState\"\n{\n\t\"appid\"\t\t\"570\"\n\t\"name\"\t\t\"Dota 2\"\n}\n",
        )
        .expect("manifest");

        let installation = SteamInstallation {
            kind: InstallKind::Native,
            root: directory.clone(),
            config_dir: directory.join("config"),
            loginusers_path: directory.join("config/loginusers.vdf"),
            registry_path: directory.join("registry.vdf"),
            userdata_dir: directory.join("userdata"),
            launch_command: vec!["steam".to_string()],
            prefix: None,
        };
        let catalog = SteamGameCatalog::new(&installation);

        assert_eq!(catalog.name_for("570"), "Dota 2");
        assert_eq!(catalog.display_name_for("999"), "App 999");
        assert!(catalog.library_roots().contains(&directory));

        let _ = std::fs::remove_dir_all(directory);
    }
}
