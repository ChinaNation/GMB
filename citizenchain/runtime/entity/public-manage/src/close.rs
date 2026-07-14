//! 公权机构多签关闭流程实现(call_index=1)。
//!
//! 仅服务于已注册的 CID 机构账户(`AccountRegisteredCid.contains_key` 命中);
//! 个人多签关闭走 personal-manage::propose_close 入口。
//!
//! 业务流程:
//! 1. 校验地址是机构地址(否则返回 `NotInstitutionAccount`)
//! 2. 校验机构账户已 Active(从 InstitutionAccounts 读)
//! 3. 校验发起人是该机构账户的活跃管理员(admins 模块::AdminAccounts[account account])
//! 4. 校验余额≥关闭门槛 + 转出金额≥ED + 无 reserved 余额
//! 5. 注销生命周期投票的全员阈值由投票引擎按管理员快照生成
//! 6. 写入 InstitutionPendingClose[address] = proposal_id 防并发
//! 7. 发射 InstitutionCloseProposed 事件

extern crate alloc;

use codec::Encode;
use frame_support::{
    ensure,
    traits::{Currency, Get, ReservableCurrency},
};
use primitives::institution_asset::{InstitutionAsset, InstitutionAssetAction};
use sp_runtime::{
    traits::{CheckedSub, Hash, Saturating, Zero},
    DispatchResult, SaturatedConversion,
};

use crate::institution::types::{CloseInstitutionAction, InstitutionLifecycleStatus};
use crate::pallet::{
    AccountRegisteredCid, CidRegisteredAccount, Config, Error, Event, InstitutionAccounts,
    InstitutionPendingClose, Pallet, RegisterNonceOf, RegisterSignatureOf, UsedDeregisterNonce,
    ACTION_CLOSE, SCOPE_ACCOUNT, SCOPE_INSTITUTION,
};
use crate::traits::{
    AccountValidator, CidInstitutionVerifier, ProtectedSourceChecker, ReservedAccountGuard,
};
use crate::BalanceOf;
use admin_primitives::AdminAccountQuery;
use votingengine::types::is_fixed_governance_code;
use votingengine::InternalVoteEngine;

