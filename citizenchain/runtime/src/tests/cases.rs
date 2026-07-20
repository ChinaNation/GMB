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
fn protected_genesis_institution_can_add_dynamic_role_without_mutating_fixed_roles() {
    use entity_primitives::{InstitutionRoleMutation, RolePermissionOperation};
    use frame_support::assert_ok;

    new_test_ext().execute_with(|| {
        let nrc = &primitives::cid::china::china_cb::CHINA_CB[0];
        let cid_number: public_manage::pallet::CidNumberOf<Runtime> = nrc
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NRC CID fits");
        let proposal_id = 9001;
        let dynamic_code =
            entity_primitives::generate_dynamic_role_code(cid_number.as_slice(), 0, proposal_id);
        assert_ok!(PublicManage::apply_institution_governance_result(
            entity_primitives::InstitutionGovernanceResult {
                institution_code: primitives::cid::code::NRC,
                cid_number: cid_number.to_vec(),
                proposal_id,
                role_mutations: vec![InstitutionRoleMutation::Create {
                    role_name: "审计工作人员".as_bytes().to_vec(),
                    term_required: false,
                    permissions: vec![entity_primitives::RolePermissionSpec {
                        business_action_id: entity_primitives::BusinessActionId {
                            module_tag: public_manage::MODULE_TAG.to_vec(),
                            action_code: u32::from(public_manage::pallet::ACTION_GOVERNANCE),
                        },
                        operation: RolePermissionOperation::Propose,
                    }],
                    assignments: vec![entity_primitives::InstitutionAssignmentTarget {
                        admin_account: AccountId::new(nrc.admins[0]),
                        term_start: 0,
                        term_end: 0,
                        assignment_source:
                            entity_primitives::InstitutionAssignmentSource::InstitutionGovernance,
                        assignment_source_ref: b"proposal-9001".to_vec(),
                        assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
                    }],
                }],
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"proposal-9001".to_vec(),
            }
        ));

        let dynamic_code: public_manage::RoleCodeOf =
            dynamic_code.try_into().expect("dynamic role code fits");
        assert!(public_manage::InstitutionRoles::<Runtime>::contains_key(
            &cid_number,
            &dynamic_code
        ));
        let fixed_code: public_manage::RoleCodeOf =
            primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                .to_vec()
                .try_into()
                .expect("fixed role code fits");
        assert!(public_manage::InstitutionRoles::<Runtime>::contains_key(
            &cid_number,
            fixed_code
        ));

        let company = primitives::cid::china::citizenchain::CITIZENCHAIN_TECHNOLOGY;
        let company_cid: private_manage::pallet::CidNumberOf<Runtime> = company
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("company CID fits");
        let company_proposal_id = 9002;
        let company_dynamic_code = entity_primitives::generate_dynamic_role_code(
            company_cid.as_slice(),
            0,
            company_proposal_id,
        );
        assert_ok!(PrivateManage::apply_institution_governance_result(
            entity_primitives::InstitutionGovernanceResult {
                institution_code: *b"SFGQ",
                cid_number: company_cid.to_vec(),
                proposal_id: company_proposal_id,
                role_mutations: vec![InstitutionRoleMutation::Create {
                    role_name: "安全工作人员".as_bytes().to_vec(),
                    term_required: false,
                    permissions: vec![entity_primitives::RolePermissionSpec {
                        business_action_id: entity_primitives::BusinessActionId {
                            module_tag: private_manage::MODULE_TAG.to_vec(),
                            action_code: u32::from(private_manage::pallet::ACTION_GOVERNANCE),
                        },
                        operation: RolePermissionOperation::Vote,
                    }],
                    assignments: vec![entity_primitives::InstitutionAssignmentTarget {
                        admin_account: AccountId::new(
                            primitives::cid::china::citizenchain::CITIZENCHAIN_GENESIS_ADMINS[0]
                                .admin_account,
                        ),
                        term_start: 0,
                        term_end: 0,
                        assignment_source:
                            entity_primitives::InstitutionAssignmentSource::InstitutionGovernance,
                        assignment_source_ref: b"proposal-9002".to_vec(),
                        assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
                    }],
                }],
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"proposal-9002".to_vec(),
            }
        ));
        let company_dynamic_code: private_manage::RoleCodeOf = company_dynamic_code
            .try_into()
            .expect("company dynamic role code fits");
        assert!(private_manage::InstitutionRoles::<Runtime>::contains_key(
            &company_cid,
            company_dynamic_code
        ));
    });
}

#[test]
fn institution_transfer_route_uses_exact_fee_account() {
    use frame_support::BoundedVec;
    use onchain::CallFeeRoute;
    use primitives::cid::china::china_cb::CHINA_CB;

    new_test_ext().execute_with(|| {
        let institution = AccountId::new(CHINA_CB[0].main_account);
        let fee_account = AccountId::new(CHINA_CB[0].fee_account);
        let beneficiary = AccountId::new([99u8; 32]);
        let call = RuntimeCall::MultisigTransfer(multisig::pallet::Call::propose_transfer {
            actor_cid_number: Some(
                CHINA_CB[0]
                    .cid_number
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .expect("NRC CID fits"),
            ),
            proposer_role_code: Some(
                primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                    .to_vec()
                    .try_into()
                    .expect("committee role fits"),
            ),
            funding_account: institution,
            beneficiary,
            amount: 10000,
            remark: BoundedVec::default(),
        });
        let signer = AccountId::new(CHINA_CB[0].admins[0]);
        let route = <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
            &signer, &call,
        );
        assert_eq!(
            route,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 0,
                payer: fee_account,
            }
        );
    });
}

