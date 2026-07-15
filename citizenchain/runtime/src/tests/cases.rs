use super::*;
// 簇 1:Runtime 整体自检(4 个用例)
#[test]
fn time_and_currency_constants_are_consistent() {
    use frame_support::traits::Get;

    assert_eq!(YUAN, 100 * FEN);
    assert_eq!(UNIT, YUAN);
    assert_eq!(primitives::pow_const::POW_TARGET_BLOCK_TIME_MS, 360_000);
    let minimum_period: u64 = <Runtime as pallet_timestamp::Config>::MinimumPeriod::get();
    assert_eq!(minimum_period, 1);
}

#[test]
fn fee_payer_returns_none_for_transfer() {
    use configs::RuntimeFeePayerExtractor;
    use frame_support::BoundedVec;
    use onchain::CallFeePayer;
    use primitives::cid::china::china_cb::CHINA_CB;

    let institution = AccountId::new(CHINA_CB[0].main_account);
    let beneficiary = AccountId::new([99u8; 32]);
    let call = RuntimeCall::MultisigTransfer(multisig::pallet::Call::propose_transfer {
        institution_code: votingengine::types::NRC,
        institution,
        beneficiary,
        amount: 10000,
        remark: BoundedVec::default(),
    });
    let signer = AccountId::new([1u8; 32]);
    // 机构转账提案交易本身由提交者按投票统一价付费；
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
    let tags: [(&str, &[u8]); 9] = [
        ("grandpakey_change", grandpakey_change::MODULE_TAG),
        ("resolution_destroy", resolution_destroy::MODULE_TAG),
        ("resolution_issuance", resolution_issuance::MODULE_TAG),
        ("runtime_upgrade", runtime_upgrade::MODULE_TAG),
        ("public_manage", public_manage::MODULE_TAG),
        ("private_manage", private_manage::MODULE_TAG),
        ("personal_admins", personal_admins::MODULE_TAG),
        ("multisig", multisig::MODULE_TAG),
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
    assert_eq!(VERSION.system_version, 1);

    let _opaque_block_id: opaque::BlockId = generic::BlockId::Number(0);
    let _runtime_block_id: BlockId = generic::BlockId::Number(0);
}
// 簇 2:装配集成测试(18 个用例)
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

        assert_ok!(ResolutionDestroy::propose_destroy(
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
        // 投票判定与业务执行已解耦；当前区块维护钩子消费 PASSED 执行队列。
        let now = System::block_number();
        <VotingEngine as frame_support::traits::Hooks<BlockNumber>>::on_initialize(now);

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
        let free = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &system_call);
        assert_eq!(free, onchain::FeeChargeKind::Free);

        let transfer_call = RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
            dest: sp_runtime::MultiAddress::Id(recipient.clone()),
            value: 123,
        });
        let amount = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &transfer_call);
        assert_eq!(amount, onchain::FeeChargeKind::Unknown);

        let remark =
            frame_support::BoundedVec::<u8, frame_support::traits::ConstU32<99>>::try_from(
                b"ordinary transfer remark".to_vec(),
            )
            .expect("remark should fit");
        let transfer_with_remark_call =
            RuntimeCall::OnchainTransaction(onchain::pallet::Call::transfer_with_remark {
                beneficiary: recipient,
                amount: 456,
                remark,
            });
        let amount_with_remark = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &transfer_with_remark_call);
        assert_eq!(
            amount_with_remark,
            onchain::FeeChargeKind::OnchainAmount(456)
        );

        let internal_vote_call = RuntimeCall::InternalVote(internal_vote::pallet::Call::cast {
            proposal_id: 1,
            approve: true,
        });
        let vote_kind = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &internal_vote_call);
        // 投票 extrinsic 本身按治理用户操作固定 1 元计费，不再套 0.1%。
        assert_eq!(vote_kind, onchain::FeeChargeKind::VoteFlat);

        let nrc_institution = AccountId::new(CHINA_CB[0].main_account);
        let resolution_destro_call =
            RuntimeCall::ResolutionDestroy(resolution_destroy::pallet::Call::propose_destroy {
                institution_code: votingengine::types::NRC,
                institution: nrc_institution,
                amount: 456,
            });
        let resolution_kind = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &resolution_destro_call);
        assert_eq!(resolution_kind, onchain::FeeChargeKind::VoteFlat);

        let unknown_balances_call =
            RuntimeCall::Balances(pallet_balances::Call::upgrade_accounts {
                who: vec![AccountId::new([9u8; 32])],
            });
        let unknown = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &unknown_balances_call);
        assert_eq!(unknown, onchain::FeeChargeKind::Unknown);
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
        // 本测试验证提案交易本身按投票统一价，而不是按提案金额套链上费率。
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
        let create_kind = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &create_call);
        assert_eq!(create_kind, onchain::FeeChargeKind::VoteFlat);

        let _ = Balances::deposit_creating(&account, 777);
        // propose_close 已加注销凭证字段(register_nonce/signature/issuer_*/signer_pubkey);
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
        let close_kind = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &close_call);
        assert_eq!(close_kind, onchain::FeeChargeKind::VoteFlat);

        let institution =
            AccountId::new(primitives::cid::china::china_cb::CHINA_CB[0].main_account);
        let transfer_call =
            RuntimeCall::MultisigTransfer(multisig::pallet::Call::propose_transfer {
                institution_code: votingengine::types::NRC,
                institution,
                beneficiary: AccountId::new([79u8; 32]),
                amount: 88_888,
                remark: frame_support::BoundedVec::default(),
            });
        let transfer_kind = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &transfer_call);
        assert_eq!(transfer_kind, onchain::FeeChargeKind::VoteFlat);
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
fn runtime_call_filter_blocks_external_balances_calls() {
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

    let blocked_from_regular_account =
        RuntimeCall::Balances(pallet_balances::Call::force_transfer {
            source: sp_runtime::MultiAddress::Id(AccountId::new([8u8; 32])),
            dest: sp_runtime::MultiAddress::Id(dst),
            value: 1,
        });
    assert!(!RuntimeCallFilter::contains(&blocked_from_regular_account));

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

    let blocked_transfer_allow_death =
        RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
            dest: sp_runtime::MultiAddress::Id(AccountId::new([7u8; 32])),
            value: 1,
        });
    assert!(!RuntimeCallFilter::contains(&blocked_transfer_allow_death));

    let blocked_transfer_keep_alive =
        RuntimeCall::Balances(pallet_balances::Call::transfer_keep_alive {
            dest: sp_runtime::MultiAddress::Id(AccountId::new([7u8; 32])),
            value: 1,
        });
    assert!(!RuntimeCallFilter::contains(&blocked_transfer_keep_alive));

    let blocked_transfer_all = RuntimeCall::Balances(pallet_balances::Call::transfer_all {
        dest: sp_runtime::MultiAddress::Id(AccountId::new([7u8; 32])),
        keep_alive: true,
    });
    assert!(!RuntimeCallFilter::contains(&blocked_transfer_all));

    let blocked_burn = RuntimeCall::Balances(pallet_balances::Call::burn {
        value: 1,
        keep_alive: true,
    });
    assert!(!RuntimeCallFilter::contains(&blocked_burn));

    let remark = frame_support::BoundedVec::<u8, frame_support::traits::ConstU32<99>>::try_from(
        b"ordinary transfer remark".to_vec(),
    )
    .expect("remark should fit");
    let allowed_onchain_transfer =
        RuntimeCall::OnchainTransaction(onchain::pallet::Call::transfer_with_remark {
            beneficiary: AccountId::new([7u8; 32]),
            amount: 1,
            remark,
        });
    assert!(RuntimeCallFilter::contains(&allowed_onchain_transfer));
}

