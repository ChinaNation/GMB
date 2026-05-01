//! # 治理投票引擎 (voting-engine)
//!
//! 治理投票基础设施模块，统一承载三类投票流程：
//! - **内部投票**（INTERNAL）：机构内部管理员按阈值投票，赞成 ≥ 阈值提前通过，
//!   剩余票不足达到阈值提前否决，30 天超时兜底否决。
//! - **联合机构投票**（JOINT）：国储会/省储会/省储行管理员按票权加权投票，
//!   105 票全票通过直接执行，任一机构反对立即进入公民投票，30 天超时进入公民投票。
//! - **公民投票**（CITIZEN）：SFID 持有者按 >50% 严格多数投票，
//!   赞成 > 50% 提前通过，反对 ≥ 50% 提前否决，30 天超时按最终票数判定。
//!
//! 关键机制：
//! - **管理员快照锁定**：提案创建时锁定管理员名单，投票期间不受链上管理员更换影响。
//! - **联合提案发起权**：国储会和省储会管理员均可发起联合投票提案。
//!
//! 通过 trait 为上层治理模块提供标准化能力：
//! - `InternalVoteEngine` / `JointVoteEngine`：仅负责提案创建(不再负责投票,
//!   Phase 1 整改后投票一律走本 pallet 的公开 `internal_vote / joint_vote /
//!   citizen_vote` extrinsic)。
//! - `InternalVoteResultCallback` / `JointVoteResultCallback`:内部/联合提案
//!   完成投票判定时,投票引擎按统一状态机调用业务 executor。
//!   业务模块只返回统一执行结果，不再直接推进投票引擎状态；PASSED 表示执行授权/可重试态。
//! - 自动超时结算、原子终结 + 回调一致性(回调返回 Err 整体回滚)、
//!   90 天延迟分块清理。

#![cfg_attr(not(feature = "std"), no_std)]

pub mod active_proposal_limit;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarks;
pub mod citizen_vote;
pub mod internal_vote;
pub mod joint_vote;
pub mod proposal_cleanup;
pub mod weights;

pub use citizen_vote::{SfidEligibility, VoteCredentialCleanup};
pub use internal_vote::ORG_DUOQIAN;
pub use pallet::*;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
use sp_runtime::DispatchError;

pub type InstitutionPalletId = [u8; 48];

/// 国储会 InstitutionPalletId（从 CHINA_CB 第一条记录派生）。
/// 公共函数，供 internal_vote、joint_vote 等子模块共用。
pub fn nrc_pallet_id_bytes() -> Option<InstitutionPalletId> {
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    CHINA_CB
        .first()
        .and_then(|n| reserve_pallet_id_to_bytes(n.shenfen_id))
}

pub const PROPOSAL_KIND_INTERNAL: u8 = 0;
pub const PROPOSAL_KIND_JOINT: u8 = 1;

pub const STAGE_INTERNAL: u8 = 0;
pub const STAGE_JOINT: u8 = 1;
pub const STAGE_CITIZEN: u8 = 2;

pub const STATUS_VOTING: u8 = 0;
pub const STATUS_PASSED: u8 = 1;
pub const STATUS_REJECTED: u8 = 2;
/// 提案已执行完成（终态）。消费模块在业务逻辑成功后推进到该状态。
pub const STATUS_EXECUTED: u8 = 3;
/// 投票通过但业务执行失败（终态）。只由投票引擎在重试耗尽、超时或业务永久失败时写入。
pub const STATUS_EXECUTION_FAILED: u8 = 4;

/// 业务模块统一执行结果。
///
/// 中文注释：业务模块只表达“业务动作执行结果”，不再直接改写提案状态。
/// 投票引擎根据该结果统一维护 PASSED / EXECUTED / EXECUTION_FAILED 状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ProposalExecutionOutcome {
    /// 不是本模块提案。
    Ignored,
    /// 业务执行成功。
    Executed,
    /// 暂时失败，保留 PASSED 并允许管理员手动重试。
    RetryableFailed,
    /// 确定不可执行，进入 EXECUTION_FAILED 终态。
    FatalFailed,
}

/// 业务模块对 `PASSED` 重试提案是否允许管理员提前取消的决策。
///
/// 中文注释：`MODULE_TAG` 只用于路由识别，不能作为权限凭据；因此取消必须由真正
/// 认领该提案的 callback 显式返回 `Allow`，默认实现一律 `Ignored`。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum ProposalCancelDecision {
    /// 不是本模块提案。
    Ignored,
    /// 本模块确认该提案已不可执行，允许进入 EXECUTION_FAILED 终态。
    Allow,
}

/// 内部提案互斥类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum InternalProposalMutexKind {
    /// 普通内部治理事项，允许同主体多个普通事项并行。
    Regular,
    /// 管理员集合变更，同主体下必须独占。
    AdminSetMutationExclusive,
}

/// 同一治理主体下的互斥状态。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct InternalProposalMutexState {
    /// 当前占用管理员集合变更独占锁的提案。
    pub admin_set_mutation_proposal: Option<u64>,
    /// 当前普通活跃提案数量。
    pub regular_active_count: u32,
}

impl InternalProposalMutexState {
    fn is_empty(&self) -> bool {
        self.admin_set_mutation_proposal.is_none() && self.regular_active_count == 0
    }
}

/// proposal_id 到互斥锁的反向绑定，用于终态/阶段切换时释放锁。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct InternalProposalMutexBinding {
    pub org: u8,
    pub institution: InstitutionPalletId,
    pub kind: InternalProposalMutexKind,
}

/// 自动执行失败后的统一重试状态。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ExecutionRetryState<BlockNumber> {
    /// 已失败的手动执行次数。自动执行失败不计入该次数。
    pub manual_attempts: u8,
    /// 第一次自动执行失败所在区块。
    pub first_auto_failed_at: BlockNumber,
    /// 超过该区块仍未执行成功，则自动转 EXECUTION_FAILED。
    pub retry_deadline: BlockNumber,
    /// 最近一次手动执行尝试所在区块。
    pub last_attempt_at: Option<BlockNumber>,
}

/// 中文注释：事项模块接入联合投票时，统一由投票引擎创建提案并写入人口快照。
pub trait JointVoteEngine<AccountId> {
    fn create_joint_proposal(
        who: AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
    ) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data(
        who: AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError>;

    fn create_joint_proposal_with_data_and_object(
        _who: AccountId,
        _eligible_total: u64,
        _snapshot_nonce: &[u8],
        _signature: &[u8],
        _module_tag: &[u8],
        _data: sp_std::vec::Vec<u8>,
        _object_kind: u8,
        _object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other(
            "JointVoteEngineObjectStoreNotConfigured",
        ))
    }

    fn cleanup_joint_proposal(_proposal_id: u64) {}
}

impl<AccountId> JointVoteEngine<AccountId> for () {
    fn create_joint_proposal(
        _who: AccountId,
        _eligible_total: u64,
        _snapshot_nonce: &[u8],
        _signature: &[u8],
    ) -> Result<u64, DispatchError> {
        Err(DispatchError::Other("JointVoteEngineNotConfigured"))
    }

