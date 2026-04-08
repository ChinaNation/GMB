//! FFI bindings for smoldot-light
//!
//! This library provides C-compatible FFI exports for the smoldot-light
//! Rust library, enabling Dart applications to use a lightweight Substrate/Polkadot client.

use parking_lot::Mutex;
use serde_json::{Value, json};
use smoldot_light::{
    AddChainConfig, AddChainConfigJsonRpc, AddChainSuccess, ChainId, Client,
    JsonRpcResponses, platform::DefaultPlatform,
};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use once_cell::sync::Lazy;

mod error;
mod ffi_types;

use ffi_types::*;

/// Global registry of clients (handle-based for safety)
static CLIENTS: Lazy<Mutex<HashMap<ClientHandle, Arc<SmoldotClientWrapper>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Global registry of chains (handle-based for safety)
static CHAINS: Lazy<Mutex<HashMap<ChainHandle, Arc<SmoldotChainWrapper>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Wrapper around smoldot Client with interior mutability
struct SmoldotClientWrapper {
    client: Mutex<Client<Arc<DefaultPlatform>, ()>>,
    runtime: tokio::runtime::Runtime,
}

/// Wrapper around Chain and ChainId
struct SmoldotChainWrapper {
    chain_id: ChainId,
    client_handle: ClientHandle,
    raw_json_rpc_responses:
        Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<String>>>,
    pending_native_requests:
        Arc<tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<String>>>>,
    next_native_request_id: AtomicU64,
}

const SYSTEM_ACCOUNT_PREFIX_HEX: &str =
    "26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9";
const NATIVE_RPC_TIMEOUT_SECS: u64 = 30;

/// Initialize a new smoldot client
///
/// # Safety
/// - `config_json` must be a valid null-terminated UTF-8 string
/// - Returns 0 on failure
#[no_mangle]
pub unsafe extern "C" fn smoldot_client_init(
    config_json: *const c_char,
    error_out: *mut *mut c_char,
) -> ClientHandle {
    if config_json.is_null() {
        set_error(error_out, "config_json is null");
        return 0;
    }

    let config_str = match CStr::from_ptr(config_json).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in config_json");
            return 0;
        }
    };

    let config: ClientConfigJson = match serde_json::from_str(config_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(error_out, &format!("Failed to parse config: {}", e));
            return 0;
        }
    };

    // 初始化 Android 日志（仅首次调用生效），使 smoldot 内部日志输出到 logcat。
    #[cfg(target_os = "android")]
    {
        let _ = android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Trace)
                .with_tag("smoldot"),
        );
    }
    // 非 Android 平台使用 env_logger
    #[cfg(not(target_os = "android"))]
    {
        let _ = env_logger::try_init();
    }

    // Create Tokio runtime
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            set_error(error_out, &format!("Failed to create runtime: {}", e));
            return 0;
        }
    };

    // Get system name and version
    let system_name = config.system_name.unwrap_or_else(|| "Polkadart".to_string());
    let system_version = config.system_version.unwrap_or_else(|| "0.1.0".to_string());

    // Initialize smoldot client (Client::new wraps platform in Arc internally)
    let platform = DefaultPlatform::new(
        system_name.into(),
        system_version.into(),
    );

    let client = Client::new(platform);

    let wrapper = Arc::new(SmoldotClientWrapper {
        client: Mutex::new(client),
        runtime,
    });

    // Generate handle
    let handle = generate_client_handle();

    // Store in registry
    CLIENTS.lock().insert(handle, wrapper);

    handle
}

/// Add a chain to the client
///
/// # Safety
/// - `client_handle` must be a valid handle returned from `smoldot_client_init`
/// - `chain_spec_json` must be a valid null-terminated UTF-8 string
/// - `callback` must be a valid function pointer
#[no_mangle]
pub unsafe extern "C" fn smoldot_add_chain(
    client_handle: ClientHandle,
    chain_spec_json: *const c_char,
    potential_relay_chains: *const ChainHandle,
    relay_chains_count: c_int,
    database_content: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if chain_spec_json.is_null() {
        set_error(error_out, "chain_spec_json is null");
        return -1;
    }

    let chain_spec = match CStr::from_ptr(chain_spec_json).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in chain_spec_json");
            return -1;
        }
    };

    let db_content = if !database_content.is_null() {
        match CStr::from_ptr(database_content).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                set_error(error_out, "Invalid UTF-8 in database_content");
                return -1;
            }
        }
    } else {
        String::new()
    };

    // Get client from registry
    let client_wrapper = {
        let clients = CLIENTS.lock();
        match clients.get(&client_handle) {
            Some(c) => Arc::clone(c),
            None => {
                set_error(error_out, "Invalid client handle");
                return -1;
            }
        }
    };

    // Parse potential relay chains
    let relay_chains: Vec<ChainId> = if !potential_relay_chains.is_null() && relay_chains_count > 0 {
        let chains_slice = std::slice::from_raw_parts(
            potential_relay_chains,
            relay_chains_count as usize,
        );

        let chains_lock = CHAINS.lock();
        chains_slice
            .iter()
            .filter_map(|&handle| {
                chains_lock.get(&handle).map(|wrapper| wrapper.chain_id)
            })
            .collect()
    } else {
        Vec::new()
    };

    // Clone Arc to move into async block
    let client_wrapper_clone = Arc::clone(&client_wrapper);

    // Spawn async task to add chain
    client_wrapper.runtime.spawn(async move {
        let config = AddChainConfig {
            specification: &chain_spec,
            json_rpc: AddChainConfigJsonRpc::Enabled {
                max_pending_requests: std::num::NonZeroU32::new(128).unwrap(),
                max_subscriptions: 1024,
            },
            potential_relay_chains: relay_chains.into_iter(),
            database_content: &db_content,
            user_data: (),
        };

        // Get mutable access to client (add_chain is NOT async in 0.18.0)
        let add_result = {
            let mut client = client_wrapper_clone.client.lock();
            client.add_chain(config)
        };

        match add_result {
            Ok(AddChainSuccess {
                chain_id,
                json_rpc_responses,
            }) => {
                let (raw_tx, raw_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

                // Create chain wrapper
                let chain_wrapper = Arc::new(SmoldotChainWrapper {
                    chain_id,
                    client_handle,
                    raw_json_rpc_responses:
                        Arc::new(tokio::sync::Mutex::new(raw_rx)),
                    pending_native_requests:
                        Arc::new(tokio::sync::Mutex::new(HashMap::new())),
                    next_native_request_id: AtomicU64::new(1),
                });
                let pending_native_requests =
                    Arc::clone(&chain_wrapper.pending_native_requests);

                if let Some(json_rpc_responses) = json_rpc_responses {
                    client_wrapper_clone.runtime.spawn(async move {
                        forward_json_rpc_responses(
                            json_rpc_responses,
                            raw_tx,
                            pending_native_requests,
                        )
                        .await;
                    });
                } else {
                    let _ = raw_tx.send(
                        json!({
                            "jsonrpc": "2.0",
                            "error": {
                                "code": -32001,
                                "message": "JSON-RPC is disabled for this chain",
                            }
                        })
                        .to_string(),
                    );
                }

                // Generate handle
                let chain_handle = generate_chain_handle();

                // Store in registry
                CHAINS.lock().insert(chain_handle, chain_wrapper);

                // Invoke callback with success
                callback(callback_id, chain_handle as i64, std::ptr::null());
            }
            Err(e) => {
                // Create error message
                let error_msg = CString::new(format!("Failed to add chain: {:?}", e))
                    .unwrap_or_else(|_| CString::new("Unknown error").unwrap());
                callback(callback_id, 0, error_msg.as_ptr());
                // Note: error_msg is leaked here, Dart side must free it
                std::mem::forget(error_msg);
            }
        }
    });

    0 // Success (async operation started)
}

