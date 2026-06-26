//! 全仓签名消息唯一原语。
//!
//! 仓库一切「先拼域+op_tag+SCALE payload 再 blake2_256」的签名消息构造统一收敛到本
//! 模块的唯一原语 [`signing_message`] + 一张 op_tag 注册表。其它任何 crate / 模块禁止
//! 本地声明 `b"GMB_*_V1"` 字符串域,或重拼 `GMB || op_tag || payload`,一律调本模块。
//!
//! ## 签名分层选型纪律(铁律:默认①,op_tag 仅四类例外)
//!
//! 全仓存在两层签名,**严格按下表选型,不得逾越**:
//! - **① Substrate 标准 extrinsic 签名(框架自带,不在本模块)**:链上交易**默认且仅用**这一种。
//!   `ensure_signed(origin)` 即认动作人本人——签名人 = 该笔交易的 origin = 执行动作的本人。
//!   机构 admin 一人一票(投票 / 转账 / 内部投票 / 立法院内表决 / 行政签署 / 三人会签 / 护宪终审)
//!   全部属此,**零 op_tag**:身份能从 origin 直接认,无需第三方背书。
//! - **② / ③ 本模块 op_tag 签名(哈希域 0x10-0x17 / 二进制前缀域 0x18-0x19)**:**只有以下四类**
//!   才允许,且必须收敛在本文件:
//!   1. **第三方背书 / 内嵌凭证**:交易里嵌着「非交易发起人」签的资格证明
//!      (如公民公投 / 联合投票公投段:公民签①发交易,机构签② CID 凭证 `OP_SIGN_VOTE/POP`)。
//!   2. **链下支付**(L2/L3 通道 / 批量结算:`OP_SIGN_L3_PAY/OFFCHAIN_BATCH/L2_ACK`)——无交易可签。
//!   3. **链下 challenge**(解密授权:`OP_SIGN_DECRYPT`)——非交易上下文。
//!   4. **跨上下文离线证明**(管理员激活:`OP_SIGN_ACTIVATE_ADMIN` 等)。
//!
//! 判别口诀:**身份能从 origin 直接认 → 只①;身份需第三方背书 / 根本没有交易 → 才用 op_tag。**
//! 新增任何 op_tag 前必须先确认它落在上述四类之一,否则一律改用①。理由:① 是框架既有、最小、
//! 最不易错;op_tag 是「①结构上做不到」时的最小必要补充,绝不为已能用①的场景引入。
//!
//! 统一消息构造:
//! ```text
//! message = BLAKE2-256( GMB(3B) || op_tag(1B) || scale_payload )
//! ```
//! 其中 `GMB` = [`crate::core_const::GMB`],`scale_payload` 是调用方对业务字段做的
//! SCALE 编码字节(`(field1, field2, ...).encode()`)。
//!
//! ## 两范式归一(字节证明)
//!
//! SCALE 元组的编码 = 各元素编码的顺序拼接;`&[u8; 3]` 编码为 3 个裸字节(无长度前缀),
//! `u8` 编码为 1 字节。故对任意字段元组:
//! ```text
//! (GMB, op_tag, f1, f2, ...).encode()  ==  GMB || op_tag || (f1, f2, ...).encode()
//! ```
//! 因此治理 5 个签名(`OP_SIGN_BIND..=OP_SIGN_DEREGISTER`,0x10-0x14)调
//! `signing_message(op_tag, (fields).encode())` 与直接 `blake2_256((GMB, op_tag, fields).encode())`
//! **消息字节逐字节相等**(回归铁证,见 `tests/sign_golden.rs`)。
//!
//! ## 单源纪律
//!
//! 禁止本地声明 `b"GMB_*_V1"` 常量,全调本模块。Dart 侧(citizenapp / citizenwallet)
//! 是本模块的手写镜像,无编译期保证;靠金标向量
//! (`tests/fixtures/signing_domain_vectors.json`)逐字节断言对齐,防跨语言漂移。

use crate::core_const::GMB; // 域分隔符(地址派生 + 签名共用),单源在 core_const
use sp_core::hashing::blake2_256;
use sp_std::vec::Vec;

// ── QR_V1 扫码签名协议编号(跨端注释真源) ──
//
// QR_V1 顶层:
//   p = "QR_V1"
//   k = 场景编号
//   b.a = 签名动作编号
//
// 这些常量不参与 runtime 执行逻辑,但作为 Rust/Dart/TS/后端统一编号表的代码真源。
// 链交易动作统一用 `qr_chain_action(pallet_index, call_index)` 生成,避免再维护
// "transfer/internal_vote/..." 这类字符串动作名。

