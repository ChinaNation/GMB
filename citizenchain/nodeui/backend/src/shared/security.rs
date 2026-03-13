// 共享安全基础设施：应用数据目录、原子写盘、安全存储、审计日志与密文封装。
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use keyring::Entry;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};
use zeroize::{Zeroize, Zeroizing};

const KEYCHAIN_SERVICE: &str = "org.chinanation.citizenchain.desktop";
const SECRET_FORMAT_VERSION: u8 = 1;
const PBKDF2_ROUNDS: u32 = 390_000;
const AUDIT_LOG_FILE_NAME: &str = "security-audit.log";
const AUDIT_LOG_MAX_BYTES: u64 = 5 * 1024 * 1024;
const AUDIT_LOG_MAX_BACKUPS: usize = 5;
static AUDIT_LOG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptedSecretEnvelope {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    cipher_b64: String,
}

/// 对路径做脱敏处理：仅保留文件名，去除父目录信息。
/// 用于返回给前端的错误消息，避免泄露服务器/本地文件系统布局。
pub(crate) fn sanitize_path(path: &Path) -> String {
    path.file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("<unknown>")
        .to_string()
}

pub(crate) fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("resolve app data dir failed: {e}"))?;
    fs::create_dir_all(&app_data).map_err(|e| format!("create app data dir failed: {e}"))?;
    Ok(app_data)
}

pub(crate) fn write_text_atomic(path: &Path, content: &str) -> Result<(), String> {
    write_bytes_atomic(path, content.as_bytes(), false)
}

pub(crate) fn write_secret_text_atomic(path: &Path, content: &str) -> Result<(), String> {
    write_bytes_atomic(path, content.as_bytes(), true)
}

/// 对非密钥但仍需限制读取权限的文件（如缓存、速率限制状态）使用受限写入。
pub(crate) fn write_text_atomic_restricted(path: &Path, content: &str) -> Result<(), String> {
    write_bytes_atomic(path, content.as_bytes(), true)
}

// 所有本地状态/密钥文件统一走“临时文件 -> fsync -> rename”路径，
// 避免掉电或异常退出时把半写入内容留给下次启动读取。
fn write_bytes_atomic(path: &Path, bytes: &[u8], secret_mode: bool) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err(format!(
            "atomic write failed: no parent for {}",
            path.display()
        ));
    };
    fs::create_dir_all(parent)
        .map_err(|e| format!("create parent dir failed ({}): {e}", parent.display()))?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_nanos())
        .unwrap_or(0);
    let random_suffix = rand::thread_rng().next_u64();
    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("atomic-data");
    let temp_path = parent.join(format!(
        ".{file_name}.tmp-{}-{stamp}-{random_suffix}",
        std::process::id()
    ));

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp_path)
        .map_err(|e| format!("create temp file failed ({}): {e}", temp_path.display()))?;

    #[cfg(unix)]
    if secret_mode {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&temp_path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            format!(
                "set temp file permission failed ({}): {e}",
                temp_path.display()
            )
        })?;
    }

    #[cfg(target_os = "windows")]
    if secret_mode {
        // Windows: 使用 NTFS 只读属性限制访问。
        // 完整的 ACL 方案需要 windows-sys 依赖，此处用标准库设置只读属性
        // 作为最低限度保护，并在注释中说明风险。
        // 注意：只读属性不等同于 Unix 0600，管理员仍可读取。
        let mut perms = fs::metadata(&temp_path)
            .map_err(|e| {
                format!(
                    "read temp file metadata failed ({}): {e}",
                    temp_path.display()
                )
            })?
            .permissions();
        perms.set_readonly(true);
        fs::set_permissions(&temp_path, perms).map_err(|e| {
            format!(
                "set temp file permission failed ({}): {e}",
                temp_path.display()
            )
        })?;
    }

    file.write_all(bytes)
        .map_err(|e| format!("write temp file failed ({}): {e}", temp_path.display()))?;
    file.sync_all()
        .map_err(|e| format!("sync temp file failed ({}): {e}", temp_path.display()))?;
    drop(file);

    #[cfg(target_os = "windows")]
    if let Err(e) = replace_file_windows(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }

    #[cfg(not(target_os = "windows"))]
    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "rename temp file failed ({} -> {}): {e}",
            temp_path.display(),
            path.display()
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn replace_file_windows(from: &Path, to: &Path) -> Result<(), String> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x8;

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(
            lp_existing_file_name: *const u16,
            lp_new_file_name: *const u16,
            dw_flags: u32,
        ) -> i32;
    }

    fn wide_null(input: &OsStr) -> Vec<u16> {
        input.encode_wide().chain(once(0)).collect()
    }

    let from_w = wide_null(from.as_os_str());
    let to_w = wide_null(to.as_os_str());
    let ok = unsafe {
        MoveFileExW(
            from_w.as_ptr(),
            to_w.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if ok == 0 {
        return Err(format!(
            "replace target file failed ({} -> {}): {}",
            from.display(),
            to.display(),
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_secs())
        .unwrap_or(0)
}

fn audit_log_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join(AUDIT_LOG_FILE_NAME))
}

fn audit_log_backup_path(path: &Path, index: usize) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(format!(".{index}"));
    PathBuf::from(backup)
}