#[test]
fn pow_digest_author_finds_pow_engine_author() {
    // pre_digest 现在存储 sr25519 公钥，PowDigestAuthor 解码后派生 AccountId。
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
            expected_pow_params_hash: Default::default(),
            new_pow_params: Default::default(),
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
        votingengine::CallbackExecutionScopes::<Runtime>::insert(proposal_id, ());
        let outcome = RuntimeJointVoteResultCallback::on_joint_vote_finalized(proposal_id, false)
            .expect("runtime-upgrade callback should succeed");
        votingengine::CallbackExecutionScopes::<Runtime>::remove(proposal_id);
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
        // 占号先行:身份写入前置。
        assert_ok!(CitizenIdentity::occupy_cid(
            RuntimeOrigin::signed(registrar.clone()),
            registrar_account.clone(),
            real_cid_number("RUNTIME-0001", "CTZN", "1")
                .try_into()
                .expect("cid number should fit"),
            [7u8; 32],
            b"43".to_vec().try_into().expect("province should fit"),
            b"4301".to_vec().try_into().expect("city should fit"),
        ));
        let payload = build_voting_identity_payload(
            wallet_account.clone(),
            &real_cid_number("RUNTIME-0001", "CTZN", "1"),
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
            &real_cid_number("RUNTIME-0002", "CTZN", "1"),
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
        // 占号先行:身份写入前置。
        assert_ok!(CitizenIdentity::occupy_cid(
            RuntimeOrigin::signed(registrar.clone()),
            registrar_account.clone(),
            real_cid_number("RUNTIME-0003", "CTZN", "1")
                .try_into()
                .expect("cid number should fit"),
            [7u8; 32],
            b"43".to_vec().try_into().expect("province should fit"),
            b"4301".to_vec().try_into().expect("city should fit"),
        ));
        let voting = build_voting_identity_payload(
            wallet_account.clone(),
            &real_cid_number("RUNTIME-0003", "CTZN", "1"),
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
            citizen_sex: citizen_identity::CitizenSex::Male,
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
fn runtime_square_post_normal_publish_allows_visitor_wallet() {
    new_test_ext().execute_with(|| {
        let visitor = AccountId::new([42u8; 32]);
        assert_ok!(SquarePost::publish_post(
            RuntimeOrigin::signed(visitor.clone()),
            b"sqp_runtime_normal".to_vec(),
            square_post::SquarePostCategory::Normal,
            [3u8; 32],
            b"sqr_runtime_normal".to_vec(),
            1_893_456_000_000,
        ));

        let stored_post_id: square_post::PostIdOf<Runtime> = b"sqp_runtime_normal"
            .to_vec()
            .try_into()
            .expect("post id fits");
        let stored = square_post::SquarePosts::<Runtime>::get(stored_post_id)
            .expect("square post should be indexed");
        assert_eq!(stored.owner_account, visitor);
        assert_eq!(stored.cid_number, None);
        assert_eq!(
            stored.post_category,
            square_post::SquarePostCategory::Normal
        );
    });
}

#[test]
fn runtime_square_post_campaign_requires_citizen_identity() {
    new_test_ext().execute_with(|| {
        let visitor = AccountId::new([43u8; 32]);
        assert_noop!(
            SquarePost::publish_post(
                RuntimeOrigin::signed(visitor),
                b"sqp_runtime_campaign_denied".to_vec(),
                square_post::SquarePostCategory::Campaign,
                [4u8; 32],
                b"sqr_runtime_campaign_denied".to_vec(),
                1_893_456_000_000,
            ),
            square_post::Error::<Runtime>::CampaignRequiresCitizen
        );
    });
}

#[test]
fn runtime_square_post_campaign_records_chain_cid_for_verified_wallet() {
    new_test_ext().execute_with(|| {
        let (_, registrar, registrar_account) = setup_frg_citizen_identity_admin(b"43");
        let wallet_pair =
            sr25519::Pair::from_string("//square-citizen-wallet", None).expect("wallet pair");
        let wallet_account = AccountId::new(wallet_pair.public().0);
        let cid_number = real_cid_number("SQUARE-0001", "CTZN", "1");

        assert_ok!(CitizenIdentity::occupy_cid(
            RuntimeOrigin::signed(registrar.clone()),
            registrar_account.clone(),
            cid_number
                .clone()
                .try_into()
                .expect("cid number should fit"),
            [8u8; 32],
            b"43".to_vec().try_into().expect("province should fit"),
            b"4301".to_vec().try_into().expect("city should fit"),
        ));
        let payload = build_voting_identity_payload(
            wallet_account.clone(),
            &cid_number,
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
        assert_ok!(SquarePost::publish_post(
            RuntimeOrigin::signed(wallet_account.clone()),
            b"sqp_runtime_campaign_ok".to_vec(),
            square_post::SquarePostCategory::Campaign,
            [5u8; 32],
            b"sqr_runtime_campaign_ok".to_vec(),
            1_893_456_000_000,
        ));

        let stored_post_id: square_post::PostIdOf<Runtime> = b"sqp_runtime_campaign_ok"
            .to_vec()
            .try_into()
            .expect("post id fits");
        let stored = square_post::SquarePosts::<Runtime>::get(stored_post_id)
            .expect("square post should be indexed");
        assert_eq!(stored.owner_account, wallet_account);
        assert_eq!(
            stored.cid_number.map(|value| value.to_vec()),
            Some(cid_number)
        );
        assert_eq!(
            square_post::PublishedPostCountByAccount::<Runtime>::get(wallet_account),
            1
        );
    });
}

#[test]
fn runtime_square_post_fee_kind_uses_onchain_minimum_fee() {
    new_test_ext().execute_with(|| {
        let who = AccountId::new([44u8; 32]);
        let call = RuntimeCall::SquarePost(square_post::pallet::Call::publish_post {
            post_id: b"sqp_fee_kind".to_vec(),
            post_category: square_post::SquarePostCategory::Normal,
            content_hash: [6u8; 32],
            storage_receipt_id: b"sqr_fee_kind".to_vec(),
            storage_until: 1_893_456_000_000,
        });
        let fee_kind = <RuntimeFeeKindClassifier as onchain::CallFeeKind<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_kind(&who, &call);
        assert_eq!(fee_kind, onchain::FeeChargeKind::OnchainAmount(0));
        assert_eq!(
            onchain::calculate_onchain_fee(0),
            primitives::fee_policy::ONCHAIN_MIN_FEE
        );
        assert_eq!(primitives::fee_policy::ONCHAIN_MIN_FEE, 10);
        // 广场降费不得改变投票和治理类统一费用。
        assert_eq!(primitives::fee_policy::VOTE_FLAT_FEE, YUAN);
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
        let cid_number_owned = real_cid_number("verifier-lookup", "CGOV", "0");
        let cid_number: &[u8] = cid_number_owned.as_slice();
        let register_nonce: public_manage::pallet::RegisterNonceOf<Runtime> =
            b"register-nonce-ah-1"
                .to_vec()
                .try_into()
                .expect("nonce should fit");
        let cid_full_name: public_manage::pallet::AccountNameOf<Runtime> = b"test-institution"
            .to_vec()
            .try_into()
            .expect("cid_full_name should fit");
        let cid_short_name: &[u8] = b"test-inst";
        let account_names: Vec<Vec<u8>> = vec![b"main-account".to_vec(), b"fee-account".to_vec()];
        let town_code: &[u8] = b"";
        let legal_representative_name: &[u8] = "测试代表".as_bytes();
        let legal_representative_cid_number: &[u8] = b"CID-LEGAL-REP-001";
        let legal_representative_account = AccountId::new([77u8; 32]);

        let make_signature = |signing_pair: &sr25519::Pair, admin_pubkey: &[u8; 32]| {
            let payload = (
                frame_system::Pallet::<Runtime>::block_hash(0),
                cid_number,
                cid_full_name.as_slice(),
                cid_short_name,
                &account_names,
                register_nonce.as_slice(),
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
                town_code,
            );
            // 注册凭证与生产 verifier 共用 signing_message 域，禁止测试保留旧双域头。
            let msg = primitives::sign::signing_message(
                primitives::sign::OP_SIGN_INST,
                &payload.encode(),
            );
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
                cid_short_name,
                &account_names,
                &register_nonce,
                &main_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &main_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
                town_code,
            ),
            "main admin signature should pass"
        );

        let creation_payload = (
            frame_system::Pallet::<Runtime>::block_hash(0),
            cid_number,
            cid_full_name.as_slice(),
            cid_short_name,
            legal_representative_name,
            legal_representative_cid_number,
            &legal_representative_account,
            &account_names,
            b"test-roles".as_slice(),
            b"test-assignments".as_slice(),
            register_nonce.as_slice(),
            issuer_cid_number.as_slice(),
            &issuer_main_account,
            &main_admin_pubkey,
            province_bytes.as_slice(),
            scope_city_name.as_slice(),
            town_code,
        );
        let creation_signature: public_manage::pallet::RegisterSignatureOf<Runtime> = main_pair
            .sign(&primitives::sign::signing_message(
                primitives::sign::OP_SIGN_INST,
                &creation_payload.encode(),
            ))
            .0
            .to_vec()
            .try_into()
            .expect("creation signature should fit");
        assert!(
            <RuntimeCidInstitutionVerifier as entity_primitives::CidInstitutionVerifier<
                AccountId,
                public_manage::pallet::AccountNameOf<Runtime>,
                public_manage::pallet::RegisterNonceOf<Runtime>,
                public_manage::pallet::RegisterSignatureOf<Runtime>,
            >>::verify_institution_creation(
                cid_number,
                &cid_full_name,
                cid_short_name,
                legal_representative_name,
                legal_representative_cid_number,
                &legal_representative_account,
                &account_names,
                b"test-roles".as_slice(),
                b"test-assignments".as_slice(),
                &register_nonce,
                &creation_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &main_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
                town_code,
            ),
            "institution creation signature covering legal representative fields should pass"
        );
        assert!(
            !<RuntimeCidInstitutionVerifier as entity_primitives::CidInstitutionVerifier<
                AccountId,
                public_manage::pallet::AccountNameOf<Runtime>,
                public_manage::pallet::RegisterNonceOf<Runtime>,
                public_manage::pallet::RegisterSignatureOf<Runtime>,
            >>::verify_institution_creation(
                cid_number,
                &cid_full_name,
                cid_short_name,
                legal_representative_name,
                legal_representative_cid_number,
                &AccountId::new([78u8; 32]),
                &account_names,
                b"test-roles".as_slice(),
                b"test-assignments".as_slice(),
                &register_nonce,
                &creation_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &main_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
                town_code,
            ),
            "tampered legal representative account must reject"
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
                cid_short_name,
                &account_names,
                &register_nonce,
                &backup_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &backup_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
                town_code,
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
                cid_short_name,
                &account_names,
                &register_nonce,
                &outsider_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &outsider_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
                town_code,
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
                cid_short_name,
                &account_names,
                &register_nonce,
                &bad_signature,
                issuer_cid_number.as_slice(),
                &issuer_main_account,
                &main_admin_pubkey,
                province_bytes.as_slice(),
                scope_city_name.as_slice(),
                town_code,
            ),
            "tampered signature must reject"
        );
    });
}
// 簇 3:机构资金白名单允许矩阵(4 个用例)
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

// ── 创世直铸全量断言(ADR-031 卡3 验收)──

/// 创世直铸当前国家/省/市骨架:常量 296 + 模板派生 49,297 = 49,593,零交易;
/// 镇行政区公权机构不进创世,运行期由注册局按 town_code 注册上链。
/// 并抽查派生首条与链上登记逐字节一致、新补国家机构入链、NJD 创世管理员在位。
#[test]
fn genesis_public_institutions_full_mint_counts() {
    new_test_ext().execute_with(|| {
        genesis_pallet::institution::build::<Runtime>();

        let builtin_count = primitives::cid::china::china_cb::CHINA_CB.len()
            + primitives::cid::china::china_ch::CHINA_CH.len()
            + primitives::cid::china::china_zf::CHINA_ZF.len()
            + primitives::cid::china::china_jc::CHINA_JC.len()
            + primitives::cid::china::china_sf::CHINA_SF.len()
            + primitives::cid::china::china_lf::CHINA_LF.len()
            + primitives::cid::china::china_jy::CHINA_JY.len();
        let total = public_manage::Institutions::<Runtime>::iter().count();
        assert_eq!(
            total,
            builtin_count + primitives::cid::official_derive::public_institution_derived_count()
        );
        assert_eq!(builtin_count, 296);

        // 抽查:派生枚举首条必须与链上登记逐字节一致。
        let mut first: Option<(Vec<u8>, Vec<u8>, Vec<u8>)> = None;
        primitives::cid::official_derive::for_each_public_institution(|cid, full, short| {
            if first.is_none() {
                first = Some((
                    cid.as_bytes().to_vec(),
                    full.as_bytes().to_vec(),
                    short.as_bytes().to_vec(),
                ));
            }
        });
        let (cid, full, short) = first.expect("derived set non-empty");
        let bounded: frame_support::BoundedVec<u8, _> = cid.try_into().expect("cid fits");
        let info = public_manage::Institutions::<Runtime>::get(&bounded).expect("derived minted");
        assert_eq!(info.cid_full_name.to_vec(), full);
        assert_eq!(info.cid_short_name.to_vec(), short);

        // 抽查:本任务新增的国家创世机构必须逐个入链,且机构码与 CID 段一致。
        let mut new_national_count = 0usize;
        for node in primitives::cid::china::china_zf::CHINA_ZF.iter().filter(|node| {
            matches!(
                primitives::cid::code::institution_code_from_cid_number(node.cid_number),
                Some(code)
                    if matches!(
                        primitives::cid::code::institution_code_text(&code),
                        Some("ARM" | "NAV" | "AIR" | "SPF" | "JOS" | "ARC" | "NVC" | "AFC" | "SFC" | "NGB" | "NGC" | "FDA")
                    )
            )
        }) {
            new_national_count += 1;
            let bounded: frame_support::BoundedVec<u8, _> = node
                .cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("new genesis cid fits");
            let info =
                public_manage::Institutions::<Runtime>::get(&bounded).expect("new genesis minted");
            assert_eq!(info.cid_full_name.to_vec(), node.cid_full_name.as_bytes());
            assert_eq!(info.cid_short_name.to_vec(), node.cid_short_name.as_bytes());
            assert_eq!(
                info.institution_code,
                primitives::cid::code::institution_code_from_cid_number(node.cid_number)
                    .expect("new genesis cid code")
            );
            assert!(
                RuntimeReservedAccountGuard::is_reserved(&AccountId::new(node.main_account)),
                "{} 主账户必须进入制度保留地址表",
                node.cid_short_name
            );
            assert!(
                RuntimeReservedAccountGuard::is_reserved(&AccountId::new(node.fee_account)),
                "{} 费用账户必须进入制度保留地址表",
                node.cid_short_name
            );
        }
        assert_eq!(new_national_count, 12);

        // NJD 创世管理员在位(常量特例保留)。
        let njd = primitives::cid::china::china_sf::CHINA_SF
            .iter()
            .find(|n| {
                primitives::cid::code::institution_code_from_cid_number(n.cid_number)
                    == Some(primitives::cid::code::NJD)
            })
            .expect("NJD in china_sf");
        let njd_main: AccountId =
            codec::Decode::decode(&mut njd.main_account.as_slice()).expect("decode");
        assert!(public_admins::AdminAccounts::<Runtime>::get(njd_main).is_some());
    });
}

/// 六个国家级单例在 block#0 精确占用约定身份；三个成员机构保持完整的“尚未组成”状态。
#[test]
fn genesis_national_singletons_exist_and_member_bodies_are_unconstituted() {
    new_test_ext().execute_with(|| {
        genesis_pallet::institution::build::<Runtime>();
        for singleton in primitives::institution_constraints::singleton_institutions() {
            let cid: public_manage::pallet::CidNumberOf<Runtime> = singleton
                .cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("singleton CID fits");
            let info = public_manage::Institutions::<Runtime>::get(&cid)
                .expect("national singleton exists at block zero");
            assert_eq!(info.institution_code, singleton.code);
            assert_eq!(
                info.status,
                entity_primitives::InstitutionLifecycleStatus::Active
            );
            let main_name: public_manage::pallet::AccountNameOf<Runtime> =
                primitives::account_derive::RESERVED_NAME_MAIN
                    .to_vec()
                    .try_into()
                    .expect("main name fits");
            assert_eq!(
                public_manage::CidRegisteredAccount::<Runtime>::get(&cid, main_name),
                Some(AccountId::new(singleton.main_account))
            );
            let main = AccountId::new(singleton.main_account);
            assert!(public_admins::AdminAccounts::<Runtime>::get(main.clone()).is_none());
            assert!(
                internal_vote::ActiveDynamicThresholds::<Runtime>::get(singleton.code, main)
                    .is_none()
            );
        }

        for spec in primitives::institution_constraints::member_composition_specs() {
            let cid: public_manage::pallet::CidNumberOf<Runtime> = spec
                .institution
                .cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("member body CID fits");
            let role_code: public_manage::RoleCodeOf = spec
                .role_code
                .to_vec()
                .try_into()
                .expect("member role code fits");
            assert!(public_manage::InstitutionRoles::<Runtime>::get(&cid, &role_code).is_none());
            assert!(
                public_manage::InstitutionRoleAssignments::<Runtime>::get(&cid, &role_code)
                    .is_empty()
            );
            assert!(public_admins::AdminAccounts::<Runtime>::get(AccountId::new(
                spec.institution.main_account
            ))
            .is_none());
        }
    });
}

/// 参议会首次组成必须原子满足法定岗位、人数区间和 admins 闭环，之后不得跌破下限。
#[test]
fn national_member_body_first_composition_and_permanent_range_are_enforced() {
    new_test_ext().execute_with(|| {
        genesis_pallet::institution::build::<Runtime>();
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        let main = AccountId::new(spec.institution.main_account);
        let members = |count: u32| {
            (0..count)
                .map(|index| {
                    let mut raw = [0u8; 32];
                    raw[..4].copy_from_slice(&index.to_le_bytes());
                    AccountId::new(raw)
                })
                .collect::<Vec<_>>()
        };
        let result = |accounts: Vec<AccountId>, include_role: bool| {
            entity_primitives::InstitutionGovernanceResult {
                institution_code: spec.institution.code,
                institution_account: main.clone(),
                role_changes: include_role
                    .then(|| {
                        vec![entity_primitives::InstitutionRoleChange {
                            role_code: spec.role_code.to_vec(),
                            role_name: spec.role_name.to_vec(),
                            term_required: false,
                            role_status: entity_primitives::InstitutionRoleStatus::Active,
                        }]
                    })
                    .unwrap_or_default(),
                assignment_changes: vec![entity_primitives::InstitutionRoleAssignmentChange {
                    role_code: spec.role_code.to_vec(),
                    assignments: accounts
                        .into_iter()
                        .map(
                            |admin_account| entity_primitives::InstitutionAssignmentTarget {
                                admin_account,
                                term_start: 0,
                                term_end: 0,
                                assignment_source:
                                    entity_primitives::InstitutionAssignmentSource::PopularElection,
                                assignment_source_ref: b"national-election".to_vec(),
                                assignment_status:
                                    entity_primitives::InstitutionAssignmentStatus::Active,
                            },
                        )
                        .collect(),
                }],
                legal_representative_change: None,
                result_source_ref: b"national-election".to_vec(),
            }
        };

        assert_noop!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(
                members(spec.min_members - 1),
                true,
            )),
            public_manage::Error::<Runtime>::RequiredMemberCountOutOfRange
        );
        assert_ok!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(
                members(spec.min_members),
                true,
            ))
        );
        let account = public_admins::AdminAccounts::<Runtime>::get(main.clone())
            .expect("first composition creates admins");
        assert_eq!(account.admins.len() as u32, spec.min_members);
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Runtime>::get(
                spec.institution.code,
                main.clone()
            ),
            None
        );
        let proposal_id = internal_vote::Pallet::<Runtime>::do_create_general_internal_proposal(
            members(1)[0].clone(),
            spec.institution.code,
            main.clone(),
            vec![spec.institution.cid_number.as_bytes().to_vec()],
        )
        .expect("composed singleton can create internal proposal");
        assert_eq!(
            internal_vote::InternalThresholdSnapshot::<Runtime>::get(proposal_id),
            Some(spec.min_members / 2 + 1)
        );
        use entity_primitives::InstitutionMultisigQuery;
        assert_eq!(
            public_manage::Pallet::<Runtime>::lookup_admin_config(&main)
                .expect("composed singleton exposes current admin snapshot")
                .threshold,
            spec.min_members / 2 + 1
        );

        assert_noop!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(
                members(spec.min_members - 1),
                false,
            )),
            public_manage::Error::<Runtime>::RequiredMemberCountOutOfRange
        );
        assert_eq!(
            public_admins::AdminAccounts::<Runtime>::get(main)
                .expect("failed change rolls back")
                .admins
                .len() as u32,
            spec.min_members
        );
    });
}

