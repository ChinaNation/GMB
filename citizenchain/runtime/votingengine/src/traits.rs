//! 投票引擎对外 trait 定义与默认 `()` 实现。
//!
//! 本文件集中所有可被 runtime / 业务 pallet 注入的 trait,以及它们的默认 `()` 实现,
//! 让 lib.rs 主体只保留 `#[pallet]` 宏与 storage/extrinsic 声明。

use frame_support::dispatch::DispatchResult;
use sp_runtime::DispatchError;

use crate::{InstitutionPalletId, ProposalCancelDecision, ProposalExecutionOutcome};

pub trait JointVoteEngine<AccountId> {
    fn create_joint_proposal(
        who: AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data(
        who: AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data_and_object(
        _who: AccountId,
        _eligible_total: u64,
        _snapshot_nonce: &[u8],
        _signature: &[u8],
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
        _object_kind: u8,
        _object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "JointVoteEngineObjectStoreNotConfigured",
        ))
    }
}

impl<AccountId> JointVoteEngine<AccountId> for () {
    fn create_joint_proposal(
        _who: AccountId,
        _eligible_total: u64,
        _snapshot_nonce: &[u8],
        _signature: &[u8],
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }

    fn create_joint_proposal_with_data(
        _who: AccountId,
        _eligible_total: u64,
        _snapshot_nonce: &[u8],
        _signature: &[u8],
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }
}

/// 事项模块接入内部投票时,统一由投票引擎创建提案并返回真实提案 ID。
///
/// 业务模块通过 `create_internal_proposal` 将普通 Active 主体提案注册到投票引擎,
/// 仅创建/激活 Pending 主体时使用 `create_pending_subject_internal_proposal`。
/// 投票动作不经此 trait 转发——所有管理员直接调公开的
/// `InternalVote::cast(proposal_id, approve)` extrinsic 投票,
/// 由投票引擎的 `InternalVoteResultCallback` 广播回调业务模块执行业务。
pub trait InternalVoteEngine<AccountId> {
    fn create_internal_proposal(
        who: AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError>;

    fn create_internal_proposal_with_data(
        who: AccountId,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;

    /// 创建普通内部提案,**显式传 threshold**(不走 InternalThresholdProvider 反查)。
    ///
    /// 用于"主体生命周期"语义的内部提案 —— 比如关闭 ORG_REN 多签,
    /// 业务规则要求**全员通过**(threshold = admins.len()),不是用户自定义 m-of-n。
    ///
    /// admins 仍从 active 主体反查(InternalAdminProvider::get_admin_list),
    /// 仅 threshold 显式传入。
    fn create_internal_proposal_with_threshold_and_data(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
        _threshold: u32,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "InternalProposalWithThresholdNotConfigured",
        ))
    }

    fn create_pending_subject_internal_proposal(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "PendingSubjectVoteEngineNotConfigured",
        ))
    }

    fn create_pending_subject_internal_proposal_with_data(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "PendingSubjectVoteEngineNotConfigured",
        ))
    }

    fn create_pending_subject_internal_proposal_with_snapshot_data(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
        _admins: sp_std::vec::Vec<AccountId>,
        _threshold: u32,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "PendingSubjectSnapshotVoteEngineNotConfigured",
        ))
    }

    fn create_admin_set_mutation_internal_proposal(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "AdminSetMutationVoteEngineNotConfigured",
        ))
    }

    fn create_admin_set_mutation_internal_proposal_with_data(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "AdminSetMutationVoteEngineNotConfigured",
        ))
    }
}

impl<AccountId> InternalVoteEngine<AccountId> for () {
    fn create_internal_proposal(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("InternalVoteEngineNotConfigured"))
    }

    fn create_internal_proposal_with_data(
        _who: AccountId,
        _org: u8,
        _institution: InstitutionPalletId,
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("InternalVoteEngineNotConfigured"))
    }
}

/// 中文注释：公民总人口快照验签接口（由 runtime 对接 SFID 系统）。
/// ADR-008 step3:`(province, signer_admin_pubkey)` 必须随 payload 一起进 SCALE 哈希,
/// runtime verifier 按 `ShengSigningPubkey` 双映射查派生签名公钥并验签;
/// 链上 0 prior knowledge of SFID,无任何"SFID main 兜底"路径。
pub trait PopulationSnapshotVerifier<AccountId, Nonce, Signature> {
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &Nonce,
        signature: &Signature,
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
    ) -> bool;
}

