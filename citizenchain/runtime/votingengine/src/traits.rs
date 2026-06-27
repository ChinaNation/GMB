//! 投票引擎对外 trait 定义与默认 `()` 实现。
//!
//! 本文件集中所有可被 runtime / 业务 pallet 注入的 trait,以及它们的默认 `()` 实现,
//! 让 lib.rs 主文件只保留 `#[pallet]` 宏与 storage/extrinsic 声明。

use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

use crate::types::InstitutionCode;
use crate::{ProposalCancelDecision, ProposalExecutionOutcome};

pub trait JointVoteEngine<AccountId> {
    fn create_joint_proposal(who: AccountId) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data(
        who: AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data_and_object(
        who: AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
        object_kind: u8,
        object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;
}

impl<AccountId> JointVoteEngine<AccountId> for () {
    fn create_joint_proposal(_who: AccountId) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }

    fn create_joint_proposal_with_data(
        _who: AccountId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }

    fn create_joint_proposal_with_data_and_object(
        _who: AccountId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
        _object_kind: u8,
        _object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }
}

/// 立法事项(立法/修法/废法)接入立法投票时,统一由投票引擎 legislation-vote 创建提案并返回真实提案 ID。
///
/// 中文注释(ADR-027):业务壳 legislation-yuan 只传"立法语义"——发起机构、机构码、表决类型、
/// 法律载荷;表决规则(参与率/赞成率/反对率上限)、两院顺序、强制公投、计票与通过判定全部归属
/// 投票引擎 legislation-vote sub-pallet,业务壳不得自行处理。
/// runtime 装配为真实 `LegislationVote`;未实装语境(如其它 pallet 的测试 mock)装 `()` 返回 NotConfigured。
pub trait LegislationVoteEngine<AccountId> {
    /// 创建立法投票提案。
    /// - `houses`:院序列 `[(机构码, 机构账户), ...]`,发起院在前、终审院在后。
    ///   单院(市立法会)= 1 项;两院(国家/省立法院)= `[众议会, 参议会]`;教委会模式 = `[教委会, 参议会]`。
    ///   发起人 `who` 必须是 `houses[0]`(发起院)的现任议员/委员(admin)。
    /// - `vote_type`:表决类型 u8(常规 0 / 常规教育 1 / 重要 2 / 重要教育 3 / 特别 4,ADR-027 修订),
    ///   用 u8 保持引擎与业务枚举解耦。特别案(4)内部全过后强制进入公投阶段(发起前须由 `who` 准备人口快照),
    ///   公投通过即生效不签署;非特别案内部全过后进入行政签署阶段(`executive` 机构法定代表人签署)。
    /// - `executive`:行政签署机构 `(机构码, 机构账户)`——市政府(市)/省政府(省)/总统府(国);其法定代表人=市长/省长/总统。
    /// - `legislature`:两院级的立法院机构 `(机构码, 机构账户)`(国家/省立法院,其法定代表人=院长,供三人会签);单院(市)= `None`。
    /// - `data`:MODULE_TAG 前缀 + 提案摘要(law_id/tier/version/content_hash)。
    /// - `object_data`:法律全文大对象(整部条文 SCALE),供通过回调读回写入新版本。
    #[allow(clippy::too_many_arguments)]
    fn create_legislation_proposal(
        who: AccountId,
        houses: sp_std::vec::Vec<(InstitutionCode, AccountId)>,
        vote_type: u8,
        executive: (InstitutionCode, AccountId),
        legislature: Option<(InstitutionCode, AccountId)>,
        needs_guard: bool,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
        object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;
}

impl<AccountId> LegislationVoteEngine<AccountId> for () {
    fn create_legislation_proposal(
        _who: AccountId,
        _houses: sp_std::vec::Vec<(InstitutionCode, AccountId)>,
        _vote_type: u8,
        _executive: (InstitutionCode, AccountId),
        _legislature: Option<(InstitutionCode, AccountId)>,
        _needs_guard: bool,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
        _object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("LegislationVoteEngineNotConfigured"))
    }
}

/// 事项模块接入内部投票时,统一由投票引擎创建提案并返回真实提案 ID。
///
/// 中文注释：业务模块只能选择“提案语义”，不能传入“本次投票通过阈值”。
/// 阈值读取、快照、计票、自动赞成票与通过/否决判定全部归属投票引擎。
pub trait InternalVoteEngine<AccountId> {
    /// 创建一般内部投票提案。用于转账、销毁、GRANDPA key 更换等普通业务。
    fn create_general_internal_proposal_with_data(
        who: AccountId,
        institution_code: InstitutionCode,
        institution: AccountId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;

    /// 创建注册/注销生命周期内部投票提案。用于注销个人多签和机构多签。
    ///
    /// 中文注释：生命周期投票由投票引擎按 active 管理员快照写入全员通过阈值。
    fn create_lifecycle_internal_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("LifecycleVoteEngineNotConfigured"))
    }