/// Send a JSON-RPC request to a chain
///
/// # Safety
/// - `chain_handle` must be a valid handle
/// - `request_json` must be a valid null-terminated UTF-8 string
#[no_mangle]
pub unsafe extern "C" fn smoldot_send_json_rpc(
    chain_handle: ChainHandle,
    request_json: *const c_char,
    error_out: *mut *mut c_char,
) -> c_int {
    if request_json.is_null() {
        set_error(error_out, "request_json is null");
        return -1;
    }

    let request = match CStr::from_ptr(request_json).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in request_json");
            return -1;
        }
    };

    // Get chain from registry
    let chain_wrapper = {
        let chains = CHAINS.lock();
        match chains.get(&chain_handle) {
            Some(c) => Arc::clone(c),
            None => {
                set_error(error_out, "Invalid chain handle");
                return -1;
            }
        }
    };

    // Get client
    let client_wrapper = {
        let clients = CLIENTS.lock();
        match clients.get(&chain_wrapper.client_handle) {
            Some(c) => Arc::clone(c),
            None => {
                set_error(error_out, "Invalid client handle");
                return -1;
            }
        }
    };

    // Send JSON-RPC request (needs mutable access)
    let mut client = client_wrapper.client.lock();
    match client.json_rpc_request(&request, chain_wrapper.chain_id) {
        Ok(_) => 0,
        Err(e) => {
            set_error(error_out, &format!("JSON-RPC error: {:?}", e));
            -1
        }
    }
}

/// Get next JSON-RPC response from a chain (blocking)
///
/// # Safety
/// - `chain_handle` must be a valid handle
/// - `callback` must be a valid function pointer
/// - Caller must free the returned string with `smoldot_free_string`
#[no_mangle]
pub unsafe extern "C" fn smoldot_next_json_rpc_response(
    chain_handle: ChainHandle,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    // Get chain from registry
    let chain_wrapper = {
        let chains = CHAINS.lock();
        match chains.get(&chain_handle) {
            Some(c) => Arc::clone(c),
            None => {
                set_error(error_out, "Invalid chain handle");
                return -1;
            }
        }
    };

    // Get client
    let client_wrapper = {
        let clients = CLIENTS.lock();
        match clients.get(&chain_wrapper.client_handle) {
            Some(c) => Arc::clone(c),
            None => {
                set_error(error_out, "Invalid client handle");
                return -1;
            }
        }
    };

    let raw_responses_arc = Arc::clone(&chain_wrapper.raw_json_rpc_responses);

    // Spawn async task
    client_wrapper.runtime.spawn(async move {
        let response_result = {
            let mut responses_lock = raw_responses_arc.lock().await;
            match responses_lock.recv().await {
                Some(response) => Ok(response),
                None => Err("Channel closed"),
            }
        };

        match response_result {
            Ok(response) => {
                // Convert response to C string
                let response_cstr = CString::new(response)
                    .unwrap_or_else(|_| CString::new("").unwrap());
                callback(callback_id, response_cstr.as_ptr() as i64, std::ptr::null());
                std::mem::forget(response_cstr); // Dart must free
            }
            Err(error_msg) => {
                let error_cstr = CString::new(error_msg).unwrap();
                callback(callback_id, 0, error_cstr.as_ptr());
                std::mem::forget(error_cstr);
            }
        }
    });

    0 // Success
}

/// Remove a chain from the client
///
/// # Safety
/// - `chain_handle` must be a valid handle
#[no_mangle]
pub unsafe extern "C" fn smoldot_remove_chain(
    chain_handle: ChainHandle,
    error_out: *mut *mut c_char,
) -> c_int {
    // Remove from registry
    let chain_wrapper = {
        let mut chains = CHAINS.lock();
        match chains.remove(&chain_handle) {
            Some(c) => c,
            None => {
                set_error(error_out, "Invalid chain handle");
                return -1;
            }
        }
    };

    // Get client
    let client_wrapper = {
        let clients = CLIENTS.lock();
        match clients.get(&chain_wrapper.client_handle) {
            Some(c) => Arc::clone(c),
            None => {
                set_error(error_out, "Invalid client handle");
                return -1;
            }
        }
    };

    // Remove chain from client (needs mutable access)
    let mut client = client_wrapper.client.lock();
    let _ = client.remove_chain(chain_wrapper.chain_id);

    0 // Success
}