    fn create_joint_proposal_with_data(
        _who: AccountId,
        _eligible_total: u64,
        _snapshot_nonce: &[u8],
        _signature: &[u8],
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
/// 投票动作不再经此 trait 转发——所有管理员直接调公开的
/// `VotingEngine::internal_vote(proposal_id, approve)` extrinsic 投票,
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

    fn cleanup_internal_proposal(_proposal_id: u64) {}
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
pub trait PopulationSnapshotVerifier<AccountId, Nonce, Signature> {
    fn verify_population_snapshot(
        who: &AccountId,
        eligible_total: u64,
        nonce: &Nonce,
        signature: &Signature,
    ) -> bool;
}

impl<AccountId, Nonce, Signature> PopulationSnapshotVerifier<AccountId, Nonce, Signature> for () {
    fn verify_population_snapshot(
        _who: &AccountId,
        _eligible_total: u64,
        _nonce: &Nonce,
        _signature: &Signature,
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
// duoqian_manage / admins_change / resolution_destro /
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
            internal_vote::ORG_NRC | internal_vote::ORG_PRC => {
                use primitives::china::china_cb::{
                    shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
                };
                CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .and_then(|n| u32::try_from(n.duoqian_admins.len()).ok())
            }
            internal_vote::ORG_PRB => {
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
    fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32>;

    /// Pending 注册多签主体创建投票使用的阈值。普通业务不得通过此方法授权。
    fn pending_pass_threshold(_org: u8, _institution: InstitutionPalletId) -> Option<u32> {
        None
    }
}

/// 默认实现：仅支持治理机构的固定制度阈值，注册多签主体需要 runtime 注入真实 Provider。
impl InternalThresholdProvider for () {
    fn pass_threshold(org: u8, _institution: InstitutionPalletId) -> Option<u32> {
        internal_vote::fixed_governance_pass_threshold(org)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct Proposal<BlockNumber> {
    /// 提案类型：内部投票/联合投票
    pub kind: u8,
    /// 当前所处投票阶段：内部/联合/公民
    pub stage: u8,
    /// 当前提案状态：投票中/通过/否决
    pub status: u8,
    /// 仅内部投票使用：机构类型（国储会/省储会/省储行）
    pub internal_org: Option<u8>,
    /// 仅内部投票使用：机构 shenfen_id 标识（全链唯一）
    pub internal_institution: Option<InstitutionPalletId>,
    /// 本阶段起始区块
    pub start: BlockNumber,
    /// 本阶段截止区块（超过则超时）
    pub end: BlockNumber,
    /// 公民投票阶段的可投票总人数（由外部资格系统给出）
    pub citizen_eligible_total: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VoteCountU32 {
    /// 赞成票
    pub yes: u32,
    /// 反对票
    pub no: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct VoteCountU64 {
    /// 赞成票
    pub yes: u64,
    /// 反对票
    pub no: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub enum PendingCleanupStage {
    AdminSnapshots,
    InternalVotes,
    JointAdminVotes,
    JointInstitutionVotes,
    JointInstitutionTallies,
    CitizenVotes,
    VoteCredentials,
    /// 清理大对象存储（ProposalObject + ProposalObjectMeta）。
    ProposalObject,
    /// 清理业务数据（ProposalData + ProposalMeta）和核心数据（Proposals + Tallies）。
    /// 这是清理流程的最后一步，单次完成。
    FinalCleanup,
}

/// 提案辅助元数据（由投票引擎统一存储，替代各业务模块的 ProposalCreatedAt / ProposalPassedAt）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ProposalMetadata<BlockNumber> {
    /// 提案创建时的区块号
    pub created_at: BlockNumber,
    /// 提案通过时的区块号（未通过时为 None）
    pub passed_at: Option<BlockNumber>,
}

/// 提案对象层元数据：记录统一对象存储的类型、长度与哈希。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct ProposalObjectMetadata<Hash> {
    /// 对象类型，由业务模块自行定义并在解码时识别。
    pub kind: u8,
    /// 对象字节长度，便于链上/链下快速判断对象规模。
    pub object_len: u32,
    /// 对象内容哈希，用于执行和审计时做一致性校验。
    pub object_hash: Hash,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::UnixTime;
    use frame_support::{
        pallet_prelude::*,
        storage::{with_transaction, TransactionOutcome},
        Blake2_128Concat,
    };
    use frame_system::pallet_prelude::*;
    use sp_runtime::traits::{Hash, One, Saturating};
    use sp_std::vec::Vec;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        #[allow(deprecated)]
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        #[pallet::constant]
        type MaxVoteNonceLength: Get<u32>;

        #[pallet::constant]
        type MaxVoteSignatureLength: Get<u32>;

        /// 每个区块自动处理的“到期提案”上限，避免 on_initialize 无界增长。
        #[pallet::constant]
        type MaxAutoFinalizePerBlock: Get<u32>;

        /// 单个到期区块允许挂载的提案 ID 上限，避免 expiry 桶无界增长。
        #[pallet::constant]
        type MaxProposalsPerExpiry: Get<u32>;

        /// 每个区块最多执行多少个清理步骤，避免历史提案清理拖垮 on_initialize。
        #[pallet::constant]
        type MaxCleanupStepsPerBlock: Get<u32>;

        /// 每个清理步骤最多删除多少条前缀项。
        #[pallet::constant]
        type CleanupKeysPerStep: Get<u32>;

        /// 提案业务数据最大长度（字节），各业务模块序列化后的数据不超过此限制。
        #[pallet::constant]
        type MaxProposalDataLen: Get<u32>;

        /// 提案大对象数据最大长度（字节），用于 runtime wasm 等大载荷。
        #[pallet::constant]
        type MaxProposalObjectLen: Get<u32>;

        /// 业务模块标识最大长度，用于 ProposalOwner 绑定。
        #[pallet::constant]
        type MaxModuleTagLen: Get<u32>;

        /// 自动执行失败后允许的最大手动失败次数。
        #[pallet::constant]
        type MaxManualExecutionAttempts: Get<u32>;

        /// 自动执行失败后等待管理员手动执行的宽限区块数。
        #[pallet::constant]
        type ExecutionRetryGraceBlocks: Get<BlockNumberFor<Self>>;

        /// 单个区块最多处理多少个执行重试超时提案。
        #[pallet::constant]
        type MaxExecutionRetryDeadlinesPerBlock: Get<u32>;

        type SfidEligibility: SfidEligibility<Self::AccountId, Self::Hash>;
        type PopulationSnapshotVerifier: PopulationSnapshotVerifier<
            Self::AccountId,
            VoteNonceOf<Self>,
            VoteSignatureOf<Self>,
        >;

        type JointVoteResultCallback: JointVoteResultCallback;
        /// 内部投票终态回调(对称于 `JointVoteResultCallback`)。
        /// Runtime 用 tuple 注册多个业务模块的 Executor,投票引擎在提案进入
        /// `STATUS_PASSED` / `STATUS_REJECTED` 时广播到每个成员。
        type InternalVoteResultCallback: InternalVoteResultCallback;
        type InternalAdminProvider: InternalAdminProvider<Self::AccountId>;
        type InternalAdminCountProvider: InternalAdminCountProvider;
        /// 内部投票阈值动态提供器（治理机构硬编码，注册多签动态读取）。
        type InternalThresholdProvider: InternalThresholdProvider;

        /// 每个机构最大管理员数量（与 admins-change 一致），用于管理员快照 BoundedVec。
        #[pallet::constant]
        type MaxAdminsPerInstitution: Get<u32>;

        /// 时间源，用于提案 ID 编码年份。
        type TimeProvider: frame_support::traits::UnixTime;

        type WeightInfo: crate::weights::WeightInfo;
    }

    use crate::weights::WeightInfo;

    pub type VoteNonceOf<T> = BoundedVec<u8, <T as Config>::MaxVoteNonceLength>;
    pub type VoteSignatureOf<T> = BoundedVec<u8, <T as Config>::MaxVoteSignatureLength>;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// 当前提案年份（用于年度计数器重置）。
    #[pallet::storage]
    pub type CurrentProposalYear<T> = StorageValue<_, u16, ValueQuery>;

    /// 当前年份内的提案计数器（每年从 0 开始）。
    #[pallet::storage]
    pub type YearProposalCounter<T> = StorageValue<_, u32, ValueQuery>;

    /// 兼容性别名：返回下一个 proposal_id（年份 × 1,000,000 + 计数器）。
    /// 仅供外部查询使用（如 App 扫描提案范围）。
    #[pallet::storage]
    #[pallet::getter(fn next_proposal_id)]
    pub type NextProposalId<T> = StorageValue<_, u64, ValueQuery>;

    /// 全局提案表：proposal_id → 提案元数据（类型/阶段/状态/起止区块/机构等）。
    /// 由 `create_internal_proposal` 写入，`set_status_and_emit` 更新状态，超时清理自动删除。
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, Proposal<BlockNumberFor<T>>, OptionQuery>;

    /// 回调执行作用域：只在 `set_status_and_emit` 调业务回调期间临时存在。
    ///
    /// 中文注释：生产业务模块通过回调返回 `ProposalExecutionOutcome`；该作用域只保护
    /// 单测兼容辅助接口，避免非回调路径绕过最终事件和互斥锁释放逻辑。
    #[pallet::storage]
    pub type CallbackExecutionScopes<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, (), OptionQuery>;

    /// 以“阶段截止区块”索引提案，用于 on_initialize 自动超时结算。
    #[pallet::storage]
    #[pallet::getter(fn proposals_by_expiry)]
    pub type ProposalsByExpiry<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u64, <T as Config>::MaxProposalsPerExpiry>,
        ValueQuery,
    >;

    /// 自动结算游标：记录上个区块未处理完的过期桶。
    #[pallet::storage]
    #[pallet::getter(fn pending_expiry_bucket)]
    pub type PendingExpiryBucket<T: Config> = StorageValue<_, BlockNumberFor<T>, OptionQuery>;

    /// 分块清理游标：按提案维度逐步清理历史投票状态，避免 finalize 路径单次无界删除。
    #[pallet::storage]
    #[pallet::getter(fn pending_cleanup_stage)]
    pub type PendingProposalCleanups<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, PendingCleanupStage, OptionQuery>;

    /// 内部投票记录：(proposal_id, 管理员公钥) → 赞成/反对。防止同一管理员重复投票。
    #[pallet::storage]
    pub type InternalVotesByAccount<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        T::AccountId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn internal_tally)]
    pub type InternalTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    /// 联合投票——管理员级记录：(proposal_id, (机构, 管理员公钥)) → 赞成/反对。
    /// 防止同一管理员在同一机构内重复投票。
    #[pallet::storage]
    pub type JointVotesByAdmin<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        (InstitutionPalletId, T::AccountId),
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_institution_tally)]
    pub type JointInstitutionTallies<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        InstitutionPalletId,
        VoteCountU32,
        ValueQuery,
    >;

    /// 联合投票——机构级汇总：(proposal_id, 机构) → 该机构内部投票的最终结果（赞成/反对）。
    /// 机构内部达到阈值后写入，用于联合阶段权重汇总。
    #[pallet::storage]
    pub type JointVotesByInstitution<T> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        InstitutionPalletId,
        bool,
        OptionQuery,
    >;

    #[pallet::storage]
    #[pallet::getter(fn joint_tally)]
    pub type JointTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU32, ValueQuery>;

    /// 公民投票记录：(proposal_id, 公民身份绑定哈希) → 赞成/反对。
    /// 每个公民身份只能投一次，由绑定哈希防重。
    #[pallet::storage]
    pub type CitizenVotesByBindingId<T: Config> =
        StorageDoubleMap<_, Blake2_128Concat, u64, Blake2_128Concat, T::Hash, bool, OptionQuery>;

    #[pallet::storage]
    #[pallet::getter(fn citizen_tally)]
    pub type CitizenTallies<T> = StorageMap<_, Blake2_128Concat, u64, VoteCountU64, ValueQuery>;

    /// 提案管理员快照：提案创建时锁定各机构管理员名单，投票期间不随链上名单变化。
    /// 内部提案只存一条（提案所属机构），联合提案存所有参与机构（约105条）。
    /// 投票时查快照判定资格，保证管理员更换不影响已有提案的投票过程。
    #[pallet::storage]
    pub type AdminSnapshot<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u64,
        Blake2_128Concat,
        InstitutionPalletId,
        BoundedVec<T::AccountId, T::MaxAdminsPerInstitution>,
        OptionQuery,
    >;

    /// 内部投票阈值快照：提案创建时锁定阈值，投票期间不受主体状态变化影响。
    #[pallet::storage]
    pub type InternalThresholdSnapshot<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, u32, OptionQuery>;

    /// 中文注释：总人口快照 nonce 防重放（全局维度，防止跨提案重放）。
    #[pallet::storage]
    #[pallet::getter(fn used_population_snapshot_nonce)]
    pub type UsedPopulationSnapshotNonce<T: Config> =
        StorageMap<_, Blake2_128Concat, T::Hash, bool, ValueQuery>;

    /// 提案业务数据（由各业务模块序列化后写入，投票引擎统一存储和清理）。
    #[pallet::storage]
    #[pallet::getter(fn proposal_data)]
    pub type ProposalData<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxProposalDataLen>, OptionQuery>;

    /// 提案 owner：proposal_id → 业务模块 MODULE_TAG。
    ///
    /// 中文注释：ProposalOwner 是投票引擎分发自动执行、手动重试和取消的唯一归属来源。
    /// 业务模块不再只依赖 ProposalData 前缀自认领，避免跨模块覆写后静默跳过。
    #[pallet::storage]
    #[pallet::getter(fn proposal_owner)]
    pub type ProposalOwner<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxModuleTagLen>, OptionQuery>;

    /// 自动执行失败后的可重试状态。
    #[pallet::storage]
    #[pallet::getter(fn proposal_execution_retry_state)]
    pub type ProposalExecutionRetryStates<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ExecutionRetryState<BlockNumberFor<T>>, OptionQuery>;

    /// 执行重试超时队列：retry_deadline → proposal_id 列表。
    #[pallet::storage]
    #[pallet::getter(fn execution_retry_deadlines)]
    pub type ExecutionRetryDeadlines<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u64, T::MaxExecutionRetryDeadlinesPerBlock>,
        ValueQuery,
    >;

    /// 提案对象层元数据（对象类型 / 长度 / 哈希），由投票引擎统一存储和清理。
    #[pallet::storage]
    #[pallet::getter(fn proposal_object_meta)]
    pub type ProposalObjectMeta<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ProposalObjectMetadata<T::Hash>, OptionQuery>;

    /// 提案对象层原始数据（例如 runtime wasm），由投票引擎统一存储和清理。
    #[pallet::storage]
    #[pallet::getter(fn proposal_object)]
    pub type ProposalObject<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, BoundedVec<u8, T::MaxProposalObjectLen>, OptionQuery>;

    /// 提案辅助元数据（创建时间、通过时间，由投票引擎统一存储和清理）。
    #[pallet::storage]
    #[pallet::getter(fn proposal_meta)]
    pub type ProposalMeta<T: Config> =
        StorageMap<_, Blake2_128Concat, u64, ProposalMetadata<BlockNumberFor<T>>, OptionQuery>;

    /// 延迟清理队列：按清理到期区块索引待清理的 proposal_id 列表。
    #[pallet::storage]
    pub type CleanupQueue<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BlockNumberFor<T>,
        BoundedVec<u64, ConstU32<50>>,
        ValueQuery,
    >;

    /// 每个机构的活跃提案 ID 列表（全局管控，不区分提案类型，上限 10 个）。
    #[pallet::storage]
    #[pallet::getter(fn active_proposals_by_institution)]
    pub type ActiveProposalsByInstitution<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        InstitutionPalletId,
        BoundedVec<u64, ConstU32<{ crate::active_proposal_limit::MAX_ACTIVE_PROPOSALS }>>,
        ValueQuery,
    >;

    /// 同一治理主体的内部提案互斥状态：(org, institution) → 锁状态。
    #[pallet::storage]
    #[pallet::getter(fn internal_proposal_mutex)]
    pub type InternalProposalMutexes<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        u8,
        Blake2_128Concat,
        InstitutionPalletId,
        InternalProposalMutexState,
        OptionQuery,
    >;

    /// 提案持有的互斥锁列表，用于终态或联合投票进入公民阶段时释放。
    #[pallet::storage]
    #[pallet::getter(fn proposal_mutex_bindings)]
    pub type ProposalMutexBindings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        u64,
        BoundedVec<InternalProposalMutexBinding, ConstU32<128>>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// 中文注释：提案已创建，记录类型、阶段和截止区块。
        ProposalCreated {
            proposal_id: u64,
            kind: u8,
            stage: u8,
            end: BlockNumberFor<T>,
        },
        /// 中文注释：联合投票阶段非全票通过或超时，提案推进到公民投票阶段。
        ProposalAdvancedToCitizen {
            proposal_id: u64,
            citizen_end: BlockNumberFor<T>,
            eligible_total: u64,
        },
        /// 中文注释：投票阶段完成或执行状态变化；PASSED 是执行授权/可重试态，不是终态。
        ProposalFinalized { proposal_id: u64, status: u8 },
        /// 中文注释：自动执行失败，提案进入 PASSED 可重试态。
        ProposalExecutionRetryScheduled {
            proposal_id: u64,
            retry_deadline: BlockNumberFor<T>,
        },
        /// 中文注释：管理员手动执行已尝试。
        ProposalExecutionRetried {
            proposal_id: u64,
            manual_attempts: u8,
            outcome: u8,
        },
        /// 中文注释：PASSED 可重试提案超过宽限期，转入执行失败终态。
        ProposalExecutionRetryExpired { proposal_id: u64 },
        /// 中文注释：管理员取消 PASSED 可重试提案，转入执行失败终态。
        ProposalExecutionCancelled { proposal_id: u64 },
        /// 中文注释：内部投票已投出一票。
        InternalVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            approve: bool,
        },
        /// 中文注释：联合投票中某机构管理员已投出一票。
        JointAdminVoteCast {
            proposal_id: u64,
            institution: InstitutionPalletId,
            who: T::AccountId,
            approve: bool,
        },
        /// 中文注释：联合投票中某机构已形成最终结果（赞成/反对）。
        JointInstitutionVoteFinalized {
            proposal_id: u64,
            institution: InstitutionPalletId,
            approved: bool,
        },
        /// 中文注释：公民投票已投出一票（binding_id 为 SFID 哈希）。
        CitizenVoteCast {
            proposal_id: u64,
            who: T::AccountId,
            binding_id: T::Hash,
            approve: bool,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// 中文注释：提案不存在或已被清理。
        ProposalNotFound,
        /// 中文注释：提案类型与当前操作不匹配（内部/联合）。
        InvalidProposalKind,
        /// 中文注释：提案所处阶段与当前操作不匹配（内部/联合/公民）。
        InvalidProposalStage,
        /// 中文注释：提案状态不允许当前操作（例如已终结的提案不可投票）。
        InvalidProposalStatus,
        /// 中文注释：内部投票的机构类型不合法。
        InvalidInternalOrg,
        /// 中文注释：机构标识不属于任何已知类型（NRC/PRC/PRB/多签）。
        InvalidInstitution,
        /// 中文注释：调用者无权执行此操作（非管理员或外部 extrinsic 直接调用）。
        NoPermission,
        /// 中文注释：投票已截止（当前区块 > end）。
        VoteClosed,
        /// 中文注释：提案尚未到期，不可手动触发超时结算。
        VoteNotExpired,
        /// 中文注释：同一身份已对该提案投过票。
        AlreadyVoted,
        /// 中文注释：SFID 资格校验未通过（binding_id 未绑定或不匹配）。
        SfidNotEligible,
        /// 中文注释：SFID 投票凭证验签失败或已被消费。
        InvalidSfidVoteCredential,
        /// 中文注释：公民投票总分母未设置（eligible_total == 0）。
        CitizenEligibleTotalNotSet,
        /// 中文注释：人口快照参数无效（nonce 为空/已使用/签名验证失败）。
        InvalidPopulationSnapshot,
        /// 中文注释：提案已终结，不可重复结算。
        ProposalAlreadyFinalized,
        /// 中文注释：提案 ID 分配溢出（年内超过 999,999 或数学溢出）。
        ProposalIdOverflow,
        /// 中文注释：单个到期区块的提案数超出上限。
        TooManyProposalsAtExpiry,
        /// 中文注释：该机构活跃提案数已达上限（10 个），需等待现有提案完成后再发起。
        ActiveProposalLimitReached,
        /// 中文注释：同一治理主体已有管理员集合变更提案活跃，普通提案需等待其结束。
        AdminSetMutationProposalActive,
        /// 中文注释：同一治理主体已有普通提案活跃，管理员更换需等待普通提案结束。
        RegularInternalProposalActive,
        /// 中文注释：内部提案互斥计数溢出。
        InternalProposalMutexOverflow,
        /// 中文注释：单个提案持有的互斥锁数量超出上限。
        TooManyInternalProposalMutexBindings,
        /// 中文注释：管理员更换提案不是当前治理主体的独占锁 owner。
        InternalProposalMutexOwnerMismatch,
        /// 中文注释：提案尚未绑定业务 owner。
        ProposalOwnerMissing,
        /// 中文注释：提案 owner 与当前业务模块不匹配。
        ProposalOwnerMismatch,
        /// 中文注释：提案业务数据已绑定，禁止跨模块覆写。
        ProposalDataAlreadyRegistered,
        /// 中文注释：提案不是可手动执行状态。
        ProposalNotRetryable,
        /// 中文注释：手动执行失败次数已达上限。
        ManualExecutionAttemptsExceeded,
        /// 中文注释：手动执行宽限期已过。
        ExecutionRetryDeadlinePassed,
        /// 中文注释：单个区块执行重试超时队列已满。
        TooManyExecutionRetryDeadlines,
        /// 中文注释：owner 模块没有明确允许取消该 PASSED 重试提案。
        ProposalCancellationNotAllowed,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();
            let max_auto_finalize = T::MaxAutoFinalizePerBlock::get() as usize;

            if max_auto_finalize > 0 {
                let db_weight = T::DbWeight::get();
                weight = weight.saturating_add(db_weight.reads(1));
                let mut budget = max_auto_finalize;
                let pending = PendingExpiryBucket::<T>::get();

                if let Some(expiry) = pending {
                    if expiry <= n {
                        let (processed, has_remaining, processed_weight) =
                            Self::auto_finalize_expiry_bucket(expiry, n, budget);
                        weight = weight.saturating_add(processed_weight);
                        budget = budget.saturating_sub(processed);
                        if has_remaining {
                            PendingExpiryBucket::<T>::put(expiry);
                            weight = weight.saturating_add(db_weight.writes(1));
                            return weight.saturating_add(Self::process_pending_cleanup_steps());
                        }
                        PendingExpiryBucket::<T>::kill();
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                }

                if budget > 0 {
                    let (_processed, has_remaining, processed_weight) =
                        Self::auto_finalize_expiry_bucket(n, n, budget);
                    weight = weight.saturating_add(processed_weight);
                    if has_remaining {
                        PendingExpiryBucket::<T>::put(n);
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                }
            }

            weight = weight.saturating_add(Self::process_execution_retry_deadlines(n));

            weight = weight.saturating_add(Self::process_pending_cleanup_steps());

            // 处理延迟清理队列：清理 90 天前完成的提案的全部数据
            weight = weight.saturating_add(proposal_cleanup::process_cleanup_queue::<T>(n));

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 内部投票(公开入口)。
        ///
        /// 所有机构管理员对内部提案的投票统一走此 call。
        /// 由 `do_internal_vote` 做完整的阶段/权限/防双投/记票/阈值校验。
        /// 达到投票判定(PASSED/REJECTED)时会通过 `set_status_and_emit` 触发
        /// `InternalVoteResultCallback` 广播给业务模块执行后续动作。
        ///
        /// 签名客户端构造 call_data 格式:
        ///   `[pallet=9][call=0][proposal_id: u64 LE][approve: bool]`
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::internal_vote())]
        pub fn internal_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_internal_vote(who, proposal_id, approve)
        }

        /// 联合投票(公开入口)。
        ///
        /// 国储会 / 省储会 / 省储行管理员按机构投票;每个机构独立计票。
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::joint_vote())]
        pub fn joint_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            institution: InstitutionPalletId,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_joint_vote(who, proposal_id, institution, approve)
        }

        /// 公民投票(公开入口)。
        ///
        /// SFID 绑定用户按 >50% 多数投票;外层可由任意签名账户代投,
        /// 内层 `signature` 必须是 `binding_id` 绑定的用户本人 sr25519 签名。
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::citizen_vote())]
        pub fn citizen_vote(
            origin: OriginFor<T>,
            proposal_id: u64,
            binding_id: T::Hash,
            nonce: VoteNonceOf<T>,
            signature: VoteSignatureOf<T>,
            approve: bool,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::do_citizen_vote(who, proposal_id, binding_id, nonce, signature, approve)
        }

        #[pallet::call_index(3)]
        #[pallet::weight(
            T::WeightInfo::finalize_proposal_internal()
                .max(T::WeightInfo::finalize_proposal_joint())
                .max(T::WeightInfo::finalize_proposal_citizen())
        )]
        pub fn finalize_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
        ) -> DispatchResultWithPostInfo {
            let _who = ensure_signed(origin)?;
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            let actual_weight = match proposal.stage {
                STAGE_INTERNAL => {
                    Self::do_finalize_internal_timeout(&proposal, proposal_id)?;
                    T::WeightInfo::finalize_proposal_internal()
                }
                STAGE_JOINT => {
                    Self::do_finalize_joint_timeout(&proposal, proposal_id)?;
                    T::WeightInfo::finalize_proposal_joint()
                }
                STAGE_CITIZEN => {
                    Self::do_finalize_citizen_timeout(&proposal, proposal_id)?;
                    T::WeightInfo::finalize_proposal_citizen()
                }
                _ => return Err(Error::<T>::InvalidProposalStage.into()),
            };

            Ok(Some(actual_weight).into())
        }

        /// 统一手动执行已通过但自动执行失败的提案。
        ///
        /// 中文注释：业务模块不得再各自暴露 execute_xxx 重试入口；所有手动执行
        /// 都必须经过投票引擎校验 PASSED 状态、管理员权限、重试次数和宽限期。
        #[pallet::call_index(4)]
        #[pallet::weight(T::DbWeight::get().reads_writes(8, 8))]
        pub fn retry_passed_proposal(origin: OriginFor<T>, proposal_id: u64) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::retry_passed_proposal_inner(&who, proposal_id)
        }

        /// 统一取消已通过但无法继续执行的提案。
        ///
        /// 中文注释：取消只允许 `PASSED -> EXECUTION_FAILED`，进入执行失败终态后
        /// 不再允许重试或再次取消。
        #[pallet::call_index(5)]
        #[pallet::weight(T::DbWeight::get().reads_writes(7, 7))]
        pub fn cancel_passed_proposal(
            origin: OriginFor<T>,
            proposal_id: u64,
            _reason: BoundedVec<u8, T::MaxProposalDataLen>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::cancel_passed_proposal_inner(&who, proposal_id)
        }
    }

    impl<T: Config> Pallet<T> {
        pub(crate) fn schedule_proposal_expiry(
            proposal_id: u64,
            end: BlockNumberFor<T>,
        ) -> DispatchResult {
            // end 表示“最后一个仍可投票区块”，因此超时结算应在 end+1 触发。
            let expiry = end.saturating_add(One::one());
            ProposalsByExpiry::<T>::try_mutate(expiry, |ids| {
                ids.try_push(proposal_id)
                    .map_err(|_| Error::<T>::TooManyProposalsAtExpiry.into())
            })
        }

        fn auto_finalize_expiry_bucket(
            expiry: BlockNumberFor<T>,
            now: BlockNumberFor<T>,
            max_count: usize,
        ) -> (usize, bool, Weight) {
            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads_writes(1, 1);
            let mut proposal_ids = ProposalsByExpiry::<T>::take(expiry);
            if proposal_ids.is_empty() {
                return (0, false, weight);
            }

            let process_count = core::cmp::min(max_count, proposal_ids.len());
            let mut retry_ids = Vec::new();
            for proposal_id in proposal_ids.drain(..process_count) {
                weight = weight.saturating_add(db_weight.reads(1));
                let Some(proposal) = Proposals::<T>::get(proposal_id) else {
                    continue;
                };
                if proposal.status != STATUS_VOTING || proposal.end >= now {
                    continue;
                }

                let finalize_result = match proposal.stage {
                    STAGE_INTERNAL => Self::do_finalize_internal_timeout(&proposal, proposal_id),
                    STAGE_JOINT => Self::do_finalize_joint_timeout(&proposal, proposal_id),
                    STAGE_CITIZEN => Self::do_finalize_citizen_timeout(&proposal, proposal_id),
                    _ => Ok(()),
                };
                if finalize_result.is_err() {
                    // 中文注释：终结失败时必须保留自动重试索引，
                    // 避免提案状态仍是 Voting，但后续再也不会被 on_initialize 处理。
                    retry_ids.push(proposal_id);
                }
            }
            for proposal_id in retry_ids {
                proposal_ids
                    .try_push(proposal_id)
                    .expect("retry ids come from the drained expiry bucket and must fit");
            }

            let has_remaining = !proposal_ids.is_empty();
            if has_remaining {
                ProposalsByExpiry::<T>::insert(expiry, proposal_ids);
                weight = weight.saturating_add(db_weight.writes(1));
            }

            let per_finalize_weight = T::WeightInfo::finalize_proposal_internal()
                .max(T::WeightInfo::finalize_proposal_joint())
                .max(T::WeightInfo::finalize_proposal_citizen());
            let finalize_weight = per_finalize_weight.saturating_mul(process_count as u64);
            weight = weight.saturating_add(finalize_weight);

            (process_count, has_remaining, weight)
        }

        /// 分配提案 ID：`年份 × 1,000,000 + 年内计数器`。
        /// 每年计数器自动重置。例如：2026000000, 2026000001, ..., 2027000000, ...
        pub(crate) fn allocate_proposal_id() -> Result<u64, DispatchError> {
            let now_ms = T::TimeProvider::now().as_millis();
            // 毫秒 → 秒 → 年份（UTC）
            let secs = u64::try_from(now_ms / 1000).map_err(|_| Error::<T>::ProposalIdOverflow)?;
            let year = Self::unix_seconds_to_year(secs)?;

            let stored_year = CurrentProposalYear::<T>::get();
            let counter = if stored_year != year {
                // 新的一年，重置计数器
                CurrentProposalYear::<T>::put(year);
                YearProposalCounter::<T>::put(1u32);
                0u32
            } else {
                let c = YearProposalCounter::<T>::get();
                ensure!(c < 999_999, Error::<T>::ProposalIdOverflow);
                YearProposalCounter::<T>::put(c + 1);
                c
            };

            let id = (year as u64)
                .checked_mul(1_000_000)
                .and_then(|base| base.checked_add(counter as u64))
                .ok_or(Error::<T>::ProposalIdOverflow)?;

            // 更新 NextProposalId（兼容外部查询）
            let next = id.checked_add(1).ok_or(Error::<T>::ProposalIdOverflow)?;
            NextProposalId::<T>::put(next);

            Ok(id)
        }

        /// Unix 秒数转 UTC 公历年份。
        pub(crate) fn unix_seconds_to_year(secs: u64) -> Result<u16, DispatchError> {
            const SECS_PER_DAY: u64 = 86_400;
            const DAYS_PER_400_YEARS: u64 = 146_097;

            let mut days = secs / SECS_PER_DAY;
            let cycles = days / DAYS_PER_400_YEARS;
            let mut year = 1970u32
                .checked_add(
                    u32::try_from(cycles)
                        .map_err(|_| Error::<T>::ProposalIdOverflow)?
                        .checked_mul(400)
                        .ok_or(Error::<T>::ProposalIdOverflow)?,
                )
                .ok_or(Error::<T>::ProposalIdOverflow)?;
            days %= DAYS_PER_400_YEARS;

            // 中文注释：提案 ID 年份段必须按真实 UTC 公历年边界切换，
            // 不能使用平均年秒数，否则元旦附近会漂移到错误年份段。
            while days >= Self::days_in_year(year) as u64 {
                days -= Self::days_in_year(year) as u64;
                year = year.checked_add(1).ok_or(Error::<T>::ProposalIdOverflow)?;
            }

            u16::try_from(year).map_err(|_| Error::<T>::ProposalIdOverflow.into())
        }

        pub(crate) fn days_in_year(year: u32) -> u16 {
            if Self::is_leap_year(year) {
                366
            } else {
                365
            }
        }

        pub(crate) fn is_leap_year(year: u32) -> bool {
            year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
        }

        pub(crate) fn ensure_open_proposal(
            proposal_id: u64,
        ) -> Result<Proposal<BlockNumberFor<T>>, DispatchError> {
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;

            ensure!(
                proposal.status == STATUS_VOTING,
                Error::<T>::InvalidProposalStatus
            );
            ensure!(
                <frame_system::Pallet<T>>::block_number() <= proposal.end,
                Error::<T>::VoteClosed
            );

            Ok(proposal)
        }

        pub(crate) fn acquire_internal_proposal_mutex(
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            kind: InternalProposalMutexKind,
        ) -> DispatchResult {
            InternalProposalMutexes::<T>::try_mutate_exists(
                org,
                institution,
                |maybe| -> DispatchResult {
                    let state = maybe.get_or_insert_with(InternalProposalMutexState::default);
                    match kind {
                        InternalProposalMutexKind::Regular => {
                            ensure!(
                                state.admin_set_mutation_proposal.is_none(),
                                Error::<T>::AdminSetMutationProposalActive
                            );
                            state.regular_active_count = state
                                .regular_active_count
                                .checked_add(1)
                                .ok_or(Error::<T>::InternalProposalMutexOverflow)?;
                        }
                        InternalProposalMutexKind::AdminSetMutationExclusive => {
                            ensure!(
                                state.admin_set_mutation_proposal.is_none(),
                                Error::<T>::AdminSetMutationProposalActive
                            );
                            ensure!(
                                state.regular_active_count == 0,
                                Error::<T>::RegularInternalProposalActive
                            );
                            state.admin_set_mutation_proposal = Some(proposal_id);
                        }
                    }
                    Ok(())
                },
            )?;

            ProposalMutexBindings::<T>::try_mutate(proposal_id, |bindings| {
                bindings
                    .try_push(InternalProposalMutexBinding {
                        org,
                        institution,
                        kind,
                    })
                    .map_err(|_| Error::<T>::TooManyInternalProposalMutexBindings)?;
                Ok(())
            })
        }

        pub(crate) fn release_internal_proposal_mutexes(proposal_id: u64) {
            let bindings = ProposalMutexBindings::<T>::take(proposal_id);
            for binding in bindings {
                InternalProposalMutexes::<T>::mutate_exists(
                    binding.org,
                    binding.institution,
                    |maybe| {
                        let Some(state) = maybe.as_mut() else {
                            return;
                        };
                        match binding.kind {
                            InternalProposalMutexKind::Regular => {
                                state.regular_active_count =
                                    state.regular_active_count.saturating_sub(1);
                            }
                            InternalProposalMutexKind::AdminSetMutationExclusive => {
                                if state.admin_set_mutation_proposal == Some(proposal_id) {
                                    state.admin_set_mutation_proposal = None;
                                }
                            }
                        }
                        if state.is_empty() {
                            *maybe = None;
                        }
                    },
                );
            }
        }

        pub fn ensure_admin_set_mutation_lock_owner(
            org: u8,
            institution: InstitutionPalletId,
            proposal_id: u64,
        ) -> DispatchResult {
            let state = InternalProposalMutexes::<T>::get(org, institution)
                .ok_or(Error::<T>::InternalProposalMutexOwnerMismatch)?;
            ensure!(
                state.admin_set_mutation_proposal == Some(proposal_id),
                Error::<T>::InternalProposalMutexOwnerMismatch
            );
            Ok(())
        }

        fn should_release_internal_proposal_mutexes(kind: u8, stage: u8, final_status: u8) -> bool {
            matches!(
                final_status,
                STATUS_REJECTED | STATUS_EXECUTED | STATUS_EXECUTION_FAILED
            ) || (kind == PROPOSAL_KIND_JOINT
                && stage == STAGE_JOINT
                && final_status == STATUS_PASSED)
        }

        fn ensure_valid_status_transition(old_status: u8, new_status: u8) -> DispatchResult {
            ensure!(
                matches!(
                    (old_status, new_status),
                    (STATUS_VOTING, STATUS_PASSED)
                        | (STATUS_VOTING, STATUS_REJECTED)
                        | (STATUS_PASSED, STATUS_EXECUTED)
                        | (STATUS_PASSED, STATUS_EXECUTION_FAILED)
                ),
                Error::<T>::InvalidProposalStatus
            );
            Ok(())
        }

        fn is_terminal_status(status: u8) -> bool {
            matches!(
                status,
                STATUS_REJECTED | STATUS_EXECUTED | STATUS_EXECUTION_FAILED
            )
        }

        fn mark_proposal_passed_at(proposal_id: u64, block: BlockNumberFor<T>) {
            ProposalMeta::<T>::mutate(proposal_id, |meta| {
                if let Some(m) = meta {
                    if m.passed_at.is_none() {
                        m.passed_at = Some(block);
                    }
                }
            });
        }

        fn set_proposal_status(proposal_id: u64, status: u8) -> DispatchResult {
            Proposals::<T>::try_mutate(proposal_id, |maybe| {
                let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                Self::ensure_valid_status_transition(proposal.status, status)?;
                proposal.status = status;
                Ok(())
            })
        }

        fn apply_terminal_side_effects(proposal_id: u64, status: u8) -> DispatchResult {
            ensure!(
                Self::is_terminal_status(status),
                Error::<T>::InvalidProposalStatus
            );
            ProposalExecutionRetryStates::<T>::remove(proposal_id);
            if status == STATUS_EXECUTION_FAILED {
                if let Some(proposal) = Proposals::<T>::get(proposal_id) {
                    // 中文注释：执行失败终态需要业务模块清理 pending 锁或独立动作存储；
                    // 清理失败不阻断投票引擎终态收口，避免提案卡在半终态。
                    let _ = Self::notify_execution_failed_terminal(proposal_id, proposal.kind);
                }
            }
            let now = frame_system::Pallet::<T>::block_number();
            proposal_cleanup::schedule_cleanup::<T>(proposal_id, now)?;
            if let Some(proposal) = Proposals::<T>::get(proposal_id) {
                if Self::should_release_internal_proposal_mutexes(
                    proposal.kind,
                    proposal.stage,
                    status,
                ) {
                    Self::release_internal_proposal_mutexes(proposal_id);
                }
            }
            Ok(())
        }

        fn finish_terminal_status(proposal_id: u64, status: u8) -> DispatchResult {
            Self::apply_terminal_side_effects(proposal_id, status)?;
            Self::deposit_event(Event::<T>::ProposalFinalized {
                proposal_id,
                status,
            });
            Ok(())
        }

        fn ensure_retry_admin(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            let institution = proposal
                .internal_institution
                .ok_or(Error::<T>::InvalidInstitution)?;
            ensure!(
                Self::is_admin_in_snapshot(proposal_id, institution, who),
                Error::<T>::NoPermission
            );
            Ok(())
        }

        fn invoke_execution_callback(
            proposal_id: u64,
            kind: u8,
            approved: bool,
        ) -> Result<ProposalExecutionOutcome, DispatchError> {
            match kind {
                PROPOSAL_KIND_INTERNAL => {
                    T::InternalVoteResultCallback::on_internal_vote_finalized(proposal_id, approved)
                }
                PROPOSAL_KIND_JOINT => {
                    T::JointVoteResultCallback::on_joint_vote_finalized(proposal_id, approved)
                }
                _ => Err(Error::<T>::InvalidProposalKind.into()),
            }
        }

        fn can_cancel_passed_proposal_by_owner(proposal_id: u64, kind: u8) -> DispatchResult {
            let decision = match kind {
                PROPOSAL_KIND_INTERNAL => {
                    T::InternalVoteResultCallback::can_cancel_passed_proposal(proposal_id)
                }
                PROPOSAL_KIND_JOINT => {
                    T::JointVoteResultCallback::can_cancel_passed_proposal(proposal_id)
                }
                _ => Err(Error::<T>::InvalidProposalKind.into()),
            }?;
            ensure!(
                decision == ProposalCancelDecision::Allow,
                Error::<T>::ProposalCancellationNotAllowed
            );
            Ok(())
        }

        fn notify_execution_failed_terminal(proposal_id: u64, kind: u8) -> DispatchResult {
            match kind {
                PROPOSAL_KIND_INTERNAL => {
                    T::InternalVoteResultCallback::on_execution_failed_terminal(proposal_id)
                }
                PROPOSAL_KIND_JOINT => {
                    T::JointVoteResultCallback::on_execution_failed_terminal(proposal_id)
                }
                _ => Err(Error::<T>::InvalidProposalKind.into()),
            }
        }

        fn schedule_execution_retry(proposal_id: u64) -> DispatchResult {
            if ProposalExecutionRetryStates::<T>::contains_key(proposal_id) {
                return Ok(());
            }
            let now = frame_system::Pallet::<T>::block_number();
            let retry_deadline = now.saturating_add(T::ExecutionRetryGraceBlocks::get());
            let state = ExecutionRetryState {
                manual_attempts: 0,
                first_auto_failed_at: now,
                retry_deadline,
                last_attempt_at: None,
            };
            ExecutionRetryDeadlines::<T>::try_mutate(retry_deadline, |ids| {
                ids.try_push(proposal_id)
                    .map_err(|_| Error::<T>::TooManyExecutionRetryDeadlines)
            })?;
            ProposalExecutionRetryStates::<T>::insert(proposal_id, state);
            Self::deposit_event(Event::<T>::ProposalExecutionRetryScheduled {
                proposal_id,
                retry_deadline,
            });
            Ok(())
        }

        fn apply_automatic_execution_outcome(
            proposal_id: u64,
            kind: u8,
            outcome: ProposalExecutionOutcome,
        ) -> DispatchResult {
            match outcome {
                ProposalExecutionOutcome::Ignored => Err(Error::<T>::ProposalOwnerMissing.into()),
                ProposalExecutionOutcome::Executed => {
                    Self::set_proposal_status(proposal_id, STATUS_EXECUTED)
                }
                ProposalExecutionOutcome::RetryableFailed => {
                    if kind == PROPOSAL_KIND_INTERNAL {
                        Self::schedule_execution_retry(proposal_id)
                    } else {
                        // 中文注释：当前统一 retry/cancel 管理员权限只支持内部提案；
                        // joint callback 若误返回 RetryableFailed，立即失败终态，避免 PASSED 卡死。
                        Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)
                    }
                }
                ProposalExecutionOutcome::FatalFailed => {
                    Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)
                }
            }
        }

        fn process_execution_retry_deadlines(now: BlockNumberFor<T>) -> Weight {
            let db_weight = T::DbWeight::get();
            let mut weight = db_weight.reads_writes(1, 1);
            let queue = ExecutionRetryDeadlines::<T>::take(now);
            if queue.is_empty() {
                return weight;
            }

            for proposal_id in queue.into_iter() {
                weight = weight.saturating_add(db_weight.reads_writes(2, 3));
                let Some(state) = ProposalExecutionRetryStates::<T>::get(proposal_id) else {
                    continue;
                };
                if state.retry_deadline > now {
                    continue;
                }
                let Some(proposal) = Proposals::<T>::get(proposal_id) else {
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    continue;
                };
                if proposal.status != STATUS_PASSED {
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    continue;
                }
                if Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED).is_ok() {
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    Self::deposit_event(Event::<T>::ProposalExecutionRetryExpired { proposal_id });
                    let _ = Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED);
                }
            }
            weight
        }

        fn retry_passed_proposal_inner(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            Self::ensure_retry_admin(who, proposal_id)?;
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotRetryable
            );
            let mut state = ProposalExecutionRetryStates::<T>::get(proposal_id)
                .ok_or(Error::<T>::ProposalNotRetryable)?;
            let now = frame_system::Pallet::<T>::block_number();
            ensure!(
                now <= state.retry_deadline,
                Error::<T>::ExecutionRetryDeadlinePassed
            );
            ensure!(
                u32::from(state.manual_attempts) < T::MaxManualExecutionAttempts::get(),
                Error::<T>::ManualExecutionAttemptsExceeded
            );

            let outcome = Self::invoke_execution_callback(proposal_id, proposal.kind, true)?;
            match outcome {
                ProposalExecutionOutcome::Executed => {
                    Self::set_proposal_status(proposal_id, STATUS_EXECUTED)?;
                    Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                        proposal_id,
                        manual_attempts: state.manual_attempts,
                        outcome: STATUS_EXECUTED,
                    });
                    Self::finish_terminal_status(proposal_id, STATUS_EXECUTED)
                }
                ProposalExecutionOutcome::RetryableFailed => {
                    state.manual_attempts = state.manual_attempts.saturating_add(1);
                    state.last_attempt_at = Some(now);
                    if u32::from(state.manual_attempts) >= T::MaxManualExecutionAttempts::get() {
                        Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
                        Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                            proposal_id,
                            manual_attempts: state.manual_attempts,
                            outcome: STATUS_EXECUTION_FAILED,
                        });
                        Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
                    } else {
                        Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                            proposal_id,
                            manual_attempts: state.manual_attempts,
                            outcome: STATUS_PASSED,
                        });
                        ProposalExecutionRetryStates::<T>::insert(proposal_id, state);
                        Ok(())
                    }
                }
                ProposalExecutionOutcome::FatalFailed => {
                    Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
                    Self::deposit_event(Event::<T>::ProposalExecutionRetried {
                        proposal_id,
                        manual_attempts: state.manual_attempts,
                        outcome: STATUS_EXECUTION_FAILED,
                    });
                    Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
                }
                ProposalExecutionOutcome::Ignored => Err(Error::<T>::ProposalOwnerMissing.into()),
            }
        }

        fn cancel_passed_proposal_inner(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            Self::ensure_retry_admin(who, proposal_id)?;
            let proposal = Proposals::<T>::get(proposal_id).ok_or(Error::<T>::ProposalNotFound)?;
            ensure!(
                proposal.status == STATUS_PASSED,
                Error::<T>::ProposalNotRetryable
            );
            Self::can_cancel_passed_proposal_by_owner(proposal_id, proposal.kind)?;
            Self::set_proposal_status(proposal_id, STATUS_EXECUTION_FAILED)?;
            Self::deposit_event(Event::<T>::ProposalExecutionCancelled { proposal_id });
            Self::finish_terminal_status(proposal_id, STATUS_EXECUTION_FAILED)
        }

        /// 兼容层：旧业务 execute_xxx extrinsic 必须委托到统一重试状态机。
        pub fn retry_passed_proposal_for(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            Self::retry_passed_proposal_inner(who, proposal_id)
        }

        /// 兼容层：旧业务 cancel_xxx extrinsic 必须委托到统一取消状态机。
        pub fn cancel_passed_proposal_for(who: &T::AccountId, proposal_id: u64) -> DispatchResult {
            Self::cancel_passed_proposal_inner(who, proposal_id)
        }

        fn with_callback_execution_scope<F, R>(
            proposal_id: u64,
            callback: F,
        ) -> Result<R, DispatchError>
        where
            F: FnOnce() -> Result<R, DispatchError>,
        {
            CallbackExecutionScopes::<T>::insert(proposal_id, ());
            let result = callback();
            CallbackExecutionScopes::<T>::remove(proposal_id);
            result
        }

        /// 更新提案状态，并按统一 executor 结果推进业务执行状态。
        pub(crate) fn set_status_and_emit(proposal_id: u64, status: u8) -> DispatchResult {
            with_transaction(|| {
                let (kind, stage, institution, should_run_callback) = match Proposals::<
                    T,
                >::try_mutate(
                    proposal_id,
                    |maybe| -> Result<(u8, u8, Option<InstitutionPalletId>, bool), DispatchError> {
                        let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                        let old_status = proposal.status;
                        Self::ensure_valid_status_transition(old_status, status)?;
                        let kind = proposal.kind;
                        let stage = proposal.stage;
                        let inst = proposal.internal_institution;
                        proposal.status = status;
                        if old_status == STATUS_VOTING && status == STATUS_PASSED {
                            let now = frame_system::Pallet::<T>::block_number();
                            Self::mark_proposal_passed_at(proposal_id, now);
                        }
                        Ok((
                            kind,
                            stage,
                            inst,
                            old_status == STATUS_VOTING
                                && matches!(status, STATUS_PASSED | STATUS_REJECTED),
                        ))
                    },
                ) {
                    Ok(v) => v,
                    Err(err) => return TransactionOutcome::Rollback(Err(err)),
                };

                // 提案结束（通过或拒绝），立即释放活跃提案名额
                if status != STATUS_VOTING {
                    if let Some(inst) = institution {
                        active_proposal_limit::remove_active_proposal::<T>(inst, proposal_id);
                    }
                }

                if should_run_callback {
                    let outcome = match Self::with_callback_execution_scope(proposal_id, || {
                        Self::invoke_execution_callback(proposal_id, kind, status == STATUS_PASSED)
                    }) {
                        Ok(outcome) => outcome,
                        Err(err) => return TransactionOutcome::Rollback(Err(err)),
                    };
                    if status == STATUS_PASSED {
                        if let Err(err) =
                            Self::apply_automatic_execution_outcome(proposal_id, kind, outcome)
                        {
                            return TransactionOutcome::Rollback(Err(err));
                        }
                    }
                }

                let final_status = match Proposals::<T>::get(proposal_id) {
                    Some(proposal) => proposal.status,
                    None => {
                        return TransactionOutcome::Rollback(Err(
                            Error::<T>::ProposalNotFound.into()
                        ))
                    }
                };
                Self::deposit_event(Event::<T>::ProposalFinalized {
                    proposal_id,
                    status: final_status,
                });

                // 中文注释：PASSED 是执行授权/可重试态，不再视为终态。
                // 90 天延迟清理只登记 REJECTED / EXECUTED / EXECUTION_FAILED。
                if Self::is_terminal_status(final_status) {
                    if let Err(err) = Self::apply_terminal_side_effects(proposal_id, final_status) {
                        return TransactionOutcome::Rollback(Err(err));
                    }
                } else if Self::should_release_internal_proposal_mutexes(kind, stage, final_status)
                {
                    Self::release_internal_proposal_mutexes(proposal_id);
                }

                TransactionOutcome::Commit(Ok(()))
            })
        }

        /// 回调专用执行结果写入。
        ///
        /// 中文注释：仅供单测验证旧回调作用域保护；生产业务回调应直接返回
        /// `ProposalExecutionOutcome`，由外层 `set_status_and_emit` 统一收口状态、事件和清理。
        #[cfg(test)]
        pub(crate) fn set_callback_execution_result(
            proposal_id: u64,
            final_status: u8,
        ) -> DispatchResult {
            ensure!(
                CallbackExecutionScopes::<T>::contains_key(proposal_id),
                Error::<T>::InvalidProposalStatus
            );
            ensure!(
                matches!(final_status, STATUS_EXECUTED | STATUS_EXECUTION_FAILED),
                Error::<T>::InvalidProposalStatus
            );
            Proposals::<T>::try_mutate(proposal_id, |maybe| {
                let proposal = maybe.as_mut().ok_or(Error::<T>::ProposalNotFound)?;
                Self::ensure_valid_status_transition(proposal.status, final_status)?;
                proposal.status = final_status;
                Ok(())
            })
        }

        fn process_pending_cleanup_steps() -> Weight {
            let max_steps = T::MaxCleanupStepsPerBlock::get() as usize;
            if max_steps == 0 {
                return Weight::zero();
            }

            let cleanup_limit = T::CleanupKeysPerStep::get().max(1);
            let db_weight = T::DbWeight::get();
            let mut weight = Weight::zero();
            // 每步的最大 weight 上界：cleanup_limit 次读 + cleanup_limit 次写 + 固定开销
            let max_weight_per_step =
                db_weight.reads_writes(u64::from(cleanup_limit) + 2, u64::from(cleanup_limit) + 2);

            for _ in 0..max_steps {
                let Some((proposal_id, stage)) = PendingProposalCleanups::<T>::iter().next() else {
                    break;
                };
                weight = weight.saturating_add(db_weight.reads(1));

                let (next_stage, _actual_weight) =
                    Self::process_pending_cleanup_step(proposal_id, stage, cleanup_limit);
                // 使用预估最大值而非实际值，确保 on_initialize 不超出声明的 weight
                weight = weight.saturating_add(max_weight_per_step);

                match next_stage {
                    Some(next) if next != stage => {
                        PendingProposalCleanups::<T>::insert(proposal_id, next);
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                    Some(_) => {}
                    None => {
                        PendingProposalCleanups::<T>::remove(proposal_id);
                        weight = weight.saturating_add(db_weight.writes(1));
                    }
                }
            }

            weight
        }

        fn process_pending_cleanup_step(
            proposal_id: u64,
            stage: PendingCleanupStage,
            cleanup_limit: u32,
        ) -> (Option<PendingCleanupStage>, Weight) {
            let db_weight = T::DbWeight::get();

            match stage {
                PendingCleanupStage::AdminSnapshots => {
                    let result = AdminSnapshot::<T>::clear_prefix(proposal_id, cleanup_limit, None);
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::AdminSnapshots)
                    } else {
                        Some(PendingCleanupStage::InternalVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::InternalVotes => {
                    let result =
                        InternalVotesByAccount::<T>::clear_prefix(proposal_id, cleanup_limit, None);
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::InternalVotes)
                    } else {
                        Some(PendingCleanupStage::JointAdminVotes) // 继续下一阶段
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointAdminVotes => {
                    let result =
                        JointVotesByAdmin::<T>::clear_prefix(proposal_id, cleanup_limit, None);
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::JointAdminVotes)
                    } else {
                        Some(PendingCleanupStage::JointInstitutionVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointInstitutionVotes => {
                    let result = JointVotesByInstitution::<T>::clear_prefix(
                        proposal_id,
                        cleanup_limit,
                        None,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::JointInstitutionVotes)
                    } else {
                        Some(PendingCleanupStage::JointInstitutionTallies)
                    };
                    (next, weight)
                }
                PendingCleanupStage::JointInstitutionTallies => {
                    let result = JointInstitutionTallies::<T>::clear_prefix(
                        proposal_id,
                        cleanup_limit,
                        None,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::JointInstitutionTallies)
                    } else {
                        Some(PendingCleanupStage::CitizenVotes)
                    };
                    (next, weight)
                }
                PendingCleanupStage::CitizenVotes => {
                    let result = CitizenVotesByBindingId::<T>::clear_prefix(
                        proposal_id,
                        cleanup_limit,
                        None,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.unique));
                    let next = if result.maybe_cursor.is_some() {
                        Some(PendingCleanupStage::CitizenVotes)
                    } else {
                        Some(PendingCleanupStage::VoteCredentials)
                    };
                    (next, weight)
                }
                PendingCleanupStage::VoteCredentials => {
                    let result = T::SfidEligibility::cleanup_vote_credentials_chunk(
                        proposal_id,
                        cleanup_limit,
                    );
                    let weight =
                        db_weight.reads_writes(u64::from(result.loops), u64::from(result.removed));
                    let next = if result.has_remaining {
                        Some(PendingCleanupStage::VoteCredentials)
                    } else {
                        Some(PendingCleanupStage::ProposalObject)
                    };
                    (next, weight)
                }
                PendingCleanupStage::ProposalObject => {
                    ProposalObject::<T>::remove(proposal_id);
                    ProposalObjectMeta::<T>::remove(proposal_id);
                    let weight = db_weight.writes(2);
                    (Some(PendingCleanupStage::FinalCleanup), weight)
                }
                PendingCleanupStage::FinalCleanup => {
                    // 清理核心数据 + 业务数据（单次完成）
                    Self::release_internal_proposal_mutexes(proposal_id);
                    Proposals::<T>::remove(proposal_id);
                    InternalTallies::<T>::remove(proposal_id);
                    InternalThresholdSnapshot::<T>::remove(proposal_id);
                    JointTallies::<T>::remove(proposal_id);
                    CitizenTallies::<T>::remove(proposal_id);
                    ProposalData::<T>::remove(proposal_id);
                    ProposalOwner::<T>::remove(proposal_id);
                    ProposalMeta::<T>::remove(proposal_id);
                    ProposalExecutionRetryStates::<T>::remove(proposal_id);
                    let weight = db_weight.writes(11);
                    (None, weight) // 全部完成
                }
            }
        }
    }

    // ──── 管理员快照查询 ────

    impl<T: Config> Pallet<T> {
        /// 查询快照中某管理员是否在指定机构的管理员名单中。
        pub fn is_admin_in_snapshot(
            proposal_id: u64,
            institution: InstitutionPalletId,
            who: &T::AccountId,
        ) -> bool {
            AdminSnapshot::<T>::get(proposal_id, institution)
                .map(|admins| admins.iter().any(|a| a == who))
                .unwrap_or(false)
        }

        /// 查询快照中某机构的管理员数量。
        pub fn snapshot_admin_count(
            proposal_id: u64,
            institution: InstitutionPalletId,
        ) -> Option<u32> {
            AdminSnapshot::<T>::get(proposal_id, institution).map(|admins| admins.len() as u32)
        }

        /// 将当前管理员列表写入快照存储。
        /// 如果管理员数量超过 MaxAdminsPerInstitution，触发 defensive 告警。
        pub(crate) fn snapshot_institution_admins(
            proposal_id: u64,
            org: u8,
            institution: InstitutionPalletId,
            pending_subject: bool,
        ) -> DispatchResult {
            let admins = if pending_subject {
                T::InternalAdminProvider::get_pending_admin_list(org, institution)
            } else {
                T::InternalAdminProvider::get_admin_list(org, institution)
            }
            .ok_or(Error::<T>::InvalidInstitution)?;

            match BoundedVec::<T::AccountId, T::MaxAdminsPerInstitution>::try_from(admins) {
                Ok(bounded) => {
                    AdminSnapshot::<T>::insert(proposal_id, institution, bounded);
                    Ok(())
                }
                Err(_) => {
                    frame_support::defensive!(
                        "snapshot_institution_admins: admin list exceeds MaxAdminsPerInstitution, snapshot not written"
                    );
                    Err(Error::<T>::InvalidInstitution.into())
                }
            }
        }
    }

    // ──── 统一提案数据存储接口 ────

    impl<T: Config> Pallet<T> {
        fn bounded_module_tag(
            module_tag: &[u8],
        ) -> Result<BoundedVec<u8, T::MaxModuleTagLen>, DispatchError> {
            module_tag
                .to_vec()
                .try_into()
                .map_err(|_| DispatchError::Other("ModuleTagTooLarge"))
        }

        /// 创建提案后原子绑定业务 owner、业务数据和创建区块。
        ///
        /// 中文注释：业务模块只能通过 `create_*_with_data` 系列入口在创建阶段写入一次。
        /// 后续生产路径不得再让 caller 自报 `module_tag` 更新 ProposalData。
        pub(crate) fn register_proposal_data(
            proposal_id: u64,
            module_tag: &[u8],
            data: sp_std::vec::Vec<u8>,
            created_at: BlockNumberFor<T>,
        ) -> DispatchResult {
            ensure!(
                Proposals::<T>::contains_key(proposal_id),
                Error::<T>::ProposalNotFound
            );
            ensure!(
                !ProposalOwner::<T>::contains_key(proposal_id)
                    && !ProposalData::<T>::contains_key(proposal_id),
                Error::<T>::ProposalDataAlreadyRegistered
            );
            let owner = Self::bounded_module_tag(module_tag)?;
            let bounded: BoundedVec<u8, T::MaxProposalDataLen> = data
                .try_into()
                .map_err(|_| DispatchError::Other("ProposalDataTooLarge"))?;
            ProposalOwner::<T>::insert(proposal_id, owner);
            ProposalData::<T>::insert(proposal_id, bounded);
            ProposalMeta::<T>::insert(
                proposal_id,
                ProposalMetadata {
                    created_at,
                    passed_at: None,
                },
            );
            Ok(())
        }

        /// 存储提案业务数据（仅保留给 voting-engine crate 内部测试/迁移使用）。
        #[cfg(test)]
        pub(crate) fn store_proposal_data(
            proposal_id: u64,
            data: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let bounded: BoundedVec<u8, T::MaxProposalDataLen> = data
                .try_into()
                .map_err(|_| DispatchError::Other("ProposalDataTooLarge"))?;
            ProposalData::<T>::insert(proposal_id, bounded);
            Ok(())
        }

        /// 读取提案业务数据。
        pub fn get_proposal_data(proposal_id: u64) -> Option<sp_std::vec::Vec<u8>> {
            ProposalData::<T>::get(proposal_id).map(|v| v.into_inner())
        }

        /// 存储提案大对象（例如 runtime wasm）。
        pub(crate) fn store_proposal_object(
            proposal_id: u64,
            kind: u8,
            data: sp_std::vec::Vec<u8>,
        ) -> DispatchResult {
            let object_len = u32::try_from(data.len())
                .map_err(|_| DispatchError::Other("ProposalObjectTooLarge"))?;
            let object_hash = T::Hashing::hash(&data);
            let bounded: BoundedVec<u8, T::MaxProposalObjectLen> = data
                .try_into()
                .map_err(|_| DispatchError::Other("ProposalObjectTooLarge"))?;
            ProposalObject::<T>::insert(proposal_id, bounded);
            ProposalObjectMeta::<T>::insert(
                proposal_id,
                ProposalObjectMetadata {
                    kind,
                    object_len,
                    object_hash,
                },
            );
            Ok(())
        }

        /// 读取提案大对象原始数据。
        pub fn get_proposal_object(proposal_id: u64) -> Option<sp_std::vec::Vec<u8>> {
            ProposalObject::<T>::get(proposal_id).map(|v| v.into_inner())
        }

        /// 读取提案对象层元数据。
        pub fn get_proposal_object_meta(
            proposal_id: u64,
        ) -> Option<ProposalObjectMetadata<T::Hash>> {
            ProposalObjectMeta::<T>::get(proposal_id)
        }

        /// 删除提案对象层数据与元数据。
        #[cfg(test)]
        pub(crate) fn remove_proposal_object(proposal_id: u64) {
            ProposalObject::<T>::remove(proposal_id);
            ProposalObjectMeta::<T>::remove(proposal_id);
        }

        /// 存储提案辅助元数据（创建时间）。
        #[cfg(test)]
        pub(crate) fn store_proposal_meta(proposal_id: u64, created_at: BlockNumberFor<T>) {
            ProposalMeta::<T>::insert(
                proposal_id,
                ProposalMetadata {
                    created_at,
                    passed_at: None,
                },
            );
        }

        /// 标记提案通过时间。
        #[cfg(test)]
        pub(crate) fn set_proposal_passed(proposal_id: u64, block: BlockNumberFor<T>) {
            Self::mark_proposal_passed_at(proposal_id, block);
        }

        /// 读取提案辅助元数据。
        pub fn get_proposal_meta(proposal_id: u64) -> Option<ProposalMetadata<BlockNumberFor<T>>> {
            ProposalMeta::<T>::get(proposal_id)
        }
    }
}

