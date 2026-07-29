import 'dart:convert';

import 'package:polkadart_keyring/polkadart_keyring.dart';

import 'package:citizenwallet/qr/envelope.dart';

class UserContactBody implements QrBody {
  const UserContactBody({
    required this.cidNumber,
    required this.ss58Address,
    required this.displayName,
  });

  final String cidNumber;
  final String ss58Address;
  final String displayName;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'cid_number': cidNumber,
        'ss58_address': ss58Address,
        'display_name': displayName,
      };

  static UserContactBody fromJson(Map<String, dynamic> data) {
    requireExactKeys(
      data,
      const {'cid_number', 'ss58_address', 'display_name'},
      'user_contact.b',
    );
    final cidNumber = data['cid_number'];
    final ss58Address = data['ss58_address'];
    final displayName = data['display_name'];
    if (cidNumber is! String ||
        cidNumber != cidNumber.trim() ||
        cidNumber.isEmpty ||
        utf8.encode(cidNumber).length > 32) {
      throw const FormatException(
        'user_contact.cid_number 必须为无首尾空格的 1 到 32 字节字符串',
      );
    }
    if (ss58Address is! String || !_isCanonicalGmbSs58(ss58Address)) {
      throw const FormatException(
        'user_contact.ss58_address 必须为本链规范 SS58 地址',
      );
    }
    if (displayName is! String ||
        displayName != displayName.trim() ||
        displayName.isEmpty ||
        displayName.runes.length > 40) {
      throw const FormatException(
        'user_contact.display_name 必须为无首尾空格的 1 到 40 字符串',
      );
    }
    return UserContactBody(
      cidNumber: cidNumber,
      ss58Address: ss58Address,
      displayName: displayName,
    );
  }

  static bool _isCanonicalGmbSs58(String value) {
    if (value.isEmpty || value != value.trim()) return false;
    try {
      final accountId = Keyring().decodeAddress(value);
      return Keyring().encodeAddress(accountId, 2027) == value;
    } catch (_) {
      return false;
    }
  }
}
