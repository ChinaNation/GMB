//! 个人多签创建流程实现。
//!
//! `do_propose_create` 由 lib.rs 内 call_index=0 入口 delegate 调用。
//! 业务流程：
//! 1. 校验发起人未被保护、账户名非空、管理员集合合法、余额充足
//! 2. 派生 `derive_personal_duoqian_account(creator, account_name)` —— 地址只依赖
//!    creator 与 account_name,与管理员列表无关,所以未来换管理员地址不变
//! 3. 同事务内：
//!    - 写 Pending PersonalDuoqians 占位
//!    - 调投票引擎 create_registered_account_create_proposal_with_data
//!    - 调 admins-change 写 Pending 账户
//! 4. 从投票引擎回读 expires_at,发射 PersonalDuoqianProposed 事件

extern crate alloc;

use codec::Encode;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::{Get, ReservableCurrency},
};
use sp_runtime::DispatchResult;

use crate::pallet::{
    AccountNameOf, Config, AdminsOf, Error, Event, Pallet, PendingPersonalCreate,
    PersonalDuoqians,
};
use crate::types::{CreateDuoqianAction, DuoqianAccount, DuoqianStatus};
use crate::BalanceOf;
use crate::ACTION_CREATE;
use primitives::multisig::{
    DuoqianAccountValidator, DuoqianReservedAccountChecker, ProtectedSourceChecker,
};
use votingengine::InternalVoteEngine;

pub(crate) fn do_propose_create<T: Config>(
    who: T::AccountId,
    account_name: AccountNameOf<T>,
    admins: AdminsOf<T>,
    regular_threshold: u32,
    amount: BalanceOf<T>,
) -> DispatchResult {
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&who),
        Error::<T>::ProtectedSource
    );
    ensure!(!account_name.is_empty(), Error::<T>::EmptyPersonalName);
    ensure!(
        amount >= T::MinCreateAmount::get(),
        Error::<T>::CreateAmountBelowMinimum
    );
    Pallet::<T>::ensure_admin_config(&who, &admins, regular_threshold)?;
    let admins_len = admins.len() as u32;

    let (reserve_total, fee) = Pallet::<T>::ensure_proposer_can_afford(&who, amount)?;

    let duoqian_account =
        Pallet::<T>::derive_personal_duoqian_account(&who, account_name.as_slice())?;
    ensure!(
        !PersonalDuoqians::<T>::contains_key(&duoqian_account),
        Error::<T>::PersonalDuoqianAlreadyExists
    );
    ensure!(
        !T::ReservedAccountChecker::is_reserved(&duoqian_account),
        Error::<T>::AccountReserved
    );
    ensure!(
        T::AccountValidator::is_valid(&duoqian_account),
        Error::<T>::InvalidAccount
    );
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&duoqian_account),
        Error::<T>::ProtectedSource
    );

    let now = <frame_system::Pallet<T>>::block_number();
    let institution = duoqian_account.clone();
    let org = votingengine::types::ORG_REN;
    let action = CreateDuoqianAction {
        duoqian_account: duoqian_account.clone(),
        proposer: who.clone(),
        amount,
        fee,
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CREATE);
    data.extend_from_slice(&action.encode());

    let proposal_id = with_transaction(|| {
        if T::Currency::reserve(&who, reserve_total).is_err() {
            return TransactionOutcome::Rollback(Err(Error::<T>::ReserveFailed.into()));
        }
        PersonalDuoqians::<T>::insert(
            &duoqian_account,
            DuoqianAccount {
                creator: who.clone(),
                account_name: account_name.clone(),
                created_at: now,
                status: DuoqianStatus::Pending,
            },
        );
        // 中文注释：regular_threshold 是账户激活后的动态阈值配置；
        // 本次注册投票的全员通过阈值由投票引擎根据管理员快照生成。
        let proposal_id = match <T as Config>::InternalVoteEngine::create_registered_account_create_proposal_with_data(
            who.clone(),
            org,
            institution.clone(),
            admins.iter().cloned().collect(),
            regular_threshold,
            crate::MODULE_TAG,
            data,
        ) {
            Ok(proposal_id) => proposal_id,
            Err(err) => return TransactionOutcome::Rollback(Err(err)),
        };
        PendingPersonalCreate::<T>::insert(proposal_id, &action);
        if let Err(err) = Pallet::<T>::create_pending_admin_account_for_proposal(
            proposal_id,
            institution.clone(),
            admins_change::AdminAccountKind::PersonalDuoqian,
            &admins,
            &who,
        ) {
            return TransactionOutcome::Rollback(Err(err));
        }
        TransactionOutcome::Commit(Ok(proposal_id))
    })?;

    let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
        .map(|p| p.end)
        .ok_or(Error::<T>::VoteEngineError)?;

    Pallet::<T>::deposit_event(Event::<T>::PersonalDuoqianProposed {
        proposal_id,
        duoqian_account,
        proposer: who,
        account_name,
        admins: admins,
        admins_len,
        threshold: regular_threshold,
        amount,
        fee,
        expires_at,
    });

    Ok(())
}