impl<T: pallet::Config> JointVoteEngine<T::AccountId> for pallet::Pallet<T> {
    fn create_joint_proposal(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
    ) -> Result<u64, DispatchError> {
        let snapshot_nonce: pallet::VoteNonceOf<T> = snapshot_nonce
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        let signature: pallet::VoteSignatureOf<T> = signature
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        pallet::Pallet::<T>::do_create_joint_proposal(
            who,
            eligible_total,
            snapshot_nonce,
            signature,
        )
    }

    fn create_joint_proposal_with_data(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let snapshot_nonce: pallet::VoteNonceOf<T> = snapshot_nonce
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        let signature: pallet::VoteSignatureOf<T> = signature
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        frame_support::storage::with_transaction(|| {
            let proposal_id = match pallet::Pallet::<T>::do_create_joint_proposal(
                who,
                eligible_total,
                snapshot_nonce,
                signature,
            ) {
                Ok(proposal_id) => proposal_id,
                Err(err) => return frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            };
            let now = frame_system::Pallet::<T>::block_number();
            match pallet::Pallet::<T>::register_proposal_data(proposal_id, module_tag, data, now) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_joint_proposal_with_data_and_object(
        who: T::AccountId,
        eligible_total: u64,
        snapshot_nonce: &[u8],
        signature: &[u8],
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
        object_kind: u8,
        object_data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        let snapshot_nonce: pallet::VoteNonceOf<T> = snapshot_nonce
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        let signature: pallet::VoteSignatureOf<T> = signature
            .to_vec()
            .try_into()
            .map_err(|_| pallet::Error::<T>::InvalidPopulationSnapshot)?;
        frame_support::storage::with_transaction(|| {
            let proposal_id = match pallet::Pallet::<T>::do_create_joint_proposal(
                who,
                eligible_total,
                snapshot_nonce,
                signature,
            ) {
                Ok(proposal_id) => proposal_id,
                Err(err) => return frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            };
            let now = frame_system::Pallet::<T>::block_number();
            if let Err(err) =
                pallet::Pallet::<T>::register_proposal_data(proposal_id, module_tag, data, now)
            {
                return frame_support::storage::TransactionOutcome::Rollback(Err(err));
            }
            match pallet::Pallet::<T>::store_proposal_object(proposal_id, object_kind, object_data)
            {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn cleanup_joint_proposal(_proposal_id: u64) {
        // 已弃用：清理现在由 set_status_and_emit 在终态转换时自动注册。
        // 保留空实现以兼容 trait 定义。
    }
}

impl<T: pallet::Config> InternalVoteEngine<T::AccountId> for pallet::Pallet<T> {
    fn create_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        pallet::Pallet::<T>::do_create_internal_proposal(who, org, institution)
    }

    fn create_internal_proposal_with_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        frame_support::storage::with_transaction(|| {
            let proposal_id =
                match pallet::Pallet::<T>::do_create_internal_proposal(who, org, institution) {
                    Ok(proposal_id) => proposal_id,
                    Err(err) => {
                        return frame_support::storage::TransactionOutcome::Rollback(Err(err))
                    }
                };
            let now = frame_system::Pallet::<T>::block_number();
            match pallet::Pallet::<T>::register_proposal_data(proposal_id, module_tag, data, now) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_pending_subject_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        pallet::Pallet::<T>::do_create_pending_subject_internal_proposal(who, org, institution)
    }

    fn create_pending_subject_internal_proposal_with_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        frame_support::storage::with_transaction(|| {
            let proposal_id = match pallet::Pallet::<T>::do_create_pending_subject_internal_proposal(
                who,
                org,
                institution,
            ) {
                Ok(proposal_id) => proposal_id,
                Err(err) => return frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            };
            let now = frame_system::Pallet::<T>::block_number();
            match pallet::Pallet::<T>::register_proposal_data(proposal_id, module_tag, data, now) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn create_admin_set_mutation_internal_proposal(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
    ) -> Result<u64, DispatchError> {
        pallet::Pallet::<T>::do_create_admin_set_mutation_internal_proposal(who, org, institution)
    }

    fn create_admin_set_mutation_internal_proposal_with_data(
        who: T::AccountId,
        org: u8,
        institution: InstitutionPalletId,
        module_tag: &[u8],
        data: sp_std::vec::Vec<u8>,
    ) -> Result<u64, DispatchError> {
        frame_support::storage::with_transaction(|| {
            let proposal_id =
                match pallet::Pallet::<T>::do_create_admin_set_mutation_internal_proposal(
                    who,
                    org,
                    institution,
                ) {
                    Ok(proposal_id) => proposal_id,
                    Err(err) => {
                        return frame_support::storage::TransactionOutcome::Rollback(Err(err))
                    }
                };
            let now = frame_system::Pallet::<T>::block_number();
            match pallet::Pallet::<T>::register_proposal_data(proposal_id, module_tag, data, now) {
                Ok(()) => frame_support::storage::TransactionOutcome::Commit(Ok(proposal_id)),
                Err(err) => frame_support::storage::TransactionOutcome::Rollback(Err(err)),
            }
        })
    }

    fn cleanup_internal_proposal(_proposal_id: u64) {
        // 清理现在由 set_status_and_emit 在终态转换时自动注册。
        // 保留空实现以兼容 trait 定义。
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::RefCell;
    use std::collections::BTreeSet;

    use frame_support::{assert_noop, assert_ok, derive_impl, traits::ConstU32, traits::Hooks};
    use frame_system as system;
    use primitives::china::china_cb::{
        shenfen_id_to_fixed48 as reserve_pallet_id_to_bytes, CHINA_CB,
    };
    use primitives::china::china_ch::{
        shenfen_id_to_fixed48 as shengbank_pallet_id_to_bytes, CHINA_CH,
    };
    use sp_runtime::{
        traits::Hash, traits::IdentityLookup, AccountId32, BuildStorage, DispatchError,
    };

    type Block = frame_system::mocking::MockBlock<Test>;

    #[frame_support::runtime]
    mod runtime {
        #[runtime::runtime]
        #[runtime::derive(
            RuntimeCall,
            RuntimeEvent,
            RuntimeError,
            RuntimeOrigin,
            RuntimeFreezeReason,
            RuntimeHoldReason,
            RuntimeSlashReason,
            RuntimeLockId,
            RuntimeTask,
            RuntimeViewFunction
        )]
        pub struct Test;

        #[runtime::pallet_index(0)]
        pub type System = frame_system;

        #[runtime::pallet_index(1)]
        pub type VotingEngine = super;
    }

    #[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
    impl system::Config for Test {
        type Block = Block;
        type AccountId = AccountId32;
        type Lookup = IdentityLookup<Self::AccountId>;
    }

    impl Config for Test {
        type RuntimeEvent = RuntimeEvent;
        type MaxVoteNonceLength = ConstU32<64>;
        type MaxVoteSignatureLength = ConstU32<64>;
        type MaxAutoFinalizePerBlock = ConstU32<64>;
        type MaxProposalsPerExpiry = ConstU32<128>;
        type MaxCleanupStepsPerBlock = ConstU32<3>;
        type CleanupKeysPerStep = ConstU32<2>;
        type MaxProposalDataLen = ConstU32<4096>;
        type MaxProposalObjectLen = ConstU32<10_240>;
        type MaxModuleTagLen = ConstU32<32>;
        type MaxManualExecutionAttempts = ConstU32<3>;
        type ExecutionRetryGraceBlocks = frame_support::traits::ConstU64<216>;
        type MaxExecutionRetryDeadlinesPerBlock = ConstU32<128>;
        type SfidEligibility = TestSfidEligibility;
        type PopulationSnapshotVerifier = TestPopulationSnapshotVerifier;
        type JointVoteResultCallback = TestJointVoteResultCallback;
        type InternalVoteResultCallback = TestInternalVoteResultCallback;
        type InternalAdminProvider = TestInternalAdminProvider;
        type InternalAdminCountProvider = ();
        type InternalThresholdProvider = TestInternalThresholdProvider;
        type MaxAdminsPerInstitution = ConstU32<32>;
        type TimeProvider = TestTimeProvider;
        type WeightInfo = ();
    }

    thread_local! {
        static USED_VOTE_NONCES: RefCell<BTreeSet<(u64, Vec<u8>, Vec<u8>)>> = RefCell::new(BTreeSet::new());
    }
    thread_local! {
        static TEST_NOW_SECS: RefCell<u64> = const { RefCell::new(DEFAULT_TEST_NOW_SECS) };
    }
    thread_local! {
        static JOINT_CALLBACK_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
    }
    thread_local! {
        static JOINT_CALLBACK_OVERRIDE_STATUS: RefCell<Option<u8>> = const { RefCell::new(None) };
    }
    // Phase 1 新增:内部投票终态回调测试桩。
    // INTERNAL_CALLBACK_SHOULD_FAIL = true → on_internal_vote_finalized 返回 Err,
    //   触发 set_status_and_emit 回滚;用于验证事务原子性。
    // INTERNAL_CALLBACK_LOG 记录每次被调用的 (proposal_id, approved),
    //   用于验证回调是否触发 / 触发参数是否正确。
    thread_local! {
        static INTERNAL_CALLBACK_SHOULD_FAIL: RefCell<bool> = const { RefCell::new(false) };
    }
    thread_local! {
        static INTERNAL_CALLBACK_LOG: RefCell<Vec<(u64, bool)>> = const { RefCell::new(Vec::new()) };
    }
    thread_local! {
        static INTERNAL_CALLBACK_OVERRIDE_STATUS: RefCell<Option<u8>> = const { RefCell::new(None) };
    }
    thread_local! {
        static INTERNAL_TERMINAL_CLEANUP_LOG: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    }
    thread_local! {
        static REGISTERED_DUOQIAN_THRESHOLD: RefCell<u32> = const { RefCell::new(3) };
    }

    pub struct TestSfidEligibility;
    pub struct TestPopulationSnapshotVerifier;
    pub struct TestJointVoteResultCallback;
    pub struct TestInternalVoteResultCallback;
    pub struct TestInternalAdminProvider;
    pub struct TestInternalThresholdProvider;

    fn pending_subject_institution() -> InstitutionPalletId {
        [77u8; 48]
    }

    fn pending_subject_admin(index: usize) -> AccountId32 {
        match index {
            0 => AccountId32::new([91u8; 32]),
            1 => AccountId32::new([92u8; 32]),
            _ => AccountId32::new([93u8; 32]),
        }
    }

    fn registered_subject_institution() -> InstitutionPalletId {
        [78u8; 48]
    }

    fn registered_subject_admin(index: usize) -> AccountId32 {
        match index {
            0 => AccountId32::new([81u8; 32]),
            1 => AccountId32::new([82u8; 32]),
            _ => AccountId32::new([83u8; 32]),
        }
    }

    fn set_registered_duoqian_threshold(threshold: u32) {
        REGISTERED_DUOQIAN_THRESHOLD.with(|value| *value.borrow_mut() = threshold);
    }

    impl InternalAdminProvider<AccountId32> for TestInternalAdminProvider {
        fn is_internal_admin(org: u8, institution: InstitutionPalletId, who: &AccountId32) -> bool {
            let who_bytes = who.encode();
            if who_bytes.len() != 32 {
                return false;
            }
            let mut who_arr = [0u8; 32];
            who_arr.copy_from_slice(&who_bytes);

            match org {
                internal_vote::ORG_NRC | internal_vote::ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                internal_vote::ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| n.duoqian_admins.iter().any(|admin| *admin == who_arr))
                    .unwrap_or(false),
                internal_vote::ORG_DUOQIAN => {
                    institution == registered_subject_institution()
                        && [
                            registered_subject_admin(0),
                            registered_subject_admin(1),
                            registered_subject_admin(2),
                        ]
                        .iter()
                        .any(|admin| admin == who)
                }
                _ => false,
            }
        }

        fn get_admin_list(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<sp_std::vec::Vec<AccountId32>> {
            match org {
                internal_vote::ORG_NRC | internal_vote::ORG_PRC => CHINA_CB
                    .iter()
                    .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    }),
                internal_vote::ORG_PRB => CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    }),
                internal_vote::ORG_DUOQIAN if institution == registered_subject_institution() => {
                    Some(sp_std::vec![
                        registered_subject_admin(0),
                        registered_subject_admin(1),
                        registered_subject_admin(2),
                    ])
                }
                _ => None,
            }
        }

        fn is_pending_internal_admin(
            org: u8,
            institution: InstitutionPalletId,
            who: &AccountId32,
        ) -> bool {
            org == internal_vote::ORG_DUOQIAN
                && institution == pending_subject_institution()
                && [pending_subject_admin(0), pending_subject_admin(1)]
                    .iter()
                    .any(|admin| admin == who)
        }

        fn get_pending_admin_list(
            org: u8,
            institution: InstitutionPalletId,
        ) -> Option<sp_std::vec::Vec<AccountId32>> {
            if org != internal_vote::ORG_DUOQIAN || institution != pending_subject_institution() {
                return None;
            }
            Some(sp_std::vec![
                pending_subject_admin(0),
                pending_subject_admin(1)
            ])
        }
    }

    impl InternalThresholdProvider for TestInternalThresholdProvider {
        fn pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            if org == internal_vote::ORG_DUOQIAN && institution == registered_subject_institution()
            {
                return REGISTERED_DUOQIAN_THRESHOLD.with(|value| Some(*value.borrow()));
            }
            // 中文注释：治理机构返回“毒化阈值”，用于证明治理投票不再依赖动态 Provider。
            if matches!(
                org,
                internal_vote::ORG_NRC | internal_vote::ORG_PRC | internal_vote::ORG_PRB
            ) {
                return Some(1);
            }
            None
        }

        fn pending_pass_threshold(org: u8, institution: InstitutionPalletId) -> Option<u32> {
            if org != internal_vote::ORG_DUOQIAN || institution != pending_subject_institution() {
                return None;
            }
            Some(2)
        }
    }

    const DEFAULT_TEST_NOW_SECS: u64 = 1_782_864_000;

    /// 测试用时间提供器：默认返回 2026 年中，可由单测覆盖为指定 UTC 秒。
    pub struct TestTimeProvider;
    impl frame_support::traits::UnixTime for TestTimeProvider {
        fn now() -> core::time::Duration {
            TEST_NOW_SECS.with(|secs| core::time::Duration::from_secs(*secs.borrow()))
        }
    }
    impl
        PopulationSnapshotVerifier<
            AccountId32,
            pallet::VoteNonceOf<Test>,
            pallet::VoteSignatureOf<Test>,
        > for TestPopulationSnapshotVerifier
    {
        fn verify_population_snapshot(
            _who: &AccountId32,
            eligible_total: u64,
            nonce: &pallet::VoteNonceOf<Test>,
            signature: &pallet::VoteSignatureOf<Test>,
        ) -> bool {
            eligible_total > 0 && !nonce.is_empty() && signature.as_slice() == b"snapshot-ok"
        }
    }

    impl SfidEligibility<AccountId32, <Test as frame_system::Config>::Hash> for TestSfidEligibility {
        fn is_eligible(
            binding_id: &<Test as frame_system::Config>::Hash,
            who: &AccountId32,
        ) -> bool {
            *binding_id == binding_id_ok() && who == &nrc_admin(0)
        }

        fn verify_and_consume_vote_credential(
            binding_id: &<Test as frame_system::Config>::Hash,
            who: &AccountId32,
            proposal_id: u64,
            nonce: &[u8],
            signature: &[u8],
        ) -> bool {
            if !Self::is_eligible(binding_id, who) || signature != b"vote-ok" || nonce.is_empty() {
                return false;
            }
            let key = (proposal_id, binding_id.encode(), nonce.to_vec());
            USED_VOTE_NONCES.with(|set| {
                let mut set = set.borrow_mut();
                if set.contains(&key) {
                    false
                } else {
                    set.insert(key);
                    true
                }
            })
        }

        fn cleanup_vote_credentials(proposal_id: u64) {
            USED_VOTE_NONCES.with(|set| {
                set.borrow_mut().retain(|(pid, _, _)| *pid != proposal_id);
            });
        }

        fn cleanup_vote_credentials_chunk(proposal_id: u64, limit: u32) -> VoteCredentialCleanup {
            let mut to_remove = Vec::new();
            USED_VOTE_NONCES.with(|set| {
                for key in set.borrow().iter() {
                    if key.0 == proposal_id {
                        to_remove.push(key.clone());
                        if to_remove.len() >= limit as usize {
                            break;
                        }
                    }
                }
            });

            let has_remaining = USED_VOTE_NONCES.with(|set| {
                let mut set = set.borrow_mut();
                for key in &to_remove {
                    set.remove(key);
                }
                set.iter().any(|(pid, _, _)| *pid == proposal_id)
            });

            VoteCredentialCleanup {
                removed: to_remove.len() as u32,
                loops: to_remove.len() as u32,
                has_remaining,
            }
        }
    }

    impl JointVoteResultCallback for TestJointVoteResultCallback {
        fn on_joint_vote_finalized(
            vote_proposal_id: u64,
            approved: bool,
        ) -> Result<ProposalExecutionOutcome, DispatchError> {
            if JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow()) {
                Err(DispatchError::Other("joint callback failed"))
            } else {
                if let Some(status) = JOINT_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow()) {
                    return Ok(match status {
                        STATUS_EXECUTED => ProposalExecutionOutcome::Executed,
                        STATUS_EXECUTION_FAILED => ProposalExecutionOutcome::FatalFailed,
                        STATUS_PASSED => ProposalExecutionOutcome::RetryableFailed,
                        _ => ProposalExecutionOutcome::Ignored,
                    });
                }
                let _ = vote_proposal_id;
                Ok(if approved {
                    ProposalExecutionOutcome::Executed
                } else {
                    ProposalExecutionOutcome::Executed
                })
            }
        }
    }

    impl InternalVoteResultCallback for TestInternalVoteResultCallback {
        fn on_internal_vote_finalized(
            proposal_id: u64,
            approved: bool,
        ) -> Result<ProposalExecutionOutcome, DispatchError> {
            // 先记日志,无论成功/失败都记 — 事务回滚会让日志外的状态回退,但
            // thread_local 不参与事务,通过对比"日志有/状态没变"即可验证回滚语义。
            INTERNAL_CALLBACK_LOG.with(|log| log.borrow_mut().push((proposal_id, approved)));
            if INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow()) {
                Err(DispatchError::Other("internal callback failed"))
            } else {
                if let Some(status) =
                    INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow())
                {
                    return Ok(match status {
                        STATUS_EXECUTED => ProposalExecutionOutcome::Executed,
                        STATUS_EXECUTION_FAILED => ProposalExecutionOutcome::FatalFailed,
                        STATUS_PASSED => ProposalExecutionOutcome::RetryableFailed,
                        _ => ProposalExecutionOutcome::Ignored,
                    });
                }
                Ok(if approved {
                    ProposalExecutionOutcome::RetryableFailed
                } else {
                    ProposalExecutionOutcome::Executed
                })
            }
        }

