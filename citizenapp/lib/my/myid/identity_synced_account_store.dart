import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import 'package:citizenapp/citizen/shared/account_derivation.dart';

/// 本机已经完整接管的 CID finalized 绑定标记。
///
/// 单独记录 account_id 会遗漏“同一账户撤销后重新绑定”等版本变化，因此必须把 CID、
/// binding_revision、当前账户及稳定数据根摘要作为一个不可拆分的完成标记。
class IdentitySyncedBinding {
  const IdentitySyncedBinding({
    required this.cidNumber,
    required this.bindingRevision,
    required this.accountId,
    required this.dataRootHash,
  });

  final String cidNumber;
  final int bindingRevision;
  final String accountId;
  final String dataRootHash;

  Map<String, Object> toJson() => <String, Object>{
        'cid_number': cidNumber,
        'binding_revision': bindingRevision,
        'account_id': accountId,
        'data_root_hash': dataRootHash,
      };

  static IdentitySyncedBinding? fromJson(String raw) {
    try {
      final value = jsonDecode(raw);
      if (value is! Map<String, dynamic>) return null;
      final cidNumber = value['cid_number'];
      final bindingRevision = value['binding_revision'];
      final accountId = value['account_id'];
      final dataRootHash = value['data_root_hash'];
      if (cidNumber is! String ||
          cidNumber.isEmpty ||
          utf8.encode(cidNumber).length > 32 ||
          bindingRevision is! int ||
          bindingRevision <= 0 ||
          accountId is! String ||
          !isAccountIdText(accountId) ||
          dataRootHash is! String ||
          !RegExp(r'^[0-9a-f]{64}$').hasMatch(dataRootHash)) {
        return null;
      }
      return IdentitySyncedBinding(
        cidNumber: cidNumber,
        bindingRevision: bindingRevision,
        accountId: accountId,
        dataRootHash: dataRootHash,
      );
    } catch (_) {
      return null;
    }
  }
}

class IdentitySyncedAccountStore {
  IdentitySyncedAccountStore({SharedPreferences? preferences})
      : _preferences = preferences;

  static const _key = 'identity_synced_binding';
  static const _removedAccountOnlyKey = 'identity_synced_account_v1';

  final SharedPreferences? _preferences;

  Future<SharedPreferences> get _prefs {
    final preferences = _preferences;
    if (preferences != null) return Future.value(preferences);
    return SharedPreferences.getInstance();
  }

  /// 已完整接管的精确绑定；无记录或损坏时返回 null，触发重新接管。
  Future<IdentitySyncedBinding?> read() async {
    final prefs = await _prefs;
    final raw = prefs.getString(_key);
    if (raw == null || raw.isEmpty) return null;
    return IdentitySyncedBinding.fromJson(raw);
  }

  Future<void> write(IdentitySyncedBinding binding) async {
    final validated =
        IdentitySyncedBinding.fromJson(jsonEncode(binding.toJson()));
    if (validated == null) {
      throw ArgumentError.value(binding, 'binding', 'CID 接管完成标记不合法');
    }
    final prefs = await _prefs;
    await prefs.setString(_key, jsonEncode(validated.toJson()));
    // 旧账户单字段标记只删不读，避免重新成为授权或完成状态真源。
    await prefs.remove(_removedAccountOnlyKey);
  }

  Future<void> clear() async {
    final prefs = await _prefs;
    await prefs.remove(_key);
    await prefs.remove(_removedAccountOnlyKey);
  }
}
