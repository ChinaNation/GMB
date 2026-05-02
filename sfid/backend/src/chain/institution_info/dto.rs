//! 机构信息查询请求/响应 DTO。
//!
//! 所有 DTO 仅给链端 / 钱包消费者使用,不暴露 SFID 内部敏感字段
//! (创建人 / 管理员 pubkey 等)。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::institutions::model::MultisigInstitution;
use crate::models::{InstitutionChainStatus, MultisigChainStatus};

// ─── 单机构详情(带 chain pull 凭证) ────────────────────────

/// `app_get_institution` 的响应包装:
/// - `#[serde(flatten)]` 把 `MultisigInstitution` 全部既有字段平铺到顶层
///   (institution_name / a3 / sub_type / parent_sfid_id / province / city / chain_status / category / ...)
/// - 末尾追加 2 字段 `register_nonce` + `signature`,供节点桌面发起
///   `propose_create_institution` extrinsic 时透传给链端 verifier
///
/// 旧调用方(钱包等仅展示场景)收到多 2 字段忽略即可。
#[derive(Serialize)]
pub(crate) struct InstitutionDetailWithCredential {
    /// 既有 MultisigInstitution 全部字段,sserde flatten 不破坏旧结构。
    #[serde(flatten)]
    pub(crate) institution: MultisigInstitution,
    /// 防重放 nonce(本次响应生成的 32 字节随机 hex)。
    /// 链端 `UsedRegisterNonce[hash(nonce)]` 标记已用,同凭证不可重放。
    pub(crate) register_nonce: String,
    /// 省级签名密钥对凭证 payload 的 sr25519 签名(64 字节 hex)。
    /// payload = `blake2_256(scale_encode(DUOQIAN_DOMAIN ++ OP_SIGN_INST ++ genesis_hash
    ///                          ++ sfid_id ++ institution_name ++ register_nonce))`
    /// 链端 `SfidInstitutionVerifier::verify_institution_registration` 重算并验签。
    pub(crate) signature: String,
}

// ─── 通用机构搜索 ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub(crate) struct AppInstitutionSearchQuery {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Clone)]
pub(crate) struct AppInstitutionSearchRow {
    pub(crate) sfid_id: String,
    pub(crate) institution_name: Option<String>,
    pub(crate) category: crate::sfid::InstitutionCategory,
    pub(crate) a3: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) chain_status: InstitutionChainStatus,
}

// ─── 机构账户列表(脱敏) ────────────────────────────────────

#[derive(Serialize)]
pub(crate) struct AppAccountEntry {
    pub(crate) account_name: String,
    pub(crate) duoqian_address: Option<String>,
    pub(crate) chain_status: MultisigChainStatus,
    pub(crate) chain_synced_at: Option<DateTime<Utc>>,
    pub(crate) is_default: bool,
    pub(crate) can_delete: bool,
}

#[derive(Serialize)]
pub(crate) struct AppInstitutionAccounts {
    pub(crate) sfid_id: String,
    pub(crate) institution_name: String,
    pub(crate) accounts: Vec<AppAccountEntry>,
}

// ─── 清算行搜索(已激活) ────────────────────────────────────

/// wuminapp 清算行搜索查询参数。
///
/// - `province`: 省份名(如"广东省"),省略=全国
/// - `city`: 市名(需搭配 province),省略=本省全部
/// - `keyword`: 关键字,匹配 sfid_id / institution_name 子串(大小写不敏感)
/// - `page`: 页码,从 1 起(默认 1)
/// - `size`: 每页条数,1~100(默认 20)
#[derive(Debug, Deserialize)]
pub(crate) struct AppClearingBankSearchQuery {
    pub province: Option<String>,
    pub city: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<u32>,
    pub size: Option<u32>,
}

/// 清算行搜索单条结果(已激活,主账户 ActiveOnChain)。
///
/// 含 `sub_type` / `parent_*` 字段方便前端展示父子层级
/// (例如"招商银行 → 招商银行广州民主路支行")。
#[derive(Serialize, Clone)]
pub(crate) struct AppClearingBankRow {
    pub(crate) sfid_id: String,
    /// 机构中文名(两步式未命名时为空串)。
    pub(crate) institution_name: String,
    /// 主体属性:SFR(私法人)或 FFR(非法人)。
    pub(crate) a3: String,
    /// 私法人子类型(仅 a3=SFR 有值)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sub_type: Option<String>,
    /// 所属法人 sfid_id(仅 a3=FFR 有值)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_sfid_id: Option<String>,
    /// 所属法人中文名(FFR 用)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_institution_name: Option<String>,
    /// 所属法人 a3(FFR 必为 SFR)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_a3: Option<String>,
    pub(crate) province: String,
    pub(crate) city: String,
    /// 主账户链上地址(hex, 不含 0x 前缀)。未上链时为 None。
    pub(crate) main_account: Option<String>,
    /// 费用账户链上地址。
    pub(crate) fee_account: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct AppClearingBankSearchOutput {
    /// 本次查询过滤后的总条数(跨省汇总)。
    pub(crate) total: usize,
    /// 当前页数据。
    pub(crate) items: Vec<AppClearingBankRow>,
    pub(crate) page: u32,
    pub(crate) size: u32,
}

// ─── 候选清算行搜索(可未激活) ──────────────────────────────

/// 候选清算行搜索参数。
#[derive(Debug, Deserialize)]
pub(crate) struct EligibleClearingBankSearchQuery {
    /// 关键字,匹配 sfid_id / institution_name 子串(大小写不敏感)。
    pub q: Option<String>,
    /// 上限(默认 20,最大 50)。
    pub limit: Option<u32>,
}

/// 候选清算行搜索单条结果。
///
/// 比 `AppClearingBankRow` 多 `main_chain_status` 字段,
/// 让节点桌面 UI 能区分"已上链/待激活/上链中"。
///
/// **注意**:本结构序列化字段名是 snake_case(serde 默认),
/// 与节点客户端 [citizenchain/node/src/offchain/types.rs] 的 deserialize
/// DTO 必须严格对齐(否则 JSON 解析失败)。
#[derive(Serialize, Clone)]
pub(crate) struct EligibleClearingBankRow {
    pub(crate) sfid_id: String,
    /// 机构中文名(两步式未命名时不出现该字段)。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) institution_name: Option<String>,
    pub(crate) a3: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_sfid_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_institution_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_a3: Option<String>,
    pub(crate) province: String,
    pub(crate) city: String,
    /// 主账户链上地址(hex, 不含 0x 前缀)。未上链时不出现该字段。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) main_account: Option<String>,
    /// 费用账户链上地址。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fee_account: Option<String>,
    /// 主账户链上状态(SCREAMING_SNAKE_CASE 枚举:
    /// `NOT_ON_CHAIN` / `PENDING_ON_CHAIN` / `ACTIVE_ON_CHAIN` / `REVOKED_ON_CHAIN`)。
    pub(crate) main_chain_status: MultisigChainStatus,
}