#[test]
fn institution_operation_debits_only_exact_fee_account_without_signer_fallback() {
    use frame_support::dispatch::GetDispatchInfo;
    use pallet_transaction_payment::OnChargeTransaction;
    use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

    type RuntimeCharge = <Runtime as pallet_transaction_payment::Config>::OnChargeTransaction;

    new_test_ext().execute_with(|| {
        let actor_cid_number: public_manage::CidNumberOf<Runtime> = CHINA_CB[0]
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NRC CID fits");
        let signer = AccountId::new(CHINA_CB[0].admins[0]);
        let fee_account = AccountId::new(CHINA_CB[0].fee_account);
        let funding_account = AccountId::new(CHINA_CB[0].main_account);
        let call = RuntimeCall::MultisigTransfer(multisig::pallet::Call::propose_transfer {
            actor_cid_number: Some(actor_cid_number.clone()),
            proposer_role_code: Some(
                primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                    .to_vec()
                    .try_into()
                    .expect("committee role fits"),
            ),
            funding_account,
            beneficiary: AccountId::new([98u8; 32]),
            amount: 50_000,
            remark: Default::default(),
        });
        let dispatch_info = call.get_dispatch_info();

        let _ = Balances::deposit_creating(&signer, 1_000);
        let _ = Balances::deposit_creating(&fee_account, 1_000);
        let signer_before = Balances::free_balance(&signer);
        let fee_before = Balances::free_balance(&fee_account);

        let liquidity = <RuntimeCharge as OnChargeTransaction<Runtime>>::withdraw_fee(
            &signer,
            &call,
            &dispatch_info,
            primitives::fee_policy::ONCHAIN_MIN_FEE,
            primitives::fee_policy::TRANSACTION_TIP,
        )
        .expect("authorized institution operation must debit its exact fee account");
        assert!(liquidity.is_some());
        assert_eq!(Balances::free_balance(&signer), signer_before);
        assert_eq!(
            Balances::free_balance(&fee_account),
            fee_before - primitives::fee_policy::ONCHAIN_MIN_FEE
        );

        // 删除唯一费用账户映射后必须直接拒绝；即使管理员钱包有钱也不允许改扣管理员。
        let fee_name: public_manage::AccountNameOf<Runtime> =
            primitives::account_derive::RESERVED_NAME_FEE
                .to_vec()
                .try_into()
                .expect("fee account name fits");
        public_manage::InstitutionAccounts::<Runtime>::remove(&actor_cid_number, &fee_name);
        public_manage::AccountRegisteredCid::<Runtime>::remove(&fee_account);
        let signer_before_reject = Balances::free_balance(&signer);
        let error = <RuntimeCharge as OnChargeTransaction<Runtime>>::can_withdraw_fee(
            &signer,
            &call,
            &dispatch_info,
            primitives::fee_policy::ONCHAIN_MIN_FEE,
            primitives::fee_policy::TRANSACTION_TIP,
        )
        .expect_err("missing exact fee account must reject instead of falling back");
        assert!(matches!(
            error,
            TransactionValidityError::Invalid(InvalidTransaction::Call)
        ));
        assert_eq!(Balances::free_balance(&signer), signer_before_reject);
    });
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
            actor_cid_number: CHINA_CB[0]
                .cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("NRC CID fits"),
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
                actor_cid_number: Some(
                    CHINA_CB[0]
                        .cid_number
                        .as_bytes()
                        .to_vec()
                        .try_into()
                        .expect("NRC CID fits"),
                ),
                execution_account: None,
                subject_cid_numbers: Default::default(),
                start: 0u32,
                end: 100u32,
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
        let nrc_institution_account = AccountId::new(CHINA_CB[0].main_account);
        let nrc_account = AccountId::new(CHINA_CB[0].main_account);
        let nrc_fee_account = AccountId::new(CHINA_CB[0].fee_account);
        let initial_balance: Balance = 1_000;
        let destroy_amount: Balance = 100;

        let _ = Balances::deposit_creating(&nrc_account, initial_balance);
        let _ = Balances::deposit_creating(&nrc_fee_account, initial_balance);
        let fee_before = Balances::free_balance(&nrc_fee_account);
        let issuance_before = Balances::total_issuance();

        assert_ok!(ResolutionDestroy::propose_destroy(
            RuntimeOrigin::signed(AccountId::new(CHINA_CB[0].admins[0])),
            CHINA_CB[0]
                .cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("NRC CID fits"),
            primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                .to_vec()
                .try_into()
                .expect("committee role fits"),
            nrc_institution_account,
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
        let execution_fee = primitives::fee_policy::calculate_onchain_fee(destroy_amount);
        let nrc_fee_share =
            execution_fee * Balance::from(primitives::fee_policy::ONCHAIN_FEE_NRC_PERCENT) / 100;
        // 测试块没有可识别作者，安全基金账户也未在空测试创世中激活；对应份额按
        // 生产分账规则销毁。NRC 份额回到 NRC 自己的费用账户，不改变总发行量。
        assert_eq!(
            Balances::total_issuance(),
            issuance_before - destroy_amount - (execution_fee - nrc_fee_share),
        );
        assert_eq!(
            Balances::free_balance(&nrc_fee_account),
            fee_before - execution_fee + nrc_fee_share,
        );
    });
}

