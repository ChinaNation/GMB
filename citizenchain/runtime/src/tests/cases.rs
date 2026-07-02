use super::*;

// ============================================================================
// 簇 1:Runtime 整体自检(4 个用例)
// ============================================================================

#[test]
fn time_and_currency_constants_are_consistent() {
    assert_eq!(YUAN, 100 * FEN);
    assert_eq!(UNIT, YUAN);
    assert_eq!(HOURS, MINUTES * 60);
    assert_eq!(DAYS, HOURS * 24);
    assert_eq!(SLOT_DURATION, MILLI_SECS_PER_BLOCK);
}

#[test]
fn fee_payer_returns_none_for_transfer() {
    use configs::RuntimeFeePayerExtractor;
    use frame_support::BoundedVec;
    use onchain_transaction::CallFeePayer;
    use primitives::cid::china::china_cb::CHINA_CB;

    let institution = AccountId::new(CHINA_CB[0].main_account);
    let beneficiary = AccountId::new([99u8; 32]);
    let call = RuntimeCall::MultisigTransfer(multisig_transfer::pallet::Call::propose_transfer {
        institution_code: votingengine::types::NRC,
        institution,
        beneficiary,
        amount: 10000,
        remark: BoundedVec::default(),
    });
    let signer = AccountId::new([1u8; 32]);
    // 中文注释：机构转账提案交易本身由提交者按投票统一价付费；
    // 真正转账手续费在 pallet 执行阶段从机构账户内部扣取，FeePayerExtractor 不代付。
    let payer = RuntimeFeePayerExtractor::fee_payer(&signer, &call);
    assert!(
        payer.is_none(),
        "fee_payer must return None for MultisigTransfer (fees handled internally)"
    );
}

/// 治理业务 pallet 的 `MODULE_TAG` 必须全局唯一。
///
/// 背景:投票引擎达终态后通过 `InternalVoteResultCallback` tuple 广播到
/// 全部业务 Executor,各 Executor 靠 `ProposalData` 前缀的 MODULE_TAG 互斥
/// 认领自己的提案。若两个模块碰撞,同一个提案可能被两个 Executor 同时执行,
/// 产生数据层异常。本测试在编译时固定捕获。
#[test]
fn governance_module_tags_are_globally_unique() {
    use std::collections::HashSet;
    let tags: [(&str, &[u8]); 11] = [
        ("public_admins", public_admins::MODULE_TAG),
        ("private_admins", private_admins::MODULE_TAG),
        ("grandpakey_change", grandpakey_change::MODULE_TAG),
        ("resolution_destro", resolution_destro::MODULE_TAG),
        ("resolution_issuance", resolution_issuance::MODULE_TAG),
        ("runtime_upgrade", runtime_upgrade::MODULE_TAG),
        ("public_manage", public_manage::MODULE_TAG),
        ("private_manage", private_manage::MODULE_TAG),
        ("personal_admins", personal_admins::MODULE_TAG),
        ("multisig_transfer", multisig_transfer::MODULE_TAG),
        ("legislation_yuan", legislation_yuan::MODULE_TAG),
    ];
    let unique: HashSet<&[u8]> = tags.iter().map(|(_, t)| *t).collect();
    assert_eq!(
        unique.len(),
        tags.len(),
        "MODULE_TAG must be globally unique across governance pallets; got: {:?}",
        tags,
    );
}

#[test]
fn runtime_version_and_block_types_are_sane() {
    assert_eq!(VERSION.spec_name.as_ref(), "citizenchain");
    assert_eq!(VERSION.impl_name.as_ref(), "citizenchain");
    assert_eq!(VERSION.authoring_version, 0);
    assert!(
        VERSION.spec_version >= 1,
        "spec_version 必须保持正向递增；WASM CI 会在编译产物时按链上版本临时提升"
    );
    assert_eq!(VERSION.impl_version, 0);
    assert_eq!(VERSION.transaction_version, 0);
    assert_eq!(VERSION.system_version, 0);

    let _opaque_block_id: opaque::BlockId = generic::BlockId::Number(0);
    let _runtime_block_id: BlockId = generic::BlockId::Number(0);
}

