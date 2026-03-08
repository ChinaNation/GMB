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
    collections::{HashMap, VecDeque},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Manager};
use zeroize::{Zeroize, Zeroizing};

const KEYCHAIN_SERVICE: &str = "org.chinanation.citizenchain.desktop";
const SECRET_FORMAT_VERSION: u8 = 1;
const PBKDF2_ROUNDS: u32 = 390_000;
const AUTH_FAIL_WINDOW_SECS: u64 = 300;
const AUTH_MAX_FAILURES_IN_WINDOW: usize = 5;
const AUTH_BACKOFF_MS: u64 = 800;
const AUTH_BACKOFF_MAX_MS: u64 = 5000;

#[derive(Default)]
struct AuthRateLimitState {
    recent_failures: VecDeque<u64>,
}

static AUTH_RATE_LIMIT: OnceLock<Mutex<HashMap<String, AuthRateLimitState>>> = OnceLock::new();

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

fn validate_system_username(user: &str) -> Result<&str, String> {
    let trimmed = user.trim();
    if trimmed.is_empty() {
        return Err("系统用户名为空".to_string());
    }
    if trimmed.chars().any(|c| c.is_control()) {
        return Err("系统用户名包含控制字符".to_string());
    }
    Ok(trimmed)
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|v| v.as_secs())
        .unwrap_or(0)
}

fn enforce_auth_rate_limit(account: &str) -> Result<(), String> {
    let map = AUTH_RATE_LIMIT.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map
        .lock()
        .map_err(|_| "设备密码限速器状态异常".to_string())?;
    let state = guard.entry(account.to_string()).or_default();
    let now = now_unix_secs();
    while state
        .recent_failures
        .front()
        .is_some_and(|ts| now.saturating_sub(*ts) > AUTH_FAIL_WINDOW_SECS)
    {
        let _ = state.recent_failures.pop_front();
    }
    if state.recent_failures.len() >= AUTH_MAX_FAILURES_IN_WINDOW {
        return Err("设备密码尝试次数过多，请稍后再试".to_string());
    }
    Ok(())
}

fn record_auth_attempt(account: &str, success: bool) {
    let map = AUTH_RATE_LIMIT.get_or_init(|| Mutex::new(HashMap::new()));
    let mut sleep_delay_ms: u64 = 0;
    let Ok(mut guard) = map.lock() else {
        return;
    };
    let state = guard.entry(account.to_string()).or_default();
    let now = now_unix_secs();
    while state
        .recent_failures
        .front()
        .is_some_and(|ts| now.saturating_sub(*ts) > AUTH_FAIL_WINDOW_SECS)
    {
        let _ = state.recent_failures.pop_front();
    }
    if success {
        state.recent_failures.clear();
        return;
    }
    state.recent_failures.push_back(now);
    let over = state
        .recent_failures
        .len()
        .saturating_sub(AUTH_MAX_FAILURES_IN_WINDOW.saturating_sub(1));
    if over > 0 {
        let mut delay = AUTH_BACKOFF_MS.saturating_mul(over as u64);
        if delay > AUTH_BACKOFF_MAX_MS {
            delay = AUTH_BACKOFF_MAX_MS;
        }
        sleep_delay_ms = delay;
    }
    drop(guard);
    if sleep_delay_ms > 0 {
        thread::sleep(std::time::Duration::from_millis(sleep_delay_ms));
    }
}

pub(crate) fn append_audit_log(app: &AppHandle, action: &str, status: &str) -> Result<(), String> {
    let path = app_data_dir(app)?.join("security-audit.log");
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

#[cfg(target_os = "macos")]
pub(crate) fn verify_device_login_password(password: &str) -> Result<(), String> {
    let user = std::env::var("USER").map_err(|e| format!("读取系统用户失败: {e}"))?;
    let user = validate_system_username(&user)?;
    enforce_auth_rate_limit(user)?;
    let output = std::process::Command::new("dscl")
        .args(["/Search", "-authonly", user, password])
        .output()
        .map_err(|e| format!("校验设备密码失败: {e}"))?;
    if output.status.success() {
        record_auth_attempt(user, true);
        return Ok(());
    }
    record_auth_attempt(user, false);
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
                let len = libc::strlen((*response).resp);
                if len > 0 {
                    ptr::write_bytes((*response).resp as *mut u8, 0, len);
                }
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
        let user = super::validate_system_username(&user)?;
        super::enforce_auth_rate_limit(user)?;
        let user_c = CString::new(user).map_err(|_| "系统用户名包含非法字符".to_string())?;
        let mut pass_raw = CString::new(password)
            .map_err(|_| "密码包含非法字符".to_string())?
            .into_bytes_with_nul();
        let mut conv = PamConv {
            conv: Some(conversation),
            appdata_ptr: pass_raw.as_ptr() as *mut c_void,
        };
        let mut success = false;

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
                success = true;
                break;
            }
        }

        pass_raw.zeroize();
        super::record_auth_attempt(user_c.to_string_lossy().as_ref(), success);
        if success {
            Ok(())
        } else {
            Err("设备开机密码错误".to_string())
        }
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
    let username = validate_system_username(&username)?;
    enforce_auth_rate_limit(&username)?;
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
        record_auth_attempt(&username, true);
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
        record_auth_attempt(&username, true);
        unsafe {
            let _ = CloseHandle(token_fallback);
        }
        return Ok(());
    }

    record_auth_attempt(&username, false);
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