        fn on_execution_failed_terminal(proposal_id: u64) -> DispatchResult {
            INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow_mut().push(proposal_id));
            Ok(())
        }
    }

    fn new_test_ext() -> sp_io::TestExternalities {
        let storage = frame_system::GenesisConfig::<Test>::default()
            .build_storage()
            .expect("frame system genesis storage should build");
        let mut ext = sp_io::TestExternalities::new(storage);
        ext.execute_with(|| {
            USED_VOTE_NONCES.with(|set| set.borrow_mut().clear());
            TEST_NOW_SECS.with(|secs| *secs.borrow_mut() = DEFAULT_TEST_NOW_SECS);
            JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
            JOINT_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
            INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
            INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
            INTERNAL_CALLBACK_LOG.with(|log| log.borrow_mut().clear());
            INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow_mut().clear());
            REGISTERED_DUOQIAN_THRESHOLD.with(|value| *value.borrow_mut() = 3);
            System::set_block_number(1);
        });
        ext
    }

    fn nrc_pid() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[0].shenfen_id)
            .expect("nrc id should be shenfen_id bytes")
    }

    fn prc_pid() -> InstitutionPalletId {
        reserve_pallet_id_to_bytes(CHINA_CB[1].shenfen_id)
            .expect("prc id should be shenfen_id bytes")
    }

    fn prb_pid() -> InstitutionPalletId {
        shengbank_pallet_id_to_bytes(CHINA_CH[0].shenfen_id)
            .expect("prb id should be shenfen_id bytes")
    }

    fn nrc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[0].duoqian_admins[index])
    }

    fn all_prc_institutions() -> Vec<(InstitutionPalletId, AccountId32)> {
        CHINA_CB
            .iter()
            .skip(1)
            .map(|n| {
                (
                    reserve_pallet_id_to_bytes(n.shenfen_id)
                        .expect("prc id should be shenfen_id bytes"),
                    AccountId32::new(n.duoqian_admins[0]),
                )
            })
            .collect()
    }

    fn all_prb_institutions() -> Vec<(InstitutionPalletId, AccountId32)> {
        CHINA_CH
            .iter()
            .map(|n| {
                (
                    shengbank_pallet_id_to_bytes(n.shenfen_id)
                        .expect("prb id should be shenfen_id bytes"),
                    AccountId32::new(n.duoqian_admins[0]),
                )
            })
            .collect()
    }

    fn prc_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CB[1].duoqian_admins[index])
    }

    fn prb_admin(index: usize) -> AccountId32 {
        AccountId32::new(CHINA_CH[0].duoqian_admins[index])
    }

    fn institution_admins(institution: InstitutionPalletId) -> Vec<AccountId32> {
        CHINA_CB
            .iter()
            .find(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
            .map(|n| {
                n.duoqian_admins
                    .iter()
                    .copied()
                    .map(AccountId32::new)
                    .collect()
            })
            .or_else(|| {
                CHINA_CH
                    .iter()
                    .find(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
                    .map(|n| {
                        n.duoqian_admins
                            .iter()
                            .copied()
                            .map(AccountId32::new)
                            .collect()
                    })
            })
            .expect("institution should have admins")
    }

    fn institution_threshold(institution: InstitutionPalletId) -> usize {
        if institution == nrc_pid() {
            return primitives::count_const::NRC_INTERNAL_THRESHOLD as usize;
        }
        if CHINA_CB
            .iter()
            .skip(1)
            .any(|n| reserve_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        {
            return primitives::count_const::PRC_INTERNAL_THRESHOLD as usize;
        }
        if CHINA_CH
            .iter()
            .any(|n| shengbank_pallet_id_to_bytes(n.shenfen_id) == Some(institution))
        {
            return primitives::count_const::PRB_INTERNAL_THRESHOLD as usize;
        }
        panic!("unknown institution");
    }

    fn cast_joint_votes_until_finalized(
        proposal_id: u64,
        institution: InstitutionPalletId,
        approve: bool,
    ) {
        let admins = institution_admins(institution);
        let threshold = institution_threshold(institution);
        let required_votes = if approve {
            threshold
        } else {
            admins.len().saturating_sub(threshold).saturating_add(1)
        };
        for admin in admins.into_iter().take(required_votes) {
            assert_ok!(submit_joint_vote(admin, proposal_id, institution, approve));
        }
    }

    fn submit_joint_vote(
        who: AccountId32,
        proposal_id: u64,
        institution: InstitutionPalletId,
        approve: bool,
    ) -> DispatchResult {
        VotingEngine::joint_vote(
            RuntimeOrigin::signed(who),
            proposal_id,
            institution,
            approve,
        )
    }

    fn binding_id_ok() -> <Test as frame_system::Config>::Hash {
        <Test as frame_system::Config>::Hashing::hash(b"sfid-ok")
    }

    fn vote_nonce(input: &str) -> pallet::VoteNonceOf<Test> {
        input
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("nonce should fit")
    }

    fn vote_sig_ok() -> pallet::VoteSignatureOf<Test> {
        b"vote-ok"
            .to_vec()
            .try_into()
            .expect("signature should fit")
    }

    fn vote_sig_bad() -> pallet::VoteSignatureOf<Test> {
        b"bad".to_vec().try_into().expect("signature should fit")
    }

    fn snapshot_nonce_ok() -> pallet::VoteNonceOf<Test> {
        b"snap-nonce"
            .to_vec()
            .try_into()
            .expect("snapshot nonce should fit")
    }

    fn snapshot_sig_ok() -> pallet::VoteSignatureOf<Test> {
        b"snapshot-ok"
            .to_vec()
            .try_into()
            .expect("snapshot signature should fit")
    }

    fn set_joint_callback_should_fail(should_fail: bool) {
        JOINT_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = should_fail);
    }

    fn set_joint_callback_override_status(status: Option<u8>) {
        JOINT_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = status);
    }

    fn set_internal_callback_override_status(status: Option<u8>) {
        INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = status);
    }

    fn set_test_now_secs(secs: u64) {
        TEST_NOW_SECS.with(|value| *value.borrow_mut() = secs);
    }

    fn mark_vote_nonce_used(
        proposal_id: u64,
        binding_id: <Test as frame_system::Config>::Hash,
        nonce: &str,
    ) {
        USED_VOTE_NONCES.with(|set| {
            set.borrow_mut()
                .insert((proposal_id, binding_id.encode(), nonce.as_bytes().to_vec()));
        });
    }

    fn has_used_vote_nonce(
        proposal_id: u64,
        binding_id: <Test as frame_system::Config>::Hash,
        nonce: &str,
    ) -> bool {
        USED_VOTE_NONCES.with(|set| {
            set.borrow()
                .contains(&(proposal_id, binding_id.encode(), nonce.as_bytes().to_vec()))
        })
    }

    fn create_internal_proposal_via_engine(
        who: AccountId32,
        org: u8,
        institution: InstitutionPalletId,
    ) -> u64 {
        <VotingEngine as InternalVoteEngine<AccountId32>>::create_internal_proposal(
            who,
            org,
            institution,
        )
        .expect("internal proposal should be created")
    }

    fn create_pending_subject_proposal_via_engine(
        who: AccountId32,
        org: u8,
        institution: InstitutionPalletId,
    ) -> u64 {
        <VotingEngine as InternalVoteEngine<AccountId32>>::create_pending_subject_internal_proposal(
            who,
            org,
            institution,
        )
        .expect("pending subject proposal should be created")
    }

    fn create_admin_set_mutation_proposal_via_engine(
        who: AccountId32,
        org: u8,
        institution: InstitutionPalletId,
    ) -> u64 {
        <VotingEngine as InternalVoteEngine<AccountId32>>::create_admin_set_mutation_internal_proposal(
            who,
            org,
            institution,
        )
        .expect("admin-set mutation proposal should be created")
    }

    /// 测试辅助:走公开 `internal_vote` extrinsic 投票。
    ///
    /// Phase 1 改造后,管理员投票只能通过公开 call(不再经 trait 转发),
    /// 此函数包裹 `RuntimeOrigin::signed(who)` 让测试代码保持简洁。
    fn cast_internal_vote_via_extrinsic(
        who: AccountId32,
        proposal_id: u64,
        approve: bool,
    ) -> DispatchResult {
        VotingEngine::internal_vote(RuntimeOrigin::signed(who), proposal_id, approve)
    }

    fn insert_citizen_proposal(proposal_id: u64, eligible_total: u64, end: u64) {
        Proposals::<Test>::insert(
            proposal_id,
            Proposal {
                kind: PROPOSAL_KIND_JOINT,
                stage: STAGE_CITIZEN,
                status: STATUS_VOTING,
                internal_org: None,
                internal_institution: None,
                start: System::block_number(),
                end,
                citizen_eligible_total: eligible_total,
            },
        );
    }

    #[test]
    fn unix_seconds_to_year_uses_utc_gregorian_boundaries() {
        new_test_ext().execute_with(|| {
            assert_eq!(
                VotingEngine::unix_seconds_to_year(1_798_761_600).expect("valid 2027 timestamp"),
                2027
            );
            assert_eq!(
                VotingEngine::unix_seconds_to_year(1_830_297_600).expect("valid 2028 timestamp"),
                2028
            );
            assert_eq!(
                VotingEngine::unix_seconds_to_year(1_861_919_999).expect("valid 2028 timestamp"),
                2028
            );
            assert_eq!(
                VotingEngine::unix_seconds_to_year(1_861_920_000).expect("valid 2029 timestamp"),
                2029
            );
            assert_eq!(
                VotingEngine::unix_seconds_to_year(1_956_528_000).expect("valid 2032 timestamp"),
                2032
            );
        });
    }

    #[test]
    fn leap_year_rules_match_gregorian_calendar() {
        new_test_ext().execute_with(|| {
            assert!(VotingEngine::is_leap_year(2000));
            assert!(!VotingEngine::is_leap_year(2100));
            assert!(VotingEngine::is_leap_year(2400));
            assert_eq!(VotingEngine::days_in_year(2028), 366);
            assert_eq!(VotingEngine::days_in_year(2029), 365);
        });
    }

    #[test]
    fn proposal_id_counter_resets_at_real_utc_year_boundary() {
        new_test_ext().execute_with(|| {
            set_test_now_secs(1_830_297_599);
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_eq!(proposal_id, 2027_000_000);
            assert_eq!(CurrentProposalYear::<Test>::get(), 2027);
            assert_eq!(YearProposalCounter::<Test>::get(), 1);

            set_test_now_secs(1_830_297_600);
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(1),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_eq!(proposal_id, 2028_000_000);
            assert_eq!(CurrentProposalYear::<Test>::get(), 2028);
            assert_eq!(YearProposalCounter::<Test>::get(), 1);
            assert_eq!(NextProposalId::<Test>::get(), 2028_000_001);
        });
    }

    #[test]
    fn internal_proposal_must_be_created_by_same_institution_admin() {
        new_test_ext().execute_with(|| {
            // Phase 1 整改:`create_internal_proposal` 公开 extrinsic 已删除,
            // 只保留 `InternalVoteEngine` trait 入口供业务模块内部调用;
            // 这里直接验证 trait 路径的权限校验。
            let outsider = AccountId32::new([7u8; 32]);

            assert_noop!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_internal_proposal(
                    outsider,
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::NoPermission
            );

            assert_noop!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_internal_proposal(
                    prc_admin(0),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::NoPermission
            );

            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_eq!(proposal_id, 2026_000_000);
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .stage,
                STAGE_INTERNAL
            );
        });
    }

    #[test]
    fn active_internal_proposal_rejects_pending_subject() {
        new_test_ext().execute_with(|| {
            assert_noop!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_internal_proposal(
                    pending_subject_admin(0),
                    internal_vote::ORG_DUOQIAN,
                    pending_subject_institution(),
                ),
                pallet::Error::<Test>::InvalidInstitution
            );
        });
    }

    #[test]
    fn governance_internal_proposal_snapshots_fixed_threshold_not_provider() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            // 中文注释：测试 Provider 对治理机构故意返回 1，这里必须仍写入固定治理阈值。
            assert_eq!(
                InternalThresholdSnapshot::<Test>::get(proposal_id),
                Some(primitives::count_const::NRC_INTERNAL_THRESHOLD)
            );
        });
    }

    #[test]
    fn pending_subject_proposal_uses_pending_snapshot_and_threshold() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_pending_subject_proposal_via_engine(
                pending_subject_admin(0),
                internal_vote::ORG_DUOQIAN,
                pending_subject_institution(),
            );

            assert_eq!(InternalThresholdSnapshot::<Test>::get(proposal_id), Some(2));
            assert!(VotingEngine::is_admin_in_snapshot(
                proposal_id,
                pending_subject_institution(),
                &pending_subject_admin(0)
            ));

            assert_ok!(cast_internal_vote_via_extrinsic(
                pending_subject_admin(0),
                proposal_id,
                true
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );

            assert_ok!(cast_internal_vote_via_extrinsic(
                pending_subject_admin(1),
                proposal_id,
                true
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn registered_duoqian_proposal_snapshots_dynamic_threshold() {
        new_test_ext().execute_with(|| {
            set_registered_duoqian_threshold(3);
            let proposal_id = create_internal_proposal_via_engine(
                registered_subject_admin(0),
                internal_vote::ORG_DUOQIAN,
                registered_subject_institution(),
            );

            assert_eq!(InternalThresholdSnapshot::<Test>::get(proposal_id), Some(3));
            set_registered_duoqian_threshold(2);

            assert_ok!(cast_internal_vote_via_extrinsic(
                registered_subject_admin(0),
                proposal_id,
                true
            ));
            assert_ok!(cast_internal_vote_via_extrinsic(
                registered_subject_admin(1),
                proposal_id,
                true
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_VOTING
            );

            assert_ok!(cast_internal_vote_via_extrinsic(
                registered_subject_admin(2),
                proposal_id,
                true
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn admin_set_mutation_mutex_blocks_same_subject_regular_proposal() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_admin_set_mutation_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            let state = VotingEngine::internal_proposal_mutex(internal_vote::ORG_NRC, nrc_pid())
                .expect("mutex should exist");
            assert_eq!(state.admin_set_mutation_proposal, Some(proposal_id));

            assert_noop!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_internal_proposal(
                    nrc_admin(1),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::AdminSetMutationProposalActive
            );
        });
    }

    #[test]
    fn regular_mutex_blocks_same_subject_admin_set_mutation() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            let state = VotingEngine::internal_proposal_mutex(internal_vote::ORG_NRC, nrc_pid())
                .expect("mutex should exist");
            assert_eq!(state.regular_active_count, 1);
            assert_eq!(state.admin_set_mutation_proposal, None);

            assert_noop!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_admin_set_mutation_internal_proposal(
                    nrc_admin(1),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::RegularInternalProposalActive
            );

            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );
        });
    }

    #[test]
    fn regular_internal_proposals_can_coexist_under_same_subject() {
        new_test_ext().execute_with(|| {
            let first = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            let second = create_internal_proposal_via_engine(
                nrc_admin(1),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            assert_ne!(first, second);
            let state = VotingEngine::internal_proposal_mutex(internal_vote::ORG_NRC, nrc_pid())
                .expect("mutex should exist");
            assert_eq!(state.regular_active_count, 2);
            assert_eq!(state.admin_set_mutation_proposal, None);
        });
    }

    #[test]
    fn admin_set_mutation_passed_status_keeps_mutex_until_terminal_status() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_admin_set_mutation_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_PASSED
            );
            assert!(
                VotingEngine::internal_proposal_mutex(internal_vote::ORG_NRC, nrc_pid()).is_some()
            );
            assert_noop!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_internal_proposal(
                    nrc_admin(1),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::AdminSetMutationProposalActive
            );

            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_EXECUTION_FAILED
            ));
            assert!(
                VotingEngine::internal_proposal_mutex(internal_vote::ORG_NRC, nrc_pid()).is_none()
            );
        });
    }

    #[test]
    fn proposal_status_transition_state_machine_is_strict() {
        new_test_ext().execute_with(|| {
            let voting_to_passed = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_noop!(
                VotingEngine::set_status_and_emit(voting_to_passed, STATUS_EXECUTED),
                pallet::Error::<Test>::InvalidProposalStatus
            );
            assert_noop!(
                VotingEngine::set_status_and_emit(voting_to_passed, STATUS_EXECUTION_FAILED),
                pallet::Error::<Test>::InvalidProposalStatus
            );
            assert_noop!(
                VotingEngine::set_status_and_emit(voting_to_passed, STATUS_VOTING),
                pallet::Error::<Test>::InvalidProposalStatus
            );
            assert_ok!(VotingEngine::set_status_and_emit(
                voting_to_passed,
                STATUS_PASSED
            ));
            assert_noop!(
                VotingEngine::set_status_and_emit(voting_to_passed, STATUS_REJECTED),
                pallet::Error::<Test>::InvalidProposalStatus
            );
            assert_noop!(
                VotingEngine::set_status_and_emit(voting_to_passed, STATUS_VOTING),
                pallet::Error::<Test>::InvalidProposalStatus
            );
            assert_ok!(VotingEngine::set_status_and_emit(
                voting_to_passed,
                STATUS_EXECUTED
            ));
            assert_noop!(
                VotingEngine::set_status_and_emit(voting_to_passed, STATUS_PASSED),
                pallet::Error::<Test>::InvalidProposalStatus
            );

            let passed_to_failed = create_internal_proposal_via_engine(
                nrc_admin(1),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_ok!(VotingEngine::set_status_and_emit(
                passed_to_failed,
                STATUS_PASSED
            ));
            assert_ok!(VotingEngine::set_status_and_emit(
                passed_to_failed,
                STATUS_EXECUTION_FAILED
            ));
            assert_noop!(
                VotingEngine::set_status_and_emit(passed_to_failed, STATUS_EXECUTED),
                pallet::Error::<Test>::InvalidProposalStatus
            );

            let rejected = create_internal_proposal_via_engine(
                nrc_admin(2),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_ok!(VotingEngine::set_status_and_emit(rejected, STATUS_REJECTED));
            assert_noop!(
                VotingEngine::set_status_and_emit(rejected, STATUS_PASSED),
                pallet::Error::<Test>::InvalidProposalStatus
            );
        });
    }

    #[test]
    fn callback_execution_result_requires_callback_scope() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            assert_noop!(
                VotingEngine::set_callback_execution_result(proposal_id, STATUS_EXECUTED),
                pallet::Error::<Test>::InvalidProposalStatus
            );
        });
    }

    #[test]
    fn internal_vote_must_be_by_same_institution_admin() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                prb_admin(0),
                internal_vote::ORG_PRB,
                prb_pid(),
            );

            assert_noop!(
                cast_internal_vote_via_extrinsic(nrc_admin(0), proposal_id, true),
                pallet::Error::<Test>::NoPermission
            );

            assert_ok!(cast_internal_vote_via_extrinsic(
                prb_admin(1),
                proposal_id,
                true
            ));
        });
    }

    #[test]
    fn nrc_internal_vote_passes_at_13_yes_votes() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            for i in 0..12 {
                assert_ok!(cast_internal_vote_via_extrinsic(
                    nrc_admin(i),
                    proposal_id,
                    true
                ));
            }
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_VOTING
            );

            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(12),
                proposal_id,
                true
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn internal_vote_is_rejected_after_timeout() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                prc_admin(0),
                internal_vote::ORG_PRC,
                prc_pid(),
            );

            let proposal = VotingEngine::proposals(proposal_id).expect("proposal exists");
            System::set_block_number(proposal.end + 1);

            assert_ok!(VotingEngine::finalize_proposal(
                RuntimeOrigin::signed(prc_admin(0)),
                proposal_id,
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn internal_vote_timeout_is_auto_rejected_on_initialize() {
        new_test_ext().execute_with(|| {
            let proposal_id = create_internal_proposal_via_engine(
                prc_admin(0),
                internal_vote::ORG_PRC,
                prc_pid(),
            );

            let proposal = VotingEngine::proposals(proposal_id).expect("proposal exists");
            System::set_block_number(proposal.end);
            <VotingEngine as Hooks<u64>>::on_initialize(proposal.end);
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );

            let next = proposal.end + 1;
            System::set_block_number(next);
            <VotingEngine as Hooks<u64>>::on_initialize(next);
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn joint_proposal_must_be_created_by_nrc_or_prc_admin() {
        new_test_ext().execute_with(|| {
            // Phase 1 整改:`create_joint_proposal` 公开 extrinsic 已删除,
            // 只保留 `JointVoteEngine` trait 入口供业务模块内部调用;
            // 这里直接验证 trait 路径的权限校验。

            // 外部人员不能创建联合提案
            let outsider = AccountId32::new([9u8; 32]);
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    outsider,
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );

            // 省储会管理员可以创建联合提案
            let nonce_prc: pallet::VoteNonceOf<Test> =
                b"snap-nonce-prc".to_vec().try_into().expect("nonce fits");
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    prc_admin(0),
                    10,
                    nonce_prc.as_slice(),
                    sig.as_slice()
                )
            );

            // 国储会管理员可以创建联合提案
            let nonce_nrc: pallet::VoteNonceOf<Test> =
                b"snap-nonce-nrc".to_vec().try_into().expect("nonce fits");
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce_nrc.as_slice(),
                    sig.as_slice()
                )
            );
        });
    }

    #[test]
    fn joint_vote_requires_current_institution_admin() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            assert_ok!(submit_joint_vote(
                prc_admin(0),
                proposal_id,
                prc_pid(),
                true
            ));

            assert_noop!(
                submit_joint_vote(prc_admin(0), proposal_id, nrc_pid(), true),
                pallet::Error::<Test>::NoPermission
            );
        });
    }

    #[test]
    fn joint_vote_rejects_duplicate_admin_vote() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            assert_noop!(
                submit_joint_vote(nrc_admin(0), proposal_id, nrc_pid(), true),
                pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn joint_vote_uses_fixed_governance_threshold_not_provider() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            // 中文注释：测试 Provider 对治理机构故意返回 1；联合投票必须等固定阈值票数才形成机构结果。
            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));
            assert_eq!(
                JointVotesByInstitution::<Test>::get(proposal_id, nrc_pid()),
                None
            );

            for i in 1..primitives::count_const::NRC_INTERNAL_THRESHOLD as usize {
                assert_ok!(submit_joint_vote(
                    nrc_admin(i),
                    proposal_id,
                    nrc_pid(),
                    true
                ));
            }
            assert_eq!(
                JointVotesByInstitution::<Test>::get(proposal_id, nrc_pid()),
                Some(true)
            );
        });
    }

    #[test]
    fn joint_vote_auto_rejects_institution_when_yes_is_no_longer_reachable() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            cast_joint_votes_until_finalized(proposal_id, nrc_pid(), false);

            assert_eq!(
                JointVotesByInstitution::<Test>::get(proposal_id, nrc_pid()),
                Some(false)
            );
            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(
                JointTallies::<Test>::get(proposal_id).no,
                primitives::count_const::NRC_JOINT_VOTE_WEIGHT
            );
        });
    }

    #[test]
    fn joint_stage_mutex_blocks_admin_set_mutation_until_citizen_stage() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert!(
                VotingEngine::internal_proposal_mutex(internal_vote::ORG_NRC, nrc_pid()).is_some()
            );
            assert!(
                VotingEngine::internal_proposal_mutex(internal_vote::ORG_PRC, prc_pid()).is_some()
            );
            assert_noop!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_admin_set_mutation_internal_proposal(
                    nrc_admin(1),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                ),
                pallet::Error::<Test>::RegularInternalProposalActive
            );

            cast_joint_votes_until_finalized(proposal_id, nrc_pid(), false);
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal should exist")
                    .stage,
                STAGE_CITIZEN
            );
            assert!(
                VotingEngine::internal_proposal_mutex(internal_vote::ORG_NRC, nrc_pid()).is_none()
            );
            assert!(
                VotingEngine::internal_proposal_mutex(internal_vote::ORG_PRC, prc_pid()).is_none()
            );

            assert_ok!(
                <VotingEngine as InternalVoteEngine<AccountId32>>::create_admin_set_mutation_internal_proposal(
                    nrc_admin(1),
                    internal_vote::ORG_NRC,
                    nrc_pid(),
                )
            );
        });
    }

    #[test]
    fn population_snapshot_nonce_cannot_be_reused_across_proposals() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert_ok!(
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice()
                )
            );

            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            assert!(
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    11,
                    nonce.as_slice(),
                    sig.as_slice()
                )
                .is_err()
            );
        });
    }

    #[test]
    fn citizen_vote_rejects_invalid_signature_and_allows_valid_vote() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_noop!(
                VotingEngine::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    binding_id_ok(),
                    vote_nonce("n-1"),
                    vote_sig_bad(),
                    true
                ),
                pallet::Error::<Test>::InvalidSfidVoteCredential
            );

            assert_ok!(VotingEngine::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("n-2"),
                vote_sig_ok(),
                true
            ));
            assert_eq!(CitizenTallies::<Test>::get(0).yes, 1);
        });
    }

    #[test]
    fn citizen_vote_same_sfid_can_only_vote_once_per_proposal() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_ok!(VotingEngine::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("n-1"),
                vote_sig_ok(),
                true
            ));

            assert_noop!(
                VotingEngine::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    binding_id_ok(),
                    vote_nonce("n-2"),
                    vote_sig_ok(),
                    false
                ),
                pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn citizen_vote_credential_nonce_is_replay_protected_per_proposal_and_sfid() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            insert_citizen_proposal(1, 10, 100);

            assert_ok!(VotingEngine::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("same"),
                vote_sig_ok(),
                true
            ));

            assert_ok!(VotingEngine::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                1,
                binding_id_ok(),
                vote_nonce("same"),
                vote_sig_ok(),
                true
            ));
        });
    }

    #[test]
    fn citizen_vote_rejects_when_eligible_total_not_set_in_proposal() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 0, 100);

            assert_noop!(
                VotingEngine::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    binding_id_ok(),
                    vote_nonce("x-1"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::CitizenEligibleTotalNotSet
            );
        });
    }

    #[test]
    fn citizen_timeout_with_half_or_less_is_rejected() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });
            System::set_block_number(6);

            assert_ok!(VotingEngine::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                0
            ));
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn citizen_timeout_is_auto_rejected_on_initialize() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            assert_ok!(VotingEngine::schedule_proposal_expiry(0, 5));
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

            System::set_block_number(6);
            <VotingEngine as Hooks<u64>>::on_initialize(6);
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn citizen_timeout_auto_registers_cleanup_and_clears_vote_nonces() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 5);
            assert_ok!(VotingEngine::schedule_proposal_expiry(0, 5));

            assert_ok!(VotingEngine::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("timeout-cleanup"),
                vote_sig_ok(),
                true
            ));
            assert!(has_used_vote_nonce(0, binding_id_ok(), "timeout-cleanup"));

            System::set_block_number(6);
            <VotingEngine as Hooks<u64>>::on_initialize(6);

            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_REJECTED
            );
            assert!(has_used_vote_nonce(0, binding_id_ok(), "timeout-cleanup"));

            // set_status_and_emit(STATUS_REJECTED) 在 on_initialize(6) 中被调用时
            // 已自动注册 90 天后清理，无需手动调用 cleanup_joint_proposal。
            // cleanup_at = 6 + retention
            let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
            let cleanup_block = 6 + retention;
            for i in 0..20u64 {
                System::set_block_number(cleanup_block + i);
                <VotingEngine as Hooks<u64>>::on_initialize(cleanup_block + i);
            }
            assert!(!has_used_vote_nonce(0, binding_id_ok(), "timeout-cleanup"));
        });
    }

    #[test]
    fn citizen_vote_rejects_ineligible_hash_and_ineligible_account() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);

            assert_noop!(
                VotingEngine::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    0,
                    <Test as frame_system::Config>::Hashing::hash(b"sfid-other"),
                    vote_nonce("n-ineligible-hash"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::SfidNotEligible
            );

            let outsider = AccountId32::new([7u8; 32]);
            assert_noop!(
                VotingEngine::citizen_vote(
                    RuntimeOrigin::signed(outsider),
                    0,
                    binding_id_ok(),
                    vote_nonce("n-ineligible"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::SfidNotEligible
            );
        });
    }

    #[test]
    fn citizen_vote_rejects_when_not_in_citizen_stage() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_noop!(
                VotingEngine::citizen_vote(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    proposal_id,
                    binding_id_ok(),
                    vote_nonce("joint-stage"),
                    vote_sig_ok(),
                    true
                ),
                pallet::Error::<Test>::InvalidProposalStage
            );
        });
    }

    #[test]
    fn citizen_vote_passes_immediately_when_yes_exceeds_half() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

            assert_ok!(VotingEngine::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("immediate-pass"),
                vote_sig_ok(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_EXECUTED);
        });
    }

    #[test]
    fn cleanup_joint_proposal_cleans_used_vote_nonce_after_retention() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            CitizenTallies::<Test>::insert(0, VoteCountU64 { yes: 5, no: 0 });

            assert_ok!(VotingEngine::citizen_vote(
                RuntimeOrigin::signed(nrc_admin(0)),
                0,
                binding_id_ok(),
                vote_nonce("immediate-cleanup"),
                vote_sig_ok(),
                true
            ));

            let proposal = Proposals::<Test>::get(0).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_EXECUTED);
            assert!(has_used_vote_nonce(0, binding_id_ok(), "immediate-cleanup"));

            // 中文注释：执行成功终态会注册 90 天延迟清理，清理前凭证仍保留。
            let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
            let cleanup_block = retention;
            for i in 0..20u64 {
                System::set_block_number(cleanup_block + i);
                <VotingEngine as Hooks<u64>>::on_initialize(cleanup_block + i);
            }
            assert!(!has_used_vote_nonce(
                0,
                binding_id_ok(),
                "immediate-cleanup"
            ));
        });
    }

    #[test]
    fn citizen_finalize_before_timeout_is_rejected() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            System::set_block_number(100);

            assert_noop!(
                VotingEngine::finalize_proposal(RuntimeOrigin::signed(nrc_admin(0)), 0),
                pallet::Error::<Test>::VoteNotExpired
            );
        });
    }

    #[test]
    fn citizen_pass_threshold_function_boundaries_are_correct() {
        assert!(!citizen_vote::is_citizen_vote_passed(0, 0));
        assert!(!citizen_vote::is_citizen_vote_passed(5, 10));
        assert!(citizen_vote::is_citizen_vote_passed(6, 10));
    }

    #[test]
    fn citizen_reject_threshold_function_boundaries_are_correct() {
        // eligible_total=0 → 不否决（无意义）
        assert!(!citizen_vote::is_citizen_vote_rejected(0, 0));
        // 反对 4/10 = 40% < 50% → 不否决（赞成仍有可能 > 50%）
        assert!(!citizen_vote::is_citizen_vote_rejected(4, 10));
        // 反对 5/10 = 50% → 否决（赞成最多 50%，无法严格 > 50%）
        assert!(citizen_vote::is_citizen_vote_rejected(5, 10));
        // 反对 6/10 = 60% → 否决
        assert!(citizen_vote::is_citizen_vote_rejected(6, 10));
    }

    #[test]
    fn joint_vote_all_yes_passes_immediately() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    100,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            cast_joint_votes_until_finalized(proposal_id, nrc_pid(), true);

            for (institution, _) in all_prc_institutions() {
                cast_joint_votes_until_finalized(proposal_id, institution, true);
            }
            for (institution, _) in all_prb_institutions() {
                cast_joint_votes_until_finalized(proposal_id, institution, true);
            }

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_EXECUTED);
            assert_eq!(proposal.stage, STAGE_JOINT);
            assert_eq!(
                JointTallies::<Test>::get(proposal_id).yes,
                primitives::count_const::JOINT_VOTE_TOTAL
            );
        });
    }

    #[test]
    fn joint_vote_non_unanimous_moves_to_citizen_immediately_after_one_institution_rejects() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    77,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");
            cast_joint_votes_until_finalized(proposal_id, nrc_pid(), true);
            let first_prc = all_prc_institutions()
                .first()
                .cloned()
                .expect("there should be at least one prc institution");
            cast_joint_votes_until_finalized(proposal_id, first_prc.0, false);

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(proposal.start, System::block_number());
            assert_eq!(
                proposal.end,
                proposal.start + primitives::count_const::VOTING_DURATION_BLOCKS as u64
            );
            assert_eq!(proposal.citizen_eligible_total, 77);
            assert_eq!(JointTallies::<Test>::get(proposal_id).no, 1);
        });
    }

    #[test]
    fn joint_vote_timeout_moves_to_citizen_when_not_unanimous() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    88,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            System::set_block_number(proposal.end + 1);
            assert_ok!(VotingEngine::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                proposal_id
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(
                proposal.end,
                (proposal.start + primitives::count_const::VOTING_DURATION_BLOCKS as u64)
            );
        });
    }

    #[test]
    fn joint_vote_timeout_auto_moves_to_citizen_on_initialize() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    88,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(submit_joint_vote(
                nrc_admin(0),
                proposal_id,
                nrc_pid(),
                true
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            let expired_at = proposal.end + 1;
            System::set_block_number(expired_at);
            <VotingEngine as Hooks<u64>>::on_initialize(expired_at);

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.stage, STAGE_CITIZEN);
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(proposal.start, expired_at);
            assert_eq!(
                proposal.end,
                expired_at + primitives::count_const::VOTING_DURATION_BLOCKS as u64
            );
        });
    }

    #[test]
    fn joint_vote_timeout_with_unanimous_tally_passes() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    66,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");
            JointTallies::<Test>::insert(
                proposal_id,
                VoteCountU32 {
                    yes: primitives::count_const::JOINT_VOTE_TOTAL,
                    no: 0,
                },
            );

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            System::set_block_number(proposal.end + 1);
            assert_ok!(VotingEngine::finalize_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                proposal_id
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_EXECUTED);
            assert_eq!(proposal.stage, STAGE_JOINT);
        });
    }

    #[test]
    fn joint_vote_callback_failure_rolls_back_final_status() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    100,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            set_joint_callback_should_fail(true);
            assert!(VotingEngine::set_status_and_emit(proposal_id, STATUS_PASSED).is_err());

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_VOTING);
            assert_eq!(proposal.stage, STAGE_JOINT);
        });
    }

    #[test]
    fn joint_vote_callback_failure_does_not_cleanup_vote_credentials() {
        new_test_ext().execute_with(|| {
            insert_citizen_proposal(0, 10, 100);
            mark_vote_nonce_used(0, binding_id_ok(), "keep-on-fail");
            set_joint_callback_should_fail(true);

            assert!(VotingEngine::set_status_and_emit(0, STATUS_PASSED).is_err());
            assert_eq!(
                Proposals::<Test>::get(0)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );
            assert!(has_used_vote_nonce(0, binding_id_ok(), "keep-on-fail"));
        });
    }

    #[test]
    fn proposal_finalized_event_uses_status_after_joint_callback_override() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    100,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            set_joint_callback_override_status(Some(STATUS_EXECUTION_FAILED));
            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            assert_eq!(proposal.status, STATUS_EXECUTION_FAILED);

            let finalized = System::events()
                .into_iter()
                .rev()
                .find_map(|record| match record.event {
                    RuntimeEvent::VotingEngine(Event::ProposalFinalized {
                        proposal_id: event_id,
                        status,
                    }) if event_id == proposal_id => Some(status),
                    _ => None,
                })
                .expect("proposal finalized event should exist");
            assert_eq!(finalized, STATUS_EXECUTION_FAILED);
            let finalized_count = System::events()
                .into_iter()
                .filter(|record| {
                    matches!(
                        &record.event,
                        RuntimeEvent::VotingEngine(Event::ProposalFinalized {
                            proposal_id: event_id,
                            ..
                        }) if *event_id == proposal_id
                    )
                })
                .count();
            assert_eq!(finalized_count, 1);
        });
    }

    #[test]
    fn auto_finalize_requeues_failed_joint_callback() {
        new_test_ext().execute_with(|| {
            let nonce = snapshot_nonce_ok();
            let sig = snapshot_sig_ok();
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    66,
                    nonce.as_slice(),
                    sig.as_slice(),
                )
                .expect("joint proposal should be created");

            JointTallies::<Test>::insert(
                proposal_id,
                VoteCountU32 {
                    yes: primitives::count_const::JOINT_VOTE_TOTAL,
                    no: 0,
                },
            );

            let proposal = Proposals::<Test>::get(proposal_id).expect("proposal should exist");
            let expired_at = proposal.end + 1;

            set_joint_callback_should_fail(true);
            System::set_block_number(expired_at);
            <VotingEngine as Hooks<u64>>::on_initialize(expired_at);

            assert_eq!(
                Proposals::<Test>::get(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_VOTING
            );
            assert_eq!(PendingExpiryBucket::<Test>::get(), Some(expired_at));
            assert_eq!(
                ProposalsByExpiry::<Test>::get(expired_at),
                vec![proposal_id]
            );

            set_joint_callback_should_fail(false);
            let next_block = expired_at + 1;
            System::set_block_number(next_block);
            <VotingEngine as Hooks<u64>>::on_initialize(next_block);

            assert_eq!(
                Proposals::<Test>::get(proposal_id)
                    .expect("proposal should exist")
                    .status,
                STATUS_EXECUTED
            );
            assert!(PendingExpiryBucket::<Test>::get().is_none());
            assert!(ProposalsByExpiry::<Test>::get(expired_at).is_empty());
        });
    }

    #[test]
    fn auto_finalize_uses_pending_cursor_when_expiry_bucket_exceeds_per_block_limit() {
        new_test_ext().execute_with(|| {
            let end = 5u64;
            let expiry = end + 1;
            let total = 70u64;
            for proposal_id in 0..total {
                insert_citizen_proposal(proposal_id, 10, end);
                assert_ok!(VotingEngine::schedule_proposal_expiry(proposal_id, end));
            }

            System::set_block_number(6);
            <VotingEngine as Hooks<u64>>::on_initialize(6);
            assert_eq!(ProposalsByExpiry::<Test>::get(expiry).len(), 6);
            assert_eq!(PendingExpiryBucket::<Test>::get(), Some(expiry));

            System::set_block_number(7);
            <VotingEngine as Hooks<u64>>::on_initialize(7);
            assert!(ProposalsByExpiry::<Test>::get(expiry).is_empty());
            assert!(PendingExpiryBucket::<Test>::get().is_none());
            for proposal_id in 0..total {
                assert_eq!(
                    Proposals::<Test>::get(proposal_id)
                        .expect("proposal should exist")
                        .status,
                    STATUS_REJECTED
                );
            }
        });
    }

    #[test]
    fn schedule_proposal_expiry_rejects_bucket_overflow() {
        new_test_ext().execute_with(|| {
            let end = 5u64;
            for proposal_id in 0..128u64 {
                assert_ok!(VotingEngine::schedule_proposal_expiry(proposal_id, end));
            }

            assert_noop!(
                VotingEngine::schedule_proposal_expiry(999, end),
                pallet::Error::<Test>::TooManyProposalsAtExpiry
            );
        });
    }

    #[test]
    fn cleanup_joint_proposal_chunks_cleanup_across_blocks() {
        new_test_ext().execute_with(|| {
            let proposal_id = 42u64;
            let citizen_hashes = [
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-1"),
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-2"),
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-3"),
            ];

            insert_citizen_proposal(proposal_id, 10, 100);
            JointVotesByInstitution::<Test>::insert(proposal_id, nrc_pid(), true);
            JointVotesByInstitution::<Test>::insert(proposal_id, prc_pid(), true);
            JointVotesByInstitution::<Test>::insert(proposal_id, prb_pid(), true);
            for (index, binding_id) in citizen_hashes.iter().enumerate() {
                CitizenVotesByBindingId::<Test>::insert(proposal_id, *binding_id, true);
                let nonce = match index {
                    0 => "cleanup-nonce-1",
                    1 => "cleanup-nonce-2",
                    _ => "cleanup-nonce-3",
                };
                mark_vote_nonce_used(proposal_id, *binding_id, nonce);
            }

            // 中文注释：投票通过后由 callback 返回 Executed，终态会注册 90 天后清理。
            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            // 此时 PendingProposalCleanups 尚未设置（要等 90 天后 process_cleanup_queue 触发）
            assert!(PendingProposalCleanups::<Test>::get(proposal_id).is_none());

            // set_status_and_emit 在 block 0 调用，cleanup_at = 0 + retention
            let retention = 90u64 * primitives::pow_const::BLOCKS_PER_DAY;
            let cleanup_block = retention;
            // 运行多轮 on_initialize 直到清理完成
            for i in 0..20u64 {
                System::set_block_number(cleanup_block + i);
                <VotingEngine as Hooks<u64>>::on_initialize(cleanup_block + i);
                if PendingProposalCleanups::<Test>::get(proposal_id).is_none()
                    && Proposals::<Test>::get(proposal_id).is_none()
                {
                    break;
                }
            }

            assert!(PendingProposalCleanups::<Test>::get(proposal_id).is_none());
            assert!(!has_used_vote_nonce(
                proposal_id,
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-1"),
                "cleanup-nonce-1"
            ));
            assert!(!has_used_vote_nonce(
                proposal_id,
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-2"),
                "cleanup-nonce-2"
            ));
            assert!(!has_used_vote_nonce(
                proposal_id,
                <Test as frame_system::Config>::Hashing::hash(b"cleanup-sfid-3"),
                "cleanup-nonce-3"
            ));
        });
    }

    #[test]
    fn store_and_get_proposal_data_works() {
        new_test_ext().execute_with(|| {
            assert!(VotingEngine::get_proposal_data(0).is_none());

            let data = b"test proposal data".to_vec();
            assert_ok!(VotingEngine::store_proposal_data(0, data.clone()));

            let stored = VotingEngine::get_proposal_data(0).expect("data should exist");
            assert_eq!(&stored[..], &data[..]);

            // 覆盖
            let data2 = b"updated data".to_vec();
            assert_ok!(VotingEngine::store_proposal_data(0, data2.clone()));
            let stored2 = VotingEngine::get_proposal_data(0).expect("data should exist");
            assert_eq!(&stored2[..], &data2[..]);
        });
    }

    #[test]
    fn store_and_get_proposal_object_works() {
        new_test_ext().execute_with(|| {
            assert!(VotingEngine::get_proposal_object(7).is_none());
            assert!(VotingEngine::get_proposal_object_meta(7).is_none());

            let object = vec![1u8, 2, 3, 4, 5, 6];
            assert_ok!(VotingEngine::store_proposal_object(7, 1, object.clone()));

            let stored = VotingEngine::get_proposal_object(7).expect("object should exist");
            assert_eq!(stored, object);

            let meta = VotingEngine::get_proposal_object_meta(7).expect("meta should exist");
            assert_eq!(meta.kind, 1);
            assert_eq!(meta.object_len, 6);
            assert_eq!(
                meta.object_hash,
                <Test as frame_system::Config>::Hashing::hash(&object)
            );

            VotingEngine::remove_proposal_object(7);
            assert!(VotingEngine::get_proposal_object(7).is_none());
            assert!(VotingEngine::get_proposal_object_meta(7).is_none());
        });
    }

    #[test]
    fn store_proposal_meta_works() {
        new_test_ext().execute_with(|| {
            VotingEngine::store_proposal_meta(42, 100);
            let meta = ProposalMeta::<Test>::get(42).expect("meta should exist");
            assert_eq!(meta.created_at, 100);
            assert!(meta.passed_at.is_none());

            VotingEngine::set_proposal_passed(42, 200);
            let meta2 = ProposalMeta::<Test>::get(42).expect("meta should exist");
            assert_eq!(meta2.passed_at, Some(200));
        });
    }

    // ──── Phase 1 新增:公开 internal_vote extrinsic + InternalVoteResultCallback ────

    /// 重置 Phase 1 新增的 thread_local 测试桩状态,避免用例间污染。
    fn reset_internal_callback_state() {
        INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = false);
        INTERNAL_CALLBACK_OVERRIDE_STATUS.with(|value| *value.borrow_mut() = None);
        INTERNAL_CALLBACK_LOG.with(|log| log.borrow_mut().clear());
        INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow_mut().clear());
    }

    #[test]
    fn internal_vote_public_call_casts_vote() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(0),
                proposal_id,
                true
            ));

            assert!(InternalVotesByAccount::<Test>::contains_key(
                proposal_id,
                &nrc_admin(0)
            ));
            assert_eq!(InternalTallies::<Test>::get(proposal_id).yes, 1);
            assert_eq!(InternalTallies::<Test>::get(proposal_id).no, 0);
        });
    }

    #[test]
    fn internal_vote_rejects_non_admin() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            // 非 NRC 管理员(比如 PRB 的管理员)不能投 NRC 的内部提案。
            assert_noop!(
                cast_internal_vote_via_extrinsic(prb_admin(0), proposal_id, true),
                pallet::Error::<Test>::NoPermission
            );
            assert_eq!(InternalTallies::<Test>::get(proposal_id).yes, 0);
        });
    }

    #[test]
    fn internal_vote_rejects_double_vote() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );
            assert_ok!(cast_internal_vote_via_extrinsic(
                nrc_admin(0),
                proposal_id,
                true
            ));
            assert_noop!(
                cast_internal_vote_via_extrinsic(nrc_admin(0), proposal_id, false),
                pallet::Error::<Test>::AlreadyVoted
            );
        });
    }

    #[test]
    fn internal_vote_passes_triggers_callback_approved_true() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            // NRC 阈值 13 票;投 13 票赞成使提案进入 STATUS_PASSED。
            for i in 0..13 {
                assert_ok!(cast_internal_vote_via_extrinsic(
                    nrc_admin(i),
                    proposal_id,
                    true
                ));
            }

            // 回调被触发且 approved = true。
            let log = INTERNAL_CALLBACK_LOG.with(|log| log.borrow().clone());
            assert_eq!(log, vec![(proposal_id, true)]);
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn internal_vote_early_rejection_triggers_callback_approved_false() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            // NRC 总管理员 19 人,阈值 13 票。7 票反对 → 剩余 12 人全同意也到不了 13,
            // 触发提前否决。
            for i in 0..7 {
                assert_ok!(cast_internal_vote_via_extrinsic(
                    nrc_admin(i),
                    proposal_id,
                    false
                ));
            }

            // 回调被触发且 approved = false。
            let log = INTERNAL_CALLBACK_LOG.with(|log| log.borrow().clone());
            assert_eq!(log, vec![(proposal_id, false)]);
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_REJECTED
            );
        });
    }

    #[test]
    fn internal_vote_callback_not_called_before_threshold() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            // 投 12 票赞成(阈值 13),未达阈值不应触发回调。
            for i in 0..12 {
                assert_ok!(cast_internal_vote_via_extrinsic(
                    nrc_admin(i),
                    proposal_id,
                    true
                ));
            }

            let log = INTERNAL_CALLBACK_LOG.with(|log| log.borrow().clone());
            assert!(log.is_empty(), "未达阈值回调不应被调用: {:?}", log);
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_VOTING
            );
        });
    }

    #[test]
    fn internal_vote_callback_err_rolls_back_status() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            // 前 12 票赞成(未达阈值,不触发回调,不受 SHOULD_FAIL 影响)。
            for i in 0..12 {
                assert_ok!(cast_internal_vote_via_extrinsic(
                    nrc_admin(i),
                    proposal_id,
                    true
                ));
            }

            // 第 13 票达阈值,回调会被触发;置 SHOULD_FAIL 让回调返回 Err。
            INTERNAL_CALLBACK_SHOULD_FAIL.with(|flag| *flag.borrow_mut() = true);
            assert!(cast_internal_vote_via_extrinsic(nrc_admin(12), proposal_id, true).is_err());

            // 提案状态、票数必须整体回滚到投票中 + 12 票。
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_VOTING
            );
            assert_eq!(InternalTallies::<Test>::get(proposal_id).yes, 12);
            assert!(!InternalVotesByAccount::<Test>::contains_key(
                proposal_id,
                &nrc_admin(12)
            ));
        });
    }

    #[test]
    fn manual_retry_third_failure_marks_execution_failed() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            assert_eq!(
                ProposalExecutionRetryStates::<Test>::get(proposal_id)
                    .expect("retry state should exist")
                    .manual_attempts,
                0
            );

            assert_ok!(VotingEngine::retry_passed_proposal(
                RuntimeOrigin::signed(nrc_admin(0)),
                proposal_id
            ));
            assert_eq!(
                ProposalExecutionRetryStates::<Test>::get(proposal_id)
                    .expect("retry state should remain")
                    .manual_attempts,
                1
            );
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );

            assert_ok!(VotingEngine::retry_passed_proposal(
                RuntimeOrigin::signed(nrc_admin(1)),
                proposal_id
            ));
            assert_eq!(
                ProposalExecutionRetryStates::<Test>::get(proposal_id)
                    .expect("retry state should remain")
                    .manual_attempts,
                2
            );

            assert_ok!(VotingEngine::retry_passed_proposal(
                RuntimeOrigin::signed(nrc_admin(2)),
                proposal_id
            ));
            assert!(ProposalExecutionRetryStates::<Test>::get(proposal_id).is_none());
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_EXECUTION_FAILED
            );

            let third_retry_outcome = System::events()
                .into_iter()
                .rev()
                .find_map(|record| match record.event {
                    RuntimeEvent::VotingEngine(Event::ProposalExecutionRetried {
                        proposal_id: event_id,
                        manual_attempts: 3,
                        outcome,
                    }) if event_id == proposal_id => Some(outcome),
                    _ => None,
                })
                .expect("third retry event should exist");
            assert_eq!(third_retry_outcome, STATUS_EXECUTION_FAILED);
        });
    }

    #[test]
    fn default_cancel_callback_rejects_passed_retry_proposal() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            assert_noop!(
                VotingEngine::cancel_passed_proposal(
                    RuntimeOrigin::signed(nrc_admin(0)),
                    proposal_id,
                    b"not allowed"
                        .to_vec()
                        .try_into()
                        .expect("reason should fit")
                ),
                Error::<Test>::ProposalCancellationNotAllowed
            );
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_PASSED
            );
        });
    }

    #[test]
    fn automatic_fatal_failed_runs_execution_failed_terminal_hook() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            set_internal_callback_override_status(Some(STATUS_EXECUTION_FAILED));
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_EXECUTION_FAILED
            );
            let cleanup_log = INTERNAL_TERMINAL_CLEANUP_LOG.with(|log| log.borrow().clone());
            assert_eq!(cleanup_log, vec![proposal_id]);
        });
    }

    #[test]
    fn joint_retryable_outcome_is_forced_to_execution_failed() {
        new_test_ext().execute_with(|| {
            set_joint_callback_override_status(Some(STATUS_PASSED));
            let proposal_id =
                <VotingEngine as JointVoteEngine<AccountId32>>::create_joint_proposal(
                    nrc_admin(0),
                    10,
                    snapshot_nonce_ok().as_slice(),
                    snapshot_sig_ok().as_slice(),
                )
                .expect("joint proposal should be created");

            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_EXECUTION_FAILED
            );
            assert!(
                ProposalExecutionRetryStates::<Test>::get(proposal_id).is_none(),
                "joint proposal must not enter internal retry state"
            );
        });
    }

    #[test]
    fn execution_retry_deadline_expires_to_execution_failed() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            let proposal_id = create_internal_proposal_via_engine(
                nrc_admin(0),
                internal_vote::ORG_NRC,
                nrc_pid(),
            );

            assert_ok!(VotingEngine::set_status_and_emit(
                proposal_id,
                STATUS_PASSED
            ));
            let deadline = ProposalExecutionRetryStates::<Test>::get(proposal_id)
                .expect("retry state should exist")
                .retry_deadline;

            System::set_block_number(deadline);
            <VotingEngine as Hooks<u64>>::on_initialize(deadline);

            assert!(ProposalExecutionRetryStates::<Test>::get(proposal_id).is_none());
            assert_eq!(
                VotingEngine::proposals(proposal_id)
                    .expect("proposal exists")
                    .status,
                STATUS_EXECUTION_FAILED
            );
            assert!(System::events().into_iter().any(|record| matches!(
                record.event,
                RuntimeEvent::VotingEngine(Event::ProposalExecutionRetryExpired {
                    proposal_id: event_id
                }) if event_id == proposal_id
            )));
        });
    }

    #[test]
    fn internal_vote_rejects_wrong_stage_joint_proposal() {
        new_test_ext().execute_with(|| {
            reset_internal_callback_state();
            // 手工写一个 kind=JOINT 的提案,用 internal_vote 去投 → 应拒绝。
            let proposal_id = 999u64;
            let now = <frame_system::Pallet<Test>>::block_number();
            Proposals::<Test>::insert(
                proposal_id,
                Proposal {
                    kind: PROPOSAL_KIND_JOINT,
                    stage: STAGE_JOINT,
                    status: STATUS_VOTING,
                    internal_org: None,
                    internal_institution: None,
                    start: now,
                    end: now + 100,
                    citizen_eligible_total: 0,
                },
            );

            assert_noop!(
                cast_internal_vote_via_extrinsic(nrc_admin(0), proposal_id, true),
                pallet::Error::<Test>::InvalidProposalKind
            );
        });
    }
}