// ============================================================================
// 簇 2:装配集成测试(18 个用例)
// ============================================================================

#[test]
fn joint_vote_callback_routes_to_resolution_issuance_and_executes() {
    use codec::Encode;
    new_test_ext().execute_with(|| {
        // 统一 ID：proposal_id 即投票引擎 ID，不再有双 ID 映射
        let proposal_id = 99u64;
        let per_recipient_amount = 123u128;
        let allocations: Vec<resolution_issuance::proposal::RecipientAmount<AccountId, Balance>> =
            CHINA_CB
                .iter()
                .skip(1)
                .map(|node| resolution_issuance::proposal::RecipientAmount {
                    recipient: AccountId::new(node.main_account),
                    amount: per_recipient_amount,
                })
                .collect();
        let recipient = allocations
            .first()
            .expect("CHINA_CB has province_name recipients")
            .recipient
            .clone();
        let recipient_before = Balances::free_balance(&recipient);
        let total_amount = allocations
            .iter()
            .fold(0u128, |sum, item| sum.saturating_add(item.amount));

        // 测试中直接写入 ProposalData/Owner，生产路径必须走 create_*_with_data 原子入口。
        let data = resolution_issuance::proposal::IssuanceProposalData {
            proposer: recipient.clone(),
            reason: b"runtime-integration".to_vec(),
            total_amount,
            allocations,
        };
        let mut encoded = Vec::from(resolution_issuance::MODULE_TAG);
        encoded.extend_from_slice(&data.encode());
        let bounded_data: frame_support::BoundedVec<
            u8,
            <Runtime as votingengine::Config>::MaxProposalDataLen,
        > = encoded.try_into().expect("proposal data bound");
        let owner: frame_support::BoundedVec<
            u8,
            <Runtime as votingengine::Config>::MaxModuleTagLen,
        > = resolution_issuance::MODULE_TAG
            .to_vec()
            .try_into()
            .expect("module tag bound");
        votingengine::ProposalData::<Runtime>::insert(proposal_id, bounded_data);
        votingengine::ProposalOwner::<Runtime>::insert(proposal_id, owner);
        votingengine::Proposals::<Runtime>::insert(
            proposal_id,
            votingengine::Proposal {
                kind: votingengine::PROPOSAL_KIND_JOINT,
                stage: votingengine::STAGE_JOINT,
                status: votingengine::STATUS_PASSED,
                internal_code: None,
                account_context: None,
                subject_cid_numbers: Default::default(),
                start: 0u32,
                end: 100u32,
                citizen_eligible_total: 10,
            },
        );

        resolution_issuance::pallet::VotingProposalCount::<Runtime>::put(1u32);
        votingengine::CallbackExecutionScopes::<Runtime>::insert(proposal_id, ());
        assert_ok!(RuntimeJointVoteResultCallback::on_joint_vote_finalized(
            proposal_id,
            true
        ));
        votingengine::CallbackExecutionScopes::<Runtime>::remove(proposal_id);

        // 验证 VotingProposalCount 已递减
        assert_eq!(
            resolution_issuance::pallet::VotingProposalCount::<Runtime>::get(),
            0u32
        );

        assert!(resolution_issuance::pallet::Executed::<Runtime>::get(proposal_id).is_some());
        assert_eq!(
            resolution_issuance::pallet::TotalIssued::<Runtime>::get(),
            total_amount
        );
        assert_eq!(
            Balances::free_balance(&recipient),
            recipient_before.saturating_add(per_recipient_amount)
        );
    });
}