#[test]
fn runtime_fee_router_covers_free_onchain_vote_institution_and_reject_paths() {
    new_test_ext().execute_with(|| {
        use onchain::CallFeeRoute;

        let who = AccountId::new([1u8; 32]);
        let recipient = AccountId::new([2u8; 32]);

        let system_call = RuntimeCall::System(frame_system::Call::remark {
            remark: b"x".to_vec(),
        });
        let free = <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
            &who,
            &system_call,
        );
        assert_eq!(free, primitives::fee_policy::FeeRoute::Free);

        let transfer_call = RuntimeCall::Balances(pallet_balances::Call::transfer_allow_death {
            dest: sp_runtime::MultiAddress::Id(recipient.clone()),
            value: 123,
        });
        let amount = <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
            &who,
            &transfer_call,
        );
        assert_eq!(amount, primitives::fee_policy::FeeRoute::Reject);

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
        let amount_with_remark = <RuntimeFeeRouter as CallFeeRoute<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_route(&who, &transfer_with_remark_call);
        assert_eq!(
            amount_with_remark,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 456,
                payer: who.clone(),
            }
        );

        let internal_vote_call = RuntimeCall::InternalVote(internal_vote::pallet::Call::cast {
            proposal_id: 1,
            approve: true,
        });
        let vote_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &who,
                &internal_vote_call,
            );
        assert_eq!(
            vote_kind,
            primitives::fee_policy::FeeRoute::Vote { payer: who.clone() }
        );

        let miner = AccountId::new([7u8; 32]);
        let fullnode_call =
            RuntimeCall::FullnodeIssuance(fullnode_issuance::pallet::Call::bind_reward_wallet {
                wallet: AccountId::new([8u8; 32]),
            });
        let fullnode_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &miner,
                &fullnode_call,
            );
        assert_eq!(
            fullnode_kind,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 0,
                payer: miner,
            }
        );

        let nrc_admin = AccountId::new(CHINA_CB[0].admins[0]);
        let nrc_institution_account = AccountId::new(CHINA_CB[0].main_account);
        let resolution_destro_call =
            RuntimeCall::ResolutionDestroy(resolution_destroy::pallet::Call::propose_destroy {
                actor_cid_number: CHINA_CB[0]
                    .cid_number
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .expect("NRC CID fits"),
                proposer_role_code: primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                    .to_vec()
                    .try_into()
                    .expect("committee role fits"),
                institution_account: nrc_institution_account,
                amount: 456,
            });
        let resolution_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &nrc_admin,
                &resolution_destro_call,
            );
        assert_eq!(
            resolution_kind,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 0,
                payer: AccountId::new(CHINA_CB[0].fee_account),
            }
        );
        let unauthorized =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &who,
                &resolution_destro_call,
            );
        assert_eq!(unauthorized, primitives::fee_policy::FeeRoute::Reject);

        let issuance_placeholder =
            RuntimeCall::OnchainIssuance(onchain_issuance::pallet::Call::propose_mint {
                actor_cid_number: CHINA_CB[0]
                    .cid_number
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .expect("NRC CID fits"),
                asset_id: 1,
                to: AccountId::new([6u8; 32]),
                amount: 100,
            });
        let issuance_placeholder_kind = <RuntimeFeeRouter as CallFeeRoute<
            AccountId,
            RuntimeCall,
            Balance,
        >>::fee_route(&nrc_admin, &issuance_placeholder);
        assert_eq!(
            issuance_placeholder_kind,
            primitives::fee_policy::FeeRoute::Reject
        );

        let clearing_bank = &primitives::cid::china::china_ch::CHINA_CH[0];
        let clearing_admin = AccountId::new(clearing_bank.admins[0]);
        let offchain_call =
            RuntimeCall::OffchainTransaction(offchain::pallet::Call::submit_offchain_batch {
                actor_cid_number: clearing_bank
                    .cid_number
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .expect("clearing bank CID fits"),
                institution_account: AccountId::new(clearing_bank.main_account),
                batch_seq: 1,
                batch: Default::default(),
                batch_signature: Default::default(),
            });
        let offchain_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &clearing_admin,
                &offchain_call,
            );
        assert_eq!(
            offchain_kind,
            primitives::fee_policy::FeeRoute::Offchain {
                fee_amount: 0,
                payer: primitives::fee_policy::OffchainFeePayer::BatchItemPayers,
            }
        );

        let unknown_balances_call =
            RuntimeCall::Balances(pallet_balances::Call::upgrade_accounts {
                who: vec![AccountId::new([9u8; 32])],
            });
        let unknown =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &who,
                &unknown_balances_call,
            );
        assert_eq!(unknown, primitives::fee_policy::FeeRoute::Reject);
    });
}

