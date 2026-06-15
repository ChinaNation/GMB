use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use super::endpoint::ImNodeEndpoint;

const MAX_KEYPACKAGES_PER_OWNER: usize = 512;
const MAX_KEYPACKAGE_HEX_LEN: usize = 128 * 1024;
const MAX_FETCH_LIMIT: u32 = 32;
pub(crate) const GMB_IM_KEYPACKAGE_PROTOCOL_VERSION: u16 = 1;

#[derive(Deserialize, Serialize)]
struct PersistentKeyPackageSnapshot {
    packages_by_account: HashMap<String, HashMap<String, ImKeyPackage>>,
}

/// 本机通信节点保存的 OpenMLS KeyPackage。
///
/// 节点只保存 OpenMLS 预密钥包字节，不保存明文消息，也不保存钱包私钥或
/// IM 设备私钥。`owner_wallet_account` 是当前 KeyPackage 所属的钱包聊天账户。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImKeyPackage {
    /// 协议版本，当前固定为 1。
    pub(crate) protocol_version: u16,
    /// KeyPackage 所属的钱包聊天账户。
    pub(crate) owner_wallet_account: String,
    /// 发布 KeyPackage 的 IM 设备 ID。
    pub(crate) device_id: String,
    /// OpenMLS 设备签名公钥 hex。
    pub(crate) device_public_key_hex: String,
    /// KeyPackage 全局去重 ID。
    pub(crate) key_package_id: String,
    /// OpenMLS KeyPackage wire bytes；Spike 阶段使用 hex 承载。
    pub(crate) key_package_hex: String,
    /// MLS cipher suite 名称或编号字符串。
    pub(crate) cipher_suite: String,
    /// 创建时间，毫秒时间戳。
    pub(crate) created_at_millis: u64,
    /// 过期时间，毫秒时间戳。
    pub(crate) expires_at_millis: u64,
    /// 被远端消费的时间；未消费为 `None`。
    pub(crate) consumed_at_millis: Option<u64>,
}

impl ImKeyPackage {
    fn validate(&self) -> Result<(), String> {
        if self.protocol_version != GMB_IM_KEYPACKAGE_PROTOCOL_VERSION {
            return Err("IM KeyPackage 协议版本不支持".to_string());
        }
        require_non_empty("owner_wallet_account", &self.owner_wallet_account)?;
        require_non_empty("device_id", &self.device_id)?;
        require_non_empty("device_public_key_hex", &self.device_public_key_hex)?;
        require_non_empty("key_package_id", &self.key_package_id)?;
        require_non_empty("key_package_hex", &self.key_package_hex)?;
        require_non_empty("cipher_suite", &self.cipher_suite)?;
        validate_hex_payload(&self.key_package_hex)?;
        if self.key_package_hex.len() > MAX_KEYPACKAGE_HEX_LEN {
            return Err("IM KeyPackage 超过大小上限".to_string());
        }
        if self.expires_at_millis <= self.created_at_millis {
            return Err("IM KeyPackage expires_at_millis 必须晚于 created_at_millis".to_string());
        }
        Ok(())
    }
}

/// 已授权手机向自己的私人通信全节点发布 KeyPackage。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct PublishImKeyPackageRequest {
    pub(crate) owner_wallet_account: String,
    pub(crate) device_id: String,
    pub(crate) device_public_key_hex: String,
    pub(crate) key_package_id: String,
    pub(crate) key_package_hex: String,
    pub(crate) cipher_suite: String,
    pub(crate) created_at_millis: u64,
    pub(crate) expires_at_millis: u64,
}

impl PublishImKeyPackageRequest {
    fn into_key_package(self) -> ImKeyPackage {
        ImKeyPackage {
            protocol_version: GMB_IM_KEYPACKAGE_PROTOCOL_VERSION,
            owner_wallet_account: self.owner_wallet_account,
            device_id: self.device_id,
            device_public_key_hex: self.device_public_key_hex,
            key_package_id: self.key_package_id,
            key_package_hex: self.key_package_hex,
            cipher_suite: self.cipher_suite,
            created_at_millis: self.created_at_millis,
            expires_at_millis: self.expires_at_millis,
            consumed_at_millis: None,
        }
    }
}