#[test]
fn resolution_destro_internal_vote_flow_executes_destroy_and_reduces_issuance() {
    new_test_ext().execute_with(|| {
        let nrc_institution = AccountId::new(CHINA_CB[0].main_account);
        let nrc_account = AccountId::new(CHINA_CB[0].main_account);
        let initial_balance: Balance = 1_000;
        let destroy_amount: Balance = 100;

        let _ = Balances::deposit_creating(&nrc_account, initial_balance);
        let issuance_before = Balances::total_issuance();

        assert_ok!(ResolutionDestro::propose_destroy(
            RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].admins[0])),
            votingengine::types::NRC,
            nrc_institution,
            destroy_amount,
        ));

        let pid = VotingEngine::next_proposal_id().saturating_sub(1);

        // 提案人 admins[0] 在 propose_destroy 时已自动计一票,从 admins[1] 起补足到阈值 13。
        for i in 1..13 {
            assert_ok!(InternalVote::cast(
                RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].admins[i])),
                pid,
                true,
            ));
        }

        // 提案数据由 votingengine 延迟清理，执行后仍保留
        assert!(VotingEngine::get_proposal_data(pid).is_some());

        assert_eq!(
            Balances::free_balance(&nrc_account),
            initial_balance - destroy_amount
        );
        assert_eq!(Balances::total_issuance(), issuance_before - destroy_amount);
    });
}

#[test]
fn runtime_fee_kind_classifier_covers_free_onchain_vote_and_unknown_paths() {
    new_test_ext().execute_with(|| {
        let who = AccountId::new([1u8; 32]);
        let recipient = AccountId::new([2u8; 32]);

        let system_call = RuntimeCall::System(frame_system::Call::remark {
            remark: b"x".to_vec(),
        });
        let free = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &system_call);
        assert_eq!(free, onchain_transaction::FeeChargeKind::Free);

        let transfer_call = RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
            dest: sp_runtime::MultiAddress::Id(recipient),
            value: 123,
        });
        let amount = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &transfer_call);
        assert_eq!(
            amount,
            onchain_transaction::FeeChargeKind::OnchainAmount(123)
        );

        let internal_vote_call = RuntimeCall::InternalVote(internal_vote::pallet::Call::cast {
            proposal_id: 1,
            approve: true,
        });
        let vote_kind = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &internal_vote_call);
        // 中文注释：投票 extrinsic 本身按治理用户操作固定 1 元计费，不再套 0.1%。
        assert_eq!(vote_kind, onchain_transaction::FeeChargeKind::VoteFlat);

        let nrc_institution = AccountId::new(CHINA_CB[0].main_account);
        let resolution_destro_call =
            RuntimeCall::ResolutionDestro(resolution_destro::pallet::Call::propose_destroy {
                institution_code: votingengine::types::NRC,
                institution: nrc_institution,
                amount: 456,
            });
        let resolution_kind = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &resolution_destro_call);
        assert_eq!(
            resolution_kind,
            onchain_transaction::FeeChargeKind::VoteFlat
        );

        let unknown_balances_call =
            RuntimeCall::Balances(pallet_balances::Call::upgrade_accounts {
                who: vec![AccountId::new([9u8; 32])],
            });
        let unknown = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &unknown_balances_call);
        assert_eq!(unknown, onchain_transaction::FeeChargeKind::Unknown);
    });
}

