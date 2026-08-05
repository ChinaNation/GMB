#ifndef SMOLDOT_H
#define SMOLDOT_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#define SmoldotGMB_SIGNER_OK 0

#define SmoldotGMB_SIGNER_ERR_NULL_ARG -1

#define SmoldotGMB_SIGNER_ERR_BAD_KEY -2

#define SmoldotGMB_SIGNER_ERR_BAD_SIGNATURE -3

#define SmoldotGMB_SIGNER_ERR_VERIFY_FAILED -4

#define SmoldotGMB_SIGNER_ERR_PANIC -5

/**
 * Opaque handle to a smoldot client
 */
typedef uint64_t SmoldotClientHandle;

/**
 * Opaque handle to a chain
 */
typedef uint64_t SmoldotChainHandle;

/**
 * Callback function type for async operations
 *
 * # Arguments
 * * `callback_id` - ID to match callback with request
 * * `result` - Result value (handle, string pointer, or 0 for error)
 * * `error` - Error message pointer (null if success)
 */
typedef void (*SmoldotDartCallback)(int64_t callback_id, int64_t result, const char *error);

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

/**
 * Initialize a new smoldot client
 *
 * # Safety
 * - `config_json` must be a valid null-terminated UTF-8 string
 * - Returns 0 on failure
 */
SmoldotClientHandle smoldot_client_init(const char *config_json, char **error_out);

/**
 * Add a chain to the client
 *
 * # Safety
 * - `client_handle` must be a valid handle returned from `smoldot_client_init`
 * - `chain_spec_json` must be a valid null-terminated UTF-8 string
 * - `callback` must be a valid function pointer
 */
int smoldot_add_chain(SmoldotClientHandle client_handle,
                      const char *chain_spec_json,
                      const SmoldotChainHandle *potential_relay_chains,
                      int relay_chains_count,
                      const char *database_content,
                      int64_t callback_id,
                      SmoldotDartCallback callback,
                      char **error_out);

/**
 * Send a JSON-RPC request to a chain
 *
 * # Safety
 * - `chain_handle` must be a valid handle
 * - `request_json` must be a valid null-terminated UTF-8 string
 */
int smoldot_send_json_rpc(SmoldotChainHandle chain_handle,
                          const char *request_json,
                          char **error_out);

/**
 * Get next JSON-RPC response from a chain (blocking)
 *
 * # Safety
 * - `chain_handle` must be a valid handle
 * - `callback` must be a valid function pointer
 * - Caller must free the returned string with `smoldot_free_string`
 */
int smoldot_next_json_rpc_response(SmoldotChainHandle chain_handle,
                                   int64_t callback_id,
                                   SmoldotDartCallback callback,
                                   char **error_out);

/**
 * Remove a chain from the client
 *
 * # Safety
 * - `chain_handle` must be a valid handle
 */
int smoldot_remove_chain(SmoldotChainHandle chain_handle, char **error_out);

/**
 * Destroy a client and all its chains
 *
 * # Safety
 * - `client_handle` must be a valid handle
 * - All chain handles for this client become invalid
 */
int smoldot_client_destroy(SmoldotClientHandle client_handle, char **error_out);

/**
 * Free a string allocated by Rust
 *
 * # Safety
 * - `ptr` must have been allocated by Rust via CString
 */
void smoldot_free_string(char *ptr);

/**
 * Get the version of the smoldot FFI library
 *
 * # Safety
 * - Returned string must be freed with `smoldot_free_string`
 */
char *smoldot_version(void);

