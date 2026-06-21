#![cfg(test)]

use super::*;
use alloc::vec::Vec;
use frame_support::{assert_noop, assert_ok};

#[test]
fn bind_succeeds_and_tracks_binding_id() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-a", "nonce-a", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            credential.clone()
        ));

        assert_eq!(
            BindingIdToAccount::<Test>::get(credential.binding_id),
            Some(1)
        );
        assert_eq!(
            AccountToBindingId::<Test>::get(1),
            Some(credential.binding_id)
        );
        assert_eq!(BoundCount::<Test>::get(), 1);
    });
}

#[test]
fn bind_rejects_reused_bind_nonce() {
    new_test_ext().execute_with(|| {
        let first = bind_credential(b"binding-a", "same-nonce", "bind-ok");
        let second = bind_credential(b"binding-b", "same-nonce", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), first));
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(2), second),
            Error::<Test>::BindNonceAlreadyUsed
        );
    });
}

#[test]
fn bind_allows_account_rebinding_to_new_binding_id() {
    new_test_ext().execute_with(|| {
        let first = bind_credential(b"binding-a", "nonce-a", "bind-ok");
        let second = bind_credential(b"binding-b", "nonce-b", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            first.clone()
        ));
        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            second.clone()
        ));

        assert!(BindingIdToAccount::<Test>::get(first.binding_id).is_none());
        assert_eq!(BindingIdToAccount::<Test>::get(second.binding_id), Some(1));
        assert_eq!(AccountToBindingId::<Test>::get(1), Some(second.binding_id));
        assert_eq!(BoundCount::<Test>::get(), 1);
    });
}

#[test]
fn bind_rejects_empty_nonce() {
    new_test_ext().execute_with(|| {
        let empty_credential = BindCredential {
            binding_id: binding_id(b"id-empty"),
            bind_nonce: Vec::<u8>::new().try_into().expect("empty vec fits"),
            issuer_sfid_number: b"SFID-ISSUER"
                .to_vec()
                .try_into()
                .expect("issuer sfid fits"),
            issuer_main_account: 99,
            signer_pubkey: [7u8; 32],
            scope_province_name: b"liaoning".to_vec().try_into().expect("scope fits"),
            scope_city_name: b"shenyang".to_vec().try_into().expect("scope fits"),
            signature: signature("bind-ok"),
        };
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(1), empty_credential),
            Error::<Test>::EmptyBindNonce
        );
    });
}

#[test]
fn bind_rejects_invalid_signature() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"id-badsig", "nonce-badsig", "bad-sig");
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential),
            Error::<Test>::InvalidSfidBindingSignature
        );
    });
}

#[test]
fn bind_rejects_binding_id_owned_by_another_account() {
    new_test_ext().execute_with(|| {
        let credential_1 = bind_credential(b"shared-id", "nonce-1", "bind-ok");
        let credential_2 = bind_credential(b"shared-id", "nonce-2", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            credential_1
        ));
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(2), credential_2),
            Error::<Test>::BindingIdAlreadyBoundToAnotherAccount
        );
    });
}

#[test]
fn bind_rejects_same_binding_id_already_bound() {
    new_test_ext().execute_with(|| {
        let credential_1 = bind_credential(b"dup-id", "nonce-dup-1", "bind-ok");
        let credential_2 = bind_credential(b"dup-id", "nonce-dup-2", "bind-ok");

        assert_ok!(SfidSystem::bind_sfid(
            RuntimeOrigin::signed(1),
            credential_1
        ));
        assert_noop!(
            SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential_2),
            Error::<Test>::SameBindingIdAlreadyBound
        );
    });
}

#[test]
fn unbind_by_root_origin_succeeds() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-unbind", "nonce-unbind", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        assert_ok!(SfidSystem::unbind_sfid(RuntimeOrigin::root(), 1));
        assert!(BindingIdToAccount::<Test>::get(bid).is_none());
        assert!(AccountToBindingId::<Test>::get(1).is_none());
        assert_eq!(BoundCount::<Test>::get(), 0);
    });
}

#[test]
fn vote_credential_is_consumed_once_per_proposal_and_binding_id() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-vote", "bind-nonce", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::is_eligible(&bid, &1));
        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            7,
            b"vote-nonce",
            b"vote-ok",
            b"SFID-ISSUER",
            &99,
            &[7u8; 32],
            b"liaoning",
            b"shenyang",
        ));
        assert!(!<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            7,
            b"vote-nonce",
            b"vote-ok",
            b"SFID-ISSUER",
            &99,
            &[7u8; 32],
            b"liaoning",
            b"shenyang",
        ));
    });
}

#[test]
fn vote_nonce_is_scoped_per_proposal() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-replay", "bind-nonce", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            10,
            b"same-nonce",
            b"vote-ok",
            b"SFID-ISSUER",
            &99,
            &[7u8; 32],
            b"liaoning",
            b"shenyang",
        ));
        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            20,
            b"same-nonce",
            b"vote-ok",
            b"SFID-ISSUER",
            &99,
            &[7u8; 32],
            b"liaoning",
            b"shenyang",
        ));
    });
}

#[test]
fn cleanup_vote_credentials_removes_nonce_state() {
    new_test_ext().execute_with(|| {
        let credential = bind_credential(b"binding-cleanup", "bind-nonce", "bind-ok");
        let bid = credential.binding_id;
        assert_ok!(SfidSystem::bind_sfid(RuntimeOrigin::signed(1), credential));

        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            7,
            b"vote-nonce",
            b"vote-ok",
            b"SFID-ISSUER",
            &99,
            &[7u8; 32],
            b"liaoning",
            b"shenyang",
        ));
        <Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::cleanup_vote_credentials(7);
        assert!(<Pallet<Test> as SfidEligibilityProvider<
            u64,
            <Test as frame_system::Config>::Hash,
        >>::verify_and_consume_vote_credential(
            &bid,
            &1,
            7,
            b"vote-nonce",
            b"vote-ok",
            b"SFID-ISSUER",
            &99,
            &[7u8; 32],
            b"liaoning",
            b"shenyang",
        ));
    });
}
