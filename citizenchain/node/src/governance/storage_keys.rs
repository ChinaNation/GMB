// 链上存储 key 构造工具，用于查询 AdminsChange 等 pallet 的存储。
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

/// 计算 twox_64 哈希(Substrate `Twox64Concat` 第一层 key 用)。
pub fn twox_64(data: &[u8]) -> [u8; 8] {
    let mut h = XxHash64::with_seed(0);
    h.write(data);
    h.finish().to_le_bytes()
}

/// 计算 blake2b_128 哈希（Substrate StorageMap key 哈希）。
pub fn blake2b_128(data: &[u8]) -> [u8; 16] {
    let hash = Blake2bParams::new().hash_length(16).hash(data);
    let mut out = [0u8; 16];
    out.copy_from_slice(hash.as_bytes());
    out
}

/// 将 shenfen_id 字符串编码为固定 48 字节(kind tag 0x01 Builtin + payload 47B 右补零)。
///
/// D 阶段(SubjectKind 协议统一,2026-05-06)起,内置主体 subject_id 协议:
///   byte[0]   = 0x01 (SubjectKind::Builtin)
///   byte[1..48] = shenfen_id 字节(≤47B,右填零)
///
/// 与 `primitives::derive::subject_id_from_shenfen_id` 算法一致。
/// 节点 offline 计算 storage key 时直接复用此实现(node 不依赖 frame 类型,本地实现)。
pub fn subject_id_from_shenfen_id(shenfen_id: &str) -> [u8; 48] {
    let raw = shenfen_id.as_bytes();
    assert!(
        !raw.is_empty() && raw.len() <= 47,
        "shenfenId 长度必须在 1..47 字节(D 协议预留 1B kind tag),实际: {}",
        raw.len()
    );
    let mut out = [0u8; 48];
    out[0] = 0x01; // SubjectKind::Builtin
    out[1..1 + raw.len()].copy_from_slice(raw);
    out
}

/// 构造 `AdminsChange` 管理员主体表存储 key(hex 字符串含 0x 前缀)。
///
/// 链上 runtime 升级后 spec=1,但 admins-change migration v1 因门控 bug 没搬数据:
/// genesis 时 `STORAGE_VERSION = 2` 已生效,migration 见 on_chain >= 2 直接 noop。
/// 92 条管理员仍在 OLD 名 `Institutions` 路径下,subject_id 也是 48B 直接 raw padded
/// (没有 D 阶段加的 0x01 kind tag)。
///
/// 客户端按 OLD 路径读,可直接覆盖 国储会 + 省储会 + 省储行的全部管理员显示。
/// 不修 migration 的话,这一段就是该客户端读链的最终形态(数据不会自己搬走)。
pub fn admin_subjects_key(shenfen_id: &str) -> String {
    let mut subject_id = [0u8; 48];
    let raw = shenfen_id.as_bytes();
    let len = raw.len().min(48);
    subject_id[..len].copy_from_slice(&raw[..len]);

    let pallet_hash = twox_128(b"AdminsChange");
    let storage_hash = twox_128(b"Institutions");
    let blake2_hash = blake2b_128(&subject_id);

    let mut key = Vec::with_capacity(16 + 16 + 16 + 48);
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&blake2_hash);
    key.extend_from_slice(&subject_id);

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

/// 构造 `StorageDoubleMap<_, Twox64Concat, K1, Twox64Concat, K2, _>` 的
/// **前缀**(只到第一层 K1,不含第二层 K2),用于 `state_getKeysPaged` 列举:
///   twox_128(pallet) + twox_128(storage) + twox_64(K1) + K1
///
/// 对应 votingengine v1 的 `ProposalsByOrg / ByInstitution / ByOwner / ByYear`
/// 4 张反向索引的列举前缀。
pub fn twox64_concat_prefix(pallet: &str, storage: &str, key1: &[u8]) -> String {
    let pallet_hash = twox_128(pallet.as_bytes());
    let storage_hash = twox_128(storage.as_bytes());
    let twox64_k1 = twox_64(key1);

    let mut key = Vec::with_capacity(16 + 16 + 8 + key1.len());
    key.extend_from_slice(&pallet_hash);
    key.extend_from_slice(&storage_hash);
    key.extend_from_slice(&twox64_k1);
    key.extend_from_slice(key1);

    format!("0x{}", hex::encode(&key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn institution_id_from_shenfen_id_with_kind_tag() {
        let id = "GFR-LN001-CB0C-617776487-20260222";
        let fixed = subject_id_from_shenfen_id(id);
        // D 阶段:byte[0]=0x01 Builtin,byte[1..1+len]=shenfen_id bytes,余下零填充
        assert_eq!(fixed[0], 0x01);
        assert_eq!(&fixed[1..1 + id.len()], id.as_bytes());
        assert!(fixed[1 + id.len()..].iter().all(|&b| b == 0));
    }

    #[test]
    fn admin_subjects_key_has_correct_length() {
        let key = admin_subjects_key("GFR-LN001-CB0C-617776487-20260222");
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
