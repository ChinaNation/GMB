//! 机构创建流程实现。
//!
//! 机构按 CID 类型自动建立全部强制协议账户，初始余额统一为零。
//!
//! 唯一入口: `do_propose_create_private_institution`(call_index=5)
//! - 载荷只包含机构最小身份和至少两个管理员人员记录
//! - 凭证带 actor CID 和签名管理员公钥，不以任何机构账户充当授权身份
//! - runtime 自动派生机构码、协议账户、默认法定代表人岗位和严格多数阈值

extern crate alloc;

use crate::institution::accounts::{build_required_protocol_accounts, validate_initial_accounts};
use crate::institution::types::{InstitutionAccountInfo, InstitutionInfo};
use crate::pallet::{
    AccountNameOf, AccountRegisteredCid, CidNumberOf, Config, Error, Event, InstitutionAccounts,
    InstitutionAdminsInputOf, Institutions, Pallet, RegisterNonceOf, RegisterSignatureOf,
    UsedRegisterNonce,
};
use crate::traits::{
    CidInstitutionVerifier, InstitutionCidQuery, ProtectedSourceChecker, RegistryAuthority,
};
use crate::RegisteredInstitution;
use codec::Encode;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::{
    traits::{Hash, SaturatedConversion},
    DispatchResult,
};

/// 机构注册创建(call_index=5)。
#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_create_private_institution<T: Config>(
    who: T::AccountId,
    cid_number: CidNumberOf<T>,
    cid_full_name: AccountNameOf<T>,
    cid_short_name: AccountNameOf<T>,
    town_code: AccountNameOf<T>,
    admins: InstitutionAdminsInputOf<T>,
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
    let institution_code = parts.institution;
    // 私权机构名称上链:链是机构信息唯一真源(注册局本地库为同步副本),
    // 公权/私权统一。全称必填,简称非空。
    ensure!(!cid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!cid_short_name.is_empty(), Error::<T>::EmptyAccountName);
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
    let threshold = admins.len() as u32 / 2 + 1;
    Pallet::<T>::ensure_admin_config(&admins, threshold)?;
    ensure!(
        T::CidInstitutionVerifier::verify_institution_creation(
            cid_number.as_slice(),
            &cid_full_name,
            cid_short_name.as_slice(),
            &admins.encode(),
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

    let protocol_accounts = build_required_protocol_accounts::<T>(&cid_number)?;
    let (created_accounts, main_account, _fee_account, initial_total) =
        validate_initial_accounts::<T>(&cid_number, &protocol_accounts)?;
    // 首次登记没有入金；外层 FeeRoute 只从 actor CID 费用账户收取机构操作最低费。
    let fee = primitives::fee_policy::calculate_onchain_fee(initial_total.saturated_into())
        .saturated_into();

    let now = <frame_system::Pallet<T>>::block_number();
    // 管理员与内部投票都按机构 CID 寻址，任何机构账户都不充当授权根。
    with_transaction(|| {
        if let Err(err) = Pallet::<T>::store_default_legal_representative_role(&cid_number) {
            return TransactionOutcome::Rollback(Err(err));
        }

        Institutions::<T>::insert(
            &cid_number,
            InstitutionInfo {
                cid_full_name: stored_full_name.clone(),
                cid_short_name: stored_short_name.clone(),
                town_code: stored_town_code.clone(),
                legal_representative_name: None,
                legal_representative_cid_number: None,
                legal_representative_account: None,
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