/// QR_V1 签名请求场景:生成方展示二维码,扫码方识别并签名。
pub const QR_KIND_SIGN_REQUEST: u8 = 1;
/// QR_V1 签名响应场景:扫码方展示签名结果,生成方扫码验签。
pub const QR_KIND_SIGN_RESPONSE: u8 = 2;
/// QR_V1 用户联系人固定码。
pub const QR_KIND_USER_CONTACT: u8 = 3;
/// QR_V1 用户转账固定码。
pub const QR_KIND_USER_TRANSFER: u8 = 4;
/// QR_V1 IM 节点配对固定码。
pub const QR_KIND_IM_NODE_PAIRING: u8 = 5;

/// QR_V1 登录签名动作。
pub const QR_ACTION_LOGIN: u16 = 1;
/// QR_V1 公民绑定签名动作。
pub const QR_ACTION_CITIZEN_BIND: u16 = 2;
/// QR_V1 CID 管理员治理/Passkey 更新签名动作。
pub const QR_ACTION_CID_ADMIN: u16 = 3;
/// QR_V1 CPMS 档案删除签名动作。
pub const QR_ACTION_CPMS_ARCHIVE_DELETE: u16 = 4;
/// QR_V1 管理员激活二进制原始签名动作。
pub const QR_ACTION_ACTIVATE_ADMIN: u16 = 5;
/// QR_V1 清算行管理员解密二进制原始签名动作。
pub const QR_ACTION_DECRYPT_ADMIN: u16 = 6;
/// QR_V1 runtime 升级 32 字节哈希直签动作。
pub const QR_ACTION_RUNTIME_UPGRADE_HASH: u16 = 7;

/// 链交易二维码动作码:高 8 位是 pallet index,低 8 位是 call index。
pub const fn qr_chain_action(pallet_index: u8, call_index: u8) -> u16 {
    ((pallet_index as u16) << 8) | call_index as u16
}

// ── 签名 payload op_tag 注册表 (0x10-0x1F),单一权威源 ──
//
// ## 两类签名域
//
// 1. **哈希域**(0x10-0x17,经 [`signing_message`] = `blake2_256(GMB||op_tag||SCALE)`):
//    - 0x10-0x14 治理/身份签名。
//    - 0x15-0x17 L3 支付 / 链下批次结算 / L2 确认。
//    只有这 8 个(0x10-0x17)经 `signing_message` 入 hash → 才进 [`SIGN_OP_TAGS`] + 金标遍历。
//
// 2. **二进制前缀域**(0x18/0x19,**不经 hash**,签**原始可解析字节**):
//    冷钱包对整段 payload 直接 sr25519 签名,node 按字节偏移解析。op_tag 常量仅作
//    payload **前 4 字节** `GMB(3B) || op_tag(1B)` 二进制前缀。
//    **不进 `SIGN_OP_TAGS`/金标 signing_message 遍历**(它们不走 `signing_message`),
//    其字节布局金标见 node 侧 `activate_admin_payload.json` / `decrypt_challenge.json`。
//
// 此外两个 IM 域(配对/绑定)是**字符串协议常量**,既非 hash 也非二进制前缀签名,
// 不占 op_tag 字节;见本模块末尾 [`IM_NODE_PAIRING_PROTO`] / [`IM_WALLET_BINDING_DOMAIN`]。
//
// 0x1A-0x1F 预留。账户地址派生 op_tag(0x00-0x0F)见 `account_derive`,命名空间不重叠。

/// 公民身份绑定。
pub const OP_SIGN_BIND: u8 = 0x10;
/// 公民投票。
pub const OP_SIGN_VOTE: u8 = 0x11;
/// 人口快照。
pub const OP_SIGN_POP: u8 = 0x12;
/// CID 机构登记。
pub const OP_SIGN_INST: u8 = 0x13;
/// CID 机构/账户注销凭证(注册局签发,链端 close 验签)。
pub const OP_SIGN_DEREGISTER: u8 = 0x14;

/// L3 支付。
pub const OP_SIGN_L3_PAY: u8 = 0x15;
/// 链下批次结算。
pub const OP_SIGN_OFFCHAIN_BATCH: u8 = 0x16;
/// L2 确认。
pub const OP_SIGN_L2_ACK: u8 = 0x17;