/// 远端发送方从对方通信节点按钱包地址拉取可用 KeyPackage。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct FetchImKeyPackagesRequest {
    pub(crate) owner_wallet_account: String,
    pub(crate) requester_chat_account: String,
    pub(crate) limit: u32,
}

impl FetchImKeyPackagesRequest {
    fn validate(&self) -> Result<(), String> {
        require_non_empty("owner_wallet_account", &self.owner_wallet_account)?;
        require_non_empty("requester_chat_account", &self.requester_chat_account)?;
        if self.limit == 0 {
            return Err("IM KeyPackage 拉取 limit 必须大于 0".to_string());
        }
        Ok(())
    }
}

/// 远端联系人声明已消费某个一次性 KeyPackage。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ConsumeImKeyPackageRequest {
    pub(crate) owner_wallet_account: String,
    pub(crate) key_package_id: String,
    pub(crate) requester_chat_account: String,
}

impl ConsumeImKeyPackageRequest {
    fn validate(&self) -> Result<(), String> {
        require_non_empty("owner_wallet_account", &self.owner_wallet_account)?;
        require_non_empty("key_package_id", &self.key_package_id)?;
        require_non_empty("requester_chat_account", &self.requester_chat_account)?;
        Ok(())
    }
}

/// 通过显式端点直连拉取 KeyPackage。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImDirectKeyPackageFetchRequest {
    pub(crate) remote_endpoint: ImNodeEndpoint,
    pub(crate) fetch: FetchImKeyPackagesRequest,
}

impl ImDirectKeyPackageFetchRequest {
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.remote_endpoint.validate()?;
        self.fetch.validate()
    }
}

/// 通过显式端点直连消费 KeyPackage。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ImDirectKeyPackageConsumeRequest {
    pub(crate) remote_endpoint: ImNodeEndpoint,
    pub(crate) consume: ConsumeImKeyPackageRequest,
}

impl ImDirectKeyPackageConsumeRequest {
    pub(crate) fn validate(&self) -> Result<(), String> {
        self.remote_endpoint.validate()?;
        self.consume.validate()
    }
}

/// 本机通信节点的多账号 KeyPackage 池。
///
/// 该池只服务本机已经授权的钱包聊天账号：每个钱包账号可以由多台手机设备
/// 发布自己的 KeyPackage，联系人通过 `/gmb/im/1` 按钱包地址拉取和消费。
/// 它不是公共目录，也不托管第三方身份。
#[derive(Debug, Default)]
pub(crate) struct ImKeyPackagePool {
    storage_path: Option<PathBuf>,
    packages_by_account: HashMap<String, HashMap<String, ImKeyPackage>>,
}

impl ImKeyPackagePool {
    /// 绑定持久化快照文件，并从磁盘恢复 KeyPackage 池。
    pub(crate) fn attach_storage(&mut self, file_path: PathBuf) -> Result<(), String> {
        if file_path.exists() {
            let bytes = fs::read(&file_path)
                .map_err(|e| format!("读取 IM KeyPackage 持久化文件失败: {e}"))?;
            let snapshot: PersistentKeyPackageSnapshot = serde_json::from_slice(&bytes)
                .map_err(|e| format!("解析 IM KeyPackage 持久化文件失败: {e}"))?;
            self.packages_by_account = snapshot.packages_by_account;
        }

        self.storage_path = Some(file_path);
        self.prune_expired(now_millis());
        self.persist()
    }

    /// 发布已授权设备生成的 KeyPackage。
    pub(crate) fn publish(
        &mut self,
        request: PublishImKeyPackageRequest,
    ) -> Result<ImKeyPackage, String> {
        let package = request.into_key_package();
        package.validate()?;
        self.prune_expired(now_millis());
        let packages = self
            .packages_by_account
            .entry(package.owner_wallet_account.clone())
            .or_default();
        if packages.contains_key(&package.key_package_id) {
            return Err("IM KeyPackage ID 已存在".to_string());
        }
        if packages.len() >= MAX_KEYPACKAGES_PER_OWNER {
            return Err("IM KeyPackage 池已达到钱包账号容量上限".to_string());
        }
        packages.insert(package.key_package_id.clone(), package.clone());
        self.persist()?;
        Ok(package)
    }

