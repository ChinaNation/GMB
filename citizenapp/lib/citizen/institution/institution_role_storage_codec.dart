import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';

import 'institution_role_models.dart';

/// entity 岗位/任职与 admins 钱包集合的严格 SCALE 解码器。
class InstitutionRoleStorageCodec {
  InstitutionRoleStorageCodec._();

  static InstitutionAdminsStorage? decodeAdmins(Uint8List data) {
    var offset = 0;
    // 机构 CID 是 `AdminAccounts` 的 storage key，不在 value 中重复保存。
    // value 唯一布局为 institution_code:[u8;4]
    // + admins:BoundedVec<(admin_account, family_name, given_name)>。
    if (offset + 4 > data.length) return null;
    final code = String.fromCharCodes(
        data.sublist(offset, offset + 4).where((b) => b != 0));
    offset += 4;
    final decodedAdmins = decodeAdminVector(data, offset);
    if (decodedAdmins == null) return null;
    final admins = decodedAdmins.$1;
    offset = decodedAdmins.$2;
    if (offset != data.length) return null;
    return InstitutionAdminsStorage(
      institutionCode: code,
      admins: admins,
    );
  }

  /// 从指定偏移严格解码统一管理员集合，并返回下一字段偏移。
  ///
  /// 机构管理员、个人多签管理员和全量管理员扫描必须共用本入口，避免
  /// 在不同页面复制 SCALE 字段顺序。账户重复、空姓名、畸形 UTF-8 均拒绝。
  static (List<AdminPerson>, int)? decodeAdminVector(
    Uint8List data,
    int offset,
  ) {
    final count = _readCompact(data, offset);
    if (count == null) return null;
    offset += count.$2;
    final admins = <AdminPerson>[];
    final accounts = <String>{};
    for (var i = 0; i < count.$1; i++) {
      if (offset + 32 > data.length) return null;
      final accountHex = _hex(data.sublist(offset, offset + 32));
      offset += 32;
      final familyName = _readBytes(
        data,
        offset,
        minLength: 1,
        maxLength: 128,
      );
      if (familyName == null) return null;
      offset = familyName.$2;
      final givenName = _readBytes(
        data,
        offset,
        minLength: 1,
        maxLength: 128,
      );
      if (givenName == null) return null;
      offset = givenName.$2;
      try {
        final admin = AdminPerson(
          admin_account: accountHex,
          family_name: utf8.decode(familyName.$1),
          given_name: utf8.decode(givenName.$1),
        );
        if (!accounts.add(admin.admin_account)) return null;
        admins.add(admin);
      } on FormatException {
        return null;
      }
    }
    return (admins, offset);
  }

  static InstitutionRole? decodeRole(Uint8List data) {
    try {
      var offset = 0;
      final cid = _readBytes(data, offset, minLength: 1, maxLength: 32);
      if (cid == null) return null;
      offset = cid.$2;
      final code = _readBytes(data, offset, minLength: 1, maxLength: 64);
      if (code == null) return null;
      offset = code.$2;
      final name = _readBytes(data, offset, minLength: 1, maxLength: 128);
      if (name == null) return null;
      offset = name.$2;
      if (offset + 2 != data.length ||
          data[offset] > 1 ||
          data[offset + 1] > 1) {
        return null;
      }
      return InstitutionRole(
        cidNumber: utf8.decode(cid.$1),
        roleCode: utf8.decode(code.$1),
        roleName: utf8.decode(name.$1),
        termRequired: data[offset] == 1,
        status: InstitutionRoleStatus.values[data[offset + 1]],
      );
    } on FormatException {
      return null;
    }
  }

