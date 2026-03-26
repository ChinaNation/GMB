use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use frame_support::{
    derive_impl,
    dispatch::{GetDispatchInfo, PostDispatchInfo},
    traits::{
        fungible::Balanced,
        tokens::{Fortitude, Precision, Preservation},
        OnUnbalanced, VariantCountOf,
    },
    weights::ConstantMultiplier,
};
use frame_system as system;
use onchain_transaction_pow::{
    AmountExtractResult, CallAmount, NrcAccountProvider, PowOnchainChargeAdapter,
    PowOnchainFeeRouter,
};
use pallet_transaction_payment::OnChargeTransaction;
use sp_runtime::{traits::IdentityLookup, AccountId32, BuildStorage};
use std::{cell::RefCell, thread_local, vec::Vec};

type Balance = u128;
type Block = frame_system::mocking::MockBlockU32<Test>;

thread_local! {
    static MOCK_AUTHOR: RefCell<Option<AccountId32>> = const { RefCell::new(None) };
}

#[frame_support::runtime]
mod runtime {
    #[runtime::runtime]
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask,
        RuntimeViewFunction
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system;
    #[runtime::pallet_index(1)]
    pub type Balances = pallet_balances;
    #[runtime::pallet_index(2)]
    pub type TransactionPayment = pallet_transaction_payment;
    #[runtime::pallet_index(3)]
    pub type FullnodePowReward = fullnode_pow_reward;
    #[runtime::pallet_index(4)]
    pub type OnchainTransactionPow = onchain_transaction_pow::pallet;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type AccountData = pallet_balances::AccountData<Balance>;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Nonce = u64;
}

impl pallet_balances::Config for Test {
    type MaxLocks = frame_support::traits::ConstU32<0>;
    type MaxReserves = frame_support::traits::ConstU32<0>;
    type ReserveIdentifier = [u8; 8];
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = frame_support::traits::ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type FreezeIdentifier = RuntimeFreezeReason;
    type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type DoneSlashHandler = ();
}

impl pallet_transaction_payment::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, ()>;
    type WeightToFee = ConstantMultiplier<Balance, frame_support::traits::ConstU128<1>>;
    type LengthToFee = ConstantMultiplier<Balance, frame_support::traits::ConstU128<1>>;
    type FeeMultiplierUpdate = ();
    type OperationalFeeMultiplier = frame_support::traits::ConstU8<1>;
    type WeightInfo = ();
}

pub struct MockFindAuthor;
impl frame_support::traits::FindAuthor<AccountId32> for MockFindAuthor {
    fn find_author<'a, I>(_digests: I) -> Option<AccountId32>
    where
        I: 'a + IntoIterator<Item = (sp_runtime::ConsensusEngineId, &'a [u8])>,
    {
        MOCK_AUTHOR.with(|v| v.borrow().clone())
    }
}

impl fullnode_pow_reward::Config for Test {
    type Currency = Balances;
    type FindAuthor = MockFindAuthor;
    type WeightInfo = ();
}

impl onchain_transaction_pow::pallet::Config for Test {}

struct MockNrcAccountProvider;
impl NrcAccountProvider<AccountId32> for MockNrcAccountProvider {
    fn nrc_account() -> Option<AccountId32> {
        Some(AccountId32::new(
            primitives::china::china_cb::CHINA_CB[0].duoqian_address,
        ))
    }
}

struct AmountExtractorAmount;
impl CallAmount<AccountId32, RuntimeCall, Balance> for AmountExtractorAmount {
    fn amount(_who: &AccountId32, _call: &RuntimeCall) -> AmountExtractResult<Balance> {
        AmountExtractResult::Amount(50_000)
    }
}

type BenchRouter = PowOnchainFeeRouter<Test, Balances, MockFindAuthor, MockNrcAccountProvider>;
type BenchAdapter = PowOnchainChargeAdapter<Balances, BenchRouter, AmountExtractorAmount, ()>;

fn account(n: u8) -> AccountId32 {
    AccountId32::new([n; 32])
}

fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .expect("system genesis build should succeed");
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(account(1), 1_000_000), (account(2), 1_000_000)],
        dev_accounts: None,
    }
    .assimilate_storage(&mut storage)
    .expect("balances genesis build should succeed");
    sp_io::TestExternalities::new(storage)
}

fn prepare_distribution_state() {
    let miner = account(9);
    let wallet = account(8);
    MOCK_AUTHOR.with(|v| {
        *v.borrow_mut() = Some(miner.clone());
    });
    fullnode_pow_reward::RewardWalletByMiner::<Test>::insert(&miner, wallet);
}

fn bench_charge_transaction_amount_path(c: &mut Criterion) {
    c.bench_function("onchain_fee_charge_transaction_amount_path", |b| {
        b.iter_batched(
            new_test_ext,
            |mut ext| {
                ext.execute_with(|| {
                    System::set_block_number(1);
                    prepare_distribution_state();

                    let payer = account(1);
                    let call =
                        RuntimeCall::System(frame_system::Call::remark { remark: Vec::new() });
                    let info = call.get_dispatch_info();
                    let post_info = PostDispatchInfo::default();
                    let tip: Balance = 0;

                    let liquidity = <BenchAdapter as OnChargeTransaction<Test>>::withdraw_fee(
                        &payer, &call, &info, 0, tip,
                    )
                    .expect("withdraw_fee should succeed");
                    <BenchAdapter as OnChargeTransaction<Test>>::correct_and_deposit_fee(
                        &payer, &info, &post_info, 0, tip, liquidity,
                    )
                    .expect("fee distribution should succeed");
                });
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_router_distribution_success(c: &mut Criterion) {
    c.bench_function("onchain_fee_router_distribution_success", |b| {
        b.iter_batched(
            new_test_ext,
            |mut ext| {
                ext.execute_with(|| {
                    System::set_block_number(1);
                    prepare_distribution_state();

                    // 中文注释：直接构造一笔已扣手续费 credit，只测分账热路径。
                    let credit = Balances::withdraw(
                        &account(2),
                        10_000,
                        Precision::Exact,
                        Preservation::Preserve,
                        Fortitude::Polite,
                    )
                    .expect("credit should withdraw");

                    <BenchRouter as OnUnbalanced<_>>::on_nonzero_unbalanced(credit);
                });
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    transaction_fee_paths,
    bench_charge_transaction_amount_path,
    bench_router_distribution_success
);
criterion_main!(transaction_fee_paths);
