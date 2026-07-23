// GMB 账户统一派生原语 —— 全 app 唯一入口。
//
// 与 citizenchain `primitives::core_const` 单一权威源严格字节对齐:
//   preimage = b"GMB"(3B) || op_tag(1B) || ss58.to_le_bytes()(2B) || payload
//   address  = blake2b_256(preimage)            // 32 字节 account id
// op_tag 与 payload(见各常量注释)逐一对齐 core_const.rs;任何模块禁止
// 自行拼接 preimage,一律调用本文件。
//
// 三种取值形态:account id(Uint8List 32B,链上读用)/ hex(小写 64 字符,
// 余额接口 + 注册表对账用)/ SS58(prefix=2027,展示用)。

import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'reserved_account_names.dart';

/// GMB 链 SS58 地址前缀(对齐 core_const::SS58_FORMAT)。
const int kGmbSs58Prefix = 2027;

// ── 账户派生 op_tag(0x00-0x0F),对齐 core_const.rs,不得复用 ──
/// 所有机构主账户 · payload = cid_number。
const int kOpMain = 0x00;

/// 所有机构费用账户 · payload = cid_number。
const int kOpFee = 0x01;

/// 永久质押(制度专属)· payload = cid_number。
const int kOpStake = 0x02;

/// 安全基金(制度专属)· payload = cid_number。
const int kOpSafetyFund = 0x03;

/// 两和基金(制度专属)· payload = cid_number。
const int kOpHe = 0x04;

/// 个人多签账户 · payload = creator(32B) || account_name。
const int kOpPersonal = 0x05;

/// 清算账户(私法人股份公司专属)· payload = cid_number。
const int kOpClearing = 0x06;

/// CID 机构自定义命名账户 · payload = cid_number || account_name。
const int kOpName = 0x07;

const String _domain = 'GMB';

/// 派生 GMB account id(32 字节)。底层唯一实现,上层全部经此。
Uint8List deriveAccountId({
  required int opTag,
  required List<int> payload,
  int ss58Prefix = kGmbSs58Prefix,
}) {
  final input = <int>[
    ...utf8.encode(_domain),
    opTag & 0xFF,
    ss58Prefix & 0xFF,
    (ss58Prefix >> 8) & 0xFF,
    ...payload,
  ];
  return Hasher.blake2b256.hash(Uint8List.fromList(input));
}

/// account id → 小写 hex(64 字符,无 0x 前缀),与链上 storage key / 注册表口径一致。
String hexFromAccountId(Uint8List id) =>
    id.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

/// account id → SS58(默认 prefix=2027),仅供展示。
String ss58FromAccountId(Uint8List id, {int ss58Prefix = kGmbSs58Prefix}) =>
    Keyring().encodeAddress(id, ss58Prefix);