    /// 创建注册个人多签/机构多签的特别内部投票提案。
    ///
    /// `dynamic_threshold` 是注册后普通业务使用的动态阈值配置，不是本次注册投票阈值。
    /// 本次注册投票阈值由投票引擎按 `admins.len()` 写全员通过快照。
    fn create_registered_account_create_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _admins: sp_std::vec::Vec<AccountId>,
        _dynamic_threshold: u32,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "RegisteredAccountCreateVoteEngineNotConfigured",
        ))
    }

    /// 创建管理员集合变更内部投票提案。只允许 admins 模块 模块接入。
    ///
    /// 中文注释：本次投票仍使用当前 active 阈值；`new_threshold` 只表示变更执行成功后
    /// 写入投票引擎的下一阶段动态阈值。
    fn create_admin_change_internal_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _new_admins_len: u32,
        _new_threshold: u32,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "AdminSetMutationVoteEngineNotConfigured",
        ))
    }

    /// 特权直设动态阈值:绕过注册/变更提案,直接写入已激活动态阈值。
    ///
    /// 中文注释:仅供 admins 模块在"联邦注册局直设市注册局管理员"(Step3 去中心化鉴权)时
    /// 同步阈值用。实现方必须按严格过半规则校验 `(admins_len, threshold)` 后写入,
    /// 失败回滚由调用方事务统一处理。默认未配置。
    fn register_active_dynamic_threshold_direct(
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _admins_len: u32,
        _threshold: u32,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "RegisterActiveDynamicThresholdDirectNotConfigured",
        ))
    }

    /// 读取已激活动态阈值。只用于展示和业务事件，不参与业务模块计票。
    fn active_dynamic_threshold(
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<u32> {
        None
    }

    /// 读取 pending 或 active 动态阈值。注册回调在激活前发事件时使用。
    fn configured_dynamic_threshold(
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<u32> {
        None
    }
}

impl<AccountId> InternalVoteEngine<AccountId> for () {
    fn create_general_internal_proposal_with_data(
        _who: AccountId,
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("InternalVoteEngineNotConfigured"))
    }
}

/// 中文注释：公民总人口快照验签接口（由 runtime 对接 CID 系统）。
/// 签发身份统一为 `issuer_cid_number + issuer_main_account + signer_pubkey`;
/// runtime verifier 必须确认 signer 属于签发机构 admins 后再验签。
pub trait PopulationSnapshotVerifier<AccountId, Nonce, Signature> {
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &Nonce,
        signature: &Signature,
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool;
}

impl<AccountId, Nonce, Signature> PopulationSnapshotVerifier<AccountId, Nonce, Signature> for () {
    fn verify_population_snapshot(
        _who: &AccountId,
        _eligible_total: u64,
        _nonce: &Nonce,
        _signature: &Signature,
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }
}