  static List<InstitutionAdminAssignment>? decodeAssignments(Uint8List data) {
    var offset = 0;
    final count = _readCompact(data, offset);
    if (count == null) return null;
    offset += count.$2;
    final out = <InstitutionAdminAssignment>[];
    for (var i = 0; i < count.$1; i++) {
      final cid = _readBytes(data, offset, minLength: 1, maxLength: 32);
      if (cid == null) return null;
      offset = cid.$2;
      if (offset + 32 > data.length) return null;
      final account = _hex(data.sublist(offset, offset + 32));
      offset += 32;
      final code = _readBytes(data, offset, minLength: 1, maxLength: 64);
      if (code == null) return null;
      offset = code.$2;
      if (offset + 9 > data.length) return null;
      final termStart =
          ByteData.sublistView(data).getUint32(offset, Endian.little);
      offset += 4;
      final termEnd =
          ByteData.sublistView(data).getUint32(offset, Endian.little);
      offset += 4;
      final source = data[offset++];
      final sourceRef = _readBytes(data, offset, maxLength: 128);
      if (sourceRef == null) return null;
      offset = sourceRef.$2;
      if (offset >= data.length ||
          source >= InstitutionAssignmentSource.values.length) {
        return null;
      }
      final status = data[offset++];
      if (status > 1) return null;
      try {
        out.add(InstitutionAdminAssignment(
          cidNumber: utf8.decode(cid.$1),
          admin_account: account,
          roleCode: utf8.decode(code.$1),
          termStart: termStart,
          termEnd: termEnd,
          source: InstitutionAssignmentSource.values[source],
          sourceRef: utf8.decode(sourceRef.$1),
          active: status == 0,
        ));
      } on FormatException {
        return null;
      }
    }
    return offset == data.length ? out : null;
  }

  /// 严格解码完整机构岗位授权主体。
  static RoleSubject? decodeRoleSubject(Uint8List data) {
    final decoded = _decodeRoleSubjectAt(data, 0);
    return decoded != null && decoded.$2 == data.length ? decoded.$1 : null;
  }

  /// 严格解码业务动作标识。
  static BusinessActionId? decodeBusinessActionId(Uint8List data) {
    final decoded = _decodeBusinessActionIdAt(data, 0);
    return decoded != null && decoded.$2 == data.length ? decoded.$1 : null;
  }

  /// 严格解码岗位业务权限。
  static RoleBusinessPermission? decodeRoleBusinessPermission(Uint8List data) {
    final decoded = _decodeRoleBusinessPermissionAt(data, 0);
    return decoded != null && decoded.$2 == data.length ? decoded.$1 : null;
  }

  /// 严格解码 `InstitutionRolePermissions[(cid, role_code)]` 的完整向量值。
  static List<RoleBusinessPermission>? decodeRolePermissions(Uint8List data) {
    final count = _readCompact(data, 0);
    if (count == null || count.$1 > 256) return null;
    var offset = count.$2;
    final permissions = <RoleBusinessPermission>[];
    final seen = <String>{};
    for (var index = 0; index < count.$1; index++) {
      final permission = _decodeRoleBusinessPermissionAt(data, offset);
      if (permission == null) return null;
      offset = permission.$2;
      final item = permission.$1;
      final key = '${item.roleSubject.cidNumber}:${item.roleSubject.roleCode}:'
          '${item.businessActionId.moduleTag}:${item.businessActionId.actionCode}:'
          '${item.operation.index}';
      if (!seen.add(key)) return null;
      permissions.add(item);
    }
    return offset == data.length ? List.unmodifiable(permissions) : null;
  }

  static (RoleBusinessPermission, int)? _decodeRoleBusinessPermissionAt(
    Uint8List data,
    int offset,
  ) {
    final role = _decodeRoleSubjectAt(data, offset);
    if (role == null) return null;
    final action = _decodeBusinessActionIdAt(data, role.$2);
    if (action == null || action.$2 >= data.length) return null;
    final operation = data[action.$2];
    if (operation >= RolePermissionOperation.values.length) return null;
    return (
      RoleBusinessPermission(
        roleSubject: role.$1,
        businessActionId: action.$1,
        operation: RolePermissionOperation.values[operation],
      ),
      action.$2 + 1,
    );
  }

  /// 严格解码机构岗位或个人多签授权主体。
  static AuthorizationSubject? decodeAuthorizationSubject(Uint8List data) {
    final decoded = _decodeAuthorizationSubjectAt(data, 0);
    return decoded != null && decoded.$2 == data.length ? decoded.$1 : null;
  }

