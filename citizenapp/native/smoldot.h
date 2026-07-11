#ifndef SMOLDOT_H
#define SMOLDOT_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

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

int smoldot_get_status_snapshot_async(SmoldotChainHandle chain_handle,
                                      int64_t callback_id,
                                      SmoldotDartCallback callback,
                                      char **error_out);

int smoldot_get_runtime_version_async(SmoldotChainHandle chain_handle,
                                      int64_t callback_id,
                                      SmoldotDartCallback callback,
                                      char **error_out);

int smoldot_get_metadata_async(SmoldotChainHandle chain_handle,
                               int64_t callback_id,
                               SmoldotDartCallback callback,
                               char **error_out);

int smoldot_get_account_next_index_async(SmoldotChainHandle chain_handle,
                                         const char *account_id_hex,
                                         int64_t callback_id,
                                         SmoldotDartCallback callback,
                                         char **error_out);

int smoldot_get_block_hash_async(SmoldotChainHandle chain_handle,
                                 const char *block_number,
                                 int64_t callback_id,
                                 SmoldotDartCallback callback,
                                 char **error_out);

int smoldot_get_block_extrinsics_async(SmoldotChainHandle chain_handle,
                                       const char *block_hash_hex,
                                       int64_t callback_id,
                                       SmoldotDartCallback callback,
                                       char **error_out);

int smoldot_submit_extrinsic_async(SmoldotChainHandle chain_handle,
                                   const char *extrinsic_hex,
                                   int64_t callback_id,
                                   SmoldotDartCallback callback,
                                   char **error_out);

int smoldot_get_system_account_async(SmoldotChainHandle chain_handle,
                                     const char *account_id_hex,
                                     int64_t callback_id,
                                     SmoldotDartCallback callback,
                                     char **error_out);

int smoldot_get_finalized_system_account_async(SmoldotChainHandle chain_handle,
                                               const char *account_id_hex,
                                               int64_t callback_id,
                                               SmoldotDartCallback callback,
                                               char **error_out);

int smoldot_get_storage_value_async(SmoldotChainHandle chain_handle,
                                    const char *storage_key_hex,
                                    int64_t callback_id,
                                    SmoldotDartCallback callback,
                                    char **error_out);

int smoldot_get_finalized_storage_value_async(SmoldotChainHandle chain_handle,
                                              const char *storage_key_hex,
                                              int64_t callback_id,
                                              SmoldotDartCallback callback,
                                              char **error_out);

int smoldot_get_storage_values_async(SmoldotChainHandle chain_handle,
                                     const char *storage_keys_json,
                                     int64_t callback_id,
                                     SmoldotDartCallback callback,
                                     char **error_out);

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

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#endif  /* SMOLDOT_H */