/// 立法院、监察院、总统府创世只占用唯一身份，首次治理结果再原子组成岗位、任职和 admins。
#[test]
fn national_singletons_without_member_ranges_can_be_composed_once() {
    new_test_ext().execute_with(|| {
        genesis_pallet::institution::build::<Runtime>();
        for singleton in primitives::institution_constraints::singleton_institutions()
            .into_iter()
            .filter(|item| {
                matches!(
                    item.code,
                    primitives::cid::code::NLG
                        | primitives::cid::code::NSP
                        | primitives::cid::code::PRS
                )
            })
        {
            let main = AccountId::new(singleton.main_account);
            let role_code_raw = b"RUNTIME_MEMBER".to_vec();
            let admins = vec![AccountId::new([91u8; 32]), AccountId::new([92u8; 32])];
            let result = entity_primitives::InstitutionGovernanceResult {
                institution_code: singleton.code,
                institution_account: main.clone(),
                role_changes: vec![entity_primitives::InstitutionRoleChange {
                    role_code: role_code_raw.clone(),
                    role_name: "运行期成员".as_bytes().to_vec(),
                    term_required: false,
                    role_status: entity_primitives::InstitutionRoleStatus::Active,
                }],
                assignment_changes: vec![entity_primitives::InstitutionRoleAssignmentChange {
                    role_code: role_code_raw.clone(),
                    assignments: admins
                        .iter()
                        .cloned()
                        .map(|admin_account| entity_primitives::InstitutionAssignmentTarget {
                            admin_account,
                            term_start: 0,
                            term_end: 0,
                            assignment_source:
                                entity_primitives::InstitutionAssignmentSource::NominationAppointment,
                            assignment_source_ref: b"first-composition".to_vec(),
                            assignment_status:
                                entity_primitives::InstitutionAssignmentStatus::Active,
                        })
                        .collect(),
                }],
                legal_representative_change: None,
                result_source_ref: b"first-composition".to_vec(),
            };

            assert_ok!(
                public_manage::Pallet::<Runtime>::apply_institution_governance_result(result)
            );
            let account = public_admins::AdminAccounts::<Runtime>::get(main.clone())
                .expect("first composition creates admins");
            assert_eq!(account.admins.to_vec(), admins);
            assert_eq!(
                internal_vote::ActiveDynamicThresholds::<Runtime>::get(singleton.code, main),
                None
            );
        }
    });
}