pub trait JointVoteResultCallback {
    fn on_joint_vote_finalized(
        vote_proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError>;

    fn can_cancel_passed_proposal(
        _proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        Ok(ProposalCancelDecision::Ignored)
    }

    fn on_execution_failed_terminal(_proposal_id: u64) -> DispatchResult {
        Ok(())
    }
}

impl JointVoteResultCallback for () {
    fn on_joint_vote_finalized(
        _vote_proposal_id: u64,
        _approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        Ok(ProposalExecutionOutcome::Ignored)
    }
}

/// 内部投票终态回调。
///
/// 投票引擎在提案进入 `STATUS_PASSED` / `STATUS_REJECTED` 时对所有注册
/// 的业务模块广播此回调，并根据返回的 [`ProposalExecutionOutcome`] 统一推进状态。
///
/// 业务模块应当:
/// - 通过 `ProposalData` 的 `MODULE_TAG` 前缀(或业务独立存储键)认领自己的提案,
///   不属于自己的提案直接返回 `ProposalExecutionOutcome::Ignored` 跳过;
/// - `approved = true` 时执行具体业务动作(转账 / 替换管理员 / 销毁 / ...);
/// - `approved = false` 时可选清理业务独立存储(如 `SweepProposalActions`)。
///
/// 回调返回 `Err` 表示业务数据异常，会让整个状态转换事务回滚。
///
/// 多业务模块通过 tuple 注册(见下方 `impl` for `(A,)`、`(A, B)` ... 等元组类型)。
pub trait InternalVoteResultCallback {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError>;

    fn can_cancel_passed_proposal(
        _proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        Ok(ProposalCancelDecision::Ignored)
    }

    fn on_execution_failed_terminal(_proposal_id: u64) -> DispatchResult {
        Ok(())
    }
}

/// 默认空实现(未挂业务回调时使用 `type X = ()`)。
impl InternalVoteResultCallback for () {
    fn on_internal_vote_finalized(
        _proposal_id: u64,
        _approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        Ok(ProposalExecutionOutcome::Ignored)
    }
}

fn merge_execution_outcome(
    current: ProposalExecutionOutcome,
    next: ProposalExecutionOutcome,
) -> ProposalExecutionOutcome {
    use ProposalExecutionOutcome::*;
    match (current, next) {
        (Ignored, outcome) => outcome,
        (outcome, Ignored) => outcome,
        (FatalFailed, _) | (_, FatalFailed) => FatalFailed,
        (RetryableFailed, _) | (_, RetryableFailed) => RetryableFailed,
        (Executed, Executed) => Executed,
    }
}

fn merge_cancel_decision(
    current: ProposalCancelDecision,
    next: ProposalCancelDecision,
) -> ProposalCancelDecision {
    match (current, next) {
        (ProposalCancelDecision::Allow, _) | (_, ProposalCancelDecision::Allow) => {
            ProposalCancelDecision::Allow
        }
        (ProposalCancelDecision::Ignored, ProposalCancelDecision::Ignored) => {
            ProposalCancelDecision::Ignored
        }
    }
}

// ──── InternalVoteResultCallback 的 tuple 实现(手写,覆盖 1~6 个成员)────
//
// 语义:依次调用每个成员的 `on_internal_vote_finalized`;任一成员返回 `Err`
// 立即短路返回,后续成员不再调用——这与 `with_transaction` 内的
// `TransactionOutcome::Rollback(Err(...))` 协作确保整个状态转换事务回滚。
//
// 注:注册 5 个业务模块(multisig_transfer /
// organization_manage / RuntimeAdminAccountQuery / resolution_destro /
// grandpakey_change),留 6 元组余量。如未来业务模块增加,补对应元组 impl。
impl<A: InternalVoteResultCallback> InternalVoteResultCallback for (A,) {
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        A::on_internal_vote_finalized(proposal_id, approved)
    }

    fn can_cancel_passed_proposal(
        proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        A::can_cancel_passed_proposal(proposal_id)
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        A::on_execution_failed_terminal(proposal_id)
    }
}