/// Destroy a client and all its chains
///
/// # Safety
/// - `client_handle` must be a valid handle
/// - All chain handles for this client become invalid
#[no_mangle]
pub unsafe extern "C" fn smoldot_client_destroy(
    client_handle: ClientHandle,
    error_out: *mut *mut c_char,
) -> c_int {
    // Remove all chains for this client
    {
        let mut chains = CHAINS.lock();
        chains.retain(|_, wrapper| wrapper.client_handle != client_handle);
    }

    // Remove client from registry
    let mut clients = CLIENTS.lock();
    if clients.remove(&client_handle).is_none() {
        set_error(error_out, "Invalid client handle");
        return -1;
    }

    0 // Success
}

/// Free a string allocated by Rust
///
/// # Safety
/// - `ptr` must have been allocated by Rust via CString
#[no_mangle]
pub unsafe extern "C" fn smoldot_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}

/// Get the version of the smoldot FFI library
///
/// # Safety
/// - Returned string must be freed with `smoldot_free_string`
#[no_mangle]
pub unsafe extern "C" fn smoldot_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    CString::new(version)
        .unwrap_or_else(|_| CString::new("unknown").unwrap())
        .into_raw()
}

/// 获取轻节点状态快照，返回 JSON 字符串。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - 返回字符串需由 `smoldot_free_string` 释放
// 以下同步版本已废弃，请使用对应的 *_async 版本。
// 保留仅为编译兼容，后续将删除。

