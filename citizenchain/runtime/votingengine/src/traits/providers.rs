//! 公民身份与机构管理员资格的只读提供者。

use crate::types::InstitutionCode;
use sp_runtime::DispatchError;

/// 公民身份只读接口。投票引擎只能读链上公民身份模块的资格和人口数，
/// 不再接收注册局链下签发的人口快照或投票凭证。
pub trait CitizenIdentityReader<AccountId> {
    fn can_vote(who: &AccountId, scope: &citizen_identity::PopulationScope) -> bool;
    fn can_be_candidate(who: &AccountId, scope: &citizen_identity::PopulationScope) -> bool;
    fn population_count(scope: &citizen_identity::PopulationScope) -> u64;

    /// 由 citizen-identity 创建同时冻结分母与成员资格的治理快照。
    fn create_population_snapshot(
        _scope: &citizen_identity::PopulationScope,
    ) -> Result<(u64, u64), DispatchError> {
        Err(DispatchError::Other(
            "citizen identity snapshot provider unavailable",
        ))
    }

    /// 验证账户在指定治理快照创建时是否具备投票资格。
    fn can_vote_at(_who: &AccountId, _snapshot_id: u64) -> bool {
        false
    }

    /// 提案历史清理完成后释放 citizen-identity 快照元数据。
    fn release_population_snapshot(_snapshot_id: u64) {}

    /// FRAME benchmark 专用：写入一个同时具备投票和参选资格的账户，
    /// 并同步人口分母。生产调用路径不会调用此函数。
    #[cfg(feature = "runtime-benchmarks")]
    fn benchmark_seed_identity(_who: &AccountId, _scope: &citizen_identity::PopulationScope) {}
}

impl<AccountId> CitizenIdentityReader<AccountId> for () {
    fn can_vote(_who: &AccountId, _scope: &citizen_identity::PopulationScope) -> bool {
        false
    }

    fn can_be_candidate(_who: &AccountId, _scope: &citizen_identity::PopulationScope) -> bool {
        false
    }

    fn population_count(_scope: &citizen_identity::PopulationScope) -> u64 {
        0
    }
}

/// 内部管理员动态提供器（可由其他治理模块提供最新管理员集合）。
///
/// 一致性契约：
/// - `is_institution_admin(institution_code, cid_number, who) == true` 时，同一链上状态读取到的
///   `get_institution_admins(institution_code, cid_number)` 必须包含 `who`。
/// - 个人多签 Pending 版本的权限与管理员列表必须满足同样强一致关系。
///
/// 投票引擎会在写入管理员快照后再次校验发起人属于快照；provider 实现若出现
/// drift，会被视为权限错误并回滚提案创建。
pub trait InternalAdminProvider<AccountId> {
    fn is_institution_admin(
        institution_code: InstitutionCode,
        cid_number: &[u8],
        who: &AccountId,
    ) -> bool;

    /// 获取机构当前管理员列表（用于提案创建时锁定快照）。
    fn get_institution_admins(
        _institution_code: InstitutionCode,
        _cid_number: &[u8],
    ) -> Option<sp_std::vec::Vec<AccountId>> {
        None
    }

    /// 查询个人多签管理员权限。
    fn is_personal_admin(_personal_account: AccountId, _who: &AccountId) -> bool {
        false
    }

    /// 获取个人多签当前管理员列表。
    fn get_personal_admins(_personal_account: AccountId) -> Option<sp_std::vec::Vec<AccountId>> {
        None
    }

    /// 查询 Pending 个人多签管理员权限。仅供创建个人多签提案使用。
    fn is_pending_personal_admin(_personal_account: AccountId, _who: &AccountId) -> bool {
        false
    }

    /// 获取机构法定代表人(ADR-027 立法签署人)。
    /// 默认 None(个人账户/尚未任命);机构公开事实由 entity 的 `InstitutionInfo` 提供。
    fn legal_representative(_cid_number: &[u8]) -> Option<AccountId> {
        None
    }

    /// 获取护宪大法官成员集(ADR-027 修订:修宪最终否决,宪法第21条)。
    /// 护宪大法官归口国家司法院,生产按管理员 `admin_role=护宪大法官` 过滤 NJD admins。
    /// 立法投票模块要求成员数恰好 7 人,并按 4 名及以上赞成判定修宪终审通过。
    fn constitution_guard_members() -> sp_std::vec::Vec<AccountId> {
        sp_std::vec::Vec::new()
    }

    /// 获取 Pending 个人多签管理员列表。
    fn get_pending_personal_admins(
        _personal_account: AccountId,
    ) -> Option<sp_std::vec::Vec<AccountId>> {
        None
    }
}

impl<AccountId> InternalAdminProvider<AccountId> for () {
    fn is_institution_admin(
        _institution_code: InstitutionCode,
        _cid_number: &[u8],
        _who: &AccountId,
    ) -> bool {
        false
    }
}

/// 内部管理员总人数提供器。
/// 联合投票会根据“剩余管理员数是否还能让赞成票达到阈值”来自动判定机构反对。
pub trait InternalAdminsLenProvider<AccountId> {
    fn institution_admins_len(institution_code: InstitutionCode, cid_number: &[u8]) -> Option<u32>;

    fn personal_admins_len(personal_account: AccountId) -> Option<u32>;
}

impl<AccountId> InternalAdminsLenProvider<AccountId> for () {
    fn institution_admins_len(
        _institution_code: InstitutionCode,
        _cid_number: &[u8],
    ) -> Option<u32> {
        None
    }

    fn personal_admins_len(_personal_account: AccountId) -> Option<u32> {
        None
    }
}