/// 89 个受保护创世机构的岗位、席位和任职必须由 genesis 构建直接写入 entity/admins。
#[test]
fn genesis_fixed_institution_roles_and_assignments_are_complete() {
    new_test_ext().execute_with(|| {
        let role_code = |raw: &[u8]| -> public_manage::RoleCodeOf {
            raw.to_vec().try_into().expect("fixed role code fits")
        };
        let cid = |raw: &str| -> public_manage::pallet::CidNumberOf<Runtime> {
            raw.as_bytes().to_vec().try_into().expect("fixed cid fits")
        };

        // 法定代表人不是创世必填项。固定机构创世时三字段必须保持全空，
        // 不得从管理员首位、机构主账户或其它钱包推导占位值。
        for institution in primitives::governance_skeleton::fixed_institutions() {
            let info = public_manage::Institutions::<Runtime>::get(cid(institution.cid_number))
                .expect("fixed genesis institution exists");
            assert!(info.legal_representative_name.is_none());
            assert!(info.legal_representative_cid_number.is_none());
            assert!(info.legal_representative_account.is_none());
        }

        // 国家储委会、省储委会统一为“委员”。
        for node in primitives::cid::china::china_cb::CHINA_CB.iter() {
            let cid_number = cid(node.cid_number);
            let code = role_code(primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER);
            let role = public_manage::InstitutionRoles::<Runtime>::get(&cid_number, &code)
                .expect("committee role exists");
            assert_eq!(role.cid_number, cid_number);
            assert_eq!(role.role_name.as_slice(), "委员".as_bytes());
            assert!(!role.term_required);
            assert_eq!(
                role.role_status,
                entity_primitives::InstitutionRoleStatus::Active
            );
            let assignments =
                public_manage::InstitutionRoleAssignments::<Runtime>::get(&cid_number, &code);
            assert_eq!(assignments.len(), node.admins.len());
            assert_eq!(
                assignments
                    .iter()
                    .map(|assignment| assignment.admin_account.clone())
                    .collect::<Vec<_>>(),
                node.admins
                    .iter()
                    .copied()
                    .map(AccountId::new)
                    .collect::<Vec<_>>()
            );
            assert!(assignments.iter().all(|assignment| {
                assignment.assignment_source
                    == entity_primitives::InstitutionAssignmentSource::Genesis
                    && assignment.assignment_source_ref.is_empty()
                    && assignment.assignment_status
                        == entity_primitives::InstitutionAssignmentStatus::Active
                    && assignment.term_start == 0
                    && assignment.term_end == 0
                    && assignment.cid_number == cid_number
                    && assignment.role_code == code
            }));

            let admin_account =
                public_admins::AdminAccounts::<Runtime>::get(AccountId::new(node.main_account))
                    .expect("committee admin account exists");
            assert_eq!(
                admin_account.cid_number.as_slice(),
                node.cid_number.as_bytes()
            );
            assert_eq!(
                admin_account.institution_code,
                primitives::cid::code::institution_code_from_cid_number(node.cid_number)
                    .expect("committee code")
            );
            assert_eq!(
                admin_account.status,
                admin_primitives::AdminAccountStatus::Active
            );
            assert_eq!(
                admin_account.admins.into_inner(),
                node.admins
                    .iter()
                    .copied()
                    .map(AccountId::new)
                    .collect::<Vec<_>>()
            );
        }

        // 省储行为“董事”。
        for node in primitives::cid::china::china_ch::CHINA_CH.iter() {
            let cid_number = cid(node.cid_number);
            let code = role_code(primitives::governance_skeleton::ROLE_CODE_DIRECTOR);
            let role = public_manage::InstitutionRoles::<Runtime>::get(&cid_number, &code)
                .expect("director role exists");
            assert_eq!(role.role_name.as_slice(), "董事".as_bytes());
            assert!(!role.term_required);
            assert_eq!(
                role.role_status,
                entity_primitives::InstitutionRoleStatus::Active
            );
            let assignments =
                public_manage::InstitutionRoleAssignments::<Runtime>::get(&cid_number, &code);
            assert_eq!(assignments.len(), node.admins.len());
            assert_eq!(
                assignments
                    .iter()
                    .map(|assignment| assignment.admin_account.clone())
                    .collect::<Vec<_>>(),
                node.admins
                    .iter()
                    .copied()
                    .map(AccountId::new)
                    .collect::<Vec<_>>()
            );
            assert!(assignments.iter().all(|assignment| {
                assignment.assignment_source
                    == entity_primitives::InstitutionAssignmentSource::Genesis
                    && assignment.assignment_source_ref.is_empty()
                    && assignment.assignment_status
                        == entity_primitives::InstitutionAssignmentStatus::Active
                    && assignment.term_start == 0
                    && assignment.term_end == 0
                    && assignment.cid_number == cid_number
                    && assignment.role_code == code
            }));
            let admin_account =
                public_admins::AdminAccounts::<Runtime>::get(AccountId::new(node.main_account))
                    .expect("director admin account exists");
            assert_eq!(
                admin_account.admins.into_inner(),
                node.admins
                    .iter()
                    .copied()
                    .map(AccountId::new)
                    .collect::<Vec<_>>()
            );
        }

        // 国家司法院固定 7 护宪、1 首席、2 次席、5 大法官。
        let njd = primitives::cid::china::china_sf::CHINA_SF
            .iter()
            .find(|node| {
                primitives::cid::code::institution_code_from_cid_number(node.cid_number)
                    == Some(primitives::cid::code::NJD)
            })
            .expect("NJD genesis node exists");
        let njd_cid = cid(njd.cid_number);
        for (raw_code, seats) in [
            (
                primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD,
                7usize,
            ),
            (primitives::governance_skeleton::ROLE_CODE_CHIEF_JUSTICE, 1),
            (
                primitives::governance_skeleton::ROLE_CODE_DEPUTY_CHIEF_JUSTICE,
                2,
            ),
            (primitives::governance_skeleton::ROLE_CODE_JUSTICE, 5),
        ] {
            let code = role_code(raw_code);
            let spec =
                primitives::governance_skeleton::fixed_role_specs(primitives::cid::code::NJD)
                    .into_iter()
                    .find(|spec| spec.role_code == raw_code)
                    .expect("NJD role spec exists");
            let role = public_manage::InstitutionRoles::<Runtime>::get(&njd_cid, &code)
                .expect("NJD role exists");
            assert_eq!(role.role_name.as_slice(), spec.role_name);
            assert!(!role.term_required);
            assert_eq!(
                role.role_status,
                entity_primitives::InstitutionRoleStatus::Active
            );
            let assignments =
                public_manage::InstitutionRoleAssignments::<Runtime>::get(&njd_cid, &code);
            assert_eq!(assignments.len(), seats);
            assert!(assignments.iter().all(|assignment| {
                assignment.assignment_source
                    == entity_primitives::InstitutionAssignmentSource::Genesis
                    && assignment.assignment_source_ref.is_empty()
                    && assignment.assignment_status
                        == entity_primitives::InstitutionAssignmentStatus::Active
                    && assignment.term_start == 0
                    && assignment.term_end == 0
                    && assignment.cid_number == njd_cid
                    && assignment.role_code == code
            }));
        }
        let njd_accounts = [
            primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD,
            primitives::governance_skeleton::ROLE_CODE_CHIEF_JUSTICE,
            primitives::governance_skeleton::ROLE_CODE_DEPUTY_CHIEF_JUSTICE,
            primitives::governance_skeleton::ROLE_CODE_JUSTICE,
        ]
        .into_iter()
        .flat_map(|raw_code| {
            public_manage::InstitutionRoleAssignments::<Runtime>::get(&njd_cid, role_code(raw_code))
                .into_iter()
                .map(|assignment| assignment.admin_account)
        })
        .collect::<Vec<_>>();
        assert_eq!(
            njd_accounts,
            primitives::cid::china::china_sf::NATIONAL_JUDICIAL_YUAN_ADMINS
                .iter()
                .copied()
                .map(AccountId::new)
                .collect::<Vec<_>>()
        );

        // 联邦注册局是一个机构、43 个省专员岗位，每岗位固定 5 人。
        let frg = primitives::cid::china::china_zf::CHINA_ZF
            .iter()
            .find(|node| {
                primitives::cid::code::institution_code_from_cid_number(node.cid_number)
                    == Some(primitives::cid::code::FRG)
            })
            .expect("FRG genesis node exists");
        let frg_cid = cid(frg.cid_number);
        let mut frg_assignments = 0usize;
        for (province_index, province) in primitives::cid::code::PROVINCE_CODE_INFOS
            .iter()
            .enumerate()
        {
            let code = role_code(
                &primitives::governance_skeleton::province_commissioner_role_code(
                    province.province_code,
                ),
            );
            let role = public_manage::InstitutionRoles::<Runtime>::get(&frg_cid, &code)
                .expect("FRG province commissioner role exists");
            assert_eq!(
                role.role_name.as_slice(),
                primitives::governance_skeleton::province_commissioner_role_name(
                    province.province_name,
                )
            );
            assert!(!role.term_required);
            assert_eq!(
                role.role_status,
                entity_primitives::InstitutionRoleStatus::Active
            );
            let assignments =
                public_manage::InstitutionRoleAssignments::<Runtime>::get(&frg_cid, &code);
            assert_eq!(
                assignments.len(),
                primitives::count_const::FRG_PROVINCE_GROUP_ADMIN_COUNT as usize
            );
            let group_size = primitives::count_const::FRG_PROVINCE_GROUP_ADMIN_COUNT as usize;
            let start = province_index * group_size;
            let expected = primitives::cid::china::china_zf::FEDERAL_REGISTRY_ADMINS
                [start..start + group_size]
                .iter()
                .copied()
                .map(AccountId::new)
                .collect::<Vec<_>>();
            assert_eq!(
                assignments
                    .iter()
                    .map(|assignment| assignment.admin_account.clone())
                    .collect::<Vec<_>>(),
                expected
            );
            assert!(assignments.iter().all(|assignment| {
                assignment.assignment_source
                    == entity_primitives::InstitutionAssignmentSource::Genesis
                    && assignment.assignment_source_ref.is_empty()
                    && assignment.assignment_status
                        == entity_primitives::InstitutionAssignmentStatus::Active
                    && assignment.term_start == 0
                    && assignment.term_end == 0
                    && assignment.cid_number == frg_cid
                    && assignment.role_code == code
            }));
            frg_assignments += assignments.len();
        }
        assert_eq!(
            frg_assignments,
            primitives::cid::china::china_zf::FEDERAL_REGISTRY_ADMINS.len()
        );
        let frg_admin_account =
            public_admins::AdminAccounts::<Runtime>::get(AccountId::new(frg.main_account))
                .expect("FRG admin account exists");
        assert_eq!(frg_admin_account.admins.len(), frg_assignments);
    });
}

