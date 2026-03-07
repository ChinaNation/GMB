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
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};

const KEYCHAIN_SERVICE: &str = "org.chinanation.citizenchain.desktop";
const SECRET_FORMAT_VERSION: u8 = 1;
const PBKDF2_ROUNDS: u32 = 210_000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EncryptedSecretEnvelope {
    version: u8,
    salt_b64: String,
    nonce_b64: String,
    cipher_b64: String,
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
    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("atomic-data");
    let temp_path = parent.join(format!(".{file_name}.tmp-{}-{stamp}", std::process::id()));

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

    file.write_all(bytes)
        .map_err(|e| format!("write temp file failed ({}): {e}", temp_path.display()))?;
    file.sync_all()
        .map_err(|e| format!("sync temp file failed ({}): {e}", temp_path.display()))?;
    drop(file);

    #[cfg(target_os = "windows")]
    if path.exists() {
        fs::remove_file(path)
            .map_err(|e| format!("replace target file failed ({}): {e}", path.display()))?;
    }

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

#[cfg(target_os = "macos")]
pub(crate) fn verify_device_login_password(password: &str) -> Result<(), String> {
    let user = std::env::var("USER").map_err(|e| format!("读取系统用户失败: {e}"))?;
    let output = std::process::Command::new("dscl")
        .args(["/Search", "-authonly", &user, password])
        .output()
        .map_err(|e| format!("校验设备密码失败: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    Err("设备开机密码错误".to_string())
}

#[cfg(target_os = "linux")]
pub(crate) fn verify_device_login_password(password: &str) -> Result<(), String> {
    linux_password_auth::verify_with_pam(password)
}

#[cfg(target_os = "linux")]
mod linux_password_auth {
    use std::{
        ffi::{c_char, c_int, c_void, CString},
        ptr,
    };

    const PAM_SUCCESS: c_int = 0;
    const PAM_BUF_ERR: c_int = 5;
    const PAM_CONV_ERR: c_int = 19;
    const PAM_PROMPT_ECHO_OFF: c_int = 1;
    const PAM_PROMPT_ECHO_ON: c_int = 2;
    const PAM_ERROR_MSG: c_int = 3;
    const PAM_TEXT_INFO: c_int = 4;

    #[repr(C)]
    struct PamMessage {
        msg_style: c_int,
        msg: *const c_char,
    }

    #[repr(C)]
    struct PamResponse {
        resp: *mut c_char,
        resp_retcode: c_int,
    }

    #[repr(C)]
    struct PamConv {
        conv: Option<
            unsafe extern "C" fn(
                num_msg: c_int,
                msg: *mut *const PamMessage,
                resp: *mut *mut PamResponse,
                appdata_ptr: *mut c_void,
            ) -> c_int,
        >,
        appdata_ptr: *mut c_void,
    }

    #[repr(C)]
    struct PamHandle {
        _private: [u8; 0],
    }

    #[link(name = "pam")]
    unsafe extern "C" {
        fn pam_start(
            service_name: *const c_char,
            user: *const c_char,
            pam_conv: *const PamConv,
            pamh: *mut *mut PamHandle,
        ) -> c_int;
        fn pam_end(pamh: *mut PamHandle, pam_status: c_int) -> c_int;
        fn pam_authenticate(pamh: *mut PamHandle, flags: c_int) -> c_int;
        fn pam_acct_mgmt(pamh: *mut PamHandle, flags: c_int) -> c_int;
    }

    unsafe fn free_responses(responses: *mut PamResponse, filled: usize) {
        for idx in 0..filled {
            let response = responses.add(idx);
            if !(*response).resp.is_null() {
                libc::free((*response).resp as *mut c_void);
                (*response).resp = ptr::null_mut();
            }
        }
        libc::free(responses as *mut c_void);
    }

    unsafe extern "C" fn conversation(
        num_msg: c_int,
        msg: *mut *const PamMessage,
        resp: *mut *mut PamResponse,
        appdata_ptr: *mut c_void,
    ) -> c_int {
        if num_msg <= 0 || msg.is_null() || resp.is_null() || appdata_ptr.is_null() {
            return PAM_CONV_ERR;
        }

        let responses =
            libc::calloc(num_msg as usize, std::mem::size_of::<PamResponse>()) as *mut PamResponse;
        if responses.is_null() {
            return PAM_BUF_ERR;
        }

        let password_ptr = appdata_ptr as *const c_char;
        for idx in 0..num_msg as usize {
            let msg_ptr = *msg.add(idx);
            if msg_ptr.is_null() {
                free_responses(responses, idx);
                return PAM_CONV_ERR;
            }
            match (*msg_ptr).msg_style {
                PAM_PROMPT_ECHO_OFF | PAM_PROMPT_ECHO_ON => {
                    let dup = libc::strdup(password_ptr);
                    if dup.is_null() {
                        free_responses(responses, idx);
                        return PAM_BUF_ERR;
                    }
                    (*responses.add(idx)).resp = dup;
                    (*responses.add(idx)).resp_retcode = 0;
                }
                PAM_ERROR_MSG | PAM_TEXT_INFO => {}
                _ => {
                    free_responses(responses, idx);
                    return PAM_CONV_ERR;
                }
            }
        }

        *resp = responses;
        PAM_SUCCESS
    }

    pub(super) fn verify_with_pam(password: &str) -> Result<(), String> {
        let user = std::env::var("USER").map_err(|e| format!("读取系统用户失败: {e}"))?;
        let user_c = CString::new(user).map_err(|_| "系统用户名包含非法字符".to_string())?;
        let pass_c = CString::new(password).map_err(|_| "密码包含非法字符".to_string())?;
        let mut conv = PamConv {
            conv: Some(conversation),
            appdata_ptr: pass_c.as_ptr() as *mut c_void,
        };

        for service in ["login", "system-auth", "common-auth"] {
            let service_c = CString::new(service).map_err(|_| "PAM 服务名非法".to_string())?;
            let mut handle: *mut PamHandle = ptr::null_mut();
            let start_status = unsafe {
                pam_start(
                    service_c.as_ptr(),
                    user_c.as_ptr(),
                    &conv,
                    &mut handle as *mut *mut PamHandle,
                )
            };
            if start_status != PAM_SUCCESS {
                continue;
            }

            let auth_status = unsafe { pam_authenticate(handle, 0) };
            if auth_status != PAM_SUCCESS {
                unsafe {
                    pam_end(handle, auth_status);
                }
                continue;
            }

            let acct_status = unsafe { pam_acct_mgmt(handle, 0) };
            unsafe {
                pam_end(handle, acct_status);
            }
            if acct_status == PAM_SUCCESS {
                return Ok(());
            }
        }

        Err("设备开机密码错误".to_string())
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn verify_device_login_password(password: &str) -> Result<(), String> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

    type Handle = isize;
    type Bool = i32;
    type Dword = u32;

    const LOGON32_LOGON_INTERACTIVE: Dword = 2;
    const LOGON32_PROVIDER_DEFAULT: Dword = 0;

    #[link(name = "Advapi32")]
    unsafe extern "system" {
        fn LogonUserW(
            user_name: *const u16,
            domain: *const u16,
            password: *const u16,
            logon_type: Dword,
            logon_provider: Dword,
            token: *mut Handle,
        ) -> Bool;
    }

    #[link(name = "Kernel32")]
    unsafe extern "system" {
        fn CloseHandle(handle: Handle) -> Bool;
    }

    fn wide_null(input: &str) -> Vec<u16> {
        OsStr::new(input).encode_wide().chain(once(0)).collect()
    }

    let username = std::env::var("USERNAME").map_err(|e| format!("读取系统用户失败: {e}"))?;
    let domain = std::env::var("USERDOMAIN").unwrap_or_else(|_| ".".to_string());
    let username_w = wide_null(&username);
    let domain_w = wide_null(&domain);
    let password_w = wide_null(password);

    let mut token: Handle = 0;
    let ok = unsafe {
        LogonUserW(
            username_w.as_ptr(),
            domain_w.as_ptr(),
            password_w.as_ptr(),
            LOGON32_LOGON_INTERACTIVE,
            LOGON32_PROVIDER_DEFAULT,
            &mut token as *mut Handle,
        )
    };
    if ok != 0 {
        unsafe {
            let _ = CloseHandle(token);
        }
        return Ok(());
    }

    let dot_w = wide_null(".");
    let mut token_fallback: Handle = 0;
    let ok_fallback = unsafe {
        LogonUserW(
            username_w.as_ptr(),
            dot_w.as_ptr(),
            password_w.as_ptr(),
            LOGON32_LOGON_INTERACTIVE,
            LOGON32_PROVIDER_DEFAULT,
            &mut token_fallback as *mut Handle,
        )
    };
    if ok_fallback != 0 {
        unsafe {
            let _ = CloseHandle(token_fallback);
        }
        return Ok(());
    }

    Err("设备开机密码错误".to_string())
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub(crate) fn verify_device_login_password(_password: &str) -> Result<(), String> {
    Err("当前操作系统暂不支持设备开机密码校验".to_string())
}

fn derive_key_from_password(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ROUNDS, &mut key);
    key
}

pub(crate) fn encrypt_secret_value(secret: &str, password: &str) -> Result<String, String> {
    let unlock = ensure_unlock_password(password)?;
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce);
    let key = derive_key_from_password(unlock, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建加密器失败: {e}"))?;
    let ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), secret.as_bytes())
        .map_err(|_| "私钥加密失败".to_string())?;
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
    let key = derive_key_from_password(unlock, &salt_arr);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("创建解密器失败: {e}"))?;
    let plain = cipher
        .decrypt(Nonce::from_slice(&nonce), cipher_bytes.as_ref())
        .map_err(|_| "解锁密码错误或密文已损坏".to_string())?;
    String::from_utf8(plain).map_err(|_| "私钥内容格式无效".to_string())
}