impl<AccountId, Nonce, Signature> PopulationSnapshotVerifier<AccountId, Nonce, Signature> for () {
    fn verify_population_snapshot(
        _who: &AccountId,
        _eligible_total: u64,
        _nonce: &Nonce,
        _signature: &Signature,
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
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

/// 默认空实现(runtime 在 Phase 2 业务模块改造前临时挂 `type X = ()`)。
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
// 注:Phase 2 预计注册 5 个业务模块(duoqian_transfer /
// org_manage / admins_change / resolution_destro /
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
/// - `is_internal_admin(org, institution, who) == true` 时，同一链上状态读取到的
///   `get_admin_list(org, institution)` 必须包含 `who`。
/// - Pending 版本的 `is_pending_internal_admin` 与 `get_pending_admin_list`
///   必须满足同样强一致关系。
///
/// 投票引擎会在写入管理员快照后再次校验发起人属于快照；provider 实现若出现
/// drift，会被视为权限错误并回滚提案创建。
pub trait InternalAdminProvider<AccountId> {
    fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId) -> bool;

    /// 获取机构当前管理员列表（用于提案创建时锁定快照）。
    fn get_admin_list(
        _org: u8,
        _institution: InstitutionPalletId,
    ) -> Option<sp_std::vec::Vec<AccountId>> {
        None
    }

    /// 查询 Pending 主体管理员权限。仅供创建/激活该主体的投票入口使用。
    fn is_pending_internal_admin(
        _org: u8,
        _institution: InstitutionPalletId,
        _who: &AccountId,
    ) -> bool {
        false
    }

    /// 获取 Pending 主体管理员列表。仅供创建/激活该主体时锁定快照。
    fn get_pending_admin_list(
        _org: u8,
        _institution: InstitutionPalletId,
    ) -> Option<sp_std::vec::Vec<AccountId>> {
        None
    }
}

impl<AccountId> InternalAdminProvider<AccountId> for () {
    fn is_internal_admin(_org: u8, _institution: InstitutionPalletId, _who: &AccountId) -> bool {
        false
    }
}

/// 内部管理员总人数提供器。
/// 联合投票会根据“剩余管理员数是否还能让赞成票达到阈值”来自动判定机构反对。
pub trait InternalAdminCountProvider {
    fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32>;
}

impl InternalAdminCountProvider for () {
    fn admin_count(org: u8, institution: InstitutionPalletId) -> Option<u32> {
        match org {
            crate::types::ORG_NRC | crate::types::ORG_PRC => {
                use primitives::china::china_cb::{
                    shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
                };
                CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok())
            }
            crate::types::ORG_PRB => {
                use primitives::china::china_ch::{
                    shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
                };
                CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok())
            }
            _ => None,
        }
    }
}

/// 注册多签内部投票阈值提供器。
/// 中文注释：治理三类机构阈值由固定制度常量提供；本 Provider 只承接注册多签主体阈值。
pub trait InternalThresholdProvider {
    /// 查询 Active 主体是否存在。用于机构合法性判断，不与阈值读取混用。
    fn is_known_subject(_org: u8, _institution: InstitutionPalletId) -> bool {
        false
    }

    /// 查询 Pending 主体是否存在。仅供创建/激活该主体的投票入口使用。
    fn is_known_pending_subject(_org: u8, _institution: InstitutionPalletId) -> bool {
        false
    }

    fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32>;

    /// Pending 注册多签主体创建投票使用的阈值。普通业务不得通过此方法授权。
    fn pending_pass_threshold(_org: u8, _institution: InstitutionPalletId) -> Option<u32> {
        None
    }
}