#[test]
fn runtime_fee_router_treats_proposals_as_operations_not_votes() {
    new_test_ext().execute_with(|| {
        use onchain::CallFeeRoute;

        let (p1, _) = sr25519::Pair::generate();
        let (p2, _) = sr25519::Pair::generate();
        let signer1 = MultiSigner::from(p1.public());
        let who: AccountId = signer1.into_account();
        let admin2: AccountId = MultiSigner::from(p2.public()).into_account();

        let beneficiary = AccountId::new([78u8; 32]);
        let admins: personal_manage::pallet::AdminsOf<Runtime> = vec![who.clone(), admin2.clone()]
            .into_iter()
            .map(|admin_account| admin_primitives::Admin {
                admin_account,
                family_name: admin_primitives::FamilyName::truncate_from(
                    "管理".as_bytes().to_vec(),
                ),
                given_name: admin_primitives::GivenName::truncate_from("员".as_bytes().to_vec()),
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("admins should fit");
        // 创建提案是普通操作，只有后续 cast 才是固定 1 元投票。
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
        let create_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &who,
                &create_call,
            );
        assert_eq!(
            create_kind,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 0,
                payer: who.clone(),
            }
        );

        let nrc_admin = AccountId::new(CHINA_CB[0].admins[0]);
        let close_call = RuntimeCall::PublicManage(
            public_manage::pallet::Call::propose_close_public_institution {
                actor_cid_number: CHINA_CB[0]
                    .cid_number
                    .as_bytes()
                    .to_vec()
                    .try_into()
                    .expect("NRC CID fits"),
                proposer_role_code: primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                    .to_vec()
                    .try_into()
                    .expect("committee role fits"),
                institution_account: AccountId::new(CHINA_CB[0].main_account),
                beneficiary,
            },
        );
        let close_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &nrc_admin,
                &close_call,
            );
        assert_eq!(
            close_kind,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 0,
                payer: AccountId::new(CHINA_CB[0].fee_account),
            }
        );

        let institution =
            AccountId::new(primitives::cid::china::china_cb::CHINA_CB[0].main_account);
        let transfer_call =
            RuntimeCall::MultisigTransfer(multisig::pallet::Call::propose_transfer {
                actor_cid_number: Some(
                    CHINA_CB[0]
                        .cid_number
                        .as_bytes()
                        .to_vec()
                        .try_into()
                        .expect("NRC CID fits"),
                ),
                proposer_role_code: Some(
                    primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER
                        .to_vec()
                        .try_into()
                        .expect("committee role fits"),
                ),
                funding_account: institution,
                beneficiary: AccountId::new([79u8; 32]),
                amount: 88_888,
                remark: frame_support::BoundedVec::default(),
            });
        let transfer_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &nrc_admin,
                &transfer_call,
            );
        assert_eq!(
            transfer_kind,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 0,
                payer: AccountId::new(CHINA_CB[0].fee_account),
            }
        );
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
            actor_cid_number: CHINA_CB[0]
                .cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("NRC CID fits"),
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
                actor_cid_number: Some(
                    CHINA_CB[0]
                        .cid_number
                        .as_bytes()
                        .to_vec()
                        .try_into()
                        .expect("NRC CID fits"),
                ),
                execution_account: None,
                subject_cid_numbers: Default::default(),
                start: 0u32,
                end: 100u32,
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
        let (_, registrar, actor_cid_number) = setup_frg_citizen_identity_admin(b"43");
        let wallet_pair =
            sr25519::Pair::from_string("//citizen-wallet-1", None).expect("wallet pair");
        let wallet_account = AccountId::new(wallet_pair.public().0);
        // 占号先行:身份写入前置。
        assert_ok!(CitizenIdentity::occupy_cid(
            RuntimeOrigin::signed(registrar.clone()),
            actor_cid_number.clone(),
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
            actor_cid_number,
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
        let (_, registrar, actor_cid_number) = setup_frg_citizen_identity_admin(b"43");
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
                actor_cid_number,
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
        let (_, registrar, actor_cid_number) = setup_frg_citizen_identity_admin(b"43");
        let wallet_pair =
            sr25519::Pair::from_string("//citizen-wallet-3", None).expect("wallet pair");
        let wallet_account = AccountId::new(wallet_pair.public().0);
        // 占号先行:身份写入前置。
        assert_ok!(CitizenIdentity::occupy_cid(
            RuntimeOrigin::signed(registrar.clone()),
            actor_cid_number.clone(),
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
            birth_date: 20000101,
        };
        let signature = sign_citizen_identity_payload(&wallet_pair, &candidate);

        assert_ok!(CitizenIdentity::upgrade_to_candidate_identity(
            RuntimeOrigin::signed(registrar),
            actor_cid_number,
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
        let (_, registrar, actor_cid_number) = setup_frg_citizen_identity_admin(b"43");
        let wallet_pair =
            sr25519::Pair::from_string("//square-citizen-wallet", None).expect("wallet pair");
        let wallet_account = AccountId::new(wallet_pair.public().0);
        let cid_number = real_cid_number("SQUARE-0001", "CTZN", "1");

        assert_ok!(CitizenIdentity::occupy_cid(
            RuntimeOrigin::signed(registrar.clone()),
            actor_cid_number.clone(),
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
            actor_cid_number,
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
        use onchain::CallFeeRoute;

        let who = AccountId::new([44u8; 32]);
        let call = RuntimeCall::SquarePost(square_post::pallet::Call::publish_post {
            post_id: b"sqp_fee_kind".to_vec(),
            post_category: square_post::SquarePostCategory::Normal,
            content_hash: [6u8; 32],
            storage_receipt_id: b"sqr_fee_kind".to_vec(),
            storage_until: 1_893_456_000_000,
        });
        let fee_kind =
            <RuntimeFeeRouter as CallFeeRoute<AccountId, RuntimeCall, Balance>>::fee_route(
                &who, &call,
            );
        assert_eq!(
            fee_kind,
            primitives::fee_policy::FeeRoute::Onchain {
                transaction_amount: 0,
                payer: who,
            }
        );
        assert_eq!(
            primitives::fee_policy::calculate_onchain_fee(0),
            primitives::fee_policy::ONCHAIN_MIN_FEE
        );
        assert_eq!(primitives::fee_policy::ONCHAIN_MIN_FEE, 10);
        // 广场发布费不得改变实际投票的统一费用。
        assert_eq!(primitives::fee_policy::VOTE_FLAT_FEE, YUAN);
    });
}

#[test]
fn ensure_nrc_admin_and_runtime_internal_admin_provider_paths() {
    new_test_ext().execute_with(|| {
        let nrc_cid: public_manage::pallet::CidNumberOf<Runtime> = CHINA_CB[0]
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NRC CID fits");
        let nrc_admin = AccountId::new(CHINA_CB[0].admins[0]);
        let outsider = AccountId::new([99u8; 32]);

        let ok_origin = RuntimeOrigin::signed(nrc_admin.clone());
        assert!(<EnsureNrcAdmin as EnsureOrigin<RuntimeOrigin>>::try_origin(ok_origin).is_ok());
        let bad_origin = RuntimeOrigin::signed(outsider.clone());
        assert!(<EnsureNrcAdmin as EnsureOrigin<RuntimeOrigin>>::try_origin(bad_origin).is_err());

        public_admins::pallet::AdminAccounts::<Runtime>::remove(&nrc_cid);
        assert!(!is_nrc_admin(&nrc_admin));
        assert!(!is_nrc_admin(&outsider));
        assert!(!RuntimeInternalAdminProvider::is_institution_admin(
            votingengine::types::NRC,
            nrc_cid.as_slice(),
            &nrc_admin
        ));
    });
}

// 机构自定义账户关闭已改为「机构在册管理员直接冷签 propose_close(不含凭证)」,由 pallet 在
// origin 处以 `is_institution_admin` 鉴权;该授权属性的回归覆盖在 public-manage/private-manage
// 的 `close_requires_matching_actor_cid_and_an_institution_admin` 等 pallet 级用例。原
// 注册局审批凭证验签(`RuntimeCidInstitutionVerifier`)连同 OnChina 平台签名钥已整体删除。
#[test]
fn onchain_issuance_requires_actor_cid_admin_and_matching_execution_account() {
    new_test_ext().execute_with(|| {
        let nrc = &CHINA_CB[0];
        let actor_cid_number: votingengine::types::CidNumber = nrc
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NRC CID fits runtime bound");
        let admin = AccountId::new(nrc.admins[0]);
        let main_account = AccountId::new(nrc.main_account);

        assert_noop!(
            OnchainIssuance::propose_issue(
                RuntimeOrigin::signed(admin.clone()),
                actor_cid_number.clone(),
                AccountId::new([250u8; 32]),
                onchain_issuance::types::AssetClass::Plain,
                b"Institution Asset".to_vec().try_into().expect("name fits"),
                b"IAS".to_vec().try_into().expect("symbol fits"),
                b"plain asset"
                    .to_vec()
                    .try_into()
                    .expect("description fits"),
                2,
                1_000,
            ),
            onchain_issuance::Error::<Runtime>::InvalidInstitutionContext
        );

        assert_ok!(OnchainIssuance::propose_issue(
            RuntimeOrigin::signed(admin),
            actor_cid_number,
            main_account,
            onchain_issuance::types::AssetClass::Plain,
            b"Institution Asset".to_vec().try_into().expect("name fits"),
            b"IAS".to_vec().try_into().expect("symbol fits"),
            b"plain asset"
                .to_vec()
                .try_into()
                .expect("description fits"),
            2,
            1_000,
        ));
    });
}

#[test]
fn onchain_issuance_rejects_non_admin_and_non_nrc_monitor_actor() {
    new_test_ext().execute_with(|| {
        let nrc_cid: votingengine::types::CidNumber = CHINA_CB[0]
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NRC CID fits runtime bound");
        assert_noop!(
            OnchainIssuance::propose_mint(
                RuntimeOrigin::signed(AccountId::new([250u8; 32])),
                nrc_cid,
                1,
                AccountId::new([1u8; 32]),
                1,
            ),
            onchain_issuance::Error::<Runtime>::ProposeOriginNotAllowed
        );

        let prc = &CHINA_CB[1];
        let prc_cid: votingengine::types::CidNumber = prc
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("PRC CID fits runtime bound");
        assert_noop!(
            OnchainIssuance::propose_monitor_freeze(
                RuntimeOrigin::signed(AccountId::new(prc.admins[0])),
                prc_cid,
                1,
                AccountId::new([1u8; 32]),
                [7u8; 32],
            ),
            onchain_issuance::Error::<Runtime>::InvalidInstitutionContext
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
        let njd_cid: public_manage::pallet::CidNumberOf<Runtime> = njd
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NJD CID fits");
        assert!(public_admins::AdminAccounts::<Runtime>::get(njd_cid).is_some());
    });
}

/// 中国公民链技术股份有限公司以私权创世机构写入，三名人员、三岗、法定代表人与阈值同源一致。
#[test]
fn genesis_citizenchain_technology_is_complete_and_protected() {
    new_test_ext().execute_with(|| {
        let company = primitives::cid::china::citizenchain::CITIZENCHAIN_TECHNOLOGY;
        let cid: private_manage::pallet::CidNumberOf<Runtime> = company
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("company CID fits");
        let info = private_manage::Institutions::<Runtime>::get(&cid)
            .expect("citizenchain technology genesis institution exists");
        assert_eq!(
            info.cid_full_name.as_slice(),
            company.cid_full_name.as_bytes()
        );
        assert_eq!(
            info.cid_short_name.as_slice(),
            company.cid_short_name.as_bytes()
        );
        assert_eq!(
            info.legal_representative_name
                .as_ref()
                .map(|value| value.as_slice()),
            Some("程伟".as_bytes())
        );
        assert_eq!(
            info.legal_representative_cid_number
                .as_ref()
                .map(|value| value.as_slice()),
            Some("GZ000-CTZN6-198805200-2026".as_bytes())
        );
        assert_eq!(
            info.legal_representative_account,
            Some(AccountId::new(
                primitives::cid::china::citizenchain::CITIZENCHAIN_GENESIS_ADMINS[0].admin_account,
            ))
        );

        let admin_cid: admin_primitives::AdminCidNumber = company
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("admin CID fits");
        let admins =
            private_admins::AdminAccounts::<Runtime>::get(admin_cid).expect("company admins exist");
        assert_eq!(admins.institution_code, *b"SFGQ");
        assert_eq!(admins.admins.len(), 3);
        assert_eq!(admins.admins[0].family_name.as_slice(), "程".as_bytes());
        assert_eq!(admins.admins[0].given_name.as_slice(), "伟".as_bytes());
        assert_eq!(admins.admins[1].family_name.as_slice(), "管理".as_bytes());
        assert_eq!(admins.admins[1].given_name.as_slice(), "员".as_bytes());

        for (index, fixed_role) in primitives::cid::china::citizenchain::CITIZENCHAIN_FIXED_ROLES
            .iter()
            .enumerate()
        {
            let role_code: private_manage::RoleCodeOf = fixed_role
                .role_code
                .to_vec()
                .try_into()
                .expect("fixed role code fits");
            let role = private_manage::InstitutionRoles::<Runtime>::get(&cid, &role_code)
                .expect("fixed company role exists");
            assert_eq!(role.role_name.as_slice(), fixed_role.role_name);
            assert_eq!(
                role.role_status,
                entity_primitives::InstitutionRoleStatus::Active
            );
            let assignments =
                private_manage::InstitutionRoleAssignments::<Runtime>::get(&cid, &role_code);
            assert_eq!(assignments.len(), 1);
            assert_eq!(
                assignments[0].admin_account,
                AccountId::new(
                    primitives::cid::china::citizenchain::CITIZENCHAIN_GENESIS_ADMINS[index]
                        .admin_account,
                )
            );
            let permissions =
                private_manage::InstitutionRolePermissions::<Runtime>::get(&cid, &role_code);
            let expected = entity_primitives::fixed_role_permission_specs(
                *b"SFGQ",
                cid.as_slice(),
                fixed_role.role_code,
            );
            assert_eq!(permissions.len(), expected.len());
            assert!(permissions.iter().zip(expected).all(|(actual, expected)| {
                actual.role_subject.cid_number == cid
                    && actual.role_subject.role_code == role_code
                    && actual.business_action_id.module_tag.as_slice() == expected.module_tag
                    && actual.business_action_id.action_code == expected.action_code
                    && actual.operation == expected.operation
            }));
        }
        assert_eq!(
            internal_vote::ActiveInstitutionThresholds::<Runtime>::get(&cid),
            Some(2)
        );
        assert!(RuntimeReservedAccountGuard::is_reserved(&AccountId::new(
            company.main_account
        )));
        assert!(RuntimeReservedAccountGuard::is_reserved(&AccountId::new(
            company.fee_account
        )));
        assert!(!RuntimeInstitutionAsset::can_spend(
            &AccountId::new(company.fee_account),
            InstitutionAssetAction::MultisigTransferExecute
        ));
        assert!(RuntimeInstitutionAsset::can_spend(
            &AccountId::new(company.fee_account),
            InstitutionAssetAction::OffchainFeeSweepExecute
        ));
        assert_eq!(private_manage::Institutions::<Runtime>::iter().count(), 1);
    });
}

/// 六个国家级单例在 block#0 精确占用约定身份；三个成员机构保持完整的“尚未组成”状态。
#[test]
fn genesis_national_singletons_exist_and_member_bodies_are_unconstituted() {
    new_test_ext().execute_with(|| {
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
            let main_name: public_manage::pallet::AccountNameOf<Runtime> =
                primitives::account_derive::RESERVED_NAME_MAIN
                    .to_vec()
                    .try_into()
                    .expect("main name fits");
            assert_eq!(
                public_manage::InstitutionAccounts::<Runtime>::get(&cid, main_name)
                    .map(|info| info.address),
                Some(AccountId::new(singleton.main_account))
            );
            assert!(public_admins::AdminAccounts::<Runtime>::get(&cid).is_none());
            assert!(internal_vote::ActiveInstitutionThresholds::<Runtime>::get(&cid).is_none());
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
            assert!(public_admins::AdminAccounts::<Runtime>::get(cid).is_none());
        }
    });
}

/// 参议会必须先独立登记 admins，治理结果只能把既有管理员任命到法定岗位。
#[test]
fn national_member_body_first_composition_and_permanent_range_are_enforced() {
    new_test_ext().execute_with(|| {
        let spec = primitives::institution_constraints::member_composition_specs()[0];
        let main = AccountId::new(spec.institution.main_account);
        let cid_number: public_manage::pallet::CidNumberOf<Runtime> = spec
            .institution
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("member body CID fits");
        let members = |count: u32| {
            (0..count)
                .map(|index| {
                    let mut raw = [0u8; 32];
                    raw[..4].copy_from_slice(&index.to_le_bytes());
                    AccountId::new(raw)
                })
                .collect::<Vec<_>>()
        };
        let result = |accounts: Vec<AccountId>| entity_primitives::InstitutionGovernanceResult {
            institution_code: spec.institution.code,
            cid_number: spec.institution.cid_number.as_bytes().to_vec(),
            proposal_id: 1,
            role_mutations: vec![],
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
        };

        let established_admins = members(spec.min_members);
        public_admins::AdminAccounts::<Runtime>::insert(
            cid_number.clone(),
            admin_primitives::InstitutionAdmins {
                institution_code: spec.institution.code,
                admins: established_admins
                    .iter()
                    .cloned()
                    .map(|admin_account| admin_primitives::Admin {
                        admin_account,
                        family_name: "管理"
                            .as_bytes()
                            .to_vec()
                            .try_into()
                            .expect("family name fits"),
                        given_name: "员"
                            .as_bytes()
                            .to_vec()
                            .try_into()
                            .expect("given name fits"),
                    })
                    .collect::<Vec<_>>()
                    .try_into()
                    .expect("member body admins fit"),
            },
        );
        let member_role_code: public_manage::RoleCodeOf = spec
            .role_code
            .to_vec()
            .try_into()
            .expect("member role code fits");
        public_manage::InstitutionRoles::<Runtime>::insert(
            &cid_number,
            &member_role_code,
            entity_primitives::InstitutionRole {
                cid_number: cid_number.clone(),
                role_code: member_role_code.clone(),
                role_name: spec
                    .role_name
                    .to_vec()
                    .try_into()
                    .expect("member role name fits"),
                term_required: false,
                role_status: entity_primitives::InstitutionRoleStatus::Active,
            },
        );

        assert_noop!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(members(
                spec.min_members - 1,
            ))),
            public_manage::Error::<Runtime>::RequiredMemberCountOutOfRange
        );
        assert_ok!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(
                established_admins.clone(),
            ))
        );
        let account = public_admins::AdminAccounts::<Runtime>::get(&cid_number)
            .expect("admins remain independently registered");
        assert_eq!(account.admins.len() as u32, spec.min_members);
        assert_eq!(
            internal_vote::ActiveInstitutionThresholds::<Runtime>::get(&cid_number),
            None
        );
        let vote_plan = votingengine::VotePlanOf::<AccountId>::try_new(
            votingengine::BusinessActionId {
                module_tag: b"test".to_vec().try_into().expect("owner fits"),
                action_code: 0,
            },
            b"test".to_vec().try_into().expect("owner fits"),
            votingengine::AuthorizationSubject::Institution(votingengine::RoleSubject {
                cid_number: cid_number.clone().try_into().expect("CID fits"),
                role_code: member_role_code.clone().try_into().expect("role fits"),
            }),
            vec![votingengine::AuthorizationSubject::Institution(
                votingengine::RoleSubject {
                    cid_number: cid_number.clone().try_into().expect("CID fits"),
                    role_code: member_role_code.clone().try_into().expect("role fits"),
                },
            )],
            votingengine::VotingEngineKind::Internal,
            [0u8; 32],
        )
        .expect("valid singleton vote plan");
        let proposal_id = internal_vote::Pallet::<Runtime>::do_create_institution_proposal(
            members(1)[0].clone(),
            spec.institution.code,
            spec.institution.cid_number.as_bytes().to_vec(),
            None,
            vec![spec.institution.cid_number.as_bytes().to_vec()],
            &vote_plan,
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
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(members(
                spec.min_members - 1,
            ))),
            public_manage::Error::<Runtime>::RequiredMemberCountOutOfRange
        );
        assert_eq!(
            public_admins::AdminAccounts::<Runtime>::get(cid_number)
                .expect("failed change rolls back")
                .admins
                .len() as u32,
            spec.min_members
        );
    });
}

