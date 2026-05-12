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

use crate::institution::accounts::{
    account_names_payload_from_initial_accounts, validate_initial_accounts,
};
use crate::institution::types::{
    CreateInstitutionAction, InstitutionAccountInfo, InstitutionInfo, InstitutionLifecycleStatus,
};
use crate::pallet::{
    AccountNameOf, AddressRegisteredSfid, Config, DuoqianAdminsOf, Error, Event,
    InstitutionAccounts, InstitutionInitialAccountsOf, Institutions, Pallet,
    PendingInstitutionCreate, RegisterNonceOf, RegisterSignatureOf, SfidNumberOf,
    SfidRegisteredAddress, UsedRegisterNonce, ACTION_CREATE_INSTITUTION,
};
use crate::traits::{ProtectedSourceChecker, SfidInstitutionVerifier};
use crate::RegisteredInstitution;
use codec::Encode;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::ReservableCurrency,
};
use primitives::derive::subject_id_from_institution_account;
use sp_runtime::{traits::Hash, DispatchResult};
use votingengine::types::{ORG_OTH, ORG_PUP};
use votingengine::InternalVoteEngine;

/// 机构整体创建提案 (call_index=5,ADR-008 step2b)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_create_institution<T: Config>(
    who: T::AccountId,
    sfid_number: SfidNumberOf<T>,
    institution_name: AccountNameOf<T>,
    accounts: InstitutionInitialAccountsOf<T>,
    admin_org: u8,
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
    ensure!(!sfid_number.is_empty(), Error::<T>::EmptySfidNumber);
    ensure!(!institution_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!province.is_empty(), Error::<T>::EmptyProvince);
    ensure!(
        !Institutions::<T>::contains_key(&sfid_number),
        Error::<T>::InstitutionAlreadyExists
    );
    Pallet::<T>::ensure_admin_config(&who, admin_count, &duoqian_admins, threshold)?;
    ensure!(
        matches!(admin_org, ORG_PUP | ORG_OTH),
        Error::<T>::InvalidAdminOrg
    );

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );
    let account_name_payload = account_names_payload_from_initial_accounts::<T>(&accounts)?;
    ensure!(
        T::SfidInstitutionVerifier::verify_institution_registration(
            sfid_number.as_slice(),
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
        validate_initial_accounts::<T>(&sfid_number, &accounts)?;
    // 共用余额预检查 helper(2026-05-03):amount + fee + ED 必须够。
    let (reserve_total, fee) = crate::common::ensure_proposer_can_afford::<T>(&who, initial_total)?;

    let now = <frame_system::Pallet<T>>::block_number();
    // 中文注释:admins-change 只接受账户级主体。SFID 机构号只负责归属/检索,
    // 管理员更换与内部投票均使用主账户地址派生的 InstitutionAccount SubjectId。
    let institution = subject_id_from_institution_account(&main_address);
    let org = admin_org;
    let action = CreateInstitutionAction {
        sfid_number: sfid_number.clone(),
        institution_name: institution_name.clone(),
        main_address: main_address.clone(),
        fee_address: fee_address.clone(),
        proposer: who.clone(),
        admin_org,
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
            &sfid_number,
            InstitutionInfo {
                institution_name: institution_name.clone(),
                main_address: main_address.clone(),
                fee_address: fee_address.clone(),
                admin_org,
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
                &sfid_number,
                &account.account_name,
                InstitutionAccountInfo {
                    address: account.address.clone(),
                    initial_balance: account.amount,
                    status: InstitutionLifecycleStatus::Pending,
                    is_default: account.is_default,
                    created_at: now,
                },
            );
            SfidRegisteredAddress::<T>::insert(
                &sfid_number,
                &account.account_name,
                &account.address,
            );
            AddressRegisteredSfid::<T>::insert(
                &account.address,
                RegisteredInstitution {
                    sfid_number: sfid_number.clone(),
                    account_name: account.account_name.clone(),
                },
            );
        }

        // B 阶段(personal-manage 拆分)起,DuoqianAccounts mirror 已删除;
        // 机构主账户的管理员配置真源在 admins-change::Subjects[main_address 派生主体]；
        // 动态阈值真源在 internal-vote，duoqian-transfer 通过查询 trait 合并读取。

        // 中文注释:threshold 是账户激活后的动态阈值配置；
        // 本次注册投票的全员通过阈值由投票引擎根据管理员快照生成。
        let proposal_id = match <T as Config>::InternalVoteEngine::create_registered_subject_create_proposal_with_data(
            who.clone(),
            org,
            institution,
            duoqian_admins.iter().cloned().collect(),
            threshold,
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
            org,
            institution,
            admins_change::AdminSubjectKind::InstitutionAccount,
            &duoqian_admins,
            &who,
        ) {
            return TransactionOutcome::Rollback(Err(err));
        }
        TransactionOutcome::Commit(Ok(proposal_id))
    })?;

    let expires_at = votingengine::Pallet::<T>::proposals(proposal_id)
        .map(|p| p.end)
        .ok_or(Error::<T>::VoteEngineError)?;

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCreateProposed {
        proposal_id,
        sfid_number,
        institution_name,
        main_address,
        proposer: who,
        accounts: created_accounts,
        admins: duoqian_admins,
        admin_org,
        admin_count,
        threshold,
        initial_total,
        reserve_total,
        expires_at,
    });

    Ok(())
}
