//! 机构创建流程实现。
//!
//! 机构最少必须有 2 个账户(主账户 + 费用账户),所以原 `propose_create`
//! (单账户机构,call_index=0) 已于 2026-05-03 删除。
//!
//! 唯一入口: `do_propose_create_institution`(call_index=5,ADR-008 step2b)
//! - 一次创建机构主账户 / 费用账户 / 自定义账户列表
//! - 凭证带 (province, signer_admin_pubkey) 双层验签
//! - 资金模型: 发起时 reserve, 通过后划转, 拒绝后 unreserve

extern crate alloc;

use codec::Encode;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::ReservableCurrency,
};
use sp_runtime::{traits::Hash, DispatchResult};

use crate::common::sfid_id_to_institution_id;
use crate::institution::accounts::{
    account_names_payload_from_initial_accounts, validate_initial_accounts,
};
use crate::institution::types::{
    CreateInstitutionAction, InstitutionAccountInfo, InstitutionInfo, InstitutionLifecycleStatus,
};
use crate::pallet::{
    AccountNameOf, AddressRegisteredSfid, Config, DuoqianAccounts, DuoqianAdminsOf, Error, Event,
    InstitutionAccounts, InstitutionInitialAccountsOf, Institutions, Pallet,
    PendingInstitutionCreate, RegisterNonceOf, RegisterSignatureOf, SfidIdOf, SfidRegisteredAddress,
    UsedRegisterNonce, ACTION_CREATE_INSTITUTION,
};
use crate::personal::types::{DuoqianAccount, DuoqianStatus};
use crate::traits::{ProtectedSourceChecker, SfidInstitutionVerifier};
use crate::RegisteredInstitution;
use voting_engine::InternalVoteEngine;

/// 机构整体创建提案 (call_index=5,ADR-008 step2b)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_create_institution<T: Config>(
    who: T::AccountId,
    sfid_id: SfidIdOf<T>,
    institution_name: AccountNameOf<T>,
    accounts: InstitutionInitialAccountsOf<T>,
    admin_count: u32,
    duoqian_admins: DuoqianAdminsOf<T>,
    threshold: u32,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    province: alloc::vec::Vec<u8>,
    signer_admin_pubkey: [u8; 32],
) -> DispatchResult {
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&who),
        Error::<T>::ProtectedSource
    );
    ensure!(!sfid_id.is_empty(), Error::<T>::EmptySfidId);
    ensure!(!institution_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!province.is_empty(), Error::<T>::EmptyProvince);
    ensure!(
        !Institutions::<T>::contains_key(&sfid_id),
        Error::<T>::InstitutionAlreadyExists
    );
    Pallet::<T>::ensure_admin_config(&who, admin_count, &duoqian_admins, threshold)?;

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );
    let account_name_payload = account_names_payload_from_initial_accounts::<T>(&accounts)?;
    ensure!(
        T::SfidInstitutionVerifier::verify_institution_registration(
            sfid_id.as_slice(),
            &institution_name,
            &account_name_payload,
            &register_nonce,
            &signature,
            province.as_slice(),
            &signer_admin_pubkey,
        ),
        Error::<T>::InvalidSfidInstitutionSignature
    );

    let (created_accounts, main_address, fee_address, initial_total) =
        validate_initial_accounts::<T>(&sfid_id, &accounts)?;
    // 共用余额预检查 helper(2026-05-03):amount + fee + ED 必须够。
    let (reserve_total, fee) =
        crate::common::ensure_proposer_can_afford::<T>(&who, initial_total)?;

    let now = <frame_system::Pallet<T>>::block_number();
    // 中文注释:机构治理索引直接由 sfid_id 派生(2026-05-03 整改),
    // 与 NRC/PRC/PRB 的 shenfen_id_to_fixed48 算法一致,不再绕道 main_address。
    let institution = sfid_id_to_institution_id(sfid_id.as_slice())
        .ok_or(Error::<T>::EmptySfidId)?;
    let org = voting_engine::internal_vote::ORG_DUOQIAN;
    let action = CreateInstitutionAction {
        sfid_id: sfid_id.clone(),
        institution_name: institution_name.clone(),
        main_address: main_address.clone(),
        fee_address: fee_address.clone(),
        proposer: who.clone(),
        admin_count,
        threshold,
        duoqian_admins: duoqian_admins.clone(),
        accounts: created_accounts.clone(),
        initial_total,
        fee,
        reserve_total,
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CREATE_INSTITUTION);
    data.extend_from_slice(&action.encode());

    let proposal_id = with_transaction(|| {
        if T::Currency::reserve(&who, reserve_total).is_err() {
            return TransactionOutcome::Rollback(Err(Error::<T>::ReserveFailed.into()));
        }
        Institutions::<T>::insert(
            &sfid_id,
            InstitutionInfo {
                institution_name: institution_name.clone(),
                main_address: main_address.clone(),
                fee_address: fee_address.clone(),
                admin_count,
                threshold,
                duoqian_admins: duoqian_admins.clone(),
                creator: who.clone(),
                created_at: now,
                status: InstitutionLifecycleStatus::Pending,
                account_count: created_accounts.len() as u32,
            },
        );

        for account in created_accounts.iter() {
            InstitutionAccounts::<T>::insert(
                &sfid_id,
                &account.account_name,
                InstitutionAccountInfo {
                    address: account.address.clone(),
                    initial_balance: account.amount,
                    status: InstitutionLifecycleStatus::Pending,
                    is_default: account.is_default,
                    created_at: now,
                },
            );
            SfidRegisteredAddress::<T>::insert(&sfid_id, &account.account_name, &account.address);
            AddressRegisteredSfid::<T>::insert(
                &account.address,
                RegisteredInstitution {
                    sfid_id: sfid_id.clone(),
                    account_name: account.account_name.clone(),
                },
            );
        }

        DuoqianAccounts::<T>::insert(
            &main_address,
            DuoqianAccount {
                admin_count,
                threshold,
                duoqian_admins: duoqian_admins.clone(),
                creator: who.clone(),
                created_at: now,
                status: DuoqianStatus::Pending,
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
        PendingInstitutionCreate::<T>::insert(proposal_id, &action);
        UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
        if let Err(err) = Pallet::<T>::create_pending_admin_subject_for_proposal(
            proposal_id,
            institution,
            admins_change::AdminSubjectKind::SfidInstitution,
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

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCreateProposed {
        proposal_id,
        sfid_id,
        institution_name,
        main_address,
        proposer: who,
        accounts: created_accounts,
        admins: duoqian_admins,
        admin_count,
        threshold,
        initial_total,
        reserve_total,
        expires_at,
    });

    Ok(())
}