/// hex(小写/大写、带或不带 0x,64 字符)→ SS58(默认 prefix=2027),仅供展示。
/// 便于把余额接口/存储解出的账户 hex 直接转成展示地址,是 hex→SS58 的唯一便捷入口。
String ss58FromHex(String hex, {int ss58Prefix = kGmbSs58Prefix}) {
  final h =
      hex.startsWith('0x') || hex.startsWith('0X') ? hex.substring(2) : hex;
  final bytes = Uint8List(h.length ~/ 2);
  for (var i = 0; i < bytes.length; i++) {
    bytes[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
  }
  return ss58FromAccountId(bytes, ss58Prefix: ss58Prefix);
}

// ── 机构账户(payload 含 cid_number)──

/// 机构主账户 id(OP_MAIN,payload = cid_number,不含名字)。
Uint8List deriveInstitutionMainAccountId(
  String cidNumber, {
  int ss58Prefix = kGmbSs58Prefix,
}) =>
    deriveAccountId(
      opTag: kOpMain,
      payload: utf8.encode(cidNumber),
      ss58Prefix: ss58Prefix,
    );

/// 机构费用账户 id(OP_FEE,payload = cid_number,不含名字)。
Uint8List deriveInstitutionFeeAccountId(
  String cidNumber, {
  int ss58Prefix = kGmbSs58Prefix,
}) =>
    deriveAccountId(
      opTag: kOpFee,
      payload: utf8.encode(cidNumber),
      ss58Prefix: ss58Prefix,
    );

/// 机构清算账户 id(OP_CLEARING,payload = cid_number,不含名字)。
///
/// 仅私法人股份公司(SFGF)自动拥有;地址派生与主/费同构,不参与自定义命名。
Uint8List deriveInstitutionClearingAccountId(
  String cidNumber, {
  int ss58Prefix = kGmbSs58Prefix,
}) =>
    deriveAccountId(
      opTag: kOpClearing,
      payload: utf8.encode(cidNumber),
      ss58Prefix: ss58Prefix,
    );

/// 机构自定义命名账户 id(OP_NAME,payload = cid_number || account_name)。
///
/// 字节对齐链端:account_name 取原始字节不 trim;空名/主/费/制度专属名一律拒绝
/// (对齐链端注册策略 is_registrable_custom_name:空→EmptyAccountName,
/// 主/费走各自 op_tag,质押/安全/两和禁注册)。
Uint8List deriveInstitutionCustomAccountId(
  String cidNumber,
  String accountName, {
  int ss58Prefix = kGmbSs58Prefix,
}) {
  if (!isRegistrableCustomName(accountName)) {
    throw ArgumentError('自定义账户名不可注册(空/主/费/制度专属): $accountName');
  }
  return deriveAccountId(
    opTag: kOpName,
    payload: <int>[...utf8.encode(cidNumber), ...utf8.encode(accountName)],
    ss58Prefix: ss58Prefix,
  );
}

/// 按 account_name 路由派生机构账户 id —— 镜像 CID `accounts/derive.rs` 单一源:
/// 主/费/质押/安全基金/两和基金 → 各自 op_tag(payload 不含名字);其他非空名
/// → OP_INSTITUTION(payload 追加名字)。空名抛错。
Uint8List deriveInstitutionAccountIdByName(
  String cidNumber,
  String accountName, {
  int ss58Prefix = kGmbSs58Prefix,
}) {
  if (accountName.isEmpty) {
    throw ArgumentError('账户名不能为空(对齐链端 EmptyAccountName)');
  }
  final (int opTag, bool appendName) = switch (accountName) {
    kReservedNameMain => (kOpMain, false),
    kReservedNameFee => (kOpFee, false),
    kReservedNameStake => (kOpStake, false),
    kReservedNameSafetyFund => (kOpSafetyFund, false),
    kReservedNameHe => (kOpHe, false),
    kReservedNameClearing => (kOpClearing, false),
    _ => (kOpName, true),
  };
  final payload = <int>[
    ...utf8.encode(cidNumber),
    if (appendName) ...utf8.encode(accountName),
  ];
  return deriveAccountId(
    opTag: opTag,
    payload: payload,
    ss58Prefix: ss58Prefix,
  );
}

// ── 个人多签(payload = creator || account_name)──

/// 个人多签账户派生(OP_PERSONAL),返回 SS58。归位自原
/// `personal-manage/personal_account_derive.dart`,行为不变。
String derivePersonalAccountSs58({
  required Uint8List creatorPubkey,
  required String accountName,
  int ss58Prefix = kGmbSs58Prefix,
}) {
  if (creatorPubkey.length != 32) {
    throw ArgumentError('creator pubkey 必须为 32 字节');
  }
  final id = deriveAccountId(
    opTag: kOpPersonal,
    payload: <int>[...creatorPubkey, ...utf8.encode(accountName)],
    ss58Prefix: ss58Prefix,
  );
  return ss58FromAccountId(id, ss58Prefix: ss58Prefix);
}