impl<A: InternalVoteResultCallback, B: InternalVoteResultCallback> InternalVoteResultCallback
    for (A, B)
{
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        let a = A::on_internal_vote_finalized(proposal_id, approved)?;
        let b = B::on_internal_vote_finalized(proposal_id, approved)?;
        Ok(merge_execution_outcome(a, b))
    }

    fn can_cancel_passed_proposal(
        proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        let a = A::can_cancel_passed_proposal(proposal_id)?;
        let b = B::can_cancel_passed_proposal(proposal_id)?;
        Ok(merge_cancel_decision(a, b))
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        A::on_execution_failed_terminal(proposal_id)?;
        B::on_execution_failed_terminal(proposal_id)
    }
}

impl<
        A: InternalVoteResultCallback,
        B: InternalVoteResultCallback,
        C: InternalVoteResultCallback,
    > InternalVoteResultCallback for (A, B, C)
{
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        let mut outcome = A::on_internal_vote_finalized(proposal_id, approved)?;
        outcome = merge_execution_outcome(
            outcome,
            B::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            C::on_internal_vote_finalized(proposal_id, approved)?,
        );
        Ok(outcome)
    }

    fn can_cancel_passed_proposal(
        proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        let a = A::can_cancel_passed_proposal(proposal_id)?;
        let b = B::can_cancel_passed_proposal(proposal_id)?;
        let c = C::can_cancel_passed_proposal(proposal_id)?;
        Ok(merge_cancel_decision(merge_cancel_decision(a, b), c))
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        A::on_execution_failed_terminal(proposal_id)?;
        B::on_execution_failed_terminal(proposal_id)?;
        C::on_execution_failed_terminal(proposal_id)
    }
}

impl<
        A: InternalVoteResultCallback,
        B: InternalVoteResultCallback,
        C: InternalVoteResultCallback,
        D: InternalVoteResultCallback,
    > InternalVoteResultCallback for (A, B, C, D)
{
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        let mut outcome = A::on_internal_vote_finalized(proposal_id, approved)?;
        outcome = merge_execution_outcome(
            outcome,
            B::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            C::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            D::on_internal_vote_finalized(proposal_id, approved)?,
        );
        Ok(outcome)
    }

    fn can_cancel_passed_proposal(
        proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        let a = A::can_cancel_passed_proposal(proposal_id)?;
        let b = B::can_cancel_passed_proposal(proposal_id)?;
        let c = C::can_cancel_passed_proposal(proposal_id)?;
        let d = D::can_cancel_passed_proposal(proposal_id)?;
        Ok(merge_cancel_decision(
            merge_cancel_decision(merge_cancel_decision(a, b), c),
            d,
        ))
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        A::on_execution_failed_terminal(proposal_id)?;
        B::on_execution_failed_terminal(proposal_id)?;
        C::on_execution_failed_terminal(proposal_id)?;
        D::on_execution_failed_terminal(proposal_id)
    }
}

impl<
        A: InternalVoteResultCallback,
        B: InternalVoteResultCallback,
        C: InternalVoteResultCallback,
        D: InternalVoteResultCallback,
        E: InternalVoteResultCallback,
    > InternalVoteResultCallback for (A, B, C, D, E)
{
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        let mut outcome = A::on_internal_vote_finalized(proposal_id, approved)?;
        outcome = merge_execution_outcome(
            outcome,
            B::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            C::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            D::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            E::on_internal_vote_finalized(proposal_id, approved)?,
        );
        Ok(outcome)
    }

    fn can_cancel_passed_proposal(
        proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        let a = A::can_cancel_passed_proposal(proposal_id)?;
        let b = B::can_cancel_passed_proposal(proposal_id)?;
        let c = C::can_cancel_passed_proposal(proposal_id)?;
        let d = D::can_cancel_passed_proposal(proposal_id)?;
        let e = E::can_cancel_passed_proposal(proposal_id)?;
        Ok(merge_cancel_decision(
            merge_cancel_decision(merge_cancel_decision(merge_cancel_decision(a, b), c), d),
            e,
        ))
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        A::on_execution_failed_terminal(proposal_id)?;
        B::on_execution_failed_terminal(proposal_id)?;
        C::on_execution_failed_terminal(proposal_id)?;
        D::on_execution_failed_terminal(proposal_id)?;
        E::on_execution_failed_terminal(proposal_id)
    }
}