/**
 * 异步读取轻节点状态快照；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_status_snapshot_async(SmoldotChainHandle chain_handle,
                                      int64_t callback_id,
                                      SmoldotDartCallback callback,
                                      char **error_out);

/**
 * 异步读取runtime 版本；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_runtime_version_async(SmoldotChainHandle chain_handle,
                                      int64_t callback_id,
                                      SmoldotDartCallback callback,
                                      char **error_out);

/**
 * 异步读取runtime metadata；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_metadata_async(SmoldotChainHandle chain_handle,
                               int64_t callback_id,
                               SmoldotDartCallback callback,
                               char **error_out);

/**
 * 异步读取账户下一个可用 nonce；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `account_id_hex` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_account_next_index_async(SmoldotChainHandle chain_handle,
                                         const char *account_id_hex,
                                         int64_t callback_id,
                                         SmoldotDartCallback callback,
                                         char **error_out);

/**
 * 异步读取指定高度的区块哈希；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `block_number` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_block_hash_async(SmoldotChainHandle chain_handle,
                                 const char *block_number,
                                 int64_t callback_id,
                                 SmoldotDartCallback callback,
                                 char **error_out);

/**
 * 异步读取指定区块的 extrinsic 列表；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `block_hash_hex` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_block_extrinsics_async(SmoldotChainHandle chain_handle,
                                       const char *block_hash_hex,
                                       int64_t callback_id,
                                       SmoldotDartCallback callback,
                                       char **error_out);

/**
 * 异步读取提交 extrinsic 到交易池；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `extrinsic_hex` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_submit_extrinsic_async(SmoldotChainHandle chain_handle,
                                   const char *extrinsic_hex,
                                   int64_t callback_id,
                                   SmoldotDartCallback callback,
                                   char **error_out);

/**
 * 异步读取最新块的 System::Account；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `account_id_hex` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_system_account_async(SmoldotChainHandle chain_handle,
                                     const char *account_id_hex,
                                     int64_t callback_id,
                                     SmoldotDartCallback callback,
                                     char **error_out);

/**
 * 异步读取finalized 的 System::Account；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `account_id_hex` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_finalized_system_account_async(SmoldotChainHandle chain_handle,
                                               const char *account_id_hex,
                                               int64_t callback_id,
                                               SmoldotDartCallback callback,
                                               char **error_out);

/**
 * 异步读取最新块的单个 storage 值；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `storage_key_hex` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_storage_value_async(SmoldotChainHandle chain_handle,
                                    const char *storage_key_hex,
                                    int64_t callback_id,
                                    SmoldotDartCallback callback,
                                    char **error_out);

/**
 * 异步读取finalized 的单个 storage 值；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `storage_key_hex` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_finalized_storage_value_async(SmoldotChainHandle chain_handle,
                                              const char *storage_key_hex,
                                              int64_t callback_id,
                                              SmoldotDartCallback callback,
                                              char **error_out);

/**
 * 异步读取最新块的批量 storage 值；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `storage_keys_json` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_storage_values_async(SmoldotChainHandle chain_handle,
                                     const char *storage_keys_json,
                                     int64_t callback_id,
                                     SmoldotDartCallback callback,
                                     char **error_out);

/**
 * 异步读取finalized 的批量 storage 值；结果经 `callback` 回传，本函数立即返回。
 *
 * # Safety
 * - `chain_handle` 必须来自 `smoldot_add_chain` 且未被 `smoldot_remove_chain` 释放；
 *   句柄失效会在运行期被检出并返回 -1，不构成 UB。
 * - `storage_keys_json` 必须是有效的 NUL 结尾字符串；空指针与非 UTF-8 已在函数内检出并返回 -1。
 * - `error_out` 允许为空；非空时必须是可写的 `*mut *mut c_char`。仅在返回 -1 时写入，
 *   写入的字符串所有权转移给调用方，必须用 `smoldot_free_string` 释放。
 * - `callback` 必须在异步操作完成前一直有效，且**会在原生 worker 线程上被调用**，
 *   不是调用本函数的那个线程。
 * - 回调拿到的 result 指针（成功路径，以 i64 传递）与 error 指针（失败路径）
 *   均转移所有权给调用方，两者都必须用 `smoldot_free_string` 释放，否则泄漏。
 */
int smoldot_get_finalized_storage_values_async(SmoldotChainHandle chain_handle,
                                               const char *storage_keys_json,
                                               int64_t callback_id,
                                               SmoldotDartCallback callback,
                                               char **error_out);

/**
 * 生成真实 OpenMLS KeyPackage，并以 JSON 返回 hex。
 *
 * # Safety
 * - `request_json` 必须是合法 UTF-8 C 字符串。
 * - 返回字符串必须由 `smoldot_free_string` 释放。
 */
char *gmb_chat_mls_create_key_package_json(const char *request_json, char **error_out);

/**
 * 执行真实 OpenMLS 双人组 round-trip smoke。
 *
 * # Safety
 * - `request_json` 必须是合法 UTF-8 C 字符串。
 * - 返回字符串必须由 `smoldot_free_string` 释放。
 */
char *gmb_chat_mls_two_party_smoke_json(const char *request_json, char **error_out);

/**
 * 使用持久化 MLS 会话加密 application message。
 *
 * # Safety
 * - `request_json` 必须是合法 UTF-8 C 字符串。
 * - 返回字符串必须由 `smoldot_free_string` 释放。
 */
char *gmb_chat_mls_encrypt_json(const char *request_json, char **error_out);

/**
 * 处理 Welcome 或解密 application message。
 *
 * # Safety
 * - `request_json` 必须是合法 UTF-8 C 字符串。
 * - 返回字符串必须由 `smoldot_free_string` 释放。
 */
char *gmb_chat_mls_decrypt_json(const char *request_json, char **error_out);

