//! 机构信息查询请求/响应 DTO。
//!
//! 所有 DTO 仅给链端 / 钱包消费者使用,不暴露 SFID 内部敏感字段
//! (创建人 / 管理员 pubkey 等)。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::app_core::chain_runtime::RuntimeSignatureMeta;
use crate::models::{InstitutionChainStatus, MultisigChainStatus};

// ─── 单机构详情(展示查询,不带注册凭证) ─────────────────────

/// `app_get_institution` 的安全展示 DTO。
///
/// 中文注释:查询与注册分开。此结构只用于链端/钱包展示 SFID 机构资料,
/// 不携带 register_nonce/signature,也不暴露 created_by 等 SFID 内部字段。
#[derive(Serialize)]
pub(crate) struct AppInstitutionDetail {
    pub(crate) sfid_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) institution_name: Option<String>,
    pub(crate) category: crate::sfid::InstitutionCategory,
    pub(crate) a3: String,
    pub(crate) p1: String,
    pub(crate) province: String,
    pub(crate) city: String,
    pub(crate) province_code: String,
    pub(crate) city_code: String,
    pub(crate) institution_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) sub_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) parent_sfid_id: Option<String>,
    pub(crate) sfid_finalized: bool,
    pub(crate) chain_status: InstitutionChainStatus,
}

// ─── 机构注册信息凭证(链端注册专用) ────────────────────────

/// 链端注册时使用的验签包装字段。
///
/// 中文注释:业务注册字段只有外层的 `sfid_id / institution_name / account_names`。
/// 本结构里的字段只用于链端确认这些信息确实由 SFID 系统签发,并做防重放。
#[derive(Serialize)]
pub(crate) struct AppInstitutionRegistrationCredential {
    pub(crate) genesis_hash: String,
    pub(crate) register_nonce: String,
    pub(crate) province: String,
    /// 签发本次凭证的省级 admin slot 公钥,统一 `0x` + 64 位小写 hex。
    pub(crate) signer_admin_pubkey: String,
    pub(crate) signature: String,
    pub(crate) meta: RuntimeSignatureMeta,
}

/// `app_get_institution_registration_info` 的响应。
///
/// 中文注释:不得在这里加入 a3/sub_type/parent_sfid_id 等链端注册不需要的业务字段。
#[derive(Serialize)]
pub(crate) struct AppInstitutionRegistrationInfo {
    pub(crate) sfid_id: String,
    pub(crate) institution_name: String,
    pub(crate) account_names: Vec<String>,
    pub(crate) credential: AppInstitutionRegistrationCredential,
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