impl<
        A: InternalVoteResultCallback,
        B: InternalVoteResultCallback,
        C: InternalVoteResultCallback,
        D: InternalVoteResultCallback,
        E: InternalVoteResultCallback,
        F: InternalVoteResultCallback,
    > InternalVoteResultCallback for (A, B, C, D, E, F)
{
    fn on_internal_vote_finalized(
        proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        let mut outcome = A::on_internal_vote_finalized(proposal_id, approved)?;
        outcome = merge_execution_outcome(
            outcome,
            B::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            C::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            D::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            E::on_internal_vote_finalized(proposal_id, approved)?,
        );
        outcome = merge_execution_outcome(
            outcome,
            F::on_internal_vote_finalized(proposal_id, approved)?,
        );
        Ok(outcome)
    }

    fn can_cancel_passed_proposal(
        proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        let a = A::can_cancel_passed_proposal(proposal_id)?;
        let b = B::can_cancel_passed_proposal(proposal_id)?;
        let c = C::can_cancel_passed_proposal(proposal_id)?;
        let d = D::can_cancel_passed_proposal(proposal_id)?;
        let e = E::can_cancel_passed_proposal(proposal_id)?;
        let f = F::can_cancel_passed_proposal(proposal_id)?;
        Ok(merge_cancel_decision(
            merge_cancel_decision(
                merge_cancel_decision(merge_cancel_decision(merge_cancel_decision(a, b), c), d),
                e,
            ),
            f,
        ))
    }

    fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
        A::on_execution_failed_terminal(proposal_id)?;
        B::on_execution_failed_terminal(proposal_id)?;
        C::on_execution_failed_terminal(proposal_id)?;
        D::on_execution_failed_terminal(proposal_id)?;
        E::on_execution_failed_terminal(proposal_id)?;
        F::on_execution_failed_terminal(proposal_id)
    }
}

/// 中文注释：内部管理员动态提供器（可由其他治理模块提供最新管理员集合）。
///
/// 一致性契约：
/// - `is_internal_admin(institution_code, institution, who) == true` 时，同一链上状态读取到的
///   `get_admin_list(institution_code, institution)` 必须包含 `who`。
/// - Pending 版本的 `is_pending_internal_admin` 与 `get_pending_admin_list`
///   必须满足同样强一致关系。
///
/// 投票引擎会在写入管理员快照后再次校验发起人属于快照；provider 实现若出现
/// drift，会被视为权限错误并回滚提案创建。
pub trait InternalAdminProvider<AccountId> {
    fn is_internal_admin(
        institution_code: InstitutionCode,
        institution: AccountId,
        who: &AccountId,
    ) -> bool;

