//! 机构创建流程实现。
//!
//! 机构最少必须有 2 个账户(主账户 + 费用账户),所以原 `propose_create`
//! (单账户机构,call_index=0) 已于 2026-05-03 删除。
//!
//! 唯一入口: `do_propose_create_institution`(call_index=5,ADR-008 step2b)
//! - 一次创建机构主账户 / 费用账户 / 自定义账户列表
//! - 凭证带签发机构 SFID、签发机构主账户和签发管理员公钥
//! - 资金模型: 发起时 reserve, 通过后划转, 拒绝后 unreserve

extern crate alloc;

use crate::institution::accounts::{
    account_names_payload_from_initial_accounts, validate_initial_accounts,
};
use crate::institution::types::{
    CreateInstitutionAction, InstitutionAccountInfo, InstitutionInfo, InstitutionLifecycleStatus,
};
use crate::pallet::{
    AccountNameOf, AccountRegisteredSfid, AdminsOf, Config, Error, Event, InstitutionAccounts,
    InstitutionInitialAccountsOf, Institutions, Pallet, PendingInstitutionCreate, RegisterNonceOf,
    RegisterSignatureOf, SfidNumberOf, SfidRegisteredAccount, UsedRegisterNonce,
    ACTION_CREATE_INSTITUTION,
};
use crate::traits::{ProtectedSourceChecker, SfidInstitutionVerifier};
use crate::RegisteredInstitution;
use codec::Encode;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::ReservableCurrency,
};
use sp_runtime::{traits::Hash, DispatchResult};
use votingengine::types::{ORG_OTH, ORG_PUP};
use votingengine::InternalVoteEngine;

/// 机构整体创建提案 (call_index=5,ADR-008 step2b)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_create_institution<T: Config>(
    who: T::AccountId,
    sfid_number: SfidNumberOf<T>,
    sfid_full_name: AccountNameOf<T>,
    accounts: InstitutionInitialAccountsOf<T>,
    org: u8,
    admins_len: u32,
    admins: AdminsOf<T>,
    threshold: u32,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    issuer_sfid_number: alloc::vec::Vec<u8>,
    issuer_main_account: T::AccountId,
    signer_pubkey: [u8; 32],
    scope_province_name: alloc::vec::Vec<u8>,
    scope_city_name: alloc::vec::Vec<u8>,
) -> DispatchResult {
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&who),
        Error::<T>::ProtectedSource
    );
    ensure!(!sfid_number.is_empty(), Error::<T>::EmptySfidNumber);
    ensure!(!sfid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(
        !issuer_sfid_number.is_empty(),
        Error::<T>::EmptyIssuerSfidNumber
    );
    ensure!(
        !scope_province_name.is_empty(),
        Error::<T>::EmptyScopeProvinceName
    );
    ensure!(
        !Institutions::<T>::contains_key(&sfid_number),
        Error::<T>::InstitutionAlreadyExists
    );
    Pallet::<T>::ensure_admin_config(&who, admins_len, &admins, threshold)?;
    ensure!(matches!(org, ORG_PUP | ORG_OTH), Error::<T>::InvalidOrg);

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );
    let account_name_payload = account_names_payload_from_initial_accounts::<T>(&accounts)?;
    ensure!(
        T::SfidInstitutionVerifier::verify_institution_registration(
            sfid_number.as_slice(),
            &sfid_full_name,
            &account_name_payload,
            &register_nonce,
            &signature,
            issuer_sfid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
        ),
        Error::<T>::InvalidSfidInstitutionSignature
    );

    let (created_accounts, main_account, fee_account, initial_total) =
        validate_initial_accounts::<T>(&sfid_number, &accounts)?;
    // 共用余额预检查 helper(2026-05-03):amount + fee + ED 必须够。
    let (reserve_total, fee) = crate::common::ensure_proposer_can_afford::<T>(&who, initial_total)?;

    let now = <frame_system::Pallet<T>>::block_number();
    // 中文注释：管理员更换与内部投票直接使用机构主账户多签地址。
    let institution = main_account.clone();
    let org = org;
    let action = CreateInstitutionAction {
        sfid_number: sfid_number.clone(),
        sfid_full_name: sfid_full_name.clone(),
        main_account: main_account.clone(),
        fee_account: fee_account.clone(),
        proposer: who.clone(),
        org,
        admins_len,
        threshold,
        admins: admins.clone(),
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
                sfid_full_name: sfid_full_name.clone(),
                main_account: main_account.clone(),
                fee_account: fee_account.clone(),
                org,
                admins_len,
                threshold,
                admins: admins.clone(),
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
            SfidRegisteredAccount::<T>::insert(
                &sfid_number,
                &account.account_name,
                &account.address,
            );
            AccountRegisteredSfid::<T>::insert(
                &account.address,
                RegisteredInstitution {
                    sfid_number: sfid_number.clone(),
                    account_name: account.account_name.clone(),
                },
            );
        }

        // B 阶段(personal-manage 拆分)起,DuoqianAccounts mirror 已删除;
        // 机构主账户的管理员配置真源在 admins-change::AdminAccounts[main_account 账户]；
        // 动态阈值真源在 internal-vote，duoqian-transfer 通过查询 trait 合并读取。

        // 中文注释:threshold 是账户激活后的动态阈值配置；
        // 本次注册投票的全员通过阈值由投票引擎根据管理员快照生成。
        let proposal_id = match <T as Config>::InternalVoteEngine::create_registered_account_create_proposal_with_data(
            who.clone(),
            org,
            institution.clone(),
            admins.iter().cloned().collect(),
            threshold,
            crate::MODULE_TAG,
            data,
        ) {
            Ok(proposal_id) => proposal_id,
            Err(err) => return TransactionOutcome::Rollback(Err(err)),
        };
        PendingInstitutionCreate::<T>::insert(proposal_id, &action);
        UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
        if let Err(err) = Pallet::<T>::create_pending_admin_account_for_proposal(
            proposal_id,
            org,
            institution.clone(),
            admins_change::AdminAccountKind::InstitutionAccount,
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

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCreateProposed {
        proposal_id,
        sfid_number,
        sfid_full_name,
        main_account,
        proposer: who,
        accounts: created_accounts,
        admins: admins,
        org,
        admins_len,
        threshold,
        initial_total,
        reserve_total,
        expires_at,
    });

    Ok(())
}
