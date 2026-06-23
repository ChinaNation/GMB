//! 中文注释:联邦注册局管理员 P0 止血播种。
//!
//! 管理员的唯一真源是链上 `admins-change::AdminAccounts`(见 ADR-023)。本模块只在
//! 【重新创世后 / 链暂不可达】这一引导窗口内,从编译期创世常量 `china_zf.rs` 直接把
//! 「总统府联邦注册局」的 215 名管理员恢复进 CID 登录投影表 `admins`,使联邦注册局
//! 管理员能立刻扫码登录,不必等链同步。稳态由 `chain_sync` 双通道(启动全量快照 +
//! indexer 增量)接管;本播种是应急 / 离线兜底,不是常态读链路径,绝不进运行时热路径。
//!
//! 幂等:`upsert_admin_conn` 以 `admin_account` 为冲突键 `ON CONFLICT DO UPDATE`,可反复运行。

use chrono::Utc;
use tracing::info;

use crate::admins::model::{AdminUser, RegistryOrgCode};
use crate::admins::repo;
use crate::gov::service::federal_registry_admins;
use crate::AppState;

/// 中文注释:`china_zf.rs`「总统府联邦注册局」条目中,每个省份注释下恰好 5 个管理员公钥。
const FEDERAL_ADMINS_PER_PROVINCE: usize = 5;

/// 中文注释:联邦注册局 43 省顺序,严格对应 `china_zf.rs`「总统府联邦注册局」条目 `admins`
/// 数组的分组注释顺序(每 5 个公钥一省)。链上只存一个扁平的联邦注册局账户、不区分省份;
/// "管理员属于哪个省"是纯 CID 元数据,仅用于管辖过滤(联邦看本省、市看本市)。
///
/// china_zf 创世后冻结(见 [[feedback_chainspec_frozen]]);若该常量的管理员数变动,
/// `run_seed_federal_admins` 的长度断言会立即报错以阻止省份错配——这是这份硬编码顺序
/// 与 china_zf 之间唯一的非编译期契约,改 china_zf 必须同步改这里。
const FEDERAL_ADMIN_PROVINCES: [&str; 43] = [
    "中枢省",
    "岭南省",
    "广东省",
    "广西省",
    "福建省",
    "海南省",
    "云南省",
    "贵州省",
    "湖南省",
    "江西省",
    "浙江省",
    "江苏省",
    "山东省",
    "山西省",
    "河南省",
    "河北省",
    "湖北省",
    "陕西省",
    "重庆省",
    "四川省",
    "甘肃省",
    "北平省",
    "海滨省",
    "松江省",
    "龙江省",
    "吉林省",
    "辽宁省",
    "宁夏省",
    "青海省",
    "安徽省",
    "台湾省",
    "西藏省",
    "新疆省",
    "西康省",
    "阿里省",
    "葱岭省",
    "伊犁省",
    "河西省",
    "昆仑省",
    "河套省",
    "热河省",
    "兴安省",
    "合江省",
];

/// 中文注释:从 china_zf 常量播种联邦注册局 215 名管理员到 `admins` + `federal_registry_scope`。
/// 单连接顺序写入,`next_admin_id_conn` + `upsert_admin_conn` 均幂等;成功返回播种条数。
pub(crate) fn run_seed_federal_admins(state: &AppState) -> Result<(), String> {
    let admins = federal_registry_admins()
        .ok_or_else(|| "china_zf 常量缺少「总统府联邦注册局」条目".to_string())?;

    let expected = FEDERAL_ADMIN_PROVINCES.len() * FEDERAL_ADMINS_PER_PROVINCE;
    if admins.len() != expected {
        return Err(format!(
            "联邦注册局管理员数 {} 与省份序 {}×{}={} 不一致;china_zf.rs 已变动,请同步 FEDERAL_ADMIN_PROVINCES",
            admins.len(),
            FEDERAL_ADMIN_PROVINCES.len(),
            FEDERAL_ADMINS_PER_PROVINCE,
            expected
        ));
    }

    let now = Utc::now();
    let seeded = state.db.with_client(move |conn| {
        for (idx, raw) in admins.iter().enumerate() {
            let province = FEDERAL_ADMIN_PROVINCES[idx / FEDERAL_ADMINS_PER_PROVINCE];
            let seat = idx % FEDERAL_ADMINS_PER_PROVINCE + 1;
            // 内部统一 0x 小写 hex(见 [[feedback_pubkey_format_rule]]);与链投影冲突键 lower(admin_account) 对齐。
            let admin_account = format!("0x{}", hex::encode(raw));
            let id = repo::next_admin_id_conn(conn)?;
            let admin = AdminUser {
                id,
                admin_account,
                // 中文注释:链上无昵称,这里给每名联邦注册局管理员写有意义的显示名
                //「{省}联邦注册局管理员{席位}」,使所有视图(含机构信息 tab 的链上 admin 展示、
                // 不过 catalog 通名回退的路径)都显示名字而非空/公钥哈希。
                admin_name: format!("{province}联邦注册局管理员{seat}"),
                registry_org_code: RegistryOrgCode::FederalRegistry,
                built_in: true,
                created_by: "SYSTEM".to_string(),
                created_at: now,
                updated_at: Some(now),
                city_name: String::new(),
            };
            repo::upsert_admin_conn(conn, &admin, Some(province))?;
        }
        Ok(admins.len())
    })?;

    info!(
        seeded,
        provinces = FEDERAL_ADMIN_PROVINCES.len(),
        "seeded federal registry admins from china_zf constants"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn province_grouping_matches_china_zf_admin_count() {
        // 中文注释:守卫省份序与创世常量一致——china_zf 一旦改动管理员数,此断言先红,
        // 阻止 run_seed_federal_admins 把公钥错配到省份。
        let admins = federal_registry_admins().expect("联邦注册局常量缺失");
        assert_eq!(
            admins.len(),
            FEDERAL_ADMIN_PROVINCES.len() * FEDERAL_ADMINS_PER_PROVINCE,
            "china_zf 联邦注册局管理员数与 FEDERAL_ADMIN_PROVINCES 不一致"
        );
    }

    #[test]
    fn province_list_has_no_duplicates() {
        let mut seen = std::collections::BTreeSet::new();
        for p in FEDERAL_ADMIN_PROVINCES {
            assert!(seen.insert(p), "省份重复: {p}");
        }
        assert_eq!(seen.len(), 43);
    }
}
