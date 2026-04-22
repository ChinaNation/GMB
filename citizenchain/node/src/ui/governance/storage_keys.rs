// 链上存储 key 构造工具，用于查询 AdminsOriginGov 等 pallet 的存储。
//
// 格式：twox_128(pallet_name) + twox_128(storage_name) + blake2_128(key) + key

use blake2b_simd::Params as Blake2bParams;
use std::hash::Hasher;
use twox_hash::XxHash64;

/// 计算 twox_128 哈希（Substrate 存储前缀专用）。
pub fn twox_128(data: &[u8]) -> [u8; 16] {
    let mut h0 = XxHash64::with_seed(0);
    h0.write(data);
    let r0 = h0.finish();

    let mut h1 = XxHash64::with_seed(1);
    h1.write(data);
    let r1 = h1.finish();

    let mut out = [0u8; 16];
    out[..8].copy_from_slice(&r0.to_le_bytes());
    out[8..].copy_from_slice(&r1.to_le_bytes());
    out
}

/// 计算 blake2b_128 哈希（Substrate StorageMap key 哈希）。
pub fn blake2b_128(data: &[u8]) -> [u8; 16] {
    let hash = Blake2bParams::new().hash_length(16).hash(data);
    let mut out = [0u8; 16];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// 将 shenfen_id 字符串编码为固定 48 字节（UTF-8 右补零）。
/// 与 Rust runtime primitives 的 `shenfen_id_to_fixed48` 一致。
pub fn shenfen_id_to_fixed48(shenfen_id: &str) -> [u8; 48] {
    let raw = shenfen_id.as_bytes();
    assert!(
        !raw.is_empty() && raw.len() <= 48,
        "shenfenId 长度必须在 1..48 字节，实际: {}",
        raw.len()
    );
    let mut out = [0u8; 48];
    out[..raw.len()].copy_from_slice(raw);
    out
}

/// 构造 `AdminsOriginGov::CurrentAdmins(institution_id)` 的存储 key（hex 字符串含 0x 前缀）。
pub fn current_admins_key(shenfen_id: &str) -> String {
    let institution_id = shenfen_id_to_fixed48(shenfen_id);
    let pallet_hash = twox_128(b"AdminsOriginGov");
    let storage_hash = twox_128(b"CurrentAdmins");
    let blake2_hash = blake2b_128(&institution_id);

    let mut key = Vec::with_capacity(16 + 16 + 16 + 48);
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash);
    key.extend_from_slice(&institution_id);

    format!("0x{}", hex::encode(&key))
}

/// 构造查询账户余额的存储 key：`System::Account(account_id)`。
/// account_id 为 32 字节公钥（hex 不含 0x）。
pub fn system_account_key(account_hex: &str) -> Result<String, String> {
    let account_bytes = hex::decode(account_hex).map_err(|e| format!("解析账户地址失败: {e}"))?;
    if account_bytes.len() != 32 {
        return Err(format!(
            "账户公钥长度必须为 32 字节，实际: {}",
            account_bytes.len()
        ));
    }

    let pallet_hash = twox_128(b"System");
    let storage_hash = twox_128(b"Account");
    let blake2_hash = blake2b_128(&account_bytes);

    let mut key = Vec::with_capacity(16 + 16 + 16 + 32);
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash);
    key.extend_from_slice(&account_bytes);

    Ok(format!("0x{}", hex::encode(&key)))
}

/// 构造无 map key 的存储 value key：twox_128(pallet) + twox_128(storage)。
/// 用于查询 NextProposalId 等 StorageValue。
pub fn value_key(pallet: &str, storage: &str) -> String {
    let pallet_hash = twox_128(pallet.as_bytes());
    let storage_hash = twox_128(storage.as_bytes());
    let mut key = Vec::with_capacity(32);
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    format!("0x{}", hex::encode(&key))
}

/// 构造 StorageMap key：twox_128(pallet) + twox_128(storage) + blake2_128_concat(key_data)。
/// blake2_128_concat = blake2_128(data) + data。
pub fn map_key(pallet: &str, storage: &str, key_data: &[u8]) -> String {
    let pallet_hash = twox_128(pallet.as_bytes());
    let storage_hash = twox_128(storage.as_bytes());
    let blake2_hash = blake2b_128(key_data);

    let mut key = Vec::with_capacity(16 + 16 + 16 + key_data.len());
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash);
    key.extend_from_slice(key_data);

    format!("0x{}", hex::encode(&key))
}

/// 构造 DoubleMap key：twox_128(pallet) + twox_128(storage)
///   + blake2_128_concat(key1) + blake2_128_concat(key2)。
pub fn double_map_key(pallet: &str, storage: &str, key1: &[u8], key2: &[u8]) -> String {
    let pallet_hash = twox_128(pallet.as_bytes());
    let storage_hash = twox_128(storage.as_bytes());
    let blake2_hash1 = blake2b_128(key1);
    let blake2_hash2 = blake2b_128(key2);

    let mut key = Vec::with_capacity(16 + 16 + 16 + key1.len() + 16 + key2.len());
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash1);
    key.extend_from_slice(key1);
    key.extend_from_slice(&blake2_hash2);
    key.extend_from_slice(key2);

    format!("0x{}", hex::encode(&key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shenfen_id_to_fixed48_pads_correctly() {
        let id = "GFR-LN001-CB0C-617776487-20260222";
        let fixed = shenfen_id_to_fixed48(id);
        assert_eq!(&fixed[..id.len()], id.as_bytes());
        assert!(fixed[id.len()..].iter().all(|&b| b == 0));
    }

    #[test]
    fn current_admins_key_has_correct_length() {
        let key = current_admins_key("GFR-LN001-CB0C-617776487-20260222");
        // 0x 前缀 + (16+16+16+48)*2 hex 字符 = 2 + 192 = 194
        assert_eq!(key.len(), 194);
        assert!(key.starts_with("0x"));
    }

    #[test]
    fn system_account_key_has_correct_length() {
        let hex32 = "a5423e483bba281da84b99620a670718d5a7eceb5ae720f7d492e8b5c2570d84";
        let key = system_account_key(hex32).unwrap();
        // 0x 前缀 + (16+16+16+32)*2 hex 字符 = 2 + 160 = 162
        assert_eq!(key.len(), 162);
    }
}
