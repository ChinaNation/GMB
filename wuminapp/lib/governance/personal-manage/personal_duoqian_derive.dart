import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

/// 个人多签地址派生的全 app 唯一入口。
///
/// 与 citizenchain `primitives::core_const::{DUOQIAN, OP_PERSONAL, derive_duoqian_account}`
/// 严格对齐：
///   preimage = b"DUOQIAN" || OP_PERSONAL(0x05) || ss58.to_le_bytes()
///              || creator(32B) || account_name
///   address  = ss58(blake2b_256(preimage))
///
/// 禁止任何页面/服务再自行拼接 preimage 派生地址；一律调用本函数。
String deriveDuoqianPersonalAddress({
  required Uint8List creatorPubkey,
  required String accountName,
  int ss58Prefix = 2027,
}) {
  if (creatorPubkey.length != 32) {
    throw ArgumentError('creator pubkey 必须为 32 字节');
  }
  final nameBytes = utf8.encode(accountName);
  final input = <int>[
    ...utf8.encode('DUOQIAN'),
    0x05, // OP_PERSONAL
    ss58Prefix & 0xFF,
    (ss58Prefix >> 8) & 0xFF,
    ...creatorPubkey,
    ...nameBytes,
  ];
  final digest = Hasher.blake2b256.hash(Uint8List.fromList(input));
  return Keyring().encodeAddress(digest, ss58Prefix);
}
