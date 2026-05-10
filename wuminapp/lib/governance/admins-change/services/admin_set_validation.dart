import 'package:wuminapp_mobile/governance/admins-change/codec/subject_id_codec.dart';
import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';

class AdminSetValidation {
  AdminSetValidation._();

  static List<String> validate({
    required AdminSubjectState subject,
    required String proposerPubkeyHex,
    required List<String> newAdmins,
  }) {
    if (!subject.isActive) {
      throw StateError('管理员主体不是已激活状态');
    }
    final proposer = _normalizePubkey(proposerPubkeyHex);
    if (!subject.admins.contains(proposer)) {
      throw StateError('当前签名钱包不是该主体管理员');
    }
    final normalized = newAdmins.map(_normalizePubkey).toList(growable: false);
    _validateCount(subject.kind, subject.org, normalized.length);
    if (normalized.toSet().length != normalized.length) {
      throw StateError('新管理员列表存在重复公钥');
    }
    if (subject.admins.toSet().containsAll(normalized) &&
        normalized.toSet().containsAll(subject.admins)) {
      throw StateError('新管理员集合与当前集合没有变化');
    }
    return normalized;
  }

  static String _normalizePubkey(String value) {
    final clean = AdminSubjectIdCodec.normalizeHex(value);
    if (clean.length != 64 || !RegExp(r'^[0-9a-f]+$').hasMatch(clean)) {
      throw FormatException('管理员公钥必须为 64 位 hex', value);
    }
    return clean;
  }

  static void _validateCount(int kind, int org, int count) {
    if (kind == 0) {
      final expected = switch (org) {
        0 => 19,
        1 || 2 => 9,
        _ => throw StateError('内置治理机构 org 无效'),
      };
      if (count != expected) throw StateError('内置治理机构管理员数量必须保持 $expected 人');
      return;
    }
    if (kind == 2) {
      if (org != 3) throw StateError('个人多签管理员更换必须使用 ORG_REN');
      if (count < 2 || count > 64) throw StateError('个人多签管理员数量必须在 2..=64 之间');
      return;
    }
    if (kind == 3) {
      if (org != 4 && org != 5) {
        throw StateError('机构账户管理员更换必须使用 ORG_PUP 或 ORG_OTH');
      }
      if (count < 2 || count > 1989) {
        throw StateError('机构账户管理员数量必须在 2..=1989 之间');
      }
      return;
    }
    if (kind == 1) {
      throw StateError('SfidInstitution 只用于机构归属/检索，不能作为管理员更换主体');
    }
    throw StateError('未知管理员主体类型');
  }
}