#[test]
fn runtime_fee_kind_classifier_treats_governance_proposals_as_vote_flat() {
    new_test_ext().execute_with(|| {
        let (p1, _) = sr25519::Pair::generate();
        let (p2, _) = sr25519::Pair::generate();
        let signer1 = MultiSigner::from(p1.public());
        let who: AccountId = signer1.into_account();
        let admin2: AccountId = MultiSigner::from(p2.public()).into_account();

        let account = AccountId::new([77u8; 32]);
        let beneficiary = AccountId::new([78u8; 32]);
        let admins: personal_manage::pallet::AdminsOf<Runtime> = vec![who.clone(), admin2.clone()]
            .try_into()
            .expect("admins should fit");
        // 中文注释：本测试验证提案交易本身按投票统一价，而不是按提案金额套链上费率。
        let account_name: personal_manage::pallet::AccountNameOf<Runtime> =
            b"runtime-test-personal"
                .to_vec()
                .try_into()
                .expect("account_name should fit");

        let create_call =
            RuntimeCall::PersonalManage(personal_manage::pallet::Call::propose_create {
                account_name,
                admins: admins.clone(),
                regular_threshold: 2,
                amount: 1_000,
            });
        let create_kind = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &create_call);
        assert_eq!(create_kind, onchain_transaction::FeeChargeKind::VoteFlat);

        let _ = Balances::deposit_creating(&account, 777);
        // 中文注释:propose_close 已加注销凭证字段(register_nonce/signature/issuer_*/signer_pubkey);
        // 本测试只验证该 Call 走投票统一价分类,凭证值无关,填默认值即可。
        let close_call = RuntimeCall::PublicManage(
            public_manage::pallet::Call::propose_close_public_institution {
                account,
                beneficiary,
                register_nonce: Default::default(),
                signature: Default::default(),
                issuer_cid_number: Vec::new(),
                issuer_main_account: AccountId::new([0u8; 32]),
                signer_pubkey: [0u8; 32],
            },
        );
        let close_kind = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &close_call);
        assert_eq!(close_kind, onchain_transaction::FeeChargeKind::VoteFlat);

        let institution =
            AccountId::new(primitives::cid::china::china_cb::CHINA_CB[0].main_account);
        let transfer_call =
            RuntimeCall::MultisigTransfer(multisig_transfer::pallet::Call::propose_transfer {
                institution_code: votingengine::types::NRC,
                institution,
                beneficiary: AccountId::new([79u8; 32]),
                amount: 88_888,
                remark: frame_support::BoundedVec::default(),
            });
        let transfer_kind = <RuntimeFeeKindClassifier as onchain_transaction::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &transfer_call);
        assert_eq!(transfer_kind, onchain_transaction::FeeChargeKind::VoteFlat);
    });
}

#[test]
fn multisig_reserved_checker_rejects_stake_and_fee_accounts() {
    let stake = AccountId::new(primitives::cid::china::china_ch::CHINA_CH[0].stake_account);
    assert!(RuntimeReservedAccountGuard::is_reserved(&stake));

    let fee_account = AccountId::new(primitives::cid::china::china_ch::CHINA_CH[0].fee_account);
    assert!(RuntimeReservedAccountGuard::is_reserved(&fee_account));
}

#[test]
fn runtime_call_filter_blocks_force_transfer_from_stake() {
    let stake = AccountId::new(primitives::cid::china::china_ch::CHINA_CH[0].stake_account);
    let dst = AccountId::new([9u8; 32]);

    let blocked_by_id = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
        source: sp_runtime::MultiAddress::Id(stake),
        dest: sp_runtime::MultiAddress::Id(dst.clone()),
        value: 1,
    });
    assert!(!RuntimeCallFilter::contains(&blocked_by_id));

    let stake_raw = primitives::cid::china::china_ch::CHINA_CH[0].stake_account;
    let blocked_by_32 = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
        source: sp_runtime::MultiAddress::Address32(stake_raw),
        dest: sp_runtime::MultiAddress::Id(dst.clone()),
        value: 1,
    });
    assert!(!RuntimeCallFilter::contains(&blocked_by_32));

    let blocked_by_raw = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
        source: sp_runtime::MultiAddress::Raw(stake_raw.to_vec()),
        dest: sp_runtime::MultiAddress::Id(dst.clone()),
        value: 1,
    });
    assert!(!RuntimeCallFilter::contains(&blocked_by_raw));

    let allowed = RuntimeCall::Balances(pallet_balances::Call::force_transfer {
        source: sp_runtime::MultiAddress::Id(AccountId::new([8u8; 32])),
        dest: sp_runtime::MultiAddress::Id(dst),
        value: 1,
    });
    assert!(RuntimeCallFilter::contains(&allowed));

    let blocked_force_unreserve = RuntimeCall::Balances(pallet_balances::Call::force_unreserve {
        who: sp_runtime::MultiAddress::Id(AccountId::new(
            primitives::cid::china::china_ch::CHINA_CH[0].stake_account,
        )),
        amount: 1,
    });
    assert!(!RuntimeCallFilter::contains(&blocked_force_unreserve));

    let blocked_force_set_balance =
        RuntimeCall::Balances(pallet_balances::Call::force_set_balance {
            who: sp_runtime::MultiAddress::Id(AccountId::new(
                primitives::cid::china::china_ch::CHINA_CH[0].stake_account,
            )),
            new_free: 1,
        });
    assert!(!RuntimeCallFilter::contains(&blocked_force_set_balance));
}

