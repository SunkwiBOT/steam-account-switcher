use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode(input: &[u8]) -> String {
    let mut output = String::with_capacity(input.len().div_ceil(3) * 4);

    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied().unwrap_or(0);
        let third = chunk.get(2).copied().unwrap_or(0);

        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 0b0000_0011) << 4) | (second >> 4)) as usize] as char);

        if chunk.len() > 1 {
            output.push(ALPHABET[(((second & 0b0000_1111) << 2) | (third >> 6)) as usize] as char);
        } else {
            output.push('=');
        }

        if chunk.len() > 2 {
            output.push(ALPHABET[(third & 0b0011_1111) as usize] as char);
        } else {
            output.push('=');
        }
    }

    output
}

type FileStamp = (SystemTime, u64);
type AvatarCache = HashMap<PathBuf, (FileStamp, String)>;

// Snapshots are built every second while the window is active; re-encoding every
// avatar each time costs more than the avatars themselves. The size joins the
// modification time because Windows can keep the same time for two quick writes.
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
    Some(format!("data:{mime};base64,{}", encode(&bytes)))
}
