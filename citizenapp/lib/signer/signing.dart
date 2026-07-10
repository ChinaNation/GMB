// GMB 全仓签名消息唯一原语 —— Dart 镜像。
//
// canonical 真源 = citizenchain `primitives::sign::signing_message`:
//   message = blake2_256( GMB(3B) || op_tag(1B) || scale_payload )
// 其中 GMB = core_const::GMB(字节 0x47 0x4D 0x42 = "GMB")。
//
// 与账户派生原语(account_derivation.dart)的区别:签名消息 **不含** ss58 字节,
// 域前缀只有 GMB(3B) + op_tag(1B)。op_tag 命名空间 0x10-0x1F 为签名段,与派生段
// 0x00-0x0F 不重叠。
//
// 本文件无编译期保证与链端一致,靠金标向量
//   citizenchain/runtime/primitives/tests/fixtures/signing_domain_vectors.json
// 的副本逐字节断言(见 test/signer/signing_golden_test.dart),防跨语言漂移。
//
// 单源纪律:任何模块禁止再本地拼 `GMB || op_tag || payload` 或写 `GMB_*_V1`
// 字符串域,一律调用本文件的 signingMessage。

import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;

// ── 签名 payload op_tag 注册表(0x10-0x1F),逐字节对齐 primitives::sign ──
//
// 0x10-0x14 治理/身份签名。
// 0x15-0x1A 业务签名段。

/// 公民身份上链确认(对齐 OP_SIGN_CITIZEN_IDENTITY)。
const int kOpSignCitizenIdentity = 0x10;

/// 机构登记(对齐 OP_SIGN_INST)。
const int kOpSignInst = 0x13;

/// 机构/账户注销凭证(对齐 OP_SIGN_DEREGISTER)。
const int kOpSignDeregister = 0x14;

/// L3 支付(对齐 OP_SIGN_L3_PAY)。
const int kOpSignL3Pay = 0x15;

/// 链下批次结算(对齐 OP_SIGN_OFFCHAIN_BATCH)。
const int kOpSignOffchainBatch = 0x16;

/// L2 确认(对齐 OP_SIGN_L2_ACK)。
const int kOpSignL2Ack = 0x17;

/// 管理员激活(对齐 OP_SIGN_ACTIVATE_ADMIN)。
const int kOpSignActivateAdmin = 0x18;

/// 解密授权(对齐 OP_SIGN_DECRYPT)。
const int kOpSignDecrypt = 0x19;

/// IM 钱包绑定(对齐 OP_SIGN_IM_WALLET_BINDING)。
const int kOpSignImWalletBinding = 0x1A;

/// 广场 BFF 登录挑战(对齐 OP_SIGN_SQUARE_LOGIN;链下 Worker 验签,设备子钥 ES256 签 digest)。
const int kOpSignSquareLogin = 0x1B;

/// 广场 BFF 设备子钥绑定(对齐 OP_SIGN_SQUARE_DEVICE_BIND;链下 Worker 验签,sr25519 主钥签)。
const int kOpSignSquareDeviceBind = 0x1C;

/// 广场 BFF 账户敏感动作:注销/退订(对齐 OP_SIGN_SQUARE_ACTION;链下 Worker 验签,sr25519 主钥签)。
const int kOpSignSquareAction = 0x1D;

// ── 二进制前缀域(0x18/0x19)──
//
// ACTIVATE_ADMIN / DECRYPT 不经 signingMessage 做 blake2 hash:冷钱包对整段
// 原始可解析 payload 直接 sr25519 签名,node 按字节偏移解析。其 op_tag
// (kOpSignActivateAdmin/kOpSignDecrypt)仅作 payload **前 4 字节**
// GMB(3B) || op_tag(1B) 二进制前缀。单源对齐 primitives::sign::
// binary_domain_prefix / BINARY_PREFIX_LEN。金标布局见
// test/signer/fixtures/binary_prefix_domain_vectors.json。

/// 二进制前缀域统一前缀长度 = GMB(3B) + op_tag(1B) = 4(对齐 BINARY_PREFIX_LEN)。
const int kBinaryPrefixLen = 4;

/// 签名域分隔符 GMB(3 字节 ASCII),单源对齐 core_const::GMB。
const List<int> kGmbSignDomain = [0x47, 0x4D, 0x42]; // "GMB"

/// 构造二进制前缀域的 4 字节前缀 GMB || op_tag(0x18/0x19 用)。
///
/// 仅用于**原始字节签名**的二进制前缀域(ACTIVATE_ADMIN/DECRYPT),不做 hash。
/// 哈希域(0x10-0x17)请改调 [signingMessage]。对齐
/// primitives::sign::binary_domain_prefix。
Uint8List binaryDomainPrefix(int opTag) {
  return Uint8List.fromList([...kGmbSignDomain, opTag & 0xFF]);
}

/// 全仓签名消息唯一原语:`blake2_256(GMB || op_tag || scalePayload)`。
///
/// [opTag] 取本文件 kOpSign* 常量;[scalePayload] 为调用方对业务字段做的
/// SCALE 编码字节(定长字段直接拼接即等价 SCALE)。返回 32 字节摘要。
Uint8List signingMessage({
  required int opTag,
  required List<int> scalePayload,
}) {
  final input = <int>[
    ...kGmbSignDomain,
    opTag & 0xFF,
    ...scalePayload,
  ];
  return Hasher.blake2b256.hash(Uint8List.fromList(input));
}

// ── SCALE 编码原语(签名 payload 拼装用),逐字节对齐链端/Worker ──

/// SCALE 编码字符串:`compact(len) || utf8(value)`。
Uint8List scaleString(String value) {
  final bytes = utf8.encode(value);
  return Uint8List.fromList([..._scaleCompact(bytes.length), ...bytes]);
}

/// u64 小端 8 字节(时间戳等定长字段)。
Uint8List u64Le(int value) {
  if (value < 0) {
    throw ArgumentError.value(value, 'value', 'u64 不允许负数');
  }
  final out = Uint8List(8);
  var current = value;
  for (var i = 0; i < out.length; i++) {
    out[i] = current & 0xff;
    current >>= 8;
  }
  return out;
}

/// SCALE compact 编码非负整数(支持到 2^30-1)。
List<int> _scaleCompact(int value) {
  if (value < 0) {
    throw ArgumentError.value(value, 'value', 'SCALE compact 不允许负数');
  }
  if (value < 1 << 6) {
    return [value << 2];
  }
  if (value < 1 << 14) {
    final v = (value << 2) | 0x01;
    return [v & 0xff, (v >> 8) & 0xff];
  }
  if (value < 1 << 30) {
    final v = (value << 2) | 0x02;
    return [v & 0xff, (v >> 8) & 0xff, (v >> 16) & 0xff, (v >> 24) & 0xff];
  }
  throw ArgumentError.value(value, 'value', 'SCALE compact 超出本地支持范围');
}