#[test]
fn pow_digest_author_finds_pow_engine_author() {
    // 中文注释：pre_digest 现在存储 sr25519 公钥，PowDigestAuthor 解码后派生 AccountId。
    let public = sp_core::sr25519::Public::from_raw([21u8; 32]);
    let expected_account: AccountId = sp_runtime::MultiSigner::from(public).into_account();
    let encoded = public.encode();
    let digests: Vec<(sp_runtime::ConsensusEngineId, &[u8])> = vec![
        (*b"TEST", b"ignored".as_ref()),
        (sp_consensus_pow::POW_ENGINE_ID, encoded.as_slice()),
    ];
    let found = PowDigestAuthor::find_author(digests);
    assert_eq!(found, Some(expected_account));
}

#[test]
fn joint_vote_callback_missing_proposal_and_runtime_upgrade_route() {
    new_test_ext().execute_with(|| {
        // 不存在的提案 ID 应返回错误
        assert!(RuntimeJointVoteResultCallback::on_joint_vote_finalized(999_999, true).is_err());

        // 测试中直接写入 votingengine 存储；生产路径必须走 create_*_with_data 原子入口。
        let proposal_id = 7u64;
        let proposer = AccountId::new(CHINA_CB[0].admins[0]);
        let reason: runtime_upgrade::pallet::ReasonOf<Runtime> =
            b"upgrade".to_vec().try_into().expect("reason");
        let code: runtime_upgrade::pallet::CodeOf<Runtime> =
            vec![1u8, 2, 3].try_into().expect("code");
        let code_hash = <Runtime as frame_system::Config>::Hashing::hash(code.as_slice());

        let proposal = runtime_upgrade::pallet::Proposal::<Runtime> {
            proposer,
            reason,
            code_hash,
        };
        let mut encoded = Vec::from(runtime_upgrade::MODULE_TAG);
        encoded.extend_from_slice(&codec::Encode::encode(&proposal));
        let bounded_data: frame_support::BoundedVec<
            u8,
            <Runtime as votingengine::Config>::MaxProposalDataLen,
        > = encoded.try_into().expect("proposal data bound");
        let owner: frame_support::BoundedVec<
            u8,
            <Runtime as votingengine::Config>::MaxModuleTagLen,
        > = runtime_upgrade::MODULE_TAG
            .to_vec()
            .try_into()
            .expect("module tag bound");
        votingengine::ProposalData::<Runtime>::insert(proposal_id, bounded_data);
        votingengine::ProposalOwner::<Runtime>::insert(proposal_id, owner);
        let code_vec = code.into_inner();
        let object_len = u32::try_from(code_vec.len()).expect("runtime code length fits u32");
        let object_hash = <Runtime as frame_system::Config>::Hashing::hash(&code_vec);
        let bounded_object: frame_support::BoundedVec<
            u8,
            <Runtime as votingengine::Config>::MaxProposalObjectLen,
        > = code_vec.try_into().expect("runtime code object bound");
        votingengine::ProposalObject::<Runtime>::insert(proposal_id, bounded_object);
        votingengine::ProposalObjectMeta::<Runtime>::insert(
            proposal_id,
            votingengine::ProposalObjectMetadata {
                kind: runtime_upgrade::pallet::PROPOSAL_OBJECT_KIND_RUNTIME_WASM,
                object_len,
                object_hash,
            },
        );
        votingengine::Proposals::<Runtime>::insert(
            proposal_id,
            votingengine::Proposal {
                kind: votingengine::PROPOSAL_KIND_JOINT,
                stage: votingengine::STAGE_JOINT,
                status: votingengine::STATUS_REJECTED,
                internal_code: None,
                account_context: None,
                subject_cid_numbers: Default::default(),
                start: 0u32,
                end: 100u32,
                citizen_eligible_total: 10,
            },
        );

        // 回调拒绝后，业务摘要保持创建时快照，终态由 votingengine 统一维护。
        let outcome = RuntimeJointVoteResultCallback::on_joint_vote_finalized(proposal_id, false)
            .expect("runtime-upgrade callback should succeed");
        assert_eq!(outcome, votingengine::ProposalExecutionOutcome::Executed);
        let raw = votingengine::Pallet::<Runtime>::get_proposal_data(proposal_id)
            .expect("proposal data should exist");
        let tag = runtime_upgrade::MODULE_TAG;
        assert!(
            raw.len() >= tag.len() && &raw[..tag.len()] == tag,
            "MODULE_TAG mismatch"
        );
        let updated = runtime_upgrade::pallet::Proposal::<Runtime>::decode(&mut &raw[tag.len()..])
            .expect("should decode");
        assert_eq!(updated.code_hash, code_hash);
        assert_eq!(
            votingengine::Proposals::<Runtime>::get(proposal_id)
                .expect("engine proposal should exist")
                .status,
            votingengine::STATUS_REJECTED
        );
    });
}

