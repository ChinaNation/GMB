import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/shared/institution_code_label.dart';

class AdminSetValidationResult {
  const AdminSetValidationResult({
    required this.admins,
    required this.threshold,
  });

  final List<String> admins;
  final int threshold;
}

class AdminSetValidation {
  AdminSetValidation._();

  static AdminSetValidationResult validate({
    required AdminAccountState account,
    required String proposerPubkeyHex,
    required List<String> admins,
    required int newThreshold,
  }) {
    if (!account.isActive) {
      throw StateError('管理员账户不是已激活状态');
    }
    final proposer = _normalizePubkey(proposerPubkeyHex);
    if (!account.admins.contains(proposer)) {
      throw StateError('当前签名钱包不是该主体管理员');
    }
    final normalized = admins.map(_normalizePubkey).toList(growable: false);
    _validateCount(account.kind, account.institutionCode, normalized.length);
    if (normalized.toSet().length != normalized.length) {
      throw StateError('新管理员列表存在重复公钥');
    }
    if (account.admins.toSet().containsAll(normalized) &&
        normalized.toSet().containsAll(account.admins)) {
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

  /// 固定治理阈值：NRC=13，PRC/PRB=6，其他=null（动态）。
  static int? fixedGovernanceThreshold(String code) {
    return switch (code) {
      'NRC' => 13,
      'PRC' || 'PRB' => 6,
      _ => null,
    };
  }

  static String _normalizePubkey(String value) {
    final clean = AdminAccountIdCodec.normalizeHex(value);
    if (clean.length != 64 || !RegExp(r'^[0-9a-f]+$').hasMatch(clean)) {
      throw FormatException('管理员公钥必须为 64 位 hex', value);
    }
    return clean;
  }

  static void _validateCount(int kind, String code, int count) {
    if (kind == 0) {
      final expected = switch (code) {
        'NRC' => 19,
        'PRC' || 'PRB' => 9,
        'FRG' => null,
        _ => throw StateError('创世管理员 institution_code 无效: $code'),
      };
      if (expected != null && count != expected) {
        throw StateError('创世治理机构管理员数量必须保持 $expected 人');
      }
      if (expected == null && (count < 1 || count > 1989)) {
        throw StateError('联邦注册局管理员数量必须在 1..=1989 之间');
      }
      return;
    }
    if (kind == 1) {
      if (!InstitutionCodeLabel.canStorePublicAdminCode(code)) {
        throw StateError('公权机构管理员更换必须使用非创世公权机构码，或已明确归属公法人的非法人机构码');
      }
      if (count < 2 || count > 1989) {
        throw StateError('公权机构管理员数量必须在 2..=1989 之间');
      }
      return;
    }
    if (kind == 2) {
      if (!InstitutionCodeLabel.canStorePrivateAdminCode(code)) {
        throw StateError('私权机构管理员更换必须使用私权机构码，或已明确归属私法人的非法人机构码');
      }
      if (count < 2 || count > 1989) {
        throw StateError('私权机构管理员数量必须在 2..=1989 之间');
      }
      return;
    }
    if (kind == 3) {
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
    if (kind == 0) {
      final expected = fixedGovernanceThreshold(code);
      if (expected != null) {
        if (threshold != expected) {
          throw StateError('创世治理机构固定阈值必须为 $expected');
        }
        return;
      }
    }
    if (kind == 0 || kind == 1 || kind == 2 || kind == 3) {
      // 中文注释：动态账户阈值只按 runtime 投票引擎公式做端上前置校验；
      // 真正保存和生效仍由 internal-vote 负责。
      if (threshold <= 0 || threshold > count || threshold * 2 <= count) {
        throw StateError('动态阈值必须严格过半且不超过管理员数量');
      }
      return;
    }
    throw StateError('该管理员账户不能设置阈值');
  }
}
