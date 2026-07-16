//! 私权机构自定义命名账户关闭流程（call_index=1）。
//!
//! 机构本身不提供关闭路径；主账户、费用账户及所有制度协议账户永久存在。
//! 只有 `InstitutionNamed` 可由 `actor_cid_number + institution_account + origin 管理员`
//! 创建内部提案并在通过后关闭。

extern crate alloc;

use admin_primitives::InstitutionAdminQuery;
use codec::Encode;
use frame_support::{
    ensure,
    traits::{Currency, ReservableCurrency},
};
use primitives::institution_asset::{InstitutionAsset, InstitutionAssetAction};
use sp_runtime::{
    traits::{CheckedSub, Hash, Zero},
    DispatchResult, SaturatedConversion,
};
use votingengine::InternalVoteEngine;

use crate::institution::types::CloseInstitutionAction;
use crate::pallet::{
    AccountRegisteredCid, CidNumberOf, Config, Error, Event, InstitutionAccounts,
    InstitutionPendingClose, Pallet, RegisterNonceOf, RegisterSignatureOf, UsedDeregisterNonce,
    ACTION_CLOSE,
};
use crate::traits::{
    AccountValidator, CidInstitutionVerifier, ProtectedSourceChecker, ReservedAccountGuard,
};
use crate::BalanceOf;

#[allow(clippy::too_many_arguments)]
pub(crate) fn do_propose_institution_close<T: Config>(
    who: T::AccountId,
    actor_cid_number: CidNumberOf<T>,
    institution_account: T::AccountId,
    beneficiary: T::AccountId,
    register_nonce: RegisterNonceOf<T>,
    signature: RegisterSignatureOf<T>,
    credential_issuer_cid_number: alloc::vec::Vec<u8>,
    credential_signer_pubkey: [u8; 32],
) -> DispatchResult {
    let registered = AccountRegisteredCid::<T>::get(&institution_account)
        .ok_or(Error::<T>::NotInstitutionAccount)?;
    ensure!(
        registered.cid_number == actor_cid_number,
        Error::<T>::NotInstitutionAccount
    );
    let account_info = InstitutionAccounts::<T>::get(
        &actor_cid_number,
        &registered.account_name,
    )
    .ok_or(Error::<T>::AccountNotFound)?;
    ensure!(
        account_info.address == institution_account,
        Error::<T>::AccountNotFound
    );
    let (_, kind) = Pallet::<T>::derive_registered_account(
        actor_cid_number.as_slice(),
        registered.account_name.as_slice(),
    )?;
    ensure!(
        kind.is_closable_institution_account(),
        Error::<T>::CannotCloseProtectedInstitution
    );

    let institution_code = Pallet::<T>::resolve_institution_code_for_account(&institution_account)
        .ok_or(Error::<T>::AccountNotFound)?;
    ensure!(
        T::InstitutionAdminQuery::is_institution_admin(
            institution_code,
            actor_cid_number.as_slice(),
            &who,
        ),
        Error::<T>::PermissionDenied
    );
    ensure!(
        !T::ProtectedSourceChecker::is_protected(&institution_account)
            && T::InstitutionAsset::can_spend(
                &institution_account,
                InstitutionAssetAction::MultisigCloseExecute,
            ),
        Error::<T>::ProtectedSource
    );
    ensure!(beneficiary != institution_account, Error::<T>::InvalidBeneficiary);
    ensure!(
        !T::ReservedAccountChecker::is_reserved(&beneficiary)
            && T::AccountValidator::is_valid(&beneficiary)
            && !T::ProtectedSourceChecker::is_protected(&beneficiary),
        Error::<T>::InvalidBeneficiary
    );

    let nonce_hash = <T as frame_system::Config>::Hashing::hash(register_nonce.as_slice());
    ensure!(
        !UsedDeregisterNonce::<T>::get(nonce_hash),
        Error::<T>::DeregisterNonceAlreadyUsed
    );
    ensure!(
        T::CidInstitutionVerifier::verify_institution_account_close(
            actor_cid_number.as_slice(),
            registered.account_name.as_slice(),
            &institution_account,
            &register_nonce,
            &signature,
            credential_issuer_cid_number.as_slice(),
            &credential_signer_pubkey,
        ),
        Error::<T>::InvalidDeregisterCredential
    );
    ensure!(
        !InstitutionPendingClose::<T>::contains_key(&institution_account),
        Error::<T>::CloseAlreadyPending
    );
    ensure!(
        T::Currency::reserved_balance(&institution_account).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );

    let action = CloseInstitutionAction {
        actor_cid_number: actor_cid_number.clone(),
        institution_account: institution_account.clone(),
        beneficiary: beneficiary.clone(),
        proposer: who.clone(),
    };
    let mut data = alloc::vec::Vec::from(crate::MODULE_TAG);
    data.push(ACTION_CLOSE);
    data.extend_from_slice(&action.encode());
    let proposal_id = T::InternalVoteEngine::create_institution_proposal_with_data(
        who.clone(),
        institution_code,
        actor_cid_number.to_vec(),
        Some(institution_account.clone()),
        alloc::vec![actor_cid_number.to_vec()],
        crate::MODULE_TAG,
        data,
    )?;
    UsedDeregisterNonce::<T>::insert(nonce_hash, true);
    InstitutionPendingClose::<T>::insert(&institution_account, proposal_id);

    Pallet::<T>::deposit_event(Event::<T>::InstitutionCloseProposed {
        proposal_id,
        account: institution_account,
        proposer: who,
        beneficiary,
    });
    Ok(())
}