#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_institution_close<T: Config>(
    who: T::AccountId,
    account: T::AccountId,
    beneficiary: T::AccountId,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    issuer_cid_number: alloc::vec::Vec<u8>,
    issuer_main_account: T::AccountId,
    signer_pubkey: [u8; 32],
) -> DispatchResult {
    // 仅机构地址走本入口
    let registered =
        AccountRegisteredCid::<T>::get(&account).ok_or(Error::<T>::NotInstitutionAccount)?;

    ensure!(
        !T::ProtectedSourceChecker::is_protected(&account),
        Error::<T>::ProtectedSource
    );
    ensure!(
        T::InstitutionAsset::can_spend(&account, InstitutionAssetAction::MultisigCloseExecute,),
        Error::<T>::ProtectedSource
    );
    ensure!(beneficiary != account, Error::<T>::InvalidBeneficiary);
    ensure!(
        !T::ReservedAccountChecker::is_reserved(&beneficiary),
        Error::<T>::InvalidBeneficiary
    );
    ensure!(
        T::AccountValidator::is_valid(&beneficiary),
        Error::<T>::InvalidAccount
    );
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&beneficiary),
        Error::<T>::InvalidBeneficiary
    );

    // 校验机构账户已 Active(InstitutionAccounts 状态)
    let account_info =
        InstitutionAccounts::<T>::get(&registered.cid_number, &registered.account_name)
            .ok_or(Error::<T>::AccountNotFound)?;
    ensure!(
        matches!(account_info.status, InstitutionLifecycleStatus::Active),
        Error::<T>::AccountNotActive
    );

    // 校验发起人是【机构】管理员(resolve 统一解析到机构主账户承载的管理员集)
    let admin_account = Pallet::<T>::resolve_admin_account_for_account(&account)
        .ok_or(Error::<T>::AccountNotFound)?;
    let institution_code = Pallet::<T>::resolve_institution_code_for_account(&account)
        .ok_or(Error::<T>::AccountNotFound)?;

    // ── 硬保护(纵深防御):创世初始机构 / 治理机构 永不可注销关闭 ──
    // 创世机构本就不在 public-manage 注册(会先报 NotInstitutionAccount),
    // 这两道是显式纵深防御；创世管理员模块会封存创世内置机构。
    ensure!(
        !T::AdminAccountQuery::is_genesis_protected(&account),
        Error::<T>::CannotCloseProtectedInstitution
    );
    ensure!(
        !is_fixed_governance_code(&institution_code)
            && !primitives::institution_constraints::is_permanent_singleton_code(&institution_code,),
        Error::<T>::CannotCloseGovernance
    );

    // 作用域由被关账户角色推出:主账户=整机构(级联关全部账户),非主=只关该账户。
    let is_main = primitives::account_derive::institution_kind_by_name(
        registered.cid_number.as_slice(),
        registered.account_name.as_slice(),
    )
    .map(|kind| Pallet::<T>::is_main_account(&kind))
    .unwrap_or(false);
    let scope = if is_main {
        SCOPE_INSTITUTION
    } else {
        SCOPE_ACCOUNT
    };

    ensure!(
        T::AdminAccountQuery::is_active_account_admin(
            institution_code,
            admin_account.clone(),
            &who
        ),
        Error::<T>::PermissionDenied
    );

    // ── 注销凭证验签(注册局在 CID 注销机构/账户后签发)+ 防重放 + scope 绑定 ──
    let nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedDeregisterNonce::<T>::get(nonce_hash),
        Error::<T>::DeregisterNonceAlreadyUsed
    );
    ensure!(
        T::CidInstitutionVerifier::verify_institution_deregistration(
            scope,
            registered.cid_number.as_slice(),
            registered.account_name.as_slice(),
            &account,
            &register_nonce,
            &signature,
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
        ),
        Error::<T>::InvalidDeregisterCredential
    );

    // 拒绝并发关闭提案
    ensure!(
        !InstitutionPendingClose::<T>::contains_key(&account),
        Error::<T>::CloseAlreadyPending
    );

    let all_balance = T::Currency::free_balance(&account);
    ensure!(
        all_balance >= T::MinCloseBalance::get(),
        Error::<T>::CloseBalanceBelowMinimum
    );
    {
        let balance_u128: u128 = all_balance.saturated_into();
        let fee_u128 = onchain::calculate_onchain_fee(balance_u128);
        let fee: BalanceOf<T> = fee_u128.saturated_into();
        let transfer_amount = all_balance
            .checked_sub(&fee)
            .ok_or(Error::<T>::FeeWithdrawFailed)?;
        let ed = T::Currency::minimum_balance();
        ensure!(transfer_amount >= ed, Error::<T>::CloseTransferBelowED);
    }
    ensure!(
        T::Currency::reserved_balance(&account).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );

    let action = CloseInstitutionAction {
        account: account.clone(),
        beneficiary: beneficiary.clone(),
        proposer: who.clone(),
        scope,
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CLOSE);
    data.extend_from_slice(&action.encode());
    let proposal_id =
        <T as Config>::InternalVoteEngine::create_lifecycle_internal_proposal_with_data(
            who.clone(),
            institution_code,
            admin_account,
            alloc::vec![registered.cid_number.to_vec()],
            crate::MODULE_TAG,
            data,
        )?;
    // 提案创建成功后标记 nonce 已用,防同一注销凭证再次发起关闭。
    UsedDeregisterNonce::<T>::insert(nonce_hash, true);
    InstitutionPendingClose::<T>::insert(&account, proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCloseProposed {
        proposal_id,
        account,
        proposer: who,
        beneficiary,
    });

    Ok(())
}