    /// 获取机构当前管理员列表（用于提案创建时锁定快照）。
    fn get_admin_list(
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<sp_std::vec::Vec<AccountId>> {
        None
    }

    /// 查询 Pending 账户管理员权限。仅供创建/激活该账户的投票入口使用。
    fn is_pending_internal_admin(
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    /// 获取机构法定代表人(机构首脑,必为该机构 admins 之一;ADR-027 立法签署人)。
    /// 默认 None(个人账户/无代表人语境);机构由 admins 模块 提供并保证 ∈ admins。
    fn legal_representative(
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<AccountId> {
        None
    }

    /// 获取护宪大法官成员集(ADR-027 修订:修宪最终否决,宪法第21条)。
    /// 护宪大法官归口国家司法院,今后按管理员「职务」字段过滤 NJD admins 取这 7 人;
    /// 字段扩展前默认空(生产解析待管理员字段扩展)。修宪护宪表决按本集合 >半数 判定。
    fn constitution_guard_members() -> sp_std::vec::Vec<AccountId> {
        sp_std::vec::Vec::new()
    }

    /// 获取 Pending 账户管理员列表。仅供创建/激活该账户时锁定快照。
    fn get_pending_admin_list(
        _institution_code: InstitutionCode,
        _institution: AccountId,
    ) -> Option<sp_std::vec::Vec<AccountId>> {
        None
    }
}

impl<AccountId> InternalAdminProvider<AccountId> for () {
    fn is_internal_admin(
        _institution_code: InstitutionCode,
        _institution: AccountId,
        _who: &AccountId,
    ) -> bool {
        false
    }
}

/// 内部管理员总人数提供器。
/// 联合投票会根据“剩余管理员数是否还能让赞成票达到阈值”来自动判定机构反对。
pub trait InternalAdminsLenProvider<AccountId> {
    fn admins_len(institution_code: InstitutionCode, institution: AccountId) -> Option<u32>;
}

impl<AccountId> InternalAdminsLenProvider<AccountId> for () {
    fn admins_len(_institution_code: InstitutionCode, _institution: AccountId) -> Option<u32> {
        None
    }
}

// ──────────────────────────────────────────────────────────────────
// 投票引擎核心 → mode pallet 的反向调用 trait
// votingengine 主 crate 的 finalize / cleanup / on_initialize 路径通过这些
// trait 派发到对应 mode pallet 的实现。
// ──────────────────────────────────────────────────────────────────

/// 内部投票超时结算入口,由 internal-vote pallet 实现。
///
/// votingengine 主 crate 的 `finalize_proposal` extrinsic 与 `on_initialize`
/// 自动结算逻辑遇到 `STAGE_INTERNAL` 时通过本 trait 派发,业务实现住在 sub-pallet。
pub trait InternalProposalFinalizer<BlockNumber, AccountId> {
    fn finalize_internal_timeout(
        proposal: &crate::Proposal<BlockNumber, AccountId>,
        proposal_id: u64,
    ) -> DispatchResult;
}

impl<BlockNumber, AccountId> InternalProposalFinalizer<BlockNumber, AccountId> for () {
    fn finalize_internal_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "InternalProposalFinalizerNotConfigured",
        ))
    }
}

/// 单步分块清理结果。`(removed_count, has_remaining)`。
pub type CleanupChunkResult = (u32, bool);

/// internal mode 的 chunked cleanup 入口。
///
/// votingengine 主 crate 维护 `PendingProposalCleanups` 状态机,但 internal mode
/// 自己的 storage(`InternalVotesByAccount` / `InternalTallies` / `InternalThresholdSnapshot`)
/// 住在 sub-pallet,所以清理动作必须通过本 trait 派发。
pub trait InternalCleanupHandler {
    /// 内部提案成功执行后的 mode 侧副作用。
    ///
    /// 中文注释：internal-vote 在这里激活/删除动态阈值，核心 votingengine 不解析
    /// 业务数据，也不把阈值职责交回业务模块。
    fn on_internal_proposal_executed(_proposal_id: u64) -> DispatchResult {
        Ok(())
    }

    /// 内部提案进入终态后的 mode 侧清理。
    ///
    /// 中文注释：注册被拒绝或执行失败时，internal-vote 用此入口清掉 pending 阈值。
    fn on_internal_proposal_terminal(_proposal_id: u64, _status: u8) -> DispatchResult {
        Ok(())
    }

    /// 分块清理 InternalVotesByAccount。
    /// 返回 `(removed_this_chunk, has_remaining)`。
    fn cleanup_internal_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;

    /// 终态清理:删 InternalTallies + InternalThresholdSnapshot(单步完成,小 storage)。
    fn cleanup_internal_terminal(proposal_id: u64);
}

impl InternalCleanupHandler for () {
    fn cleanup_internal_votes_chunk(_proposal_id: u64, _limit: u32) -> CleanupChunkResult {
        (0, false)
    }

    fn cleanup_internal_terminal(_proposal_id: u64) {}
}

/// 联合投票超时结算入口。joint-vote sub-pallet 实现。
///
/// 联合投票分两阶段:内部投票阶段(STAGE_JOINT)+ 联合公投阶段(STAGE_REFERENDUM)。
/// votingengine 主 crate 的 finalize 路径根据 stage 选择派发到这两个 fn。
pub trait JointProposalFinalizer<BlockNumber, AccountId> {
    fn finalize_joint_timeout(
        proposal: &crate::Proposal<BlockNumber, AccountId>,
        proposal_id: u64,
    ) -> DispatchResult;