/// 默认实现不提供任何阈值，强制 runtime / mock runtime 显式注入真实 Provider。
impl InternalThresholdProvider for () {
    fn pass_threshold(_org: u8, _institution: InstitutionPalletId) -> Option<u32> {
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
pub trait InternalProposalFinalizer<BlockNumber> {
    fn finalize_internal_timeout(
        proposal: &crate::Proposal<BlockNumber>,
        proposal_id: u64,
    ) -> DispatchResult;
}

impl<BlockNumber> InternalProposalFinalizer<BlockNumber> for () {
    fn finalize_internal_timeout(
        _proposal: &crate::Proposal<BlockNumber>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other("InternalProposalFinalizerNotConfigured"))
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
/// 联合投票分两阶段:管理员阶段(STAGE_JOINT)+ 全民兜底阶段(STAGE_CITIZEN)。
/// votingengine 主 crate 的 finalize 路径根据 stage 选择派发到这两个 fn。
pub trait JointProposalFinalizer<BlockNumber> {
    fn finalize_joint_timeout(
        proposal: &crate::Proposal<BlockNumber>,
        proposal_id: u64,
    ) -> DispatchResult;

    fn finalize_jointreferendum_timeout(
        proposal: &crate::Proposal<BlockNumber>,
        proposal_id: u64,
    ) -> DispatchResult;
}

impl<BlockNumber> JointProposalFinalizer<BlockNumber> for () {
    fn finalize_joint_timeout(
        _proposal: &crate::Proposal<BlockNumber>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other("JointProposalFinalizerNotConfigured"))
    }

    fn finalize_jointreferendum_timeout(
        _proposal: &crate::Proposal<BlockNumber>,
        _proposal_id: u64,
    ) -> DispatchResult {
        Err(DispatchError::Other("JointProposalFinalizerNotConfigured"))
    }
}

/// joint mode 的 chunked cleanup 入口。
///
/// joint storage(JointVotesByAdmin / JointInstitutionTallies / JointVotesByInstitution
/// / JointTallies / CitizenVotesByBindingId / CitizenTallies / UsedPopulationSnapshotNonce)
/// 住在 joint-vote pallet,votingengine 主 crate 通过本 trait 派发清理。
pub trait JointCleanupHandler {
    fn cleanup_joint_admin_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;
    fn cleanup_joint_institution_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;
    fn cleanup_joint_institution_tallies_chunk(proposal_id: u64, limit: u32)
        -> CleanupChunkResult;
    fn cleanup_citizen_votes_chunk(proposal_id: u64, limit: u32) -> CleanupChunkResult;

    /// 终态清理:删 JointTallies + CitizenTallies(单步)。
    fn cleanup_joint_terminal(proposal_id: u64);
}

impl JointCleanupHandler for () {
    fn cleanup_joint_admin_votes_chunk(_proposal_id: u64, _limit: u32) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_joint_institution_votes_chunk(
        _proposal_id: u64,
        _limit: u32,
    ) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_joint_institution_tallies_chunk(
        _proposal_id: u64,
        _limit: u32,
    ) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_citizen_votes_chunk(_proposal_id: u64, _limit: u32) -> CleanupChunkResult {
        (0, false)
    }
    fn cleanup_joint_terminal(_proposal_id: u64) {}
}

// ──────────────────────────────────────────────────────────────────
// SFID 资格 / 凭证 trait
// votingengine::Config 用作 bound,joint-vote pallet 在 jointreferendum 阶段
// 调用以判定 SFID 持有者投票资格并消耗一次性凭证。
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

/// ADR-008 step3:`verify_and_consume_vote_credential` 加 `(province, signer_admin_pubkey)`
/// 双层匹配字段,链上不再保留任何"SFID main 兜底"路径。
pub trait SfidEligibility<AccountId, Hash> {
    fn is_eligible(binding_id: &Hash, who: &AccountId) -> bool;
    fn verify_and_consume_vote_credential(
        binding_id: &Hash,
        who: &AccountId,
        proposal_id: u64,
        nonce: &[u8],
        signature: &[u8],
        province: &[u8],
        signer_admin_pubkey: &[u8; 32],
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

impl<AccountId, Hash> SfidEligibility<AccountId, Hash> for () {
    fn is_eligible(_binding_id: &Hash, _who: &AccountId) -> bool {
        false
    }

    fn verify_and_consume_vote_credential(
        _binding_id: &Hash,
        _who: &AccountId,
        _proposal_id: u64,
        _nonce: &[u8],
        _signature: &[u8],
        _province: &[u8],
        _signer_admin_pubkey: &[u8; 32],
    ) -> bool {
        false
    }
}
