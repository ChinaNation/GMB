use crate::governance::signing::encode_compact_u32;

const RUNTIME_UPGRADE_PALLET_INDEX: u8 = 12;
const PROPOSE_RUNTIME_UPGRADE_CALL_INDEX: u8 = 0;
const DEVELOPER_DIRECT_UPGRADE_CALL_INDEX: u8 = 2;
const MAX_WASM_BYTES: usize = 5 * 1_048_576;

/// 读取并校验 Runtime WASM 文件。
pub(crate) fn read_wasm(wasm_path: &str) -> Result<(Vec<u8>, f64), String> {
    let wasm_code = std::fs::read(wasm_path).map_err(|e| format!("读取 WASM 文件失败: {e}"))?;
    if wasm_code.is_empty() {
        return Err("WASM 文件为空".to_string());
    }
    let wasm_size_mb = wasm_code.len() as f64 / 1_048_576.0;
    if wasm_code.len() > MAX_WASM_BYTES {
        return Err(format!("WASM 文件超过 5MB 上限，当前 {wasm_size_mb:.2} MB"));
    }
    Ok((wasm_code, wasm_size_mb))
}

/// 构建开发期直接升级 call_data: RuntimeUpgrade.developer_direct_upgrade(code)。
pub(crate) fn developer_direct_upgrade(wasm_code: &[u8]) -> Vec<u8> {
    let wasm_len_compact = encode_compact_u32(wasm_code.len() as u32);
    let mut call_data = Vec::with_capacity(2 + wasm_len_compact.len() + wasm_code.len());
    call_data.push(RUNTIME_UPGRADE_PALLET_INDEX);
    call_data.push(DEVELOPER_DIRECT_UPGRADE_CALL_INDEX);
    call_data.extend_from_slice(&wasm_len_compact);
    call_data.extend_from_slice(wasm_code);
    call_data
}

/// 从文件重建开发期直接升级 call_data,用于签名响应提交阶段。
pub(crate) fn developer_direct_upgrade_from_file(wasm_path: &str) -> Result<Vec<u8>, String> {
    let (wasm_code, _) = read_wasm(wasm_path)?;
    Ok(developer_direct_upgrade(&wasm_code))
}

/// 构建运行期协议升级提案 call_data: RuntimeUpgrade.propose_runtime_upgrade(...)。
pub(crate) fn propose_runtime_upgrade(wasm_code: &[u8], reason: &str) -> Result<Vec<u8>, String> {
    let reason_bytes = reason.as_bytes();
    if reason_bytes.is_empty() {
        return Err("升级理由不能为空".to_string());
    }

    let reason_compact = encode_compact_u32(reason_bytes.len() as u32);
    let wasm_compact = encode_compact_u32(wasm_code.len() as u32);

    let mut call_data = Vec::with_capacity(
        2 + reason_compact.len() + reason_bytes.len() + wasm_compact.len() + wasm_code.len(),
    );
    call_data.push(RUNTIME_UPGRADE_PALLET_INDEX);
    call_data.push(PROPOSE_RUNTIME_UPGRADE_CALL_INDEX);
    call_data.extend_from_slice(&reason_compact);
    call_data.extend_from_slice(reason_bytes);
    call_data.extend_from_slice(&wasm_compact);
    call_data.extend_from_slice(wasm_code);
    Ok(call_data)
}

/// 从文件重建运行期协议升级提案 call_data。
pub(crate) fn propose_runtime_upgrade_from_file(
    wasm_path: &str,
    reason: &str,
) -> Result<Vec<u8>, String> {
    let (wasm_code, _) = read_wasm(wasm_path)?;
    propose_runtime_upgrade(&wasm_code, reason)
}
