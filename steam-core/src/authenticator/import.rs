use super::vault::{base64_decode, validate_secret, KEY_LENGTH};
use super::{io_error, AuthError};
use crate::error_code::ErrorCode;
use aes::{
    cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit},
    Aes256,
};
use serde::Deserialize;
use std::{fs, path::Path};
use steamguard::{token::TwoFactorSecret, SteamGuardAccount};
use zeroize::Zeroizing;

#[derive(Debug, Deserialize)]
struct SdaManifest {
    #[serde(default)]
    entries: Vec<SdaManifestEntry>,
}

#[derive(Debug, Deserialize)]
struct SdaManifestEntry {
    filename: String,
    #[serde(
        default,
        rename = "steamid",
        deserialize_with = "deserialize_optional_u64"
    )]
    steam_id: Option<u64>,
    #[serde(default, rename = "encryption_iv")]
    iv: Option<String>,
    #[serde(default, rename = "encryption_salt")]
    salt: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SdaAccount {
    #[serde(default)]
    account_name: String,
    #[serde(default)]
    serial_number: String,
    #[serde(default)]
    revocation_code: String,
    shared_secret: String,
    #[serde(default)]
    token_gid: String,
    identity_secret: String,
    #[serde(default)]
    uri: String,
    #[serde(default)]
    device_id: String,
    #[serde(default)]
    secret_1: String,
    #[serde(default, rename = "Session")]
    session: Option<SdaSession>,
}

#[derive(Debug, Deserialize)]
struct SdaSession {
    #[serde(
        default,
        rename = "SteamID",
        deserialize_with = "deserialize_optional_u64"
    )]
    steam_id: Option<u64>,
}

fn deserialize_optional_u64<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<u64>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrU64 {
        String(String),
        U64(u64),
    }

    Option::<StringOrU64>::deserialize(deserializer)?
        .map(|value| match value {
            StringOrU64::String(value) => value.parse().map_err(serde::de::Error::custom),
            StringOrU64::U64(value) => Ok(value),
        })
        .transpose()
}

/// Encrypted .maFile backups use AES-256-CBC with a PBKDF2-HMAC-SHA1 key.
/// Their IV and salt are stored in the sibling `manifest.json`.
fn decrypt_backup(
    ciphertext: &[u8],
    passkey: &str,
    iv_b64: &str,
    salt_b64: &str,
) -> Result<Vec<u8>, AuthError> {
    let salt = base64_decode(salt_b64).map_err(|_| AuthError::InvalidBackupManifest)?;
    let iv = base64_decode(iv_b64).map_err(|_| AuthError::InvalidBackupManifest)?;
    if iv.len() != 16 || salt.is_empty() {
        return Err(AuthError::InvalidBackupManifest);
    }

    let mut key = Zeroizing::new([0u8; KEY_LENGTH]);
    pbkdf2::pbkdf2_hmac::<sha1::Sha1>(passkey.as_bytes(), &salt, 50_000, key.as_mut());

    let text = std::str::from_utf8(ciphertext).map_err(|_| AuthError::InvalidBackup)?;
    let decoded = base64_decode(text).map_err(|_| AuthError::InvalidBackup)?;
    if decoded.is_empty() || !decoded.len().is_multiple_of(16) {
        return Err(AuthError::InvalidBackup);
    }

    let mut buffer = Zeroizing::new(decoded);
    let decryptor = cbc::Decryptor::<Aes256>::new_from_slices(key.as_ref(), &iv)
        .map_err(|_| AuthError::InvalidBackupManifest)?;
    let plaintext = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| AuthError::BackupDecryptionFailed)?;
    Ok(plaintext.to_vec())
}

pub fn import_mafile(path: &Path, passkey: Option<&str>) -> Result<SteamGuardAccount, AuthError> {
    let bytes = fs::read(path).map_err(io_error)?;
    let looks_like_json = bytes
        .iter()
        .find(|byte| !byte.is_ascii_whitespace())
        .map(|byte| *byte == b'{')
        .unwrap_or(false);

    let (json, manifest_steam_id) = if looks_like_json {
        (bytes, 0u64)
    } else {
        let passkey = passkey.ok_or(AuthError::Code(ErrorCode::BackupPasswordRequired))?;
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_default();
        let manifest_path = path.with_file_name("manifest.json");
        let manifest_text = fs::read_to_string(&manifest_path)
            .map_err(|_| AuthError::Code(ErrorCode::BackupManifestMissing))?;
        let manifest: SdaManifest =
            serde_json::from_str(&manifest_text).map_err(|_| AuthError::InvalidBackupManifest)?;
        let entry = manifest
            .entries
            .iter()
            .find(|entry| entry.filename == file_name)
            .ok_or(AuthError::Code(ErrorCode::BackupManifestMismatch))?;
        let iv = entry
            .iv
            .as_deref()
            .ok_or(AuthError::InvalidBackupManifest)?;
        let salt = entry
            .salt
            .as_deref()
            .ok_or(AuthError::InvalidBackupManifest)?;
        let plaintext = decrypt_backup(&bytes, passkey, iv, salt)?;
        (plaintext, entry.steam_id.unwrap_or(0))
    };

    let json = Zeroizing::new(json);
    let raw: SdaAccount = serde_json::from_slice(&json).map_err(|_| AuthError::InvalidBackup)?;
    sda_to_account(raw, manifest_steam_id)
}

fn sda_to_account(raw: SdaAccount, fallback_steam_id: u64) -> Result<SteamGuardAccount, AuthError> {
    let steam_id = raw
        .session
        .as_ref()
        .and_then(|session| session.steam_id)
        .filter(|value| *value != 0)
        .unwrap_or(fallback_steam_id);
    if steam_id == 0 {
        return Err(AuthError::Code(ErrorCode::BackupSteamIdMissing));
    }

    let shared_secret = TwoFactorSecret::from_bytes(
        validate_secret(&raw.shared_secret).map_err(|_| AuthError::InvalidBackup)?,
    );
    validate_secret(&raw.identity_secret).map_err(|_| AuthError::InvalidBackup)?;
    if raw.device_id.trim().is_empty() {
        return Err(AuthError::Code(ErrorCode::BackupDeviceMissing));
    }

    Ok(SteamGuardAccount {
        account_name: raw.account_name,
        steam_id,
        serial_number: raw.serial_number,
        revocation_code: raw.revocation_code.into(),
        shared_secret,
        token_gid: raw.token_gid,
        identity_secret: raw.identity_secret.into(),
        uri: raw.uri.into(),
        device_id: raw.device_id,
        secret_1: raw.secret_1.into(),
        tokens: None,
    })
}