/// 执行关闭:转出余额 + 物理删除账户级索引(InstitutionAccounts/CidRegisteredAccount/
/// AccountRegisteredCid)+ 关闭 admin account;机构级 Institutions 永不删除,
/// 整机构注销时状态置 Closed(墓碑,CID 号永不复用)。
pub(crate) fn execute_institution_close_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CloseInstitutionAction<T::AccountId>,
) -> DispatchResult {
    use frame_support::traits::{ExistenceRequirement, OnUnbalanced, WithdrawReasons};

    ensure!(
        T::InstitutionAsset::can_spend(
            &action.account,
            InstitutionAssetAction::MultisigCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );
    let admin_account = Pallet::<T>::resolve_admin_account_for_account(&action.account)
        .ok_or(Error::<T>::AccountNotFound)?;
    let institution_code = Pallet::<T>::resolve_institution_code_for_account(&action.account)
        .ok_or(Error::<T>::AccountNotFound)?;
    let registered =
        AccountRegisteredCid::<T>::get(&action.account).ok_or(Error::<T>::AccountNotFound)?;
    let cid_number = registered.cid_number.clone();
    let account_info = InstitutionAccounts::<T>::get(&cid_number, &registered.account_name)
        .ok_or(Error::<T>::AccountNotFound)?;
    ensure!(
        matches!(account_info.status, InstitutionLifecycleStatus::Active),
        Error::<T>::AccountNotActive
    );
    let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
        .ok_or(Error::<T>::ProposalActionNotFound)?;
    ensure!(
        votingengine::Pallet::<T>::is_callback_execution_scope(proposal_id)
            && votingengine::Pallet::<T>::is_proposal_owner(proposal_id, crate::MODULE_TAG)
            && proposal.kind == votingengine::PROPOSAL_KIND_INTERNAL
            && proposal.stage == votingengine::STAGE_INTERNAL
            && proposal.status == votingengine::STATUS_PASSED
            && proposal.internal_code == Some(institution_code)
            && proposal.account_context == Some(admin_account.clone())
            && proposal
                .subject_cid_numbers
                .iter()
                .any(|subject| subject.as_slice() == cid_number.as_slice())
            && InstitutionPendingClose::<T>::get(&action.account) == Some(proposal_id),
        Error::<T>::ProposalActionNotFound
    );
    ensure!(
        !T::AdminAccountQuery::is_genesis_protected(&action.account)
            && !is_fixed_governance_code(&institution_code)
            && !primitives::institution_constraints::is_permanent_singleton_code(&institution_code,),
        Error::<T>::CannotCloseGovernance
    );
    ensure!(
        action.beneficiary != action.account
            && !T::ReservedAccountChecker::is_reserved(&action.beneficiary)
            && T::AccountValidator::is_valid(&action.beneficiary)
            && !T::ProtectedSourceChecker::is_protected(&action.beneficiary),
        Error::<T>::InvalidBeneficiary
    );
    let is_main = primitives::account_derive::institution_kind_by_name(
        cid_number.as_slice(),
        registered.account_name.as_slice(),
    )
    .map(|kind| Pallet::<T>::is_main_account(&kind))
    .unwrap_or(false);
    let expected_scope = if is_main {
        SCOPE_INSTITUTION
    } else {
        SCOPE_ACCOUNT
    };
    ensure!(
        action.scope == expected_scope,
        Error::<T>::ProposalActionNotFound
    );

    // 整机构注销=该 cid 下全部账户;单账户注销=仅本账户。
    // 先 collect 再处理,避免边遍历 StorageDoubleMap 边删。
    let targets: alloc::vec::Vec<(crate::pallet::AccountNameOf<T>, T::AccountId)> =
        if action.scope == SCOPE_INSTITUTION {
            InstitutionAccounts::<T>::iter_prefix(&cid_number)
                .map(|(name, info)| (name, info.address))
                .collect()
        } else {
            let mut v = alloc::vec::Vec::new();
            v.push((registered.account_name.clone(), action.account.clone()));
            v
        };

    let ed = T::Currency::minimum_balance();
    let mut total_transferred: BalanceOf<T> = Zero::zero();
    let mut total_fee: BalanceOf<T> = Zero::zero();
    for (account_name, addr) in targets.iter() {
        // 执行阶段复核 reserved,保证账户能被彻底清空复用。
        ensure!(
            T::Currency::reserved_balance(addr).is_zero(),
            Error::<T>::ReservedBalanceRemaining
        );
        let bal = T::Currency::free_balance(addr);
        if !bal.is_zero() {
            let fee_u128 = onchain::calculate_onchain_fee(bal.saturated_into());
            let mut fee: BalanceOf<T> = fee_u128.saturated_into();
            // 扣费后不足 ED 的 dust 子账户整额转出、不收费,避免转账失败留残。
            let transfer_amount = match bal.checked_sub(&fee) {
                Some(rem) if rem >= ed => rem,
                _ => {
                    fee = Zero::zero();
                    bal
                }
            };
            if !fee.is_zero() {
                let fee_imbalance = T::Currency::withdraw(
                    addr,
                    fee,
                    WithdrawReasons::FEE,
                    ExistenceRequirement::AllowDeath,
                )
                .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
                T::FeeRouter::on_unbalanced(fee_imbalance);
            }
            T::Currency::transfer(
                addr,
                &action.beneficiary,
                transfer_amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::TransferFailed)?;
            total_transferred = total_transferred.saturating_add(transfer_amount);
            total_fee = total_fee.saturating_add(fee);
        }
        // 删除账户索引;历史事件/提案保留在链历史。
        InstitutionAccounts::<T>::remove(&cid_number, account_name);
        CidRegisteredAccount::<T>::remove(&cid_number, account_name);
        AccountRegisteredCid::<T>::remove(addr);
    }

    // 整机构注销才关闭机构唯一的 AdminAccount(机构消亡);
    // 单账户注销保留机构与其管理员。
    if action.scope == SCOPE_INSTITUTION {
        Pallet::<T>::close_admin_account(proposal_id, institution_code, admin_account)?;
        // 机构级墓碑:Institutions 永不删除,状态置 Closed,该 CID 号永不复用。
        crate::pallet::Institutions::<T>::mutate(&cid_number, |info| {
            if let Some(info) = info {
                info.status = crate::institution::types::InstitutionLifecycleStatus::Closed;
            }
        });
    }
    InstitutionPendingClose::<T>::remove(&action.account);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionClosed {
        proposal_id,
        account: action.account.clone(),
        beneficiary: action.beneficiary.clone(),
        amount: total_transferred,
        fee: total_fee,
    });

    Ok(())
}

// pallet::Call::propose_close 入口仍在 lib.rs 内,delegate 到 do_propose_institution_close。