  /// 严格解码业务模块绑定的投票计划。
  static VotePlan? decodeVotePlan(Uint8List data) {
    final action = _decodeBusinessActionIdAt(data, 0);
    if (action == null) return null;
    final owner = _readUtf8(data, action.$2, minLength: 1, maxLength: 32);
    if (owner == null || owner.$1 != action.$1.moduleTag) return null;
    final proposer = _decodeAuthorizationSubjectAt(data, owner.$2);
    if (proposer == null) return null;
    final count = _readCompact(data, proposer.$2);
    if (count == null || count.$1 < 1 || count.$1 > 256) return null;
    var offset = proposer.$2 + count.$2;
    final voters = <AuthorizationSubject>[];
    final voterKeys = <String>{};
    for (var index = 0; index < count.$1; index++) {
      final voter = _decodeAuthorizationSubjectAt(data, offset);
      if (voter == null) return null;
      offset = voter.$2;
      final key = voter.$1.isInstitution
          ? 'i:${voter.$1.roleSubject!.cidNumber}:${voter.$1.roleSubject!.roleCode}'
          : 'p:${voter.$1.personalAccountHex}';
      if (!voterKeys.add(key)) return null;
      voters.add(voter.$1);
    }
    if (offset >= data.length) return null;
    final engine = data[offset++];
    if (engine >= VotingEngineKind.values.length ||
        offset + 32 != data.length) {
      return null;
    }
    if (proposer.$1.isInstitution) {
      if (voters.any((voter) => !voter.isInstitution)) return null;
    } else {
      if (voters.length != 1 ||
          voters.single.isInstitution ||
          voters.single.personalAccountHex != proposer.$1.personalAccountHex) {
        return null;
      }
    }
    return VotePlan(
      businessActionId: action.$1,
      proposalOwner: owner.$1,
      proposerSubject: proposer.$1,
      voterSubjects: List.unmodifiable(voters),
      votingEngine: VotingEngineKind.values[engine],
      businessObjectHash: _hex(data.sublist(offset, offset + 32)),
    );
  }

  static (RoleSubject, int)? _decodeRoleSubjectAt(
    Uint8List data,
    int offset,
  ) {
    final cid = _readUtf8(data, offset, minLength: 1, maxLength: 32);
    if (cid == null) return null;
    final roleCode = _readUtf8(data, cid.$2, minLength: 1, maxLength: 64);
    if (roleCode == null) return null;
    return (
      RoleSubject(cidNumber: cid.$1, roleCode: roleCode.$1),
      roleCode.$2,
    );
  }

  static (BusinessActionId, int)? _decodeBusinessActionIdAt(
    Uint8List data,
    int offset,
  ) {
    final moduleTag = _readUtf8(data, offset, minLength: 1, maxLength: 32);
    if (moduleTag == null || moduleTag.$2 + 4 > data.length) return null;
    final actionCode =
        ByteData.sublistView(data).getUint32(moduleTag.$2, Endian.little);
    return (
      BusinessActionId(moduleTag: moduleTag.$1, actionCode: actionCode),
      moduleTag.$2 + 4,
    );
  }

  static (AuthorizationSubject, int)? _decodeAuthorizationSubjectAt(
    Uint8List data,
    int offset,
  ) {
    if (offset >= data.length) return null;
    final kind = data[offset++];
    if (kind == 0) {
      final role = _decodeRoleSubjectAt(data, offset);
      return role == null
          ? null
          : (AuthorizationSubject.institution(role.$1), role.$2);
    }
    if (kind == 1 && offset + 32 <= data.length) {
      return (
        AuthorizationSubject.personalMultisig(
          _hex(data.sublist(offset, offset + 32)),
        ),
        offset + 32,
      );
    }
    return null;
  }

  static (String, int)? _readUtf8(
    Uint8List data,
    int offset, {
    required int minLength,
    required int maxLength,
  }) {
    final value = _readBytes(
      data,
      offset,
      minLength: minLength,
      maxLength: maxLength,
    );
    if (value == null) return null;
    try {
      return (utf8.decode(value.$1), value.$2);
    } on FormatException {
      return null;
    }
  }

  static (Uint8List, int)? _readBytes(
    Uint8List data,
    int offset, {
    int minLength = 0,
    required int maxLength,
  }) {
    final compact = _readCompact(data, offset);
    if (compact == null) return null;
    if (compact.$1 < minLength || compact.$1 > maxLength) return null;
    final start = offset + compact.$2;
    final end = start + compact.$1;
    if (end > data.length) return null;
    return (Uint8List.sublistView(data, start, end), end);
  }

  static (int, int)? _readCompact(Uint8List data, int offset) {
    if (offset >= data.length) return null;
    final first = data[offset];
    final mode = first & 3;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1 && offset + 2 <= data.length) {
      return ((((data[offset + 1] << 8) | first) >> 2), 2);
    }
    if (mode == 2 && offset + 4 <= data.length) {
      final raw = ByteData.sublistView(data).getUint32(offset, Endian.little);
      return (raw >> 2, 4);
    }
    return null;
  }

  static String _hex(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