    /// 拉取未消费且未过期的 KeyPackage。
    pub(crate) fn fetch_available(
        &mut self,
        request: FetchImKeyPackagesRequest,
    ) -> Result<Vec<ImKeyPackage>, String> {
        request.validate()?;
        self.prune_expired(now_millis());
        self.persist()?;

        let account_packages = self
            .packages_by_account
            .get(&request.owner_wallet_account)
            .ok_or_else(|| "IM KeyPackage 池尚未发布该钱包账户".to_string())?;
        let mut packages = account_packages
            .values()
            .filter(|package| package.consumed_at_millis.is_none())
            .cloned()
            .collect::<Vec<_>>();
        packages.sort_by(|left, right| {
            left.created_at_millis
                .cmp(&right.created_at_millis)
                .then_with(|| left.key_package_id.cmp(&right.key_package_id))
        });
        let limit = request.limit.min(MAX_FETCH_LIMIT) as usize;
        packages.truncate(limit);
        Ok(packages)
    }

    /// 消费一次性 KeyPackage。
    pub(crate) fn consume(
        &mut self,
        request: ConsumeImKeyPackageRequest,
    ) -> Result<ImKeyPackage, String> {
        request.validate()?;
        let now = now_millis();
        self.prune_expired(now);
        let account_packages = self
            .packages_by_account
            .get_mut(&request.owner_wallet_account)
            .ok_or_else(|| "IM KeyPackage 池尚未发布该钱包账户".to_string())?;
        let package = account_packages
            .get_mut(&request.key_package_id)
            .ok_or_else(|| "IM KeyPackage 不存在或已过期".to_string())?;
        if package.consumed_at_millis.is_some() {
            return Err("IM KeyPackage 已被消费".to_string());
        }
        package.consumed_at_millis = Some(now);
        let consumed = package.clone();
        self.persist()?;
        Ok(consumed)
    }

    fn prune_expired(&mut self, now: u64) {
        self.packages_by_account.retain(|_, packages| {
            packages.retain(|_, package| package.expires_at_millis > now);
            !packages.is_empty()
        });
    }

    fn persist(&self) -> Result<(), String> {
        let Some(path) = &self.storage_path else {
            return Ok(());
        };
        let snapshot = PersistentKeyPackageSnapshot {
            packages_by_account: self.packages_by_account.clone(),
        };
        persist_snapshot(path, &snapshot)
    }
}

fn persist_snapshot(path: &Path, snapshot: &PersistentKeyPackageSnapshot) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "IM KeyPackage 持久化路径缺少父目录".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("创建 IM KeyPackage 目录失败: {e}"))?;
    let tmp_path = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(snapshot)
        .map_err(|e| format!("序列化 IM KeyPackage 快照失败: {e}"))?;
    fs::write(&tmp_path, bytes).map_err(|e| format!("写入 IM KeyPackage 临时文件失败: {e}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| format!("替换 IM KeyPackage 旧快照失败: {e}"))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| format!("提交 IM KeyPackage 快照失败: {e}"))?;
    Ok(())
}

fn require_non_empty(field_name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("IM KeyPackage 字段 {field_name} 不能为空"));
    }
    Ok(())
}