// 公民身份上链统一由 citizen-identity 承载：注册局管理员提交交易，公民钱包签名确认。
#[test]
fn runtime_citizen_identity_frg_province_admin_registers_voting_identity() {
    new_test_ext().execute_with(|| {
        let (_, registrar, registrar_account) = setup_frg_citizen_identity_admin(b"43");
        let wallet_pair =
            sr25519::Pair::from_string("//citizen-wallet-1", None).expect("wallet pair");
        let wallet_account = AccountId::new(wallet_pair.public().0);
        let payload = build_voting_identity_payload(
            wallet_account.clone(),
            b"CTZN-RUNTIME-0001",
            b"43",
            b"4301",
            b"4301001",
        );
        let signature = sign_citizen_identity_payload(&wallet_pair, &payload);

        assert_ok!(CitizenIdentity::register_voting_identity(
            RuntimeOrigin::signed(registrar),
            registrar_account,
            payload,
            signature,
        ));

        assert!(
            citizen_identity::VotingIdentityByAccount::<Runtime>::contains_key(&wallet_account)
        );
        assert_eq!(citizen_identity::CountryVotingCount::<Runtime>::get(), 1);
    });
}

#[test]
fn runtime_citizen_identity_frg_admin_cannot_register_other_province() {
    new_test_ext().execute_with(|| {
        let (_, registrar, registrar_account) = setup_frg_citizen_identity_admin(b"43");
        let wallet_pair =
            sr25519::Pair::from_string("//citizen-wallet-2", None).expect("wallet pair");
        let wallet_account = AccountId::new(wallet_pair.public().0);
        let payload = build_voting_identity_payload(
            wallet_account,
            b"CTZN-RUNTIME-0002",
            b"44",
            b"4401",
            b"4401001",
        );
        let signature = sign_citizen_identity_payload(&wallet_pair, &payload);

        assert_noop!(
            CitizenIdentity::register_voting_identity(
                RuntimeOrigin::signed(registrar),
                registrar_account,
                payload,
                signature,
            ),
            citizen_identity::Error::<Runtime>::UnauthorizedRegistrar
        );
    });
}

