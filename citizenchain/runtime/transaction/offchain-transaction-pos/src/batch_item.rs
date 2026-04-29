//! 扫码支付清算体系 Step 1 新增:支付意图与签名数据结构。
//!
//! 中文注释:
//! - `PaymentIntent` 是 L3 用私钥签名的原始数据,wuminapp 本地签,链上验。
//! - 本文件**不引入新的 Storage**,仅提供纯结构与签名哈希函数。
//! - `OffchainBatchItem` 在 Step 2 引入(当前 lib.rs 的旧结构 Step 2 重写)。

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_std::vec::Vec;

/// L3 扫码支付意图,wuminapp 本地 sr25519 签名的原始数据。
///
/// 字段顺序**必须与 wuminapp 的 Dart 实现逐字段对齐**,否则签名哈希不一致。
#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, TypeInfo, MaxEncodedLen)]
pub struct PaymentIntent<AccountId, BlockNumber> {
    /// 全局唯一交易 ID(防重放,与链上 ProcessedOffchainTx 联动)。
    pub tx_id: H256,
    /// 付款方 L3 公钥(L3 的 AccountId,同链上地址)。
    pub payer: AccountId,
    /// 付款方绑定的清算行**主账户**地址。
    pub payer_bank: AccountId,
    /// 收款方 L3 公钥。
    pub recipient: AccountId,
    /// 收款方绑定的清算行**主账户**地址。
    pub recipient_bank: AccountId,
    /// 转账金额(分)。
    pub amount: u128,
    /// 本笔手续费(分),按收款方清算行费率计算。
    pub fee: u128,
    /// L3 的单调递增 nonce,与链上 `L3PaymentNonce` 对应。
    pub nonce: u64,
    /// 签名过期高度(链上 `execute` 时校验 `current_block <= expires_at`)。
    pub expires_at: BlockNumber,
}

/// L3 签名域分隔符,与 wuminapp 保持**逐字节一致**。
pub const L3_PAY_SIGNING_DOMAIN: &[u8] = b"GMB_L3_PAY_V1";

/// 清算行批次级签名域分隔符,必须与 node/offchain/packer.rs 保持一致。
pub const BATCH_SIGNING_DOMAIN: &[u8] = b"GMB_OFFCHAIN_BATCH_V1";

impl<AccountId: Encode, BlockNumber: Encode> PaymentIntent<AccountId, BlockNumber> {
    /// 生成签名消息哈希:`blake2_256(DOMAIN || SCALE(intent))`。
    ///
    /// wuminapp 的 Dart 端必须用同样的拼接顺序,否则签名验证失败。
    pub fn signing_hash(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(L3_PAY_SIGNING_DOMAIN);
        data.extend_from_slice(&self.encode());
        sp_io::hashing::blake2_256(&data)
    }
}

/// 生成清算行批次签名哈希:
/// `blake2_256(DOMAIN || SCALE(institution_main) || batch_seq_le || SCALE(batch))`。
///
/// node 侧 `AccountId32.as_ref()` 与 SCALE 编码同为 32 字节,这里使用 `Encode`
/// 是为了让 runtime 维持泛型边界,同时锁定与节点打包器逐字节一致的消息格式。
pub fn batch_signing_hash<AccountId: Encode>(
    institution_main: &AccountId,
    batch_seq: u64,
    batch_bytes: &[u8],
) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(BATCH_SIGNING_DOMAIN);
    data.extend_from_slice(&institution_main.encode());
    data.extend_from_slice(&batch_seq.to_le_bytes());
    data.extend_from_slice(batch_bytes);
    sp_io::hashing::blake2_256(&data)
}

