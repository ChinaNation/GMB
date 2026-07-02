//! 机构创建流程实现。
//!
//! 机构最少必须有 2 个账户(主账户 + 费用账户)。
//!
//! 唯一入口: `do_propose_create_private_institution`(call_index=5)
//! - 一次创建机构主账户 / 费用账户 / 自定义账户列表
//! - 凭证带签发机构 CID、签发机构主账户和签发管理员公钥
//! - 资金模型: 注册局交易成功即划转初始余额并激活机构与管理员集合

extern crate alloc;

use crate::institution::accounts::{
    account_names_payload_from_initial_accounts, validate_initial_accounts,
};
use crate::institution::types::{
    InstitutionAccountInfo, InstitutionInfo, InstitutionLifecycleStatus,
};
use crate::pallet::{
    AccountNameOf, AccountRegisteredCid, AdminProfilesOf, CidNumberOf, CidRegisteredAccount,
    Config, Error, Event, InstitutionAccounts, InstitutionInitialAccountsOf, Institutions, Pallet,
    RegisterNonceOf, RegisterSignatureOf, UsedRegisterNonce,
};
use crate::traits::{
    CidInstitutionVerifier, InstitutionCidQuery, ProtectedSourceChecker, RegistryAuthority,
};
use crate::RegisteredInstitution;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReasons},
};
use sp_runtime::{
    traits::{Hash, Zero},
    DispatchResult,
};
use votingengine::types::InstitutionCode;

/// 机构注册创建(call_index=5)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_create_private_institution<T: Config>(
    who: T::AccountId,
    cid_number: CidNumberOf<T>,
    cid_full_name: AccountNameOf<T>,
    _cid_short_name: AccountNameOf<T>,
    accounts: InstitutionInitialAccountsOf<T>,
    institution_code: InstitutionCode,
    admins_len: u32,
    admins: AdminProfilesOf<T>,
    threshold: u32,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    issuer_cid_number: alloc::vec::Vec<u8>,
    issuer_main_account: T::AccountId,
    signer_pubkey: [u8; 32],
    scope_province_name: alloc::vec::Vec<u8>,
    scope_city_name: alloc::vec::Vec<u8>,
) -> DispatchResult {
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&who),
        Error::<T>::ProtectedSource
    );
    ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
    // private-manage 只管理私权机构;私权机构名称留在注册端业务库,
    // 链上只保存 CID 号、账户、管理员与生命周期状态。
    let (stored_full_name, stored_short_name) =
        (AccountNameOf::<T>::default(), AccountNameOf::<T>::default());
    ensure!(
        !issuer_cid_number.is_empty(),
        Error::<T>::EmptyIssuerCidNumber
    );
    ensure!(
        !scope_province_name.is_empty(),
        Error::<T>::EmptyScopeProvinceName
    );
    ensure!(
        !Institutions::<T>::contains_key(&cid_number),
        Error::<T>::InstitutionAlreadyExists
    );
    ensure!(
        !T::SiblingInstitutionQuery::cid_exists(&cid_number),
        Error::<T>::InstitutionAlreadyExists
    );
    Pallet::<T>::ensure_admin_config(admins_len, &admins, threshold)?;
    Pallet::<T>::ensure_lifecycle_institution_code(&institution_code)?;

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );
    let account_name_payload = account_names_payload_from_initial_accounts::<T>(&accounts)?;
    ensure!(
        T::CidInstitutionVerifier::verify_institution_registration(
            cid_number.as_slice(),
            &cid_full_name,
            &account_name_payload,
            &register_nonce,
            &signature,
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
        ),
        Error::<T>::InvalidCidInstitutionSignature
    );
    ensure!(
        T::RegistryAuthority::can_register_institution(
            &who,
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &signer_pubkey,
            cid_number.as_slice(),
            institution_code,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
        ),
        Error::<T>::RegistryAuthorityDenied
    );

    let (created_accounts, main_account, _fee_account, initial_total) =
        validate_initial_accounts::<T>(&cid_number, &accounts)?;
    // 共用余额预检查 helper:amount + fee + ED 必须够。
    let (_total_with_fee, fee) =
        crate::common::ensure_proposer_can_afford::<T>(&who, initial_total)?;

    let now = <frame_system::Pallet<T>>::block_number();
    // 管理员更换与内部投票直接使用机构主账户。
    let institution = main_account.clone();

    with_transaction(|| {
        if !fee.is_zero() {
            let fee_imbalance = match T::Currency::withdraw(
                &who,
                fee,
                WithdrawReasons::FEE,
                ExistenceRequirement::KeepAlive,
            ) {
                Ok(imbalance) => imbalance,
                Err(_) => {
                    return TransactionOutcome::Rollback(Err(Error::<T>::FeeWithdrawFailed.into()))
                }
            };
            T::FeeRouter::on_unbalanced(fee_imbalance);
        }

        for account in created_accounts.iter() {
            if T::Currency::transfer(
                &who,
                &account.address,
                account.amount,
                ExistenceRequirement::KeepAlive,
            )
            .is_err()
            {
                return TransactionOutcome::Rollback(Err(Error::<T>::TransferFailed.into()));
            }
        }

        Institutions::<T>::insert(
            &cid_number,
            InstitutionInfo {
                cid_full_name: stored_full_name.clone(),
                cid_short_name: stored_short_name.clone(),
                institution_code,
                created_at: now,
                status: InstitutionLifecycleStatus::Active,
            },
        );

        for account in created_accounts.iter() {
            InstitutionAccounts::<T>::insert(
                &cid_number,
                &account.account_name,
                InstitutionAccountInfo {
                    address: account.address.clone(),
                    initial_balance: account.amount,
                    status: InstitutionLifecycleStatus::Active,
                    is_default: account.is_default,
                    created_at: now,
                },
            );
            CidRegisteredAccount::<T>::insert(&cid_number, &account.account_name, &account.address);
            AccountRegisteredCid::<T>::insert(
                &account.address,
                RegisteredInstitution {
                    cid_number: cid_number.clone(),
                    account_name: account.account_name.clone(),
                },
            );
        }

        // 注册局创建机构时直接提交目标机构管理员合集;交易成功即写 Active。
        if let Err(err) = Pallet::<T>::set_active_admin_account_direct(
            institution_code,
            institution.clone(),
            &admins,
            threshold,
            &who,
        ) {
            return TransactionOutcome::Rollback(Err(err));
        }
        UsedRegisterNonce::<T>::insert(register_nonce_hash, true);
        TransactionOutcome::Commit(Ok(()))
    })?;

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCreated {
        cid_number,
        main_account,
        account_count: created_accounts.len() as u32,
        initial_total,
        fee,
    });

    Ok(())
}