/**
 * 为 CID 钱包换绑暂存、提交或丢弃 MLS 状态的新账户密文。
 *
 * `stage` 只在内存解开此前密文并写旁路新账户密文；`commit` 在 finalized 后替换正式
 * 文件；`discard` 删除旁路文件。任何动作都不会把 OpenMLS 状态明文写盘。
 *
 * # Safety
 * - `request_json` 必须是合法 UTF-8 C 字符串。
 * - 返回字符串必须由 `smoldot_free_string` 释放。
 */
char *gmb_chat_mls_rekey_state_json(const char *request_json,
                                    char **error_out);

/**
 * 创建 MLS 群(创建者为唯一成员,epoch 0)。
 *
 * # Safety
 * 见 `gmb_chat_mls_create_key_package_json`。
 */
char *gmb_chat_mls_group_create_json(const char *request_json, char **error_out);

/**
 * 批量加人:产 1 个 Commit(发给现有成员)+ 1 个 Welcome(发给全部新人)。
 *
 * # Safety
 * 见 `gmb_chat_mls_create_key_package_json`。
 */
char *gmb_chat_mls_group_add_members_json(const char *request_json, char **error_out);

/**
 * 删人:产 Commit(发给剩余成员 + 被删者)。
 *
 * # Safety
 * 见 `gmb_chat_mls_create_key_package_json`。
 */
char *gmb_chat_mls_group_remove_members_json(const char *request_json, char **error_out);

/**
 * 群 application message:单次加密,Dart 侧按名册扇 N 信封。
 *
 * # Safety
 * 见 `gmb_chat_mls_create_key_package_json`。
 */
char *gmb_chat_mls_group_create_message_json(const char *request_json, char **error_out);

/**
 * 处理入站群消息(Welcome / Commit / Application)。收端唯一入口,按 epoch 判定
 * applied / out_of_order / stale,乱序缓冲由 Dart 依此状态负责。
 *
 * # Safety
 * 见 `gmb_chat_mls_create_key_package_json`。
 */
char *gmb_chat_mls_group_process_json(const char *request_json, char **error_out);

/**
 * 只读群状态:当前 epoch + 成员名册(MLS 真源,供 Dart 镜像对账与上限守)。
 *
 * # Safety
 * 见 `gmb_chat_mls_create_key_package_json`。
 */
char *gmb_chat_mls_group_state_json(const char *request_json, char **error_out);

/**
 * 从 32 字节母种子按 `chain_code` 硬派生一层 child mini-secret（32 字节）。
 *
 * 等价于 Dart 侧 `MiniSecretKey.fromRawKey(seed).expandEd25519()
 * .hardDeriveMiniSecretKey(const <int>[], cc)`；多层派生由调用方按 junction
 * 顺序逐层调用（每层的输入是上一层的输出），与 Dart 循环逐字节一致。
 *
 * # Safety
 * `seed`/`chain_code` 须各指向 32 字节可读内存，`out_child` 指向 32 字节可写内存。
 */
int32_t gmb_sr25519_derive_hard(const uint8_t *seed,
                                const uint8_t *chain_code,
                                uint8_t *out_child);

/**
 * child mini-secret（32 字节）→ 公钥（32 字节，即 AccountId32）。
 *
 * 等价于 Dart 侧 `Keyring.sr25519.fromSeed(child).bytes()`。
 *
 * # Safety
 * `child` 指向 32 字节可读内存，`out_public` 指向 32 字节可写内存。
 */
int32_t gmb_sr25519_public_key(const uint8_t *child, uint8_t *out_public);

/**
 * 用 child mini-secret 对 `message` 签名，输出 64 字节签名。
 *
 * 等价于 Dart 侧 `Keyring.sr25519.fromSeed(child).sign(message)`。sr25519 签名含
 * 随机数，**同一输入两次签名字节不同**（正常），只能靠验签比对，不能比字节。
 *
 * # Safety
 * `child` 指向 32 字节可读内存；`message` 指向 `message_len` 字节可读内存
 * （`message_len` 为 0 时允许空指针）；`out_signature` 指向 64 字节可写内存。
 */
int32_t gmb_sr25519_sign(const uint8_t *child,
                         const uint8_t *message,
                         uintptr_t message_len,
                         uint8_t *out_signature);

/**
 * 验签：公钥（32B）+ 签名（64B）+ 消息。通过返回 [`GMB_SIGNER_OK`]。
 *
 * # Safety
 * `public`/`signature` 分别指向 32/64 字节可读内存；`message` 指向 `message_len`
 * 字节可读内存（`message_len` 为 0 时允许空指针）。
 */
int32_t gmb_sr25519_verify(const uint8_t *public_,
                           const uint8_t *signature,
                           const uint8_t *message,
                           uintptr_t message_len);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SMOLDOT_H */
