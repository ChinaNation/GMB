//! 机构创建流程实现。
//!
//! 机构必须完整携带其 CID 类型要求的协议账户；普通私权机构至少包含主账户和费用账户。
//!
//! 唯一入口: `do_propose_create_private_institution`(call_index=5)
//! - 一次创建该机构全部必需协议账户及可选自定义账户
//! - 凭证带 actor CID 和签名管理员公钥，不以任何机构账户充当授权身份
//! - 资金模型: 注册局交易成功即划转初始余额并激活机构与管理员集合

extern crate alloc;

use codec::Encode;
use entity_primitives::InstitutionMultisigQuery;

use crate::institution::accounts::{
    account_names_payload_from_initial_accounts, validate_initial_accounts,
};
use crate::institution::types::{InstitutionAccountInfo, InstitutionInfo};
use crate::pallet::{
    AccountNameOf, AccountRegisteredCid, CidNumberOf, Config, Error, Event, InstitutionAccounts,
    InstitutionInitialAccountsOf, Institutions, Pallet, RegisterNonceOf, RegisterSignatureOf,
    UsedRegisterNonce,
};
use crate::traits::{
    CidInstitutionVerifier, InstitutionCidQuery, ProtectedSourceChecker, RegistryAuthority,
};
use crate::RegisteredInstitution;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
    traits::{Currency, ExistenceRequirement},
};
use primitives::institution_asset::{InstitutionAsset, InstitutionAssetAction};
use sp_runtime::{
    traits::{CheckedAdd, Hash, SaturatedConversion, Zero},
    DispatchResult,
};
use votingengine::types::InstitutionCode;

