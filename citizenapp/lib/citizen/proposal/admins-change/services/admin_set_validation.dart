import 'dart:convert';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';

class AdminSetValidationResult {
  const AdminSetValidationResult({
    required this.admins,
    required this.threshold,
  });

  final List<AdminPerson> admins;
  final int threshold;
}

class AdminSetValidation {
  AdminSetValidation._();

  static AdminSetValidationResult validate({
    required AdminAccountState account,
    required String proposerPubkeyHex,
    required List<AdminPerson> admins,
    required int newThreshold,
  }) {
    if (!account.isActive) {
      throw StateError('管理员账户不是已激活状态');
    }
    if (account.kind != 2 || account.institutionCode != 'PMUL') {
      throw StateError('机构管理员由 entity 任职结果管理；本流程只允许个人多签');
    }
    final proposer = _normalizePubkey(proposerPubkeyHex);
    if (!account.admins.any((admin) => admin.admin_account == proposer)) {
      throw StateError('当前签名钱包不是该主体管理员');
    }
    final normalized = admins
        .map(
          (admin) => AdminPerson(
            admin_account: _normalizePubkey(admin.admin_account),
            family_name: _normalizeName(admin.family_name, '管理', '姓'),
            given_name: _normalizeName(admin.given_name, '员', '名'),
          ),
        )
        .toList(growable: false);
    _validateCount(account.kind, account.institutionCode, normalized.length);
    final nextAccounts = normalized.map((admin) => admin.admin_account).toSet();
    if (nextAccounts.length != normalized.length) {
      throw StateError('新管理员列表存在重复公钥');
    }
    if (_sameAdmins(account.admins, normalized)) {
      throw StateError('新管理员集合与当前集合没有变化');
    }
    _validateThreshold(
        account.kind, account.institutionCode, normalized.length, newThreshold);
    return AdminSetValidationResult(
        admins: normalized, threshold: newThreshold);
  }

  static int minimumDynamicThreshold(int adminsLen) {
    return (adminsLen ~/ 2) + 1;
  }

  static String _normalizePubkey(String value) {
    final clean = AdminAccountIdCodec.normalizeHex(value);
    if (clean.length != 64 || !RegExp(r'^[0-9a-f]+$').hasMatch(clean)) {
      throw FormatException('管理员公钥必须为 64 位 hex', value);
    }
    return clean;
  }

  static String _normalizeName(String value, String fallback, String label) {
    final normalized = value.trim().isEmpty ? fallback : value.trim();
    if (utf8.encode(normalized).length > 128) {
      throw FormatException('管理员$label不得超过 128 字节', value);
    }
    return normalized;
  }

  static bool _sameAdmins(List<AdminPerson> left, List<AdminPerson> right) {
    if (left.length != right.length) return false;
    final leftByAccount = {
      for (final admin in left) admin.admin_account: admin,
    };
    for (final admin in right) {
      final current = leftByAccount[admin.admin_account];
      if (current == null ||
          current.family_name != admin.family_name ||
          current.given_name != admin.given_name) {
        return false;
      }
    }
    return true;
  }

  static void _validateCount(int kind, String code, int count) {
    if (kind == 2) {
      if (code != 'PMUL') {
        throw StateError('个人多签管理员更换必须使用 PMUL');
      }
      if (count < 2 || count > 64) throw StateError('个人多签管理员数量必须在 2..=64 之间');
      return;
    }
    throw StateError('未知管理员账户类型');
  }

  static void _validateThreshold(
    int kind,
    String code,
    int count,
    int threshold,
  ) {
    if (kind == 2 && code == 'PMUL') {
      // 动态账户阈值只按 runtime 投票引擎公式做端上前置校验；
      // 真正保存和生效仍由 internal-vote 负责。
      if (threshold <= 0 || threshold > count || threshold * 2 <= count) {
        throw StateError('动态阈值必须严格过半且不超过管理员数量');
      }
      return;
    }
    throw StateError('该管理员账户不能设置阈值');
  }
}
