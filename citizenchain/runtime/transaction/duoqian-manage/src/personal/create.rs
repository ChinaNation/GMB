//! 个人多签创建流程实现。
//!
//! `do_propose_create_personal` 由 lib.rs 内 call_index=3 入口 delegate 调用。
//! 业务流程：
//! 1. 校验发起人未被保护、账户名非空、admin/threshold 合法、余额充足
//! 2. 派生 `derive_personal_duoqian_address(creator, account_name)` —— 地址只依赖
//!    creator 与 account_name,与管理员列表无关,所以未来换管理员地址不变
//! 3. 同事务内：
//!    - 写 Pending DuoqianAccounts 占位
//!    - 写 PersonalDuoqianInfo 反向索引
//!    - 调投票引擎 create_pending_subject_internal_proposal_with_snapshot_data
//!    - 调 admins-change 写 Pending 主体
//! 4. 从投票引擎回读 expires_at,发射 PersonalDuoqianProposed 事件

extern crate alloc;

use codec::Encode;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::{Get, ReservableCurrency},
};
use sp_runtime::DispatchResult;

use crate::common::account_to_institution_id;
use crate::pallet::{
    self, AccountNameOf, Config, DuoqianAccounts, DuoqianAdminsOf, Error, Event, Pallet,
    PendingPersonalCreate, PersonalDuoqianInfo, ACTION_CREATE_PERSONAL,
};
use crate::BalanceOf;
use crate::personal::types::{CreateDuoqianAction, DuoqianAccount, DuoqianStatus, PersonalDuoqianMeta};
use crate::traits::{
    DuoqianAddressValidator, DuoqianReservedAddressChecker, ProtectedSourceChecker,
};
use voting_engine::InternalVoteEngine;

pub(crate) fn do_propose_create_personal<T: Config>(
    who: T::AccountId,
    account_name: AccountNameOf<T>,
    admin_count: u32,
    duoqian_admins: DuoqianAdminsOf<T>,
    threshold: u32,
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
    // 中文注释:admin/threshold/排重/发起人是管理员等参数共用 ensure_admin_config 校验。
    Pallet::<T>::ensure_admin_config(&who, admin_count, &duoqian_admins, threshold)?;

    // 预检查余额(amount + fee + ED) — 共用 helper
    let (reserve_total, _fee) =
        crate::common::ensure_proposer_can_afford::<T>(&who, amount)?;

    let duoqian_address = Pallet::<T>::derive_personal_duoqian_address(&who, account_name.as_slice())?;
    ensure!(
        !DuoqianAccounts::<T>::contains_key(&duoqian_address),
        Error::<T>::PersonalDuoqianAlreadyExists
    );
    ensure!(
        !T::ReservedAddressChecker::is_reserved(&duoqian_address),
        Error::<T>::AddressReserved
    );
    ensure!(
        T::AddressValidator::is_valid(&duoqian_address),
        Error::<T>::InvalidAddress
    );
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&duoqian_address),
        Error::<T>::ProtectedSource
    );

    let now = <frame_system::Pallet<T>>::block_number();
    let institution = account_to_institution_id(&duoqian_address);
    let org = voting_engine::internal_vote::ORG_DUOQIAN;
    let action = CreateDuoqianAction {
        duoqian_address: duoqian_address.clone(),
        proposer: who.clone(),
        admin_count,
        threshold,
        amount,
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CREATE_PERSONAL);
    data.extend_from_slice(&action.encode());

    let proposal_id = with_transaction(|| {
        // 中文注释:统一资金模型(2026-05-03):reserve amount + fee,
        // 投票通过后 unreserve→transfer→withdraw,投票否决/失败 unreserve。
        if T::Currency::reserve(&who, reserve_total).is_err() {
            return TransactionOutcome::Rollback(Err(Error::<T>::ReserveFailed.into()));
        }
        DuoqianAccounts::<T>::insert(
            &duoqian_address,
            DuoqianAccount {
                admin_count,
                threshold,
                duoqian_admins: duoqian_admins.clone(),
                creator: who.clone(),
                created_at: now,
                status: DuoqianStatus::Pending,
            },
        );
        PersonalDuoqianInfo::<T>::insert(
            &duoqian_address,
            PersonalDuoqianMeta {
                creator: who.clone(),
                account_name: account_name.clone(),
            },
        );
        // 中文注释:创建提案需全员管理员通过(2026-05-03 整改)。
        // admins-change 主体里 threshold 字段保存用户自定义 m-of-n,
        // 用于激活后日常治理(转账等),不影响此处投票阈值。
        let create_threshold = duoqian_admins.len() as u32;
        let proposal_id = match <T as Config>::InternalVoteEngine::create_pending_subject_internal_proposal_with_snapshot_data(
            who.clone(),
            org,
            institution,
            duoqian_admins.iter().cloned().collect(),
            create_threshold,
            crate::MODULE_TAG,
            data,
        ) {
            Ok(proposal_id) => proposal_id,
            Err(err) => return TransactionOutcome::Rollback(Err(err)),
        };
        // 备份 action 用于 reserve 释放(否决/失败终态时取出释放)。
        PendingPersonalCreate::<T>::insert(proposal_id, &action);
        if let Err(err) = Pallet::<T>::create_pending_admin_subject_for_proposal(
            proposal_id,
            institution,
            admins_change::AdminSubjectKind::PersonalDuoqian,
            &duoqian_admins,
            threshold,
            &who,
        ) {
            return TransactionOutcome::Rollback(Err(err));
        }
        TransactionOutcome::Commit(Ok(proposal_id))
    })?;

    let expires_at = voting_engine::Pallet::<T>::proposals(proposal_id)
        .map(|p| p.end)
        .ok_or(Error::<T>::VoteEngineError)?;

    Pallet::<T>::deposit_event(Event::<T>::PersonalDuoqianProposed {
        proposal_id,
        duoqian_address,
        proposer: who,
        account_name,
        admins: duoqian_admins,
        admin_count,
        threshold,
        amount,
        expires_at,
    });

    Ok(())
}

// 防 dead_code: pallet 模块声明保留。
#[allow(dead_code)]
fn _force_pallet_use() {
    let _ = pallet::ACTION_CREATE_PERSONAL;
}