    fn finalize_jointreferendum_timeout(
        proposal: &crate::Proposal<BlockNumber, AccountId>,
        proposal_id: u64,
    ) -> DispatchResult;
}

impl<BlockNumber, AccountId> JointProposalFinalizer<BlockNumber, AccountId> for () {
    fn finalize_joint_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other("JointProposalFinalizerNotConfigured"))
    }

    fn finalize_jointreferendum_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other("JointProposalFinalizerNotConfigured"))
    }
}

/// joint mode 的 chunked cleanup 入口。
///
/// joint storage(JointVotesByAdmin / JointInstitutionTallies / JointVotesByInstitution
/// / JointTallies / ReferendumVotesByBindingId / ReferendumTallies / UsedPopulationSnapshotNonce)
/// 住在 joint-vote pallet,votingengine 主 crate 通过本 trait 派发清理。
pub trait JointCleanupHandler {
    fn cleanup_joint_admin_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;
    fn cleanup_joint_institution_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;
    fn cleanup_joint_institution_tallies_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;
    fn cleanup_referendum_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;

    /// 终态清理:删 JointTallies + ReferendumTallies(单步)。
    fn cleanup_joint_terminal(proposal_id: u64);
}

impl JointCleanupHandler for () {
    fn cleanup_joint_admin_votes_chunk(_proposal_id: u64, _limit: u32) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_joint_institution_votes_chunk(_proposal_id: u64, _limit: u32) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_joint_institution_tallies_chunk(
        _proposal_id: u64,
        _limit: u32,
    ) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_referendum_votes_chunk(_proposal_id: u64, _limit: u32) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_joint_terminal(_proposal_id: u64) {}
}

// ──────────────────────────────────────────────────────────────────
// 立法投票(legislation-vote)mode trait(ADR-027)
// 核心 votingengine 按 PROPOSAL_KIND_LEGISLATION / STAGE_LEG_* 分发到这些 trait。
// 三个投票 sub-pallet(internal/joint/citizen)逻辑零改动。
// ──────────────────────────────────────────────────────────────────

/// 立法投票超时结算入口。legislation-vote sub-pallet 实现。
/// 四阶段(ADR-027 修订 2026-06-25):内部表决(STAGE_LEG_HOUSE,单院一段/两院顺序两段)
/// + 强制公投(STAGE_LEG_REFERENDUM)+ 行政签署(STAGE_LEG_SIGN)+ 三人会签(STAGE_LEG_OVERRIDE)。
pub trait LegislationProposalFinalizer<BlockNumber, AccountId> {
    fn finalize_legislation_house_timeout(
        proposal: &crate::Proposal<BlockNumber, AccountId>,
        proposal_id: u64,
    ) -> DispatchResult;

    fn finalize_legislation_referendum_timeout(
        proposal: &crate::Proposal<BlockNumber, AccountId>,
        proposal_id: u64,
    ) -> DispatchResult;

    /// 行政签署阶段超时:市级(无 legislature)= 视为通过(PASSED);省/国级 = 退回三人会签。
    fn finalize_legislation_sign_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Ok(())
    }

    /// 三人会签阶段超时:法案否决(REJECTED)。
    fn finalize_legislation_override_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Ok(())
    }

    /// 护宪大法官终审阶段超时(仅修宪):未获多数通过 → 法案否决(REJECTED)。
    fn finalize_legislation_guard_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Ok(())
    }
}

impl<BlockNumber, AccountId> LegislationProposalFinalizer<BlockNumber, AccountId> for () {
    fn finalize_legislation_house_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "LegislationProposalFinalizerNotConfigured",
        ))
    }

    fn finalize_legislation_referendum_timeout(
        _proposal: &crate::Proposal<BlockNumber, AccountId>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other(
            "LegislationProposalFinalizerNotConfigured",
        ))
    }
}

