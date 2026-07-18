//! 机构创建流程实现。
//!
//! 机构按 CID 类型自动建立全部强制协议账户，初始余额统一为零。
//!
//! 唯一入口: `do_propose_create_public_institution`(call_index=5)
//! - 载荷只包含机构最小身份和至少两个管理员人员记录
//! - 操作机构 CID 表示注册局 actor,签名者只来自 extrinsic origin
//! - runtime 自动派生机构码、协议账户、默认法定代表人岗位和严格多数阈值

extern crate alloc;

use crate::institution::accounts::{build_required_protocol_accounts, validate_initial_accounts};
use crate::institution::types::{InstitutionAccountInfo, InstitutionInfo};
use crate::pallet::{
    AccountNameOf, AccountRegisteredCid, CidNumberOf, Config, Error, Event, InstitutionAccounts,
    InstitutionAdminsInputOf, Institutions, Pallet,
};
use crate::traits::{InstitutionCidQuery, ProtectedSourceChecker, RegistryAuthority};
use crate::RegisteredInstitution;
use frame_support::{
    ensure,
    storage::{with_transaction, TransactionOutcome},
};
use sp_runtime::{traits::SaturatedConversion, DispatchResult};
use votingengine::types::InstitutionCode;

fn ensure_public_town_code<T: Config>(
    institution_code: &InstitutionCode,
    town_code: &AccountNameOf<T>,
) -> DispatchResult {
    let is_town = matches!(
        primitives::cid::code::admin_level(institution_code),
        Some(primitives::cid::code::AdminLevel::Town)
    );
    if is_town {
        ensure!(
            town_code.as_slice().len() == 3
                && town_code.as_slice().iter().all(u8::is_ascii_alphanumeric),
            Error::<T>::InvalidTownCode
        );
    } else {
        ensure!(town_code.is_empty(), Error::<T>::InvalidTownCode);
    }
    Ok(())
}

/// 机构注册创建(call_index=5)。
pub(crate) fn do_propose_create_public_institution<T: Config>(
    who: T::AccountId,
    cid_number: CidNumberOf<T>,
    cid_full_name: AccountNameOf<T>,
    cid_short_name: AccountNameOf<T>,
    town_code: AccountNameOf<T>,
    admins: InstitutionAdminsInputOf<T>,
    actor_cid_number: alloc::vec::Vec<u8>,
) -> DispatchResult {
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&who),
        Error::<T>::ProtectedSource
    );
    ensure!(!cid_number.is_empty(), Error::<T>::EmptyCidNumber);
    // CID 号全量校验单源 primitives::cid;机构码必须是公权家族且与参数一致。
    let parts = primitives::cid::number::parse_cid_number_parts_bytes(cid_number.as_slice())
        .map_err(|_| Error::<T>::InvalidCidNumber)?;
    ensure!(
        primitives::cid::code::is_public_legal_code(&parts.institution),
        Error::<T>::InvalidCidNumber
    );
    let institution_code = parts.institution;
    // public-manage 只管理公权机构,公权机构全称/简称必须上链供 App 直读。
    ensure!(!cid_full_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure!(!cid_short_name.is_empty(), Error::<T>::EmptyAccountName);
    ensure_public_town_code::<T>(&institution_code, &town_code)?;
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
        !Institutions::<T>::contains_key(&cid_number),
        Error::<T>::InstitutionAlreadyExists
    );
    ensure!(
        !T::SiblingInstitutionQuery::cid_exists(&cid_number),
        Error::<T>::InstitutionAlreadyExists
    );
    Pallet::<T>::ensure_lifecycle_institution_code(&institution_code)?;

    let threshold = admins.len() as u32 / 2 + 1;
    Pallet::<T>::ensure_admin_config(&admins, threshold)?;
    ensure!(
        T::RegistryAuthority::can_register_institution_origin(
            &who,
            actor_cid_number.as_slice(),
            cid_number.as_slice(),
            institution_code,
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

        // 注册局创建机构时直接提交目标机构管理员合集;交易成功即写 Active。
        if let Err(err) =
            Pallet::<T>::set_institution_admins(&cid_number, institution_code, &admins, threshold)
        {
            return TransactionOutcome::Rollback(Err(err));
        }
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