/// 扫码支付清算体系 Step 2 新增:批次上链的**单条 item 结构**(清算行体系)。
///
/// 与现有 `pallet::OffchainBatchItem`(旧省储行模型,字段少)并存,Step 2 起
/// 新 `submit_offchain_batch_v2` extrinsic 使用本结构。
///
/// 字段顺序必须与 wuminapp Dart 端的 SCALE 编码逐字段对齐。
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Encode,
    Decode,
    frame_support::pallet_prelude::DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
)]
pub struct OffchainBatchItemV2<AccountId, BlockNumber> {
    pub tx_id: H256,
    pub payer: AccountId,
    pub payer_bank: AccountId,
    pub recipient: AccountId,
    pub recipient_bank: AccountId,
    pub transfer_amount: u128,
    pub fee_amount: u128,
    /// L3 sr25519 签名(64 字节)。
    pub payer_sig: [u8; 64],
    pub payer_nonce: u64,
    pub expires_at: BlockNumber,
}

impl<AccountId: Clone + Encode, BlockNumber: Clone + Encode>
    OffchainBatchItemV2<AccountId, BlockNumber>
{
    /// 还原为 `PaymentIntent`(用于重算签名哈希以验签)。
    pub fn to_intent(&self) -> PaymentIntent<AccountId, BlockNumber> {
        PaymentIntent {
            tx_id: self.tx_id,
            payer: self.payer.clone(),
            payer_bank: self.payer_bank.clone(),
            recipient: self.recipient.clone(),
            recipient_bank: self.recipient_bank.clone(),
            amount: self.transfer_amount,
            fee: self.fee_amount,
            nonce: self.payer_nonce,
            expires_at: self.expires_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_runtime::AccountId32;

    fn acc(b: u8) -> AccountId32 {
        AccountId32::new([b; 32])
    }

    #[test]
    fn signing_hash_is_deterministic() {
        let intent = PaymentIntent::<AccountId32, u32> {
            tx_id: H256::repeat_byte(9),
            payer: acc(1),
            payer_bank: acc(2),
            recipient: acc(3),
            recipient_bank: acc(2),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let h1 = intent.signing_hash();
        let h2 = intent.signing_hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn signing_hash_changes_with_any_field() {
        let base = PaymentIntent::<AccountId32, u32> {
            tx_id: H256::repeat_byte(9),
            payer: acc(1),
            payer_bank: acc(2),
            recipient: acc(3),
            recipient_bank: acc(2),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let mut changed = base.clone();
        changed.amount = 10_001;
        assert_ne!(base.signing_hash(), changed.signing_hash());
    }

    // ─── 扫码支付 Step 2c-ii-a golden vectors ────────────────────────────
    //
    // 目的:锁定 `PaymentIntent` 的 SCALE 编码布局 + signing_hash 算法,使
    // wuminapp `payment_intent.dart::NodePaymentIntent` 端的编码/哈希必须逐
    // 字节一致。wuminapp 端 `test/trade/payment_intent_golden_test.dart` 写入
    // **相同的 fixture + 相同的期望 hex**,任一端实现漂移 → 两端 CI 同时红。
    //
    // 布局(固定 204 字节,详见 batch_item.rs 结构注释):
    //   tx_id(32) || payer(32) || payer_bank(32) || recipient(32) ||
    //   recipient_bank(32) || amount(u128 LE,16) || fee(u128 LE,16) ||
    //   nonce(u64 LE,8) || expires_at(u32 LE,4)
    //
    // 签名域 `GMB_L3_PAY_V1`(13 字节 ASCII) || encoded(204) → blake2_256 = 32 字节。

    /// 把 `&[u8]` 转 hex(无 `0x` 前缀,小写),测试断言格式。
    fn hex_lower(bytes: &[u8]) -> sp_std::vec::Vec<u8> {
        let mut out = sp_std::vec::Vec::with_capacity(bytes.len() * 2);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for b in bytes {
            out.push(HEX[(*b >> 4) as usize]);
            out.push(HEX[(*b & 0x0F) as usize]);
        }
        out
    }

    fn assert_hex_eq(label: &str, bytes: &[u8], expected: &str) {
        let got = hex_lower(bytes);
        let got_str = core::str::from_utf8(&got).expect("hex ascii");
        assert_eq!(got_str, expected, "{label} hex mismatch");
    }

    /// Fixture 1:简单同行支付。所有定长字段用重复字节、金额 10_000 分 / 费 5 分
    /// / nonce 1 / expires_at 100。
    #[test]
    fn golden_fixture1_simple_same_bank() {
        let intent = PaymentIntent::<AccountId32, u32> {
            tx_id: H256::zero(),
            payer: acc(1),
            payer_bank: acc(2),
            recipient: acc(3),
            recipient_bank: acc(2),
            amount: 10_000,
            fee: 5,
            nonce: 1,
            expires_at: 100,
        };
        let encoded = intent.encode();
        assert_eq!(encoded.len(), 204);
        assert_hex_eq(
            "fixture1 encoded",
            &encoded,
            "\
0000000000000000000000000000000000000000000000000000000000000000\
0101010101010101010101010101010101010101010101010101010101010101\
0202020202020202020202020202020202020202020202020202020202020202\
0303030303030303030303030303030303030303030303030303030303030303\
0202020202020202020202020202020202020202020202020202020202020202\
10270000000000000000000000000000\
05000000000000000000000000000000\
0100000000000000\
64000000",
        );
        assert_hex_eq(
            "fixture1 signing_hash",
            &intent.signing_hash(),
            "f50eeb66b681e445ee6fcffa318288b915fdea9791eae1d094645d4eb5f7008f",
        );
    }

    /// Fixture 2:跨行 + 大金额 + 大 nonce + 大 expires_at。锁字节序与字段位置。
    #[test]
    fn golden_fixture2_cross_bank_big_values() {
        let intent = PaymentIntent::<AccountId32, u32> {
            tx_id: H256::repeat_byte(0xFF),
            payer: acc(0x11),
            payer_bank: acc(0xAA),
            recipient: acc(0x22),
            recipient_bank: acc(0xBB),
            amount: u128::MAX,
            fee: u128::MAX,
            nonce: u64::MAX,
            expires_at: u32::MAX,
        };
        let encoded = intent.encode();
        assert_eq!(encoded.len(), 204);
        assert_hex_eq(
            "fixture2 encoded",
            &encoded,
            "\
ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff\
1111111111111111111111111111111111111111111111111111111111111111\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
2222222222222222222222222222222222222222222222222222222222222222\
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\
ffffffffffffffffffffffffffffffff\
ffffffffffffffffffffffffffffffff\
ffffffffffffffff\
ffffffff",
        );
        assert_hex_eq(
            "fixture2 signing_hash",
            &intent.signing_hash(),
            "d6f381b931ad0f2c7f7fba5d83bdd24892ccbd0e063d831ebc00d2e6d21c9bd8",
        );
    }

    /// Fixture 3:零金额边界。tx_id 递增字节,其他保守。
    #[test]
    fn golden_fixture3_zero_amount_incrementing_tx() {
        let mut tx_bytes = [0u8; 32];
        for (i, b) in tx_bytes.iter_mut().enumerate() {
            *b = i as u8; // 0x00..0x1F
        }
        let intent = PaymentIntent::<AccountId32, u32> {
            tx_id: H256::from(tx_bytes),
            payer: acc(0x55),
            payer_bank: acc(0x77),
            recipient: acc(0x66),
            recipient_bank: acc(0x77),
            amount: 0,
            fee: 0,
            nonce: 0,
            expires_at: 0,
        };
        let encoded = intent.encode();
        assert_eq!(encoded.len(), 204);
        assert_hex_eq(
            "fixture3 encoded",
            &encoded,
            "\
000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\
5555555555555555555555555555555555555555555555555555555555555555\
7777777777777777777777777777777777777777777777777777777777777777\
6666666666666666666666666666666666666666666666666666666666666666\
7777777777777777777777777777777777777777777777777777777777777777\
00000000000000000000000000000000\
00000000000000000000000000000000\
0000000000000000\
00000000",
        );
        assert_hex_eq(
            "fixture3 signing_hash",
            &intent.signing_hash(),
            "8e99dbc826503544b240ed3e113f999bc3928048aa69989118f517309286a1b2",
        );
    }
}
