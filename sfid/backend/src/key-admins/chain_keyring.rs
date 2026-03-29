use chrono::Utc;
use hex::FromHex;
use schnorrkel::{signing_context, PublicKey as Sr25519PublicKey, Signature as Sr25519Signature};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519::Pair as Sr25519Pair, Pair};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainKeyringState {
    pub version: u64,
    pub main_pubkey: String,
    pub backup_a_pubkey: String,
    pub backup_b_pubkey: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeySlot {
    Main,
    BackupA,
    BackupB,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotateMainRequest {
    pub initiator_pubkey: String,
    pub new_backup_pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RotateMainResult {
    pub old_main_pubkey: String,
    pub promoted_slot: KeySlot,
    pub state: ChainKeyringState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RotateMainError {
    InitiatorMustBeBackup,
    NewBackupPubkeyRequired,
    NewBackupPubkeyConflict,
}

impl ChainKeyringState {
    pub fn new(main_pubkey: String, backup_a_pubkey: String, backup_b_pubkey: String) -> Self {
        Self {
            version: 1,
            main_pubkey,
            backup_a_pubkey,
            backup_b_pubkey,
            updated_at: Utc::now().timestamp(),
        }
    }

    pub fn rotate_main(&self, req: RotateMainRequest) -> Result<RotateMainResult, RotateMainError> {
        let initiator = req.initiator_pubkey.trim();
        let new_backup = req.new_backup_pubkey.trim();

        if new_backup.is_empty() {
            return Err(RotateMainError::NewBackupPubkeyRequired);
        }

        if new_backup.eq_ignore_ascii_case(self.main_pubkey.as_str())
            || new_backup.eq_ignore_ascii_case(self.backup_a_pubkey.as_str())
            || new_backup.eq_ignore_ascii_case(self.backup_b_pubkey.as_str())
        {
            return Err(RotateMainError::NewBackupPubkeyConflict);
        }

        let (promoted_slot, next_main, next_backup_a, next_backup_b) =
            if initiator.eq_ignore_ascii_case(self.backup_a_pubkey.as_str()) {
                (
                    KeySlot::BackupA,
                    self.backup_a_pubkey.clone(),
                    new_backup.to_string(),
                    self.backup_b_pubkey.clone(),
                )
            } else if initiator.eq_ignore_ascii_case(self.backup_b_pubkey.as_str()) {
                (
                    KeySlot::BackupB,
                    self.backup_b_pubkey.clone(),
                    self.backup_a_pubkey.clone(),
                    new_backup.to_string(),
                )
            } else {
                return Err(RotateMainError::InitiatorMustBeBackup);
            };

        let next_state = ChainKeyringState {
            version: self.version.saturating_add(1),
            main_pubkey: next_main,
            backup_a_pubkey: next_backup_a,
            backup_b_pubkey: next_backup_b,
            updated_at: Utc::now().timestamp(),
        };

        Ok(RotateMainResult {
            old_main_pubkey: self.main_pubkey.clone(),
            promoted_slot,
            state: next_state,
        })
    }

    #[allow(dead_code)]
    pub fn all_pubkeys(&self) -> [&str; 3] {
        [
            &self.main_pubkey,
            &self.backup_a_pubkey,
            &self.backup_b_pubkey,
        ]
    }
}

#[allow(dead_code)]
pub fn load_signing_key() -> Sr25519Pair {
    let raw = std::env::var("SFID_SIGNING_SEED_HEX")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .expect("SFID_SIGNING_SEED_HEX is required");
    load_signing_key_from_seed(raw.as_str())
}

pub fn try_load_signing_key_from_seed(seed_text: &str) -> Result<Sr25519Pair, String> {
    let seed = decode_seed_to_32(seed_text)?;
    Sr25519Pair::from_seed_slice(&seed)
        .map_err(|_| "invalid sr25519 seed for substrate pair derivation".to_string())
}

pub fn load_signing_key_from_seed(seed_text: &str) -> Sr25519Pair {
    try_load_signing_key_from_seed(seed_text)
        .unwrap_or_else(|err| panic!("invalid signing seed hex: {err}"))
}

pub fn try_derive_pubkey_hex_from_seed(seed_text: &str) -> Result<String, String> {
    let keypair = try_load_signing_key_from_seed(seed_text)?;
    Ok(format!("0x{}", hex::encode(keypair.public().0)))
}

#[allow(dead_code)]
pub fn derive_pubkey_hex_from_seed(seed_text: &str) -> String {
    try_derive_pubkey_hex_from_seed(seed_text)
        .unwrap_or_else(|err| panic!("invalid signing seed hex: {err}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RotationSignatureMode {
    Raw,
    BytesWrapped,
}

fn rotation_signature_mode(message: &str) -> RotationSignatureMode {
    // Reserved for forward compatibility: server currently emits `sigfmt=raw-v1`.
    if message.contains("|sigfmt=bytes-wrap-v1") {
        RotationSignatureMode::BytesWrapped
    } else {
        RotationSignatureMode::Raw
    }
}

pub fn verify_rotation_signature(pubkey: &str, message: &str, signature: &str) -> bool {
    let Some(pubkey_bytes) = parse_sr25519_pubkey_bytes(pubkey) else {
        return false;
    };
    let Some(signature_bytes) = parse_sr25519_signature_bytes(signature) else {
        return false;
    };
    let verifying_key = match Sr25519PublicKey::from_bytes(&pubkey_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let sig = match Sr25519Signature::from_bytes(&signature_bytes) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let ctx = signing_context(b"substrate");
    match rotation_signature_mode(message) {
        RotationSignatureMode::Raw => verifying_key
            .verify(ctx.bytes(message.as_bytes()), &sig)
            .is_ok(),
        RotationSignatureMode::BytesWrapped => {
            let wrapped = format!("<Bytes>{}</Bytes>", message);
            verifying_key
                .verify(ctx.bytes(wrapped.as_bytes()), &sig)
                .is_ok()
        }
    }
}

fn parse_sr25519_pubkey_bytes(value: &str) -> Option<[u8; 32]> {
    let normalized = normalize_hex(value);
    if normalized.len() == 64 && normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        let bytes = Vec::from_hex(normalized).ok()?;
        return bytes.as_slice().try_into().ok();
    }
    None
}

fn parse_sr25519_signature_bytes(value: &str) -> Option<[u8; 64]> {
    let normalized = normalize_hex(value);
    if normalized.len() != 128 || !normalized.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let bytes = Vec::from_hex(normalized).ok()?;
    bytes.as_slice().try_into().ok()
}

fn normalize_hex(value: &str) -> &str {
    value
        .trim()
        .strip_prefix("0x")
        .or_else(|| value.trim().strip_prefix("0X"))
        .unwrap_or(value.trim())
}

fn decode_seed_to_32(raw: &str) -> Result<[u8; 32], String> {
    let trimmed = normalize_hex(raw);
    if trimmed.len() != 64 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("seed must be exactly 64 hex characters".to_string());
    }
    let bytes = Vec::from_hex(trimmed).map_err(|_| "seed contains invalid hex".to_string())?;
    if bytes.len() != 32 {
        return Err("seed must decode to 32 bytes".to_string());
    }
    let mut out = [0_u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> ChainKeyringState {
        ChainKeyringState::new(
            "main-pubkey".to_string(),
            "backup-a".to_string(),
            "backup-b".to_string(),
        )
    }

    #[test]
    fn rotate_main_from_backup_a_keeps_one_main_two_backups() {
        let state = sample_state();
        let result = state
            .rotate_main(RotateMainRequest {
                initiator_pubkey: "backup-a".to_string(),
                new_backup_pubkey: "backup-new".to_string(),
            })
            .expect("rotate from backup_a should succeed");
        assert_eq!(result.promoted_slot, KeySlot::BackupA);
        assert_eq!(result.state.main_pubkey, "backup-a");
        assert_eq!(result.state.backup_a_pubkey, "backup-new");
        assert_eq!(result.state.backup_b_pubkey, "backup-b");
    }

    #[test]
    fn rotate_main_requires_backup_initiator() {
        let state = sample_state();
        let err = state
            .rotate_main(RotateMainRequest {
                initiator_pubkey: "main-pubkey".to_string(),
                new_backup_pubkey: "backup-new".to_string(),
            })
            .expect_err("main cannot initiate rotation");
        assert_eq!(err, RotateMainError::InitiatorMustBeBackup);
    }

    #[test]
    fn rotate_main_requires_unique_new_backup() {
        let state = sample_state();
        let err = state
            .rotate_main(RotateMainRequest {
                initiator_pubkey: "backup-b".to_string(),
                new_backup_pubkey: "backup-a".to_string(),
            })
            .expect_err("new backup must not clash with existing key");
        assert_eq!(err, RotateMainError::NewBackupPubkeyConflict);
    }

    #[test]
    fn rotate_main_compares_pubkeys_case_insensitively() {
        let state = ChainKeyringState::new(
            "0xMAIN".to_string(),
            "0xAbCd".to_string(),
            "0xBEEF".to_string(),
        );
        let result = state
            .rotate_main(RotateMainRequest {
                initiator_pubkey: "0xabcd".to_string(),
                new_backup_pubkey: "0x1234".to_string(),
            })
            .expect("case-insensitive initiator should rotate");
        assert_eq!(result.state.main_pubkey, "0xAbCd");
    }

    #[test]
    fn weak_non_hex_seed_is_rejected() {
        assert!(try_load_signing_key_from_seed("password123").is_err());
        assert!(try_derive_pubkey_hex_from_seed("test-seed").is_err());
    }

    #[test]
    fn substrate_seed_derivation_matches_dev_chain_main_pubkey() {
        let pubkey = derive_pubkey_hex_from_seed(
            "0xb642a34db79f5adbc800415b27bd271a5459e5e53f80d63c4e4c920fc247f4da",
        );
        assert_eq!(
            pubkey,
            "0x14e4f684453a0ccf9ebb3113d05ae1da934b7f7b2dbd3b9dcdf4138357ab1607"
        );
    }
}