/// 法定代表人治理必须支持整体任命/更换与整体解除，不能留下姓名、CID 或账户半字段。
#[test]
fn institution_governance_can_clear_legal_representative_atomically() {
    new_test_ext().execute_with(|| {
        let cid_number = CHINA_CB[0].cid_number.as_bytes().to_vec();
        let institution_code =
            primitives::cid::code::institution_code_from_cid_number(CHINA_CB[0].cid_number)
                .expect("CHINA_CB CID must contain institution code");
        let cid: public_manage::CidNumberOf<Runtime> =
            cid_number.clone().try_into().expect("CID fits");
        let representative_account = public_admins::AdminAccounts::<Runtime>::get(&cid)
            .expect("NRC genesis admins exist")
            .admins[0]
            .admin_account
            .clone();
        let result = |legal_representative_change, assignments| {
            entity_primitives::InstitutionGovernanceResult {
                institution_code,
                cid_number: cid_number.clone(),
                proposal_id: 2,
                role_mutations: vec![],
                assignment_changes: vec![entity_primitives::InstitutionRoleAssignmentChange {
                    role_code: primitives::institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE
                        .to_vec(),
                    assignments,
                }],
                legal_representative_change,
                result_source_ref: b"legal-representative-governance".to_vec(),
            }
        };
        let representative_assignment = entity_primitives::InstitutionAssignmentTarget {
            admin_account: representative_account.clone(),
            term_start: 0,
            term_end: 0,
            assignment_source:
                entity_primitives::InstitutionAssignmentSource::InstitutionGovernance,
            assignment_source_ref: b"legal-representative-governance".to_vec(),
            assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
        };

        assert_ok!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(
                Some(
                    entity_primitives::InstitutionLegalRepresentativeChange::Set {
                        legal_representative_name: "法定代表人".as_bytes().to_vec(),
                        legal_representative_cid_number: b"CITIZEN-LR-001".to_vec(),
                        legal_representative_account: representative_account.clone(),
                    },
                ),
                vec![representative_assignment],
            ))
        );
        let stored =
            public_manage::Institutions::<Runtime>::get(&cid).expect("genesis institution exists");
        assert!(stored.legal_representative_name.is_some());
        assert_eq!(
            stored.legal_representative_account,
            Some(representative_account.clone())
        );

        assert_ok!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(
                Some(entity_primitives::InstitutionLegalRepresentativeChange::Clear),
                vec![],
            ))
        );
        let cleared = public_manage::Institutions::<Runtime>::get(&cid)
            .expect("genesis institution still exists");
        assert!(cleared.legal_representative_name.is_none());
        assert!(cleared.legal_representative_cid_number.is_none());
        assert!(cleared.legal_representative_account.is_none());

        let second_account = public_admins::AdminAccounts::<Runtime>::get(&cid)
            .expect("NRC genesis admins remain")
            .admins[1]
            .admin_account
            .clone();
        let second_assignment = entity_primitives::InstitutionAssignmentTarget {
            admin_account: second_account,
            term_start: 0,
            term_end: 0,
            assignment_source:
                entity_primitives::InstitutionAssignmentSource::InstitutionGovernance,
            assignment_source_ref: b"legal-representative-governance".to_vec(),
            assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
        };
        assert_noop!(
            public_manage::Pallet::<Runtime>::apply_institution_governance_result(result(
                Some(
                    entity_primitives::InstitutionLegalRepresentativeChange::Set {
                        legal_representative_name: "重复法定代表人".as_bytes().to_vec(),
                        legal_representative_cid_number: b"CITIZEN-LR-002".to_vec(),
                        legal_representative_account: representative_account,
                    },
                ),
                vec![
                    entity_primitives::InstitutionAssignmentTarget {
                        admin_account: public_admins::AdminAccounts::<Runtime>::get(&cid)
                            .expect("NRC genesis admins remain")
                            .admins[0]
                            .admin_account
                            .clone(),
                        term_start: 0,
                        term_end: 0,
                        assignment_source:
                            entity_primitives::InstitutionAssignmentSource::InstitutionGovernance,
                        assignment_source_ref: b"legal-representative-governance".to_vec(),
                        assignment_status: entity_primitives::InstitutionAssignmentStatus::Active,
                    },
                    second_assignment,
                ],
            )),
            public_manage::Error::<Runtime>::FixedRoleSeatsMismatch
        );
    });
}