/// 机构注册创建(call_index=5)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_create_private_institution<T: Config>(
    who: T::AccountId,
    cid_number: CidNumberOf<T>,
    cid_full_name: AccountNameOf<T>,
    cid_short_name: AccountNameOf<T>,
    town_code: AccountNameOf<T>,
    legal_representative_name: AccountNameOf<T>,
    legal_representative_cid_number: CidNumberOf<T>,
    legal_representative_account: T::AccountId,
    accounts: InstitutionInitialAccountsOf<T>,
    funding_account: Option<T::AccountId>,
    institution_code: InstitutionCode,
    roles: crate::institution::role::InstitutionRolesOf<T>,
    assignments: crate::institution::role::InstitutionAdminAssignmentsOf<T>,
    threshold: u32,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    actor_cid_number: alloc::vec::Vec<u8>,
    credential_signer_pubkey: [u8; 32],
    scope_province_name: alloc::vec::Vec<u8>,
    scope_city_name: alloc::vec::Vec<u8>,
) -> DispatchResult {
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&who),
        Error::<T>::ProtectedSource
    );
    ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
    // CID 号全量校验单源 primitives::cid;机构码必须是私权法人/非法人家族且与参数一致。
    let parts = primitives::cid::number::parse_cid_number_parts_bytes(cid_number.as_slice())
        .map_err(|_| Error::<T>::InvalidCidNumber)?;
    ensure!(
        primitives::cid::code::is_private_legal_code(&parts.institution)
            || primitives::cid::code::is_unincorporated_code(&parts.institution),
        Error::<T>::InvalidCidNumber
    );
    ensure!(
        parts.institution == institution_code,
        Error::<T>::InvalidCidNumber
    );
    // 私权机构名称上链:链是机构信息唯一真源(注册局本地库为同步副本),
    // 公权/私权统一。全称必填,简称非空。
    ensure!(!cid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!cid_short_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(
        !legal_representative_name.is_empty(),
        Error::<T>::EmptyLegalRepresentativeName
    );
    ensure!(
        !legal_representative_cid_number.is_empty(),
        Error::<T>::EmptyLegalRepresentativeCidNumber
    );
    ensure!(town_code.is_empty(), Error::<T>::InvalidTownCode);
    let (stored_full_name, stored_short_name, stored_town_code) = (
        cid_full_name.clone(),
        cid_short_name.clone(),
        town_code.clone(),
    );
    ensure!(
        !actor_cid_number.is_empty(),
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
    Pallet::<T>::ensure_lifecycle_institution_code(&institution_code)?;

    let register_nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedRegisterNonce::<T>::get(register_nonce_hash),
        Error::<T>::RegisterNonceAlreadyUsed
    );
    let account_name_payload = account_names_payload_from_initial_accounts::<T>(&accounts)?;
    ensure!(
        T::CidInstitutionVerifier::verify_institution_creation(
            cid_number.as_slice(),
            &cid_full_name,
            cid_short_name.as_slice(),
            legal_representative_name.as_slice(),
            legal_representative_cid_number.as_slice(),
            &legal_representative_account,
            &account_name_payload,
            funding_account.as_ref(),
            &roles.encode(),
            &assignments.encode(),
            &register_nonce,
            &signature,
            actor_cid_number.as_slice(),
            &credential_signer_pubkey,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
            town_code.as_slice(),
        ),
        Error::<T>::InvalidCidInstitutionSignature
    );
    ensure!(
        T::RegistryAuthority::can_register_institution(
            &who,
            actor_cid_number.as_slice(),
            &credential_signer_pubkey,
            cid_number.as_slice(),
            institution_code,
            scope_province_name.as_slice(),
            scope_city_name.as_slice(),
        ),
        Error::<T>::RegistryAuthorityDenied
    );

    let (created_accounts, main_account, _fee_account, initial_total) =
        validate_initial_accounts::<T>(&cid_number, &accounts)?;
    let funding_account = match (initial_total.is_zero(), funding_account) {
        (true, None) => None,
        (true, Some(_)) => return Err(Error::<T>::UnexpectedFundingAccount.into()),
        (false, None) => return Err(Error::<T>::FundingAccountRequired.into()),
        (false, Some(account)) => {
            ensure!(
                T::InstitutionQuery::account_belongs_to(actor_cid_number.as_slice(), &account)
                    && T::InstitutionAsset::can_spend(
                        &account,
                        InstitutionAssetAction::InstitutionCreateFunding,
                    ),
                Error::<T>::InvalidFundingAccount
            );
            let required = initial_total
                .checked_add(&T::Currency::minimum_balance())
                .ok_or(Error::<T>::InsufficientAmount)?;
            ensure!(
                T::Currency::free_balance(&account) >= required,
                Error::<T>::InsufficientAmount
            );
            Some(account)
        }
    };
    // 外层 FeeRoute 已按 initial_total 从 actor CID 费用账户收取一次完整链上费。
    let fee = primitives::fee_policy::calculate_onchain_fee(initial_total.saturated_into())
        .saturated_into();

    let now = <frame_system::Pallet<T>>::block_number();
    // 管理员与内部投票都按机构 CID 寻址，任何机构账户都不充当授权根。
    with_transaction(|| {
        let admins = match Pallet::<T>::store_initial_roles_and_assignments(
            &cid_number,
            &roles,
            &assignments,
            entity_primitives::InstitutionAssignmentSource::Registry,
        ) {
            Ok(admins) => admins,
            Err(err) => return TransactionOutcome::Rollback(Err(err)),
        };
        if let Err(err) = Pallet::<T>::ensure_admin_config(&admins, threshold) {
            return TransactionOutcome::Rollback(Err(err));
        }
        if let Some(source) = funding_account.as_ref() {
            for account in created_accounts.iter() {
                if !account.amount.is_zero()
                    && T::Currency::transfer(
                        source,
                        &account.address,
                        account.amount,
                        ExistenceRequirement::KeepAlive,
                    )
                    .is_err()
                {
                    return TransactionOutcome::Rollback(Err(Error::<T>::TransferFailed.into()));
                }
            }
        }

        Institutions::<T>::insert(
            &cid_number,
            InstitutionInfo {
                cid_full_name: stored_full_name.clone(),
                cid_short_name: stored_short_name.clone(),
                town_code: stored_town_code.clone(),
                legal_representative_name: Some(legal_representative_name.clone()),
                legal_representative_cid_number: Some(legal_representative_cid_number.clone()),
                legal_representative_account: Some(legal_representative_account.clone()),
                institution_code,
                created_at: now,
            },
        );

        for account in created_accounts.iter() {
            InstitutionAccounts::<T>::insert(
                &cid_number,
                &account.account_name,
                InstitutionAccountInfo {
                    address: account.address.clone(),
                    initial_balance: account.amount,
                    created_at: now,
                },
            );
            AccountRegisteredCid::<T>::insert(
                &account.address,
                RegisteredInstitution {
                    cid_number: cid_number.clone(),
                    account_name: account.account_name.clone(),
                },
            );
        }

        // 注册局创建机构时直接提交目标机构有效管理员集合；授权与阈值均以 CID 为 key。
        if let Err(err) =
            Pallet::<T>::set_institution_admins(&cid_number, institution_code, &admins, threshold)
        {
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