fn validate_hex_payload(value: &str) -> Result<(), String> {
    if value.len() % 2 != 0 {
        return Err("IM KeyPackage hex 长度必须为偶数".to_string());
    }
    hex::decode(value)
        .map(|_| ())
        .map_err(|_| "IM KeyPackage 必须是合法小写或大写 hex".to_string())
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{
        now_millis, ConsumeImKeyPackageRequest, FetchImKeyPackagesRequest, ImKeyPackagePool,
        PublishImKeyPackageRequest,
    };
    use std::fs;

    fn sample_publish(id: &str) -> PublishImKeyPackageRequest {
        let now = now_millis();
        PublishImKeyPackageRequest {
            owner_wallet_account: "bob-wallet".to_string(),
            device_id: "bob-phone".to_string(),
            device_public_key_hex: "aabbcc".to_string(),
            key_package_id: id.to_string(),
            key_package_hex: "aabbccdd".to_string(),
            cipher_suite: "MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519".to_string(),
            created_at_millis: now,
            expires_at_millis: now + 600_000,
        }
    }

    #[test]
    fn supports_multiple_wallet_accounts_on_one_node() {
        let mut pool = ImKeyPackagePool::default();
        pool.publish(sample_publish("bob-kp"))
            .expect("bob keypackage should publish");
        let mut alice = sample_publish("alice-kp");
        alice.owner_wallet_account = "alice-wallet".to_string();
        alice.device_id = "alice-phone".to_string();
        alice.device_public_key_hex = "ddeeff".to_string();
        pool.publish(alice)
            .expect("alice keypackage should publish on same node");

        let bob = pool
            .fetch_available(FetchImKeyPackagesRequest {
                owner_wallet_account: "bob-wallet".to_string(),
                requester_chat_account: "carol-wallet".to_string(),
                limit: 1,
            })
            .expect("bob keypackage should fetch");
        let alice = pool
            .fetch_available(FetchImKeyPackagesRequest {
                owner_wallet_account: "alice-wallet".to_string(),
                requester_chat_account: "carol-wallet".to_string(),
                limit: 1,
            })
            .expect("alice keypackage should fetch");

        assert_eq!(bob[0].key_package_id, "bob-kp");
        assert_eq!(alice[0].key_package_id, "alice-kp");
    }

    #[test]
    fn publish_fetch_and_consume_keypackage() {
        let mut pool = ImKeyPackagePool::default();
        pool.publish(sample_publish("kp-1"))
            .expect("keypackage should publish");

        let fetched = pool
            .fetch_available(FetchImKeyPackagesRequest {
                owner_wallet_account: "bob-wallet".to_string(),
                requester_chat_account: "alice-wallet".to_string(),
                limit: 1,
            })
            .expect("keypackage should fetch");
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].key_package_id, "kp-1");

        let consumed = pool
            .consume(ConsumeImKeyPackageRequest {
                owner_wallet_account: "bob-wallet".to_string(),
                key_package_id: "kp-1".to_string(),
                requester_chat_account: "alice-wallet".to_string(),
            })
            .expect("keypackage should consume");
        assert!(consumed.consumed_at_millis.is_some());

        let fetched_after_consume = pool
            .fetch_available(FetchImKeyPackagesRequest {
                owner_wallet_account: "bob-wallet".to_string(),
                requester_chat_account: "alice-wallet".to_string(),
                limit: 1,
            })
            .expect("keypackage should fetch after consume");
        assert!(fetched_after_consume.is_empty());
    }

    #[test]
    fn persists_keypackage_pool() {
        let file_path =
            std::env::temp_dir().join(format!("gmb-im-keypackage-test-{}.json", now_millis()));
        let _ = fs::remove_file(&file_path);

        let mut pool = ImKeyPackagePool::default();
        pool.attach_storage(file_path.clone())
            .expect("storage should attach");
        pool.publish(sample_publish("kp-persist"))
            .expect("keypackage should publish");

        let mut reloaded = ImKeyPackagePool::default();
        reloaded
            .attach_storage(file_path.clone())
            .expect("storage should reload");
        let fetched = reloaded
            .fetch_available(FetchImKeyPackagesRequest {
                owner_wallet_account: "bob-wallet".to_string(),
                requester_chat_account: "alice-wallet".to_string(),
                limit: 1,
            })
            .expect("keypackage should survive reload");
        assert_eq!(fetched[0].key_package_id, "kp-persist");

        let _ = fs::remove_file(file_path);
    }
}
