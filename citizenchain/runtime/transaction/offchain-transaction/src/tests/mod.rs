//! 清算行 pallet 集成测试(Step 2b-iv-b + D 步新增)。
//!
//! 覆盖**端到端**路径:构造 mock `Test` runtime(含 `frame_system` +
//! `pallet_balances` + 本 pallet + mock SFID),用真实 sr25519 密钥签 L3 支付
//! 意图,经 `submit_offchain_batch_v2` 执行后断言链上 Storage 全部吻合。
//!
//! 测试清单:
//! - `bind_deposit_withdraw_full_cycle`      绑定 / 充值 / 提现三合一
//! - `double_bind_rejected`                  禁止重复绑定
//! - `bind_rejects_unregistered_bank`        拒绝未注册清算行
//! - `switch_requires_zero_balance`          先清零再切换
//! - `switch_after_withdraw_all_works`
//! - `submit_batch_rejects_non_admin`        非管理员提交批次应失败
//! - `submit_batch_same_bank_end_to_end`     单笔同行支付完整结算 + PaymentSettled 事件
//!
//! Mock 约束(与 `settlement.rs::pubkey_from_accountid` 对齐):`AccountId` 的
//! 32 字节 = sr25519 公钥;因此 fixture 里用 `Pair::from_seed()` 派生 L3 与
//! 清算行管理员账户,清算行主账户 / 费用账户用固定字节数组。

#![cfg(test)]

use crate as offchain_transaction;
use crate::{
    batch_item::OffchainBatchItemV2, pallet::*, BankTotalDeposits, DepositBalance, L2FeeRateBp,
    L3PaymentNonce, LastClearingBatchSeq, UserBank,
};
use codec::Encode;
use frame_support::{
    assert_noop, assert_ok, construct_runtime, derive_impl, parameter_types,
    traits::{ConstU32, Currency},
    BoundedVec,
};
use sp_core::{sr25519, Pair, H256};
use sp_io::TestExternalities;
use sp_runtime::{AccountId32, BuildStorage};

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
    pub enum Test {
        System: frame_system,
        Balances: pallet_balances,
        OffchainTx: offchain_transaction,
    }
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountId = AccountId32;
    type Lookup = sp_runtime::traits::IdentityLookup<AccountId32>;
    type AccountData = pallet_balances::AccountData<u128>;
}

parameter_types! {
    pub const ExistentialDeposit: u128 = 1;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type Balance = u128;
    type AccountStore = System;
    type ExistentialDeposit = ExistentialDeposit;
}

// ─── 清算行 fixture ───────────────────────────────────────────────────────

const BANK_MAIN_BYTES: [u8; 32] = [0xAA; 32];
const BANK_FEE_BYTES: [u8; 32] = [0xAB; 32];
const BANK_ADMIN_SEED: [u8; 32] = [0xAC; 32];
const OTHER_BANK_BYTES: [u8; 32] = [0xBA; 32];
const BANK_SFID: &[u8] = b"SFR-GD-SZ01-CB01-N9-D8";

fn bank_main() -> AccountId32 {
    AccountId32::new(BANK_MAIN_BYTES)
}
fn bank_fee() -> AccountId32 {
    AccountId32::new(BANK_FEE_BYTES)
}
fn bank_admin_pair() -> sr25519::Pair {
    sr25519::Pair::from_seed(&BANK_ADMIN_SEED)
}
fn bank_admin() -> AccountId32 {
    AccountId32::new(bank_admin_pair().public().0)
}

/// Mock `SfidAccountQuery`:把 `BANK_MAIN_BYTES` 注册为 SFR 主账户 Active,
/// `BANK_FEE_BYTES` 注册为 SFR 费用账户 Active;`bank_admin()` 是主账户唯一
/// 管理员。`OTHER_BANK_BYTES` 故意不注册,用于负路径。
pub struct MockSfid;

impl crate::bank_check::SfidAccountQuery<AccountId32> for MockSfid {
    fn account_info(addr: &AccountId32) -> Option<(sp_std::vec::Vec<u8>, sp_std::vec::Vec<u8>)> {
        let bytes: &[u8; 32] = addr.as_ref();
        if *bytes == BANK_MAIN_BYTES {
            Some((BANK_SFID.to_vec(), "主账户".as_bytes().to_vec()))
        } else if *bytes == BANK_FEE_BYTES {
            Some((BANK_SFID.to_vec(), "费用账户".as_bytes().to_vec()))
        } else {
            None
        }
    }

    fn find_address(sfid_number: &[u8], account_name: &[u8]) -> Option<AccountId32> {
        if sfid_number != BANK_SFID {
            return None;
        }
        if account_name == "主账户".as_bytes() {
            Some(AccountId32::new(BANK_MAIN_BYTES))
        } else if account_name == "费用账户".as_bytes() {
            Some(AccountId32::new(BANK_FEE_BYTES))
        } else {
            None
        }
    }

    fn is_active(addr: &AccountId32) -> bool {
        let bytes: &[u8; 32] = addr.as_ref();
        *bytes == BANK_MAIN_BYTES || *bytes == BANK_FEE_BYTES
    }

    fn is_admin_of(bank: &AccountId32, who: &AccountId32) -> bool {
        let bank_bytes: &[u8; 32] = bank.as_ref();
        *bank_bytes == BANK_MAIN_BYTES && who == &bank_admin()
    }

    /// Step 2 mock:测试场景默认 BANK_MAIN 满足资格白名单,负路径单测自行覆盖。
    fn is_clearing_bank_eligible(addr: &AccountId32) -> bool {
        let bytes: &[u8; 32] = addr.as_ref();
        *bytes == BANK_MAIN_BYTES
    }

    /// Step 2 mock:测试场景默认 BANK_MAIN 已声明清算行节点。
    fn is_registered_clearing_node(bank: &AccountId32) -> bool {
        let bytes: &[u8; 32] = bank.as_ref();
        *bytes == BANK_MAIN_BYTES
    }
}

impl offchain_transaction::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MaxBatchSize = ConstU32<256>;
    type MaxBatchSignatureLength = ConstU32<128>;
    type InstitutionAsset = (); // fail-open,测试白名单放行
    type SfidAccountQuery = MockSfid;
    type WeightInfo = ();
}

// ─── 公共 setup ───────────────────────────────────────────────────────────

fn new_test_ext() -> TestExternalities {
    let mut t: TestExternalities = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into();
    t.execute_with(|| {
        // 给清算行主账户 / 费用账户 / L3 预置充足余额,覆盖充值/提现/settlement 转账。
        Balances::make_free_balance_be(&bank_main(), 10_000_000_000u128);
        Balances::make_free_balance_be(&bank_fee(), 1_000_000u128);
        System::set_block_number(1);
    });
    t
}

/// 从 sr25519 种子生成 `(AccountId32, Pair)`,并给该账户预置余额。
fn new_l3_user(seed: &[u8; 32], balance: u128) -> (AccountId32, sr25519::Pair) {
    let pair = sr25519::Pair::from_seed(seed);
    let acc = AccountId32::new(pair.public().0);
    Balances::make_free_balance_be(&acc, balance);
    (acc, pair)
}

mod cases;