#[test]
fn runtime_citizen_identity_reader_reads_voting_and_candidate_identity() {
    new_test_ext().execute_with(|| {
        let (_, registrar, registrar_account) = setup_frg_citizen_identity_admin(b"43");
        let wallet_pair =
            sr25519::Pair::from_string("//citizen-wallet-3", None).expect("wallet pair");
        let wallet_account = AccountId::new(wallet_pair.public().0);
        let voting = build_voting_identity_payload(
            wallet_account.clone(),
            b"CTZN-RUNTIME-0003",
            b"43",
            b"4301",
            b"4301001",
        );
        let candidate = citizen_identity::CandidateIdentityPayload {
            voting,
            birth_province_code: test_area_code(b"43"),
            birth_city_code: test_area_code(b"4301"),
            birth_town_code: test_area_code(b"4301001"),
            citizen_full_name: b"Runtime Citizen"
                .to_vec()
                .try_into()
                .expect("citizen name fits"),
        };
        let signature = sign_citizen_identity_payload(&wallet_pair, &candidate);

        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(registrar),
            registrar_account,
            candidate,
            signature,
        ));

        let town_scope = citizen_identity::PopulationScope::Town(
            test_area_code(b"43"),
            test_area_code(b"4301"),
            test_area_code(b"4301001"),
        );
        assert!(RuntimeCitizenIdentityReader::can_vote(
            &wallet_account,
            &town_scope
        ));
        assert!(RuntimeCitizenIdentityReader::can_be_candidate(
            &wallet_account,
            &town_scope
        ));
        assert_eq!(
            RuntimeCitizenIdentityReader::population_count(&town_scope),
            1
        );
    });
}

#[test]
fn ensure_nrc_admin_and_runtime_internal_admin_provider_paths() {
    new_test_ext().execute_with(|| {
        let nrc_id = AccountId::new(CHINA_CB[0].main_account);
        let nrc_admin = AccountId::new(CHINA_CB[0].admins[0]);
        let outsider = AccountId::new([99u8; 32]);

        let ok_origin = RuntimeOrigin::signed(nrc_admin.clone());
        assert!(<EnsureNrcAdmin as EnsureOrigin<RuntimeOrigin>>::try_origin(ok_origin).is_ok());
        let bad_origin = RuntimeOrigin::signed(outsider.clone());
        assert!(<EnsureNrcAdmin as EnsureOrigin<RuntimeOrigin>>::try_origin(bad_origin).is_err());

        public_admins::pallet::AdminAccounts::<Runtime>::remove(&nrc_id);
        assert!(!is_nrc_admin(&nrc_admin));
        assert!(!is_nrc_admin(&outsider));
        assert!(!RuntimeInternalAdminProvider::is_internal_admin(
            votingengine::types::NRC,
            nrc_id,
            &nrc_admin
        ));
    });
}

