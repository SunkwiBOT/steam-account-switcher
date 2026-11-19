use base64::Engine;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

type FileStamp = (SystemTime, u64);
type AvatarCache = HashMap<PathBuf, (FileStamp, String)>;

// ! The size joins the modification time because Windows can reuse the same timestamp across two quick writes.
static AVATAR_CACHE: Mutex<Option<AvatarCache>> = Mutex::new(None);

pub fn data_url(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    let stamp = (metadata.modified().ok()?, metadata.len());
    let mut guard = AVATAR_CACHE
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let cache = guard.get_or_insert_with(HashMap::new);
    if let Some((cached, url)) = cache.get(path) {
        if *cached == stamp {
            return Some(url.clone());
        }
    }
    let url = encode_avatar(path)?;
    cache.insert(path.to_path_buf(), (stamp, url.clone()));
    Some(url)
}

fn encode_avatar(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mime = match path
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        _ => return None,
    };
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{encoded}"))
}