/// 立法院、监察院、总统府必须先独立登记 admins，岗位任职不得反向生成管理员。
#[test]
fn national_singletons_without_member_ranges_can_be_composed_once() {
    new_test_ext().execute_with(|| {
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
            let cid_number: public_manage::pallet::CidNumberOf<Runtime> = singleton
                .cid_number
                .as_bytes()
                .to_vec()
                .try_into()
                .expect("singleton CID fits");
            let admins = vec![AccountId::new([91u8; 32]), AccountId::new([92u8; 32])];
            public_admins::AdminAccounts::<Runtime>::insert(
                cid_number.clone(),
                admin_primitives::InstitutionAdmins {
                    institution_code: singleton.code,
                    admins: admins
                        .iter()
                        .cloned()
                        .map(|admin_account| admin_primitives::Admin {
                            admin_account,
                            family_name: "管理"
                                .as_bytes()
                                .to_vec()
                                .try_into()
                                .expect("family name fits"),
                            given_name: "员"
                                .as_bytes()
                                .to_vec()
                                .try_into()
                                .expect("given name fits"),
                        })
                        .collect::<Vec<_>>()
                        .try_into()
                        .expect("singleton admins fit"),
                },
            );
            let result = entity_primitives::InstitutionGovernanceResult {
                institution_code: singleton.code,
                cid_number: singleton.cid_number.as_bytes().to_vec(),
                proposal_id: 100,
                role_mutations: vec![entity_primitives::InstitutionRoleMutation::Create {
                    role_name: "运行期成员".as_bytes().to_vec(),
                    term_required: false,
                    permissions: vec![entity_primitives::RolePermissionSpec {
                        business_action_id: entity_primitives::BusinessActionId {
                            module_tag: public_manage::MODULE_TAG.to_vec(),
                            action_code: u32::from(public_manage::pallet::ACTION_GOVERNANCE),
                        },
                        operation: entity_primitives::RolePermissionOperation::Propose,
                    }],
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
                assignment_changes: vec![],
                legal_representative_change: None,
                result_source_ref: b"first-composition".to_vec(),
            };

            assert_ok!(
                public_manage::Pallet::<Runtime>::apply_institution_governance_result(result)
            );
            let account = public_admins::AdminAccounts::<Runtime>::get(&cid_number)
                .expect("admins remain independently registered");
            assert_eq!(
                account
                    .admins
                    .iter()
                    .map(|admin| admin.admin_account.clone())
                    .collect::<Vec<_>>(),
                admins
            );
            assert_eq!(
                internal_vote::ActiveInstitutionThresholds::<Runtime>::get(cid_number),
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

            let admin_account = public_admins::AdminAccounts::<Runtime>::get(&cid_number)
                .expect("committee admin account exists");
            assert_eq!(
                admin_account.institution_code,
                primitives::cid::code::institution_code_from_cid_number(node.cid_number)
                    .expect("committee code")
            );
            assert_eq!(
                admin_account
                    .admins
                    .into_inner()
                    .into_iter()
                    .map(|admin| admin.admin_account)
                    .collect::<Vec<_>>(),
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
            let admin_account = public_admins::AdminAccounts::<Runtime>::get(&cid_number)
                .expect("director admin account exists");
            assert_eq!(
                admin_account
                    .admins
                    .into_inner()
                    .into_iter()
                    .map(|admin| admin.admin_account)
                    .collect::<Vec<_>>(),
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
        let frg_admin_account = public_admins::AdminAccounts::<Runtime>::get(&frg_cid)
            .expect("FRG admin account exists");
        assert_eq!(frg_admin_account.admins.len(), frg_assignments);
    });
}

/// 固定权限必须由创世直接写入；协议升级和决议发行均为 NRC/PRC 发起并投票、PRB 仅投票。
#[test]
fn genesis_fixed_role_permissions_match_the_shared_catalog() {
    use entity_primitives::{
        business_action::{
            ACTION_RESOLUTION_ISSUANCE, ACTION_RUNTIME_UPGRADE, MODULE_RESOLUTION_ISSUANCE,
            MODULE_RUNTIME_UPGRADE,
        },
        RolePermissionOperation,
    };

    new_test_ext().execute_with(|| {
        let role_code = |raw: &[u8]| -> public_manage::RoleCodeOf {
            raw.to_vec().try_into().expect("fixed role code fits")
        };
        let cid = |raw: &str| -> public_manage::pallet::CidNumberOf<Runtime> {
            raw.as_bytes().to_vec().try_into().expect("fixed cid fits")
        };
        let has = |permissions: &public_manage::RolePermissionsOf<Runtime>,
                   module_tag: &[u8],
                   action_code: u32,
                   operation: RolePermissionOperation| {
            permissions.iter().any(|permission| {
                permission.business_action_id.module_tag.as_slice() == module_tag
                    && permission.business_action_id.action_code == action_code
                    && permission.operation == operation
            })
        };

        for node in primitives::cid::china::china_cb::CHINA_CB.iter() {
            let cid_number = cid(node.cid_number);
            let code = role_code(primitives::governance_skeleton::ROLE_CODE_COMMITTEE_MEMBER);
            let permissions =
                public_manage::InstitutionRolePermissions::<Runtime>::get(&cid_number, &code);
            assert!(has(
                &permissions,
                MODULE_RUNTIME_UPGRADE,
                ACTION_RUNTIME_UPGRADE,
                RolePermissionOperation::Propose,
            ));
            assert!(has(
                &permissions,
                MODULE_RUNTIME_UPGRADE,
                ACTION_RUNTIME_UPGRADE,
                RolePermissionOperation::Vote,
            ));
        }

        let prb = &primitives::cid::china::china_ch::CHINA_CH[0];
        let prb_cid = cid(prb.cid_number);
        let director = role_code(primitives::governance_skeleton::ROLE_CODE_DIRECTOR);
        let permissions =
            public_manage::InstitutionRolePermissions::<Runtime>::get(&prb_cid, &director);
        assert!(has(
            &permissions,
            MODULE_RUNTIME_UPGRADE,
            ACTION_RUNTIME_UPGRADE,
            RolePermissionOperation::Vote,
        ));
        assert!(!has(
            &permissions,
            MODULE_RUNTIME_UPGRADE,
            ACTION_RUNTIME_UPGRADE,
            RolePermissionOperation::Propose,
        ));
        assert!(has(
            &permissions,
            MODULE_RESOLUTION_ISSUANCE,
            ACTION_RESOLUTION_ISSUANCE,
            RolePermissionOperation::Vote,
        ));
        assert!(!has(
            &permissions,
            MODULE_RESOLUTION_ISSUANCE,
            ACTION_RESOLUTION_ISSUANCE,
            RolePermissionOperation::Propose,
        ));

        let nrc = &primitives::cid::china::china_cb::CHINA_CB[0];
        let nrc_cid = cid(nrc.cid_number);
        let lr = role_code(primitives::institution_constraints::ROLE_CODE_LEGAL_REPRESENTATIVE);
        assert!(public_manage::InstitutionRolePermissions::<Runtime>::contains_key(&nrc_cid, &lr));
        assert!(public_manage::InstitutionRolePermissions::<Runtime>::get(&nrc_cid, lr).is_empty());
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
        let result_source_ref = 700u64.encode();
        let njd_cid_number: public_manage::pallet::CidNumberOf<Runtime> = njd
            .cid_number
            .as_bytes()
            .to_vec()
            .try_into()
            .expect("NJD cid fits");
        let njd_admins = public_admins::AdminAccounts::<Runtime>::get(&njd_cid_number)
            .expect("NJD genesis admins exist")
            .admins
            .into_iter()
            .map(|admin| admin.admin_account)
            .collect::<Vec<_>>();
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
                cid_number: njd.cid_number.as_bytes().to_vec(),
                proposal_id: 700,
                role_mutations: vec![],
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
        role_change.role_mutations = vec![entity_primitives::InstitutionRoleMutation::Rename {
            role_code: primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD.to_vec(),
            role_name: "修改固定岗位".as_bytes().to_vec(),
        }];
        assert_noop!(
            <RuntimeInstitutionGovernanceResultHandler as entity_primitives::InstitutionGovernanceResultHandler<AccountId>>::apply_institution_governance_result(
                role_change
            ),
            public_manage::Error::<Runtime>::FixedRoleDefinitionImmutable
        );

        assert_noop!(
            <RuntimeInstitutionGovernanceResultHandler as entity_primitives::InstitutionGovernanceResultHandler<AccountId>>::apply_institution_governance_result(
                result(vec![njd_admins[0].clone()])
            ),
            public_manage::Error::<Runtime>::FixedRoleSeatsMismatch
        );

        let mut replacement = njd_admins
            .iter()
            .take(primitives::governance_skeleton::NJD_CONSTITUTION_GUARD_SEATS as usize)
            .cloned()
            .collect::<Vec<_>>();
        replacement.rotate_left(1);
        assert_ok!(
            <RuntimeInstitutionGovernanceResultHandler as entity_primitives::InstitutionGovernanceResultHandler<AccountId>>::apply_institution_governance_result(
                result(replacement.clone())
            )
        );

        let role_code: public_manage::RoleCodeOf = primitives::governance_skeleton::ROLE_CODE_CONSTITUTION_GUARD
            .to_vec()
            .try_into()
            .expect("NJD role code fits");
        let stored = public_manage::InstitutionRoleAssignments::<Runtime>::get(
            njd_cid_number.clone(),
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
            internal_vote::ActiveInstitutionThresholds::<Runtime>::get(njd_cid_number),
            None
        );
    });
}