pub(crate) fn execute_institution_close_with_finalizer<T: Config>(
    proposal_id: u64,
    action: &CloseInstitutionAction<T::AccountId, CidNumberOf<T>>,
) -> DispatchResult {
    use frame_support::traits::{ExistenceRequirement, OnUnbalanced, WithdrawReasons};

    let registered = AccountRegisteredCid::<T>::get(&action.institution_account)
        .ok_or(Error::<T>::AccountNotFound)?;
    ensure!(
        registered.cid_number == action.actor_cid_number,
        Error::<T>::AccountNotFound
    );
    let account_info = InstitutionAccounts::<T>::get(
        &action.actor_cid_number,
        &registered.account_name,
    )
    .ok_or(Error::<T>::AccountNotFound)?;
    ensure!(
        account_info.address == action.institution_account,
        Error::<T>::AccountNotFound
    );
    let (_, kind) = Pallet::<T>::derive_registered_account(
        action.actor_cid_number.as_slice(),
        registered.account_name.as_slice(),
    )?;
    ensure!(
        kind.is_closable_institution_account(),
        Error::<T>::CannotCloseProtectedInstitution
    );
    ensure!(
        T::InstitutionAsset::can_spend(
            &action.institution_account,
            InstitutionAssetAction::MultisigCloseExecute,
        ),
        Error::<T>::ProtectedSource
    );

    let institution_code = Pallet::<T>::resolve_institution_code_for_account(
        &action.institution_account,
    )
    .ok_or(Error::<T>::AccountNotFound)?;
    let proposal = votingengine::Pallet::<T>::proposals(proposal_id)
        .ok_or(Error::<T>::ProposalActionNotFound)?;
    ensure!(
        votingengine::Pallet::<T>::is_callback_execution_scope(proposal_id)
            && votingengine::Pallet::<T>::is_proposal_owner(proposal_id, crate::MODULE_TAG)
            && proposal.kind == votingengine::PROPOSAL_KIND_INTERNAL
            && proposal.stage == votingengine::STAGE_INTERNAL
            && proposal.status == votingengine::STATUS_PASSED
            && proposal.internal_code == Some(institution_code)
            && proposal.actor_cid_number.as_ref().map(|cid| cid.as_slice())
                == Some(action.actor_cid_number.as_slice())
            && proposal.execution_account.as_ref() == Some(&action.institution_account)
            && InstitutionPendingClose::<T>::get(&action.institution_account) == Some(proposal_id),
        Error::<T>::ProposalActionNotFound
    );
    ensure!(
        action.beneficiary != action.institution_account
            && !T::ReservedAccountChecker::is_reserved(&action.beneficiary)
            && T::AccountValidator::is_valid(&action.beneficiary)
            && !T::ProtectedSourceChecker::is_protected(&action.beneficiary),
        Error::<T>::InvalidBeneficiary
    );
    ensure!(
        T::Currency::reserved_balance(&action.institution_account).is_zero(),
        Error::<T>::ReservedBalanceRemaining
    );

    let balance = T::Currency::free_balance(&action.institution_account);
    let mut transferred = BalanceOf::<T>::zero();
    let mut fee = BalanceOf::<T>::zero();
    if !balance.is_zero() {
        fee = onchain::calculate_onchain_fee(balance.saturated_into()).saturated_into();
        let transfer_amount = balance.checked_sub(&fee).unwrap_or_else(Zero::zero);
        if !fee.is_zero() {
            let fee_imbalance = T::Currency::withdraw(
                &action.institution_account,
                fee,
                WithdrawReasons::FEE,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::FeeWithdrawFailed)?;
            T::FeeRouter::on_unbalanced(fee_imbalance);
        }
        if !transfer_amount.is_zero() {
            T::Currency::transfer(
                &action.institution_account,
                &action.beneficiary,
                transfer_amount,
                ExistenceRequirement::AllowDeath,
            )
            .map_err(|_| Error::<T>::TransferFailed)?;
            transferred = transfer_amount;
        }
    }

    InstitutionAccounts::<T>::remove(&action.actor_cid_number, &registered.account_name);
    AccountRegisteredCid::<T>::remove(&action.institution_account);
    InstitutionPendingClose::<T>::remove(&action.institution_account);
    Pallet::<T>::deposit_event(Event::<T>::InstitutionClosed {
        proposal_id,
        account: action.institution_account.clone(),
        beneficiary: action.beneficiary.clone(),
        amount: transferred,
        fee,
    });
    Ok(())
}