#[test]
fn runtime_cid_institution_verifier_runtime_admin_account_query_lookup() {
    new_test_ext().execute_with(|| {
        let (main_pair, main_admin_pubkey, backup_pair, backup_admin_pubkey, province_bytes) =
            setup_step3_test_admins();
        let issuer_cid_number = test_issuer_cid_number();
        let issuer_main_account = test_issuer_main_account();
        let scope_city_name = test_scope_city_name();
        let cid_number: &[u8] = b"AH001-GCB07-000000001-2026";
        let register_nonce: public_manage::pallet::RegisterNonceOf<Runtime> =
            b"register-nonce-ah-1"
                .to_vec()
                .try_into()
                .expect("nonce should fit");
        let cid_full_name: public_manage::pallet::AccountNameOf<Runtime> = b"test-institution"
            .to_vec()
            .try_into()
            .expect("cid_full_name should fit");
        let account_names: Vec<Vec<u8>> = vec![b"main-account".to_vec(), b"fee-account".to_vec()];

        let make_signature = |signing_pair: &sr25519::Pair, admin_pubkey: &[u8; 32]| {
            let payload = (
                primitives::core_const::GMB,
                primitives::core_const::OP_SIGN_INST,
                frame_system::Pallet::<Runtime>::block_hash(0),
                cid_number,
                cid_full_name.as_slice(),
                &account_names,
                register_nonce.as_slice(),
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
            );
            let msg = blake2_256(&payload.encode());
            let sig = signing_pair.sign(&msg);
            let bounded: public_manage::pallet::RegisterSignatureOf<Runtime> =
                sig.0.to_vec().try_into().expect("signature should fit");
            bounded
        };

        let main_signature = make_signature(&main_pair, &main_admin_pubkey);
        assert!(
            <RuntimeCidInstitutionVerifier as entity_primitives::CidInstitutionVerifier<
                AccountId,
                public_manage::pallet::AccountNameOf<Runtime>,
                public_manage::pallet::RegisterNonceOf<Runtime>,
                public_manage::pallet::RegisterSignatureOf<Runtime>,
            >>::verify_institution_registration(
                cid_number,
                &cid_full_name,
                &account_names,
                &register_nonce,
                &main_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &main_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
            ),
            "main admin signature should pass"
        );

        let backup_signature = make_signature(&backup_pair, &backup_admin_pubkey);
        assert!(
            <RuntimeCidInstitutionVerifier as entity_primitives::CidInstitutionVerifier<
                AccountId,
                public_manage::pallet::AccountNameOf<Runtime>,
                public_manage::pallet::RegisterNonceOf<Runtime>,
                public_manage::pallet::RegisterSignatureOf<Runtime>,
            >>::verify_institution_registration(
                cid_number,
                &cid_full_name,
                &account_names,
                &register_nonce,
                &backup_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &backup_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
            ),
            "backup admin signature should pass"
        );

        let outsider_pair = sr25519::Pair::from_string("//outsider-inst", None).expect("pair");
        let outsider_pubkey = outsider_pair.public().0;
        let outsider_signature = make_signature(&outsider_pair, &outsider_pubkey);
        assert!(
            !<RuntimeCidInstitutionVerifier as entity_primitives::CidInstitutionVerifier<
                AccountId,
                public_manage::pallet::AccountNameOf<Runtime>,
                public_manage::pallet::RegisterNonceOf<Runtime>,
                public_manage::pallet::RegisterSignatureOf<Runtime>,
            >>::verify_institution_registration(
                cid_number,
                &cid_full_name,
                &account_names,
                &register_nonce,
                &outsider_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &outsider_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
            ),
            "outsider admin pubkey must reject"
        );

        let bad_signature: public_manage::pallet::RegisterSignatureOf<Runtime> =
            vec![9u8; 64].try_into().expect("signature should fit");
        assert!(
            !<RuntimeCidInstitutionVerifier as entity_primitives::CidInstitutionVerifier<
                AccountId,
                public_manage::pallet::AccountNameOf<Runtime>,
                public_manage::pallet::RegisterNonceOf<Runtime>,
                public_manage::pallet::RegisterSignatureOf<Runtime>,
            >>::verify_institution_registration(
                cid_number,
                &cid_full_name,
                &account_names,
                &register_nonce,
                &bad_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &main_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
            ),
            "tampered signature must reject"
        );
    });
}

// ============================================================================
// 簇 3:机构资金白名单允许矩阵(4 个用例)
// ============================================================================

#[test]
fn stake_account_is_completely_blocked() {
    let account = stake_account();
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigTransferExecute
    ));
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigCloseExecute
    ));
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainBatchDebit
    ));
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainFeeSweepExecute
    ));
}

#[test]
fn reserved_multisig_only_allows_transfer_and_close() {
    let account = reserved_main_account();
    assert!(RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigTransferExecute
    ));
    assert!(RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigCloseExecute
    ));
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainBatchDebit
    ));
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainFeeSweepExecute
    ));
}

#[test]
fn reserved_fee_account_only_allows_fee_sweep() {
    let account = reserved_fee_account();
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigTransferExecute
    ));
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigCloseExecute
    ));
    assert!(!RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainBatchDebit
    ));
    assert!(RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainFeeSweepExecute
    ));
}

#[test]
fn ordinary_account_allows_all_actions() {
    let account = ordinary_account();
    assert!(RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigTransferExecute
    ));
    assert!(RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::MultisigCloseExecute
    ));
    assert!(RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainBatchDebit
    ));
    assert!(RuntimeInstitutionAsset::can_spend(
        &account,
        InstitutionAssetAction::OffchainFeeSweepExecute
    ));
}