#[deprecated(note = "Use smoldot_get_status_snapshot_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_status_snapshot(
    chain_handle: ChainHandle,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match block_on_native_capability(chain_handle, |chain_wrapper, client_wrapper| async move {
        let snapshot_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_status_snapshot(chain_wrapper.chain_id)
                .map_err(|error| error.to_string())?
        };
        let snapshot = snapshot_future.await.map_err(|error| error.to_string())?;

        let snapshot = json!({
            "peerCount": snapshot.peer_count,
            "isSyncing": snapshot.is_syncing,
            "bestBlockNumber": snapshot.best_block_number,
            "bestBlockHash": format!("0x{}", hex::encode(snapshot.best_block_hash)),
            "finalizedBlockNumber": snapshot.finalized_block_number,
            "finalizedBlockHash": format!("0x{}", hex::encode(snapshot.finalized_block_hash)),
        });
        Ok(snapshot.to_string())
    }) {
        Ok(json_str) => string_into_raw(json_str, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 获取运行时版本，返回 JSON 字符串。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_runtime_version_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_runtime_version(
    chain_handle: ChainHandle,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match block_on_native_capability(chain_handle, |chain_wrapper, client_wrapper| async move {
        let snapshot_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_runtime_version_snapshot(chain_wrapper.chain_id)
                .map_err(|error| error.to_string())?
        };
        let runtime_version = snapshot_future.await.map_err(|error| error.to_string())?;
        let apis = runtime_version
            .apis
            .iter()
            .map(|(name_hash, version)| json!([format!("0x{}", hex::encode(name_hash)), *version]))
            .collect::<Vec<_>>();
        Ok(json!({
            "specName": runtime_version.spec_name,
            "implName": runtime_version.impl_name,
            "authoringVersion": runtime_version.authoring_version,
            "specVersion": runtime_version.spec_version,
            "implVersion": runtime_version.impl_version,
            "transactionVersion": runtime_version.transaction_version,
            "stateVersion": runtime_version.state_version,
            "apis": apis,
        })
        .to_string())
    }) {
        Ok(json_str) => string_into_raw(json_str, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 获取运行时 metadata hex。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_metadata_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_metadata(
    chain_handle: ChainHandle,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    match block_on_native_capability(chain_handle, |chain_wrapper, client_wrapper| async move {
        let metadata_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_metadata(chain_wrapper.chain_id)
                .map_err(|error| error.to_string())?
        };
        let metadata = metadata_future.await.map_err(|error| error.to_string())?;
        Ok(format!("0x{}", hex::encode(metadata)))
    }) {
        Ok(metadata_hex) => string_into_raw(metadata_hex, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 获取账户下一个可用 nonce，返回十进制字符串。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - `account_id_hex` 必须是合法的 32 字节 hex 字符串
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_account_next_index_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_account_next_index(
    chain_handle: ChainHandle,
    account_id_hex: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if account_id_hex.is_null() {
        set_error(error_out, "account_id_hex is null");
        return std::ptr::null_mut();
    }

    let account_id_hex = match CStr::from_ptr(account_id_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in account_id_hex");
            return std::ptr::null_mut();
        }
    };
    let account_id = match decode_account_id_hex(&account_id_hex) {
        Ok(bytes) => bytes,
        Err(message) => {
            set_error(error_out, &message);
            return std::ptr::null_mut();
        }
    };

    match block_on_native_capability(chain_handle, move |chain_wrapper, client_wrapper| async move {
        let next_index_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_account_next_index(chain_wrapper.chain_id, account_id)
                .map_err(|error| error.to_string())?
        };
        let next_index = next_index_future.await.map_err(|error| error.to_string())?;
        Ok(next_index.to_string())
    }) {
        Ok(next_index) => string_into_raw(next_index, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 获取指定块高的 block hash。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - `block_number` 必须是合法十进制字符串
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_block_hash_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_block_hash(
    chain_handle: ChainHandle,
    block_number: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if block_number.is_null() {
        set_error(error_out, "block_number is null");
        return std::ptr::null_mut();
    }

    let block_number = match CStr::from_ptr(block_number).to_str() {
        Ok(value) => value,
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in block_number");
            return std::ptr::null_mut();
        }
    };

    let block_number = match block_number.parse::<u64>() {
        Ok(value) => value,
        Err(error) => {
            set_error(error_out, &format!("Invalid block_number: {error}"));
            return std::ptr::null_mut();
        }
    };

    match block_on_native_capability(chain_handle, move |chain_wrapper, client_wrapper| async move {
        // 优先查本地缓存（快路径，无网络开销）。
        let known_block_hash_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_known_block_hash(chain_wrapper.chain_id, block_number)
                .map_err(|error| error.to_string())?
        };

        if let Some(block_hash) = known_block_hash_future
            .await
            .map_err(|error| error.to_string())?
        {
            return Ok(format!("0x{}", hex::encode(block_hash)));
        }

        // 本地缓存未命中，回退到 JSON-RPC（通过 smoldot P2P 网络查询）。
        let result = native_json_rpc_request(
            Arc::clone(&chain_wrapper),
            Arc::clone(&client_wrapper),
            "chain_getBlockHash",
            json!([block_number]),
        )
        .await?;

        // 轻节点正常情况：finalized 之前的旧区块没在 smoldot 缓存里，
        // chain_getBlockHash 返回 null。把 null 当作"未知"返回空串，
        // 由 dart 层判定为 None，绝不抛错（否则 PendingTxReconciler 会
        // 对每个老区块号刷一条 non-string 错误日志，淹没真问题）。
        if result.is_null() {
            return Ok(String::new());
        }
        result
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!(
                "chain_getBlockHash returned non-string for height {block_number}: {result}"
            ))
    }) {
        Ok(block_hash) => string_into_raw(block_hash, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 获取指定区块中的 extrinsics 列表，返回 JSON 数组字符串。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - `block_hash_hex` 必须是合法 UTF-8
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_block_extrinsics_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_block_extrinsics(
    chain_handle: ChainHandle,
    block_hash_hex: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if block_hash_hex.is_null() {
        set_error(error_out, "block_hash_hex is null");
        return std::ptr::null_mut();
    }

    let block_hash_hex = match CStr::from_ptr(block_hash_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in block_hash_hex");
            return std::ptr::null_mut();
        }
    };
    let block_hash = match decode_prefixed_hex(&block_hash_hex) {
        Ok(bytes) => match <[u8; 32]>::try_from(bytes.as_slice()) {
            Ok(hash) => hash,
            Err(_) => {
                set_error(error_out, "block_hash_hex must decode to 32 bytes");
                return std::ptr::null_mut();
            }
        },
        Err(message) => {
            set_error(error_out, &message);
            return std::ptr::null_mut();
        }
    };

    match block_on_native_capability(chain_handle, move |chain_wrapper, client_wrapper| async move {
        let native_extrinsics_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_block_extrinsics(chain_wrapper.chain_id, block_hash)
                .map_err(|error| error.to_string())?
        };

        let values = match native_extrinsics_future.await {
            Ok(extrinsics) => extrinsics
                .into_iter()
                .map(|extrinsic| format!("0x{}", hex::encode(extrinsic)))
                .collect::<Vec<_>>(),
            Err(error) => {
                return Err(format!(
                    "Failed to download block body via light-client path for {block_hash_hex}: {error}"
                ))
            }
        };

        serde_json::to_string(&values)
            .map_err(|error| format!("Failed to encode block extrinsics JSON: {error}"))
    }) {
        Ok(extrinsics_json) => string_into_raw(extrinsics_json, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 提交已编码 extrinsic，返回交易哈希 hex。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - `extrinsic_hex` 必须是合法 UTF-8
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_submit_extrinsic_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_submit_extrinsic(
    chain_handle: ChainHandle,
    extrinsic_hex: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if extrinsic_hex.is_null() {
        set_error(error_out, "extrinsic_hex is null");
        return std::ptr::null_mut();
    }

    let extrinsic_hex = match CStr::from_ptr(extrinsic_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in extrinsic_hex");
            return std::ptr::null_mut();
        }
    };

    match block_on_native_capability(chain_handle, move |chain_wrapper, client_wrapper| async move {
        let result = native_json_rpc_request(
            Arc::clone(&chain_wrapper),
            Arc::clone(&client_wrapper),
            "author_submitExtrinsic",
            json!([extrinsic_hex]),
        )
        .await?;

        let tx_hash = result
            .as_str()
            .ok_or_else(|| "author_submitExtrinsic result is not a string".to_string())?;
        Ok(tx_hash.to_string())
    }) {
        Ok(tx_hash) => string_into_raw(tx_hash, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 读取 `System.Account`，返回 JSON 字符串。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - `account_id_hex` 必须是合法的 32 字节 hex 字符串
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_system_account_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_system_account(
    chain_handle: ChainHandle,
    account_id_hex: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if account_id_hex.is_null() {
        set_error(error_out, "account_id_hex is null");
        return std::ptr::null_mut();
    }

    let account_id_hex = match CStr::from_ptr(account_id_hex).to_str() {
        Ok(value) => value,
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in account_id_hex");
            return std::ptr::null_mut();
        }
    };

    let account_id = match decode_account_id_hex(account_id_hex) {
        Ok(bytes) => bytes,
        Err(message) => {
            set_error(error_out, &message);
            return std::ptr::null_mut();
        }
    };

    match block_on_native_capability(chain_handle, move |chain_wrapper, client_wrapper| async move {
        let storage_key = build_system_account_storage_key(&account_id);
        let storage_key_bytes = decode_prefixed_hex(&storage_key)?;
        let native_storage_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_storage_values(chain_wrapper.chain_id, vec![storage_key_bytes])
                .map_err(|error| error.to_string())?
        };

        // 中文注释：余额/nonce 主路径已经切到原生 storage proof，这里不再回退 legacy `state_getStorage`。
        let storage_value_hex = native_storage_future
            .await
            .map_err(|error| error.to_string())?
            .pop()
            .flatten()
            .map(|value_bytes| format!("0x{}", hex::encode(value_bytes)));

        if storage_value_hex.is_none() {
            return Ok(json!({
                "storageKey": storage_key,
                "exists": false,
            })
            .to_string());
        }

        let value_hex = storage_value_hex.unwrap();
        let value_bytes = decode_prefixed_hex(&value_hex)?;

        let nonce = if value_bytes.len() >= 4 {
            Some(u32::from_le_bytes([
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
            ]) as u64)
        } else {
            None
        };
        let free_fen = if value_bytes.len() >= 32 {
            Some(read_u128_le_string(&value_bytes, 16)?)
        } else {
            None
        };

        Ok(json!({
            "storageKey": storage_key,
            "exists": true,
            "valueHex": value_hex,
            "nonce": nonce,
            "freeFen": free_fen,
        })
        .to_string())
    }) {
        Ok(json_str) => string_into_raw(json_str, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 读取任意 storage value，返回 JSON 字符串。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - `storage_key_hex` 必须是合法 hex 字符串
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_storage_value_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_storage_value(
    chain_handle: ChainHandle,
    storage_key_hex: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if storage_key_hex.is_null() {
        set_error(error_out, "storage_key_hex is null");
        return std::ptr::null_mut();
    }

    let storage_key_hex = match CStr::from_ptr(storage_key_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in storage_key_hex");
            return std::ptr::null_mut();
        }
    };
    let storage_key_bytes = match decode_prefixed_hex(&storage_key_hex) {
        Ok(bytes) => bytes,
        Err(message) => {
            set_error(error_out, &message);
            return std::ptr::null_mut();
        }
    };

    match block_on_native_capability(chain_handle, move |chain_wrapper, client_wrapper| async move {
        let native_storage_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_storage_values(chain_wrapper.chain_id, vec![storage_key_bytes])
                .map_err(|error| error.to_string())?
        };

        let storage_value = native_storage_future
            .await
            .map_err(|error| error.to_string())?
            .pop()
            .flatten();
        Ok(json_storage_value_response_from_bytes(&storage_key_hex, storage_value).to_string())
    }) {
        Ok(json_str) => string_into_raw(json_str, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

/// 批量读取多个 storage value，返回 JSON 对象字符串。
///
/// # Safety
/// - `chain_handle` 必须是有效链句柄
/// - `storage_keys_json` 必须是 JSON 数组字符串
/// - 返回字符串需由 `smoldot_free_string` 释放
#[deprecated(note = "Use smoldot_get_storage_values_async instead")]
#[no_mangle]
pub unsafe extern "C" fn smoldot_get_storage_values(
    chain_handle: ChainHandle,
    storage_keys_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if storage_keys_json.is_null() {
        set_error(error_out, "storage_keys_json is null");
        return std::ptr::null_mut();
    }

    let storage_keys_json = match CStr::from_ptr(storage_keys_json).to_str() {
        Ok(value) => value,
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in storage_keys_json");
            return std::ptr::null_mut();
        }
    };

    let storage_keys: Vec<String> = match serde_json::from_str(storage_keys_json) {
        Ok(value) => value,
        Err(error) => {
            set_error(
                error_out,
                &format!("Failed to parse storage_keys_json: {error}"),
            );
            return std::ptr::null_mut();
        }
    };

    match block_on_native_capability(chain_handle, move |chain_wrapper, client_wrapper| async move {
        let decoded_storage_keys = storage_keys
            .iter()
            .map(|storage_key_hex| decode_prefixed_hex(storage_key_hex))
            .collect::<Result<Vec<_>, _>>()?;
        let native_storage_future = {
            let client = client_wrapper.client.lock();
            client
                .chain_storage_values(chain_wrapper.chain_id, decoded_storage_keys)
                .map_err(|error| error.to_string())?
        };

        let native_values = native_storage_future
            .await
            .map_err(|error| error.to_string())?;
        let mut values = serde_json::Map::with_capacity(storage_keys.len());
        for (storage_key_hex, storage_value) in
            storage_keys.iter().zip(native_values.into_iter())
        {
            let value_hex = storage_value
                .map(|value_bytes| Value::String(format!("0x{}", hex::encode(value_bytes))))
                .unwrap_or(Value::Null);
            values.insert(storage_key_hex.clone(), value_hex);
        }

        Ok(Value::Object(values).to_string())
    }) {
        Ok(json_str) => string_into_raw(json_str, error_out),
        Err(message) => {
            set_error(error_out, &message);
            std::ptr::null_mut()
        }
    }
}

// ──── 异步 FFI 导出（不阻塞 Dart 主线程） ────

/// 异步回调辅助：在独立线程上执行 async 闭包，完成后通过 DartCallback 回调。
///
/// 使用 `std::thread::spawn` + `runtime.block_on` 而非 `tokio::spawn`，
/// 因为 smoldot 的部分原生 API 返回的 Future 没有 `Send` 约束。
/// 独立线程上的 `block_on` 不阻塞 Dart 主线程，同时兼容非 Send futures。
fn spawn_native_capability_async<F, Fut>(
    chain_handle: ChainHandle,
    callback_id: i64,
    callback: DartCallback,
    f: F,
) -> Result<(), String>
where
    F: FnOnce(Arc<SmoldotChainWrapper>, Arc<SmoldotClientWrapper>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    let chain_wrapper = get_chain_wrapper(chain_handle)?;
    let client_wrapper = get_client_wrapper(chain_wrapper.client_handle)?;
    std::thread::spawn(move || {
        let result = client_wrapper.runtime.block_on(f(
            Arc::clone(&chain_wrapper),
            Arc::clone(&client_wrapper),
        ));
        match result {
            Ok(json_str) => {
                let cstr = CString::new(json_str)
                    .unwrap_or_else(|_| CString::new("{}").unwrap());
                unsafe { callback(callback_id, cstr.as_ptr() as i64, std::ptr::null()) };
                std::mem::forget(cstr);
            }
            Err(msg) => {
                let cstr = CString::new(msg)
                    .unwrap_or_else(|_| CString::new("Unknown error").unwrap());
                unsafe { callback(callback_id, 0, cstr.as_ptr()) };
                std::mem::forget(cstr);
            }
        }
    });
    Ok(())
}

/// 异步 FFI 入口的错误处理宏：参数校验失败时设置 error_out 并返回 -1。
macro_rules! async_ffi_entry {
    ($chain_handle:expr, $callback_id:expr, $callback:expr, $error_out:expr, $body:expr) => {
        match spawn_native_capability_async($chain_handle, $callback_id, $callback, $body) {
            Ok(()) => 0,
            Err(message) => {
                set_error($error_out, &message);
                -1
            }
        }
    };
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_status_snapshot_async(
    chain_handle: ChainHandle,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        |chain_wrapper, client_wrapper| async move {
            let snapshot_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_status_snapshot(chain_wrapper.chain_id)
                    .map_err(|error| error.to_string())?
            };
            let snapshot = snapshot_future.await.map_err(|error| error.to_string())?;
            Ok(json!({
                "peerCount": snapshot.peer_count,
                "isSyncing": snapshot.is_syncing,
                "bestBlockNumber": snapshot.best_block_number,
                "bestBlockHash": format!("0x{}", hex::encode(snapshot.best_block_hash)),
                "finalizedBlockNumber": snapshot.finalized_block_number,
                "finalizedBlockHash": format!("0x{}", hex::encode(snapshot.finalized_block_hash)),
            })
            .to_string())
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_runtime_version_async(
    chain_handle: ChainHandle,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        |chain_wrapper, client_wrapper| async move {
            let snapshot_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_runtime_version_snapshot(chain_wrapper.chain_id)
                    .map_err(|error| error.to_string())?
            };
            let runtime_version = snapshot_future.await.map_err(|error| error.to_string())?;
            let apis = runtime_version
                .apis
                .iter()
                .map(|(name_hash, version)| json!([format!("0x{}", hex::encode(name_hash)), *version]))
                .collect::<Vec<_>>();
            Ok(json!({
                "specName": runtime_version.spec_name,
                "implName": runtime_version.impl_name,
                "authoringVersion": runtime_version.authoring_version,
                "specVersion": runtime_version.spec_version,
                "implVersion": runtime_version.impl_version,
                "transactionVersion": runtime_version.transaction_version,
                "stateVersion": runtime_version.state_version,
                "apis": apis,
            })
            .to_string())
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_metadata_async(
    chain_handle: ChainHandle,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        |chain_wrapper, client_wrapper| async move {
            let metadata_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_metadata(chain_wrapper.chain_id)
                    .map_err(|error| error.to_string())?
            };
            let metadata = metadata_future.await.map_err(|error| error.to_string())?;
            Ok(format!("0x{}", hex::encode(metadata)))
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_account_next_index_async(
    chain_handle: ChainHandle,
    account_id_hex: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if account_id_hex.is_null() {
        set_error(error_out, "account_id_hex is null");
        return -1;
    }
    let account_id_hex = match CStr::from_ptr(account_id_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in account_id_hex");
            return -1;
        }
    };
    let account_id = match decode_account_id_hex(&account_id_hex) {
        Ok(bytes) => bytes,
        Err(message) => {
            set_error(error_out, &message);
            return -1;
        }
    };

    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        move |chain_wrapper, client_wrapper| async move {
            let next_index_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_account_next_index(chain_wrapper.chain_id, account_id)
                    .map_err(|error| error.to_string())?
            };
            let next_index = next_index_future.await.map_err(|error| error.to_string())?;
            Ok(next_index.to_string())
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_block_hash_async(
    chain_handle: ChainHandle,
    block_number: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if block_number.is_null() {
        set_error(error_out, "block_number is null");
        return -1;
    }
    let block_number = match CStr::from_ptr(block_number).to_str() {
        Ok(value) => value,
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in block_number");
            return -1;
        }
    };
    let block_number = match block_number.parse::<u64>() {
        Ok(value) => value,
        Err(error) => {
            set_error(error_out, &format!("Invalid block_number: {error}"));
            return -1;
        }
    };

    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        move |chain_wrapper, client_wrapper| async move {
            let known_block_hash_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_known_block_hash(chain_wrapper.chain_id, block_number)
                    .map_err(|error| error.to_string())?
            };
            if let Some(block_hash) = known_block_hash_future
                .await
                .map_err(|error| error.to_string())?
            {
                return Ok(format!("0x{}", hex::encode(block_hash)));
            }
            let result = native_json_rpc_request(
                Arc::clone(&chain_wrapper),
                Arc::clone(&client_wrapper),
                "chain_getBlockHash",
                json!([block_number]),
            )
            .await?;
            // 中文注释：轻节点正常情况——finalized 之前的旧区块没在 smoldot
            // 缓存里，chain_getBlockHash 返回 null。把 null 当作"未知"返回空串，
            // 由 dart 层判定为 None，绝不抛错（否则 PendingTxReconciler 会
            // 对每个老区块号刷一条 non-string 错误日志，淹没真问题）。
            if result.is_null() {
                return Ok(String::new());
            }
            result
                .as_str()
                .map(|s| s.to_string())
                .ok_or_else(|| format!(
                    "chain_getBlockHash returned non-string for height {block_number}: {result}"
                ))
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_block_extrinsics_async(
    chain_handle: ChainHandle,
    block_hash_hex: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if block_hash_hex.is_null() {
        set_error(error_out, "block_hash_hex is null");
        return -1;
    }
    let block_hash_hex = match CStr::from_ptr(block_hash_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in block_hash_hex");
            return -1;
        }
    };
    let block_hash = match decode_prefixed_hex(&block_hash_hex) {
        Ok(bytes) => match <[u8; 32]>::try_from(bytes.as_slice()) {
            Ok(hash) => hash,
            Err(_) => {
                set_error(error_out, "block_hash_hex must decode to 32 bytes");
                return -1;
            }
        },
        Err(message) => {
            set_error(error_out, &message);
            return -1;
        }
    };

    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        move |chain_wrapper, client_wrapper| async move {
            let native_extrinsics_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_block_extrinsics(chain_wrapper.chain_id, block_hash)
                    .map_err(|error| error.to_string())?
            };
            let values = match native_extrinsics_future.await {
                Ok(extrinsics) => extrinsics
                    .into_iter()
                    .map(|extrinsic| format!("0x{}", hex::encode(extrinsic)))
                    .collect::<Vec<_>>(),
                Err(error) => {
                    return Err(format!(
                        "Failed to download block body for {block_hash_hex}: {error}"
                    ))
                }
            };
            serde_json::to_string(&values)
                .map_err(|error| format!("Failed to encode block extrinsics JSON: {error}"))
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_submit_extrinsic_async(
    chain_handle: ChainHandle,
    extrinsic_hex: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if extrinsic_hex.is_null() {
        set_error(error_out, "extrinsic_hex is null");
        return -1;
    }
    let extrinsic_hex = match CStr::from_ptr(extrinsic_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in extrinsic_hex");
            return -1;
        }
    };

    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        move |chain_wrapper, client_wrapper| async move {
            let result = native_json_rpc_request(
                Arc::clone(&chain_wrapper),
                Arc::clone(&client_wrapper),
                "author_submitExtrinsic",
                json!([extrinsic_hex]),
            )
            .await?;
            let tx_hash = result
                .as_str()
                .ok_or_else(|| "author_submitExtrinsic result is not a string".to_string())?;
            Ok(tx_hash.to_string())
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_system_account_async(
    chain_handle: ChainHandle,
    account_id_hex: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if account_id_hex.is_null() {
        set_error(error_out, "account_id_hex is null");
        return -1;
    }
    let account_id_hex = match CStr::from_ptr(account_id_hex).to_str() {
        Ok(value) => value,
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in account_id_hex");
            return -1;
        }
    };
    let account_id = match decode_account_id_hex(account_id_hex) {
        Ok(bytes) => bytes,
        Err(message) => {
            set_error(error_out, &message);
            return -1;
        }
    };

    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        move |chain_wrapper, client_wrapper| async move {
            let storage_key = build_system_account_storage_key(&account_id);
            let storage_key_bytes = decode_prefixed_hex(&storage_key)?;
            let native_storage_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_storage_values(chain_wrapper.chain_id, vec![storage_key_bytes])
                    .map_err(|error| error.to_string())?
            };
            let storage_value_hex = native_storage_future
                .await
                .map_err(|error| error.to_string())?
                .pop()
                .flatten()
                .map(|value_bytes| format!("0x{}", hex::encode(value_bytes)));

            if storage_value_hex.is_none() {
                return Ok(json!({
                    "storageKey": storage_key,
                    "exists": false,
                })
                .to_string());
            }

            let value_hex = storage_value_hex.unwrap();
            let value_bytes = decode_prefixed_hex(&value_hex)?;
            let nonce = if value_bytes.len() >= 4 {
                Some(u32::from_le_bytes([
                    value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3],
                ]) as u64)
            } else {
                None
            };
            let free_fen = if value_bytes.len() >= 32 {
                Some(read_u128_le_string(&value_bytes, 16)?)
            } else {
                None
            };

            Ok(json!({
                "storageKey": storage_key,
                "exists": true,
                "valueHex": value_hex,
                "nonce": nonce,
                "freeFen": free_fen,
            })
            .to_string())
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_storage_value_async(
    chain_handle: ChainHandle,
    storage_key_hex: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if storage_key_hex.is_null() {
        set_error(error_out, "storage_key_hex is null");
        return -1;
    }
    let storage_key_hex = match CStr::from_ptr(storage_key_hex).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in storage_key_hex");
            return -1;
        }
    };
    let storage_key_bytes = match decode_prefixed_hex(&storage_key_hex) {
        Ok(bytes) => bytes,
        Err(message) => {
            set_error(error_out, &message);
            return -1;
        }
    };

    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        move |chain_wrapper, client_wrapper| async move {
            let native_storage_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_storage_values(chain_wrapper.chain_id, vec![storage_key_bytes])
                    .map_err(|error| error.to_string())?
            };
            let storage_value = native_storage_future
                .await
                .map_err(|error| error.to_string())?
                .pop()
                .flatten();
            Ok(json_storage_value_response_from_bytes(&storage_key_hex, storage_value).to_string())
        }
    )
}

#[no_mangle]
pub unsafe extern "C" fn smoldot_get_storage_values_async(
    chain_handle: ChainHandle,
    storage_keys_json: *const c_char,
    callback_id: i64,
    callback: DartCallback,
    error_out: *mut *mut c_char,
) -> c_int {
    if storage_keys_json.is_null() {
        set_error(error_out, "storage_keys_json is null");
        return -1;
    }
    let storage_keys_json = match CStr::from_ptr(storage_keys_json).to_str() {
        Ok(value) => value.to_string(),
        Err(_) => {
            set_error(error_out, "Invalid UTF-8 in storage_keys_json");
            return -1;
        }
    };
    let storage_keys: Vec<String> = match serde_json::from_str(&storage_keys_json) {
        Ok(value) => value,
        Err(error) => {
            set_error(error_out, &format!("Failed to parse storage_keys_json: {error}"));
            return -1;
        }
    };

    async_ffi_entry!(chain_handle, callback_id, callback, error_out,
        move |chain_wrapper, client_wrapper| async move {
            let decoded_storage_keys = storage_keys
                .iter()
                .map(|storage_key_hex| decode_prefixed_hex(storage_key_hex))
                .collect::<Result<Vec<_>, _>>()?;
            let native_storage_future = {
                let client = client_wrapper.client.lock();
                client
                    .chain_storage_values(chain_wrapper.chain_id, decoded_storage_keys)
                    .map_err(|error| error.to_string())?
            };
            let native_values = native_storage_future
                .await
                .map_err(|error| error.to_string())?;
            let mut values = serde_json::Map::with_capacity(storage_keys.len());
            for (storage_key_hex, storage_value) in
                storage_keys.iter().zip(native_values.into_iter())
            {
                let value_hex = storage_value
                    .map(|value_bytes| Value::String(format!("0x{}", hex::encode(value_bytes))))
                    .unwrap_or(Value::Null);
                values.insert(storage_key_hex.clone(), value_hex);
            }
            Ok(Value::Object(values).to_string())
        }
    )
}

// Helper functions

fn generate_client_handle() -> ClientHandle {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn generate_chain_handle() -> ChainHandle {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

unsafe fn set_error(error_out: *mut *mut c_char, message: &str) {
    if !error_out.is_null() {
        let error_cstr = CString::new(message)
            .unwrap_or_else(|_| CString::new("Unknown error").unwrap());
        *error_out = error_cstr.into_raw();
    }
}

async fn forward_json_rpc_responses(
    mut responses: JsonRpcResponses<Arc<DefaultPlatform>>,
    raw_tx: tokio::sync::mpsc::UnboundedSender<String>,
    pending_native_requests: Arc<
        tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<String>>>,
    >,
) {
    while let Some(response) = responses.next().await {
        if !dispatch_native_response(&pending_native_requests, &response).await {
            if raw_tx.send(response).is_err() {
                break;
            }
        }
    }
}

async fn dispatch_native_response(
    pending_native_requests: &Arc<
        tokio::sync::Mutex<HashMap<String, tokio::sync::oneshot::Sender<String>>>,
    >,
    response: &str,
) -> bool {
    let Ok(json_value) = serde_json::from_str::<Value>(response) else {
        return false;
    };
    let Some(id_value) = json_value.get("id") else {
        return false;
    };
    let request_id = match id_value {
        Value::String(value) => value.clone(),
        _ => id_value.to_string(),
    };

    let sender = {
        let mut pending = pending_native_requests.lock().await;
        pending.remove(&request_id)
    };
    if let Some(sender) = sender {
        let _ = sender.send(response.to_string());
        return true;
    }
    false
}

/// 在 tokio runtime 上同步执行原生 capability 闭包。
///
/// # Safety (threading)
/// 必须从非 tokio 线程调用（即 Dart FFI 同步回调线程）。
/// 如果从 tokio runtime 内部调用会导致死锁。
fn block_on_native_capability<T, F, Fut>(
    chain_handle: ChainHandle,
    f: F,
) -> Result<T, String>
where
    F: FnOnce(Arc<SmoldotChainWrapper>, Arc<SmoldotClientWrapper>) -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let chain_wrapper = get_chain_wrapper(chain_handle)?;
    let client_wrapper = get_client_wrapper(chain_wrapper.client_handle)?;
    let runtime = &client_wrapper.runtime;
    runtime.block_on(f(chain_wrapper, Arc::clone(&client_wrapper)))
}

fn get_chain_wrapper(chain_handle: ChainHandle) -> Result<Arc<SmoldotChainWrapper>, String> {
    let chains = CHAINS.lock();
    chains
        .get(&chain_handle)
        .cloned()
        .ok_or_else(|| "Invalid chain handle".to_string())
}

fn get_client_wrapper(client_handle: ClientHandle) -> Result<Arc<SmoldotClientWrapper>, String> {
    let clients = CLIENTS.lock();
    clients
        .get(&client_handle)
        .cloned()
        .ok_or_else(|| "Invalid client handle".to_string())
}

async fn native_json_rpc_request(
    chain_wrapper: Arc<SmoldotChainWrapper>,
    client_wrapper: Arc<SmoldotClientWrapper>,
    method: &str,
    params: Value,
) -> Result<Value, String> {
    let request_id = format!(
        "__native_{}",
        chain_wrapper.next_native_request_id.fetch_add(1, Ordering::Relaxed)
    );
    let (sender, receiver) = tokio::sync::oneshot::channel::<String>();

    {
        let mut pending = chain_wrapper.pending_native_requests.lock().await;
        pending.insert(request_id.clone(), sender);
    }

    let request = json!({
        "id": request_id,
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
    })
    .to_string();

    {
        let mut client = client_wrapper.client.lock();
        client
            .json_rpc_request(request, chain_wrapper.chain_id)
            .map_err(|error| format!("JSON-RPC queue error for {method}: {error:?}"))?;
    }

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(NATIVE_RPC_TIMEOUT_SECS),
        receiver,
    )
    .await
    .map_err(|_| format!("JSON-RPC timeout for {method}"))?
    .map_err(|_| format!("JSON-RPC channel closed for {method}"))?;

    let response_json: Value = serde_json::from_str(&response)
        .map_err(|error| format!("Invalid JSON-RPC response for {method}: {error}"))?;
    if let Some(error) = response_json.get("error") {
        return Err(format!("JSON-RPC error for {method}: {error}"));
    }
    Ok(response_json.get("result").cloned().unwrap_or(Value::Null))
}

fn decode_account_id_hex(account_id_hex: &str) -> Result<Vec<u8>, String> {
    let bytes = decode_prefixed_hex(account_id_hex)?;
    if bytes.len() != 32 {
        return Err(format!(
            "account_id_hex must decode to 32 bytes, got {}",
            bytes.len()
        ));
    }
    Ok(bytes)
}

fn build_system_account_storage_key(account_id: &[u8]) -> String {
    let mut key = hex::decode(SYSTEM_ACCOUNT_PREFIX_HEX).unwrap_or_default();
    let blake2 = blake2_rfc::blake2b::blake2b(16, &[], account_id);
    key.extend_from_slice(blake2.as_bytes());
    key.extend_from_slice(account_id);
    format!("0x{}", hex::encode(key))
}

fn decode_prefixed_hex(value: &str) -> Result<Vec<u8>, String> {
    let clean = value.strip_prefix("0x").unwrap_or(value);
    hex::decode(clean).map_err(|error| format!("Invalid hex string: {error}"))
}

fn read_u128_le_string(bytes: &[u8], offset: usize) -> Result<String, String> {
    let slice = bytes
        .get(offset..offset + 16)
        .ok_or_else(|| "u128 slice out of bounds".to_string())?;
    let mut value = 0u128;
    for (index, byte) in slice.iter().enumerate() {
        value |= (*byte as u128) << (index * 8);
    }
    Ok(value.to_string())
}

fn string_into_raw(value: String, error_out: *mut *mut c_char) -> *mut c_char {
    match CString::new(value) {
        Ok(string) => string.into_raw(),
        Err(_) => {
            unsafe { set_error(error_out, "Failed to build response string") };
            std::ptr::null_mut()
        }
    }
}

fn json_storage_value_response_from_bytes(
    storage_key_hex: &str,
    storage_value: Option<Vec<u8>>,
) -> Value {
    match storage_value {
        Some(value_bytes) => json!({
            "storageKey": storage_key_hex,
            "exists": true,
            "valueHex": format!("0x{}", hex::encode(value_bytes)),
        }),
        None => json!({
            "storageKey": storage_key_hex,
            "exists": false,
        }),
    }
}