/// 立法投票 mode 的 chunked cleanup 入口。
/// legislation-vote 自有账本(LegHouseVotesByAdmin / LegReferendumVotesByBindingId /
/// LegHouseTally / LegReferendumTally / LegMeta 等)住在 sub-pallet,核心通过本 trait 派发清理。
pub trait LegislationCleanupHandler {
    fn cleanup_legislation_house_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;
    fn cleanup_legislation_referendum_votes_chunk(
        proposal_id: u64,
        limit: u32,
    ) -> CleanupChunkResult;

    /// 终态清理:删 LegMeta + LegHouseTally + LegReferendumTally 等小 storage(单步)。
    fn cleanup_legislation_terminal(proposal_id: u64);
}

impl LegislationCleanupHandler for () {
    fn cleanup_legislation_house_votes_chunk(_proposal_id: u64, _limit: u32) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_legislation_referendum_votes_chunk(
        _proposal_id: u64,
        _limit: u32,
    ) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_legislation_terminal(_proposal_id: u64) {}
}

/// 立法投票终态业务回调(对称于 `JointVoteResultCallback`)。
/// 核心在立法提案进入 PASSED/REJECTED/EXECUTION_FAILED 时按 kind 广播到业务壳 legislation-yuan。
pub trait LegislationVoteResultCallback {
    fn on_legislation_vote_finalized(
        vote_proposal_id: u64,
        approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError>;

    fn can_cancel_passed_proposal(
        _proposal_id: u64,
    ) -> Result<ProposalCancelDecision, DispatchError> {
        Ok(ProposalCancelDecision::Ignored)
    }

    fn on_execution_failed_terminal(_proposal_id: u64) -> DispatchResult {
        Ok(())
    }
}

impl LegislationVoteResultCallback for () {
    fn on_legislation_vote_finalized(
        _vote_proposal_id: u64,
        _approved: bool,
    ) -> Result<ProposalExecutionOutcome, DispatchError> {
        Ok(ProposalExecutionOutcome::Ignored)
    }
}

// ──────────────────────────────────────────────────────────────────
// CID 资格 / 凭证 trait
// votingengine::Config 用作 bound,joint-vote pallet 在 jointreferendum 阶段
// 调用以判定 CID 持有者投票资格并消耗一次性凭证。
// ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VoteCredentialCleanup {
    pub removed: u32,
    pub loops: u32,
    pub has_remaining: bool,
}

impl VoteCredentialCleanup {
    pub const fn done() -> Self {
        Self {
            removed: 0,
            loops: 0,
            has_remaining: false,
        }
    }
}

/// 中文注释：公民投票资格实时验签。签发身份统一从机构 admins 校验。
pub trait CidEligibility<AccountId, Hash> {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        binding_id: &Hash,
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
        issuer_cid_number: &[u8],
        issuer_main_account: &AccountId,
        signer_pubkey: &[u8; 32],
        scope_province_name: &[u8],
        scope_city_name: &[u8],
    ) -> bool;

    /// 清理某个联合/公民提案对应的投票凭证防重放状态。
    fn cleanup_vote_credentials(_proposal_id: u64) {}

    /// 分块清理某个提案维度下的投票凭证。
    fn cleanup_vote_credentials_chunk(proposal_id: u64, _limit: u32) -> VoteCredentialCleanup {
        Self::cleanup_vote_credentials(proposal_id);
        let _ = proposal_id;
        VoteCredentialCleanup::done()
    }
}

impl<AccountId, Hash> CidEligibility<AccountId, Hash> for () {
    fn is_eligible(_binding_id: &Hash, _who: &AccountId) -> bool {
        false
    }

    fn verify_and_consume_vote_credential(
        _binding_id: &Hash,
        _who: &AccountId,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
        _issuer_cid_number: &[u8],
        _issuer_main_account: &AccountId,
        _signer_pubkey: &[u8; 32],
        _scope_province_name: &[u8],
        _scope_city_name: &[u8],
    ) -> bool {
        false
    }
}