/// 管理员激活 **二进制前缀域**。
///
/// 非 hash 域:payload 前 4 字节为 `GMB || OP_SIGN_ACTIVATE_ADMIN`,其后接原始可解析字段
/// (account_id/code/kind/pubkey/timestamp/nonce),冷钱包对整段 payload sr25519 签名。
/// **不进 [`SIGN_OP_TAGS`]**(不走 `signing_message`)。
pub const OP_SIGN_ACTIVATE_ADMIN: u8 = 0x18;
/// 解密授权 **二进制前缀域**。
///
/// 非 hash 域:challenge payload 前 4 字节为 `GMB || OP_SIGN_DECRYPT`,其后接原始可解析
/// 字段(cid_number/pubkey/timestamp/nonce),冷钱包对整段 payload sr25519 签名。
/// **不进 [`SIGN_OP_TAGS`]**(不走 `signing_message`)。
pub const OP_SIGN_DECRYPT: u8 = 0x19;

/// 二进制前缀域(0x18/0x19)统一前缀长度:`GMB`(3B) + op_tag(1B) = 4 字节。
///
/// 冷钱包/node/citizenapp/citizenwallet 四方逐字节一致,所有偏移/长度常量以本值为基准。
pub const BINARY_PREFIX_LEN: usize = 4;

/// 构造二进制前缀域的 4 字节前缀 `GMB || op_tag`(0x18/0x19 用)。
///
/// 仅用于**原始字节签名**的二进制前缀域(ACTIVATE_ADMIN/DECRYPT),不做 hash。
/// 哈希域(0x10-0x17)请改调 [`signing_message`]。
pub fn binary_domain_prefix(op_tag: u8) -> [u8; BINARY_PREFIX_LEN] {
    let mut prefix = [0u8; BINARY_PREFIX_LEN];
    prefix[..GMB.len()].copy_from_slice(GMB);
    prefix[GMB.len()] = op_tag;
    prefix
}

/// 全部**哈希域**签名 op_tag(0x10-0x17,经 [`signing_message`])的注册表,
/// 供金标遍历与残留扫描。顺序与注册表声明一致;新增哈希域 op_tag 必须同步追加此数组
/// + 刷新金标。二进制前缀域(0x18/0x19)与 IM 字符串常量**不在此列**。
pub const SIGN_OP_TAGS: [u8; 8] = [
    OP_SIGN_BIND,
    OP_SIGN_VOTE,
    OP_SIGN_POP,
    OP_SIGN_INST,
    OP_SIGN_DEREGISTER,
    OP_SIGN_L3_PAY,
    OP_SIGN_OFFCHAIN_BATCH,
    OP_SIGN_L2_ACK,
];

/// 全仓签名消息唯一原语。
///
/// `message = BLAKE2-256( GMB(3B) || op_tag(1B) || scale_payload )`。
///
/// `scale_payload` 由调用方对业务字段做 SCALE 编码(`(f1, f2, ...).encode()`)。
/// 任何签名消息(治理/身份/支付/结算/IM/激活/解密)都必须经本入口构造,
/// 禁止在其它模块重拼域前缀或另写 `b"GMB_*_V1"` 字符串域。
pub fn signing_message(op_tag: u8, scale_payload: &[u8]) -> [u8; 32] {
    let mut data = Vec::with_capacity(GMB.len() + 1 + scale_payload.len());
    data.extend_from_slice(GMB);
    data.push(op_tag);
    data.extend_from_slice(scale_payload);
    blake2_256(&data)
}

// ── IM 协议字符串常量(单一权威源) ──
//
// 这两个**不是**签名 op_tag,既不经 [`signing_message`] 做 hash,也不作二进制前缀签名:
// - [`IM_WALLET_BINDING_DOMAIN`] 是管道分隔 UTF-8 canonical 字符串的首段(钱包对整段
//   canonical 字符串签名,但域是字符串字面,不进 op_tag hash 命名空间)。
// - [`IM_NODE_PAIRING_PROTO`] 是节点配对 QR body 的协议版本串,**不签名**。
//
// node `im::binding` + `settings::communication-node` + Dart 两端共用本常量为单源,
// 各端 import/镜像本值,不得本地另写副本。

/// IM 钱包绑定 canonical payload 的域首段。
///
/// 钱包对 `DOMAIN|wallet_account|im_device_id|...|nonce` 管道分隔 UTF-8 字符串签名;
/// 本值是该字符串的第一段(非 op_tag hash 域)。
pub const IM_WALLET_BINDING_DOMAIN: &str = "GMB_IM_WALLET_BINDING_V1";

/// IM 节点配对 QR body 的协议版本串。
///
/// 仅作配对 QR body 内 `proto` 字段值,**不参与任何签名**;本值是单一权威源。
pub const IM_NODE_PAIRING_PROTO: &str = "GMB_IM_NODE_PAIRING_V1";