fn rotate_audit_log_if_needed(path: &Path) -> Result<(), String> {
    let size = match fs::metadata(path) {
        Ok(meta) => meta.len(),
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(format!(
                "read audit log metadata failed ({}): {err}",
                path.display()
            ));
        }
    };
    if size < AUDIT_LOG_MAX_BYTES {
        return Ok(());
    }

    let oldest = audit_log_backup_path(path, AUDIT_LOG_MAX_BACKUPS);
    match fs::remove_file(&oldest) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(format!(
                "remove oldest audit log backup failed ({}): {err}",
                oldest.display()
            ));
        }
    }

    for idx in (1..AUDIT_LOG_MAX_BACKUPS).rev() {
        let src = audit_log_backup_path(path, idx);
        let dst = audit_log_backup_path(path, idx + 1);
        match fs::rename(&src, &dst) {
            Ok(_) => {}
            Err(err) if err.kind() == ErrorKind::NotFound => {}
            Err(err) => {
                return Err(format!(
                    "rotate audit log backup failed ({} -> {}): {err}",
                    src.display(),
                    dst.display()
                ));
            }
        }
    }

    let first = audit_log_backup_path(path, 1);
    fs::rename(path, &first).map_err(|err| {
        format!(
            "rotate current audit log failed ({} -> {}): {err}",
            path.display(),
            first.display()
        )
    })?;
    Ok(())
}

pub(crate) fn append_audit_log(app: &AppHandle, action: &str, status: &str) -> Result<(), String> {
    let path = audit_log_path(app)?;
    let lock = AUDIT_LOG_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock
        .lock()
        .map_err(|_| "audit log state poisoned".to_string())?;
    rotate_audit_log_if_needed(&path)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open audit log failed ({}): {e}", path.display()))?;
    let ts = now_unix_secs();
    let line = format!("{ts}\taction={action}\tstatus={status}\n");
    file.write_all(line.as_bytes())
        .map_err(|e| format!("write audit log failed ({}): {e}", path.display()))
}

fn secure_store_entry(account: &str) -> Result<Entry, String> {
    Entry::new(KEYCHAIN_SERVICE, account).map_err(|e| format!("初始化系统安全存储失败: {e}"))
}

pub(crate) fn secure_store_set(account: &str, value: &str) -> Result<(), String> {
    let entry = secure_store_entry(account)?;
    entry
        .set_password(value)
        .map_err(|e| format!("写入系统安全存储失败: {e}"))
}

pub(crate) fn secure_store_get(account: &str) -> Result<Option<String>, String> {
    let entry = secure_store_entry(account)?;
    match entry.get_password() {
        Ok(v) => {
            let value = v.trim().to_string();
            if value.is_empty() {
                return Ok(None);
            }
            Ok(Some(value))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(format!("读取系统安全存储失败: {e}")),
    }
}

pub(crate) fn ensure_unlock_password(password: &str) -> Result<&str, String> {
    let trimmed = password.trim();
    if trimmed.is_empty() {
        return Err("设备开机密码不能为空".to_string());
    }
    Ok(trimmed)
}

fn derive_key_from_password(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ROUNDS, &mut key);
    key
}

// 安全存储里保存的是“设备密码派生密钥二次加密后的密文封装”，
// 即使系统 keyring 条目泄露，也需要用户设备密码才能还原真正秘密。
pub(crate) fn encrypt_secret_value(secret: &str, password: &str) -> Result<String, String> {
    let unlock = ensure_unlock_password(password)?;
    let unlock_guard = Zeroizing::new(unlock.to_string());
    let secret_guard = Zeroizing::new(secret.to_string());
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce);
    let mut key = derive_key_from_password(&unlock_guard, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建加密器失败: {e}"))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), secret_guard.as_bytes())
        .map_err(|_| "私钥加密失败".to_string())?;
    key.zeroize();
    let envelope = EncryptedSecretEnvelope {
        version: SECRET_FORMAT_VERSION,
        salt_b64: BASE64.encode(salt),
        nonce_b64: BASE64.encode(nonce),
        cipher_b64: BASE64.encode(ciphertext),
    };
    serde_json::to_string(&envelope).map_err(|e| format!("编码加密数据失败: {e}"))
}

pub(crate) fn decrypt_secret_value(enveloped: &str, password: &str) -> Result<String, String> {
    let unlock = ensure_unlock_password(password)?;
    let unlock_guard = Zeroizing::new(unlock.to_string());
    let envelope: EncryptedSecretEnvelope =
        serde_json::from_str(enveloped).map_err(|e| format!("解析密文数据失败: {e}"))?;
    if envelope.version != SECRET_FORMAT_VERSION {
        return Err("密文版本不支持".to_string());
    }

    let salt = BASE64
        .decode(envelope.salt_b64)
        .map_err(|_| "密文盐值损坏".to_string())?;
    let nonce = BASE64
        .decode(envelope.nonce_b64)
        .map_err(|_| "密文随机数损坏".to_string())?;
    let cipher_bytes = BASE64
        .decode(envelope.cipher_b64)
        .map_err(|_| "密文载荷损坏".to_string())?;
    if salt.len() != 16 || nonce.len() != 12 {
        return Err("密文参数长度无效".to_string());
    }

    let mut salt_arr = [0u8; 16];
    salt_arr.copy_from_slice(&salt);
    let mut key = derive_key_from_password(&unlock_guard, &salt_arr);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建解密器失败: {e}"))?;
    let mut plain = cipher
        .decrypt(Nonce::from_slice(&nonce), cipher_bytes.as_ref())
        .map_err(|_| "解锁密码错误或密文已损坏".to_string())?;
    key.zeroize();
    let decoded = String::from_utf8(std::mem::take(&mut plain)).map_err(|e| {
        let mut bytes = e.into_bytes();
        bytes.zeroize();
        "私钥内容格式无效".to_string()
    });
    plain.zeroize();
    decoded
}