/// runtime 必须把选举结果路由到 public-manage，并在写入前拒绝固定岗位席位漂移。
#[test]
fn runtime_governance_result_router_enforces_fixed_role_seats() {
    new_test_ext().execute_with(|| {
        let njd = primitives::cid::china::china_sf::CHINA_SF
            .iter()
            .find(|node| {
                primitives::cid::code::institution_code_from_cid_number(node.cid_number)
                    == Some(primitives::cid::code::NJD)
            })
            .expect("NJD genesis node exists");
        let main = AccountId::new(njd.main_account);
        let result_source_ref = 700u64.encode();
        let result = |accounts: Vec<AccountId>| {
            let assignments = accounts
                .into_iter()
                .map(|admin_account| entity_primitives::InstitutionAssignmentTarget {
                    admin_account,
                    term_start: 0,
                    term_end: 0,
                    assignment_source:
                        entity_primitives::InstitutionAssignmentSource::MutualElection,
                    assignment_source_ref: result_source_ref.clone(),
                    assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
                })
                .collect();
            entity_primitives::InstitutionGovernanceResult {
                institution_code: primitives::cid::code::NJD,
                institution_account: main.clone(),
                role_changes: vec![],
                assignment_changes: vec![
                    entity_primitives::InstitutionRoleAssignmentChange {
                        role_code:
                            primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD
                                .to_vec(),
                        assignments,
                    },
                ],
                legal_representative_change: None,
                result_source_ref: result_source_ref.clone(),
            }
        };

        let mut role_change = result(vec![]);
        role_change.role_changes = vec![entity_primitives::InstitutionRoleChange {
            role_code: primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD.to_vec(),
            role_name: "修改固定岗位".as_bytes().to_vec(),
            term_required: false,
            role_status: entity_primitives::InstitutionRoleStatus::Active,
        }];
        assert_noop!(
            <RuntimeInstitutionGovernanceResultHandler as entity_primitives::InstitutionGovernanceResultHandler<AccountId>>::apply_institution_governance_result(
                role_change
            ),
            public_manage::Error::<Runtime>::FixedRoleDefinitionImmutable
        );

        assert_noop!(
            <RuntimeInstitutionGovernanceResultHandler as entity_primitives::InstitutionGovernanceResultHandler<AccountId>>::apply_institution_governance_result(
                result(vec![AccountId::new([61u8; 32])])
            ),
            public_manage::Error::<Runtime>::FixedRoleSeatsMismatch
        );

        let replacement = (70u8..77)
            .map(|seed| AccountId::new([seed; 32]))
            .collect::<Vec<_>>();
        assert_ok!(
            <RuntimeInstitutionGovernanceResultHandler as entity_primitives::InstitutionGovernanceResultHandler<AccountId>>::apply_institution_governance_result(
                result(replacement.clone())
            )
        );

        let cid_number: public_manage::pallet::CidNumberOf<Runtime> = njd
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NJD cid fits");
        let role_code: public_manage::RoleCodeOf = primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD
            .to_vec()
            .try_into()
            .expect("NJD role code fits");
        let stored = public_manage::InstitutionRoleAssignments::<Runtime>::get(
            cid_number,
            role_code,
        );
        assert_eq!(
            stored
                .into_iter()
                .map(|assignment| assignment.admin_account)
                .collect::<Vec<_>>(),
            replacement
        );
        assert_eq!(
            internal_vote::ActiveDynamicThresholds::<Runtime>::get(
                primitives::cid::code::NJD,
                main,
            ),
            None
        );
    });
}
