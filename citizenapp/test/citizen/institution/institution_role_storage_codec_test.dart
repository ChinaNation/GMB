import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:citizenapp/citizen/institution/institution_role_models.dart';
import 'package:citizenapp/citizen/institution/institution_role_storage_codec.dart';
import 'package:citizenapp/citizen/institution/institution_chain_service.dart';
import 'package:citizenapp/citizen/institution/institution_models.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  List<int> bytes(String text) =>
      [(utf8.encode(text).length << 2), ...utf8.encode(text)];

  List<int> compact(int value) {
    if (value < 64) return [value << 2];
    final encoded = (value << 2) | 1;
    return [encoded & 0xff, (encoded >> 8) & 0xff];
  }

  Map<String, dynamic> readRolePermissionFixture() {
    final candidates = [
      File('../memory/06-quality/fixtures/institution_role_permission_v1.json'),
      File('memory/06-quality/fixtures/institution_role_permission_v1.json'),
    ];
    final file = candidates.firstWhere((candidate) => candidate.existsSync());
    return jsonDecode(file.readAsStringSync()) as Map<String, dynamic>;
  }

  Uint8List fixtureBytes(Map<String, dynamic> fixture, String name) {
    final cases = fixture['cases']! as List<dynamic>;
    final entry = cases
        .cast<Map<String, dynamic>>()
        .singleWhere((item) => item['name'] == name);
    final hex = entry['encoded_hex']! as String;
    return Uint8List.fromList([
      for (var index = 0; index < hex.length; index += 2)
        int.parse(hex.substring(index, index + 2), radix: 16),
    ]);
  }

  test('岗位权限与 VotePlan 严格解码统一 SCALE fixture', () {
    final fixture = readRolePermissionFixture();
    final role = InstitutionRoleStorageCodec.decodeRoleSubject(
      fixtureBytes(fixture, 'role_subject_nrc_committee'),
    )!;
    expect(role.cidNumber, 'LN001-NRC0G-944805165-2026');
    expect(role.roleCode, 'COMMITTEE_MEMBER');

    final action = InstitutionRoleStorageCodec.decodeBusinessActionId(
      fixtureBytes(fixture, 'business_action_resolution_issuance'),
    )!;
    expect(action.moduleTag, 'res-iss');
    expect(action.actionCode, 0);

    final permission = InstitutionRoleStorageCodec.decodeRoleBusinessPermission(
      fixtureBytes(fixture, 'permission_resolution_issuance_propose'),
    )!;
    expect(permission.operation, RolePermissionOperation.propose);
    expect(permission.roleSubject.roleCode, 'COMMITTEE_MEMBER');

    final personal = InstitutionRoleStorageCodec.decodeAuthorizationSubject(
      fixtureBytes(fixture, 'authorization_personal_multisig'),
    )!;
    expect(personal.isInstitution, isFalse);
    expect(personal.personalAccountId, '07' * 32);

    final plan = InstitutionRoleStorageCodec.decodeVotePlan(
      fixtureBytes(fixture, 'vote_plan_resolution_issuance_joint'),
    )!;
    expect(plan.businessActionId.moduleTag, 'res-iss');
    expect(plan.proposalOwner, 'res-iss');
    expect(plan.proposerSubject.roleSubject!.roleCode, 'COMMITTEE_MEMBER');
    expect(plan.voterSubjects, hasLength(3));
    expect(plan.voterSubjects.last.roleSubject!.roleCode, 'DIRECTOR');
    expect(plan.votingEngine, VotingEngineKind.joint);
    expect(plan.businessObjectHash, 'ab' * 32);

    final withTrailingByte = Uint8List.fromList([
      ...fixtureBytes(fixture, 'vote_plan_resolution_issuance_joint'),
      0,
    ]);
    expect(
        InstitutionRoleStorageCodec.decodeVotePlan(withTrailingByte), isNull);
  });

  test('岗位权限 storage 向量严格解码并拒绝重复项', () {
    final fixture = readRolePermissionFixture();
    final permission =
        fixtureBytes(fixture, 'permission_resolution_issuance_propose');
    final decoded = InstitutionRoleStorageCodec.decodeRolePermissions(
      Uint8List.fromList([4, ...permission]),
    )!;
    expect(decoded, hasLength(1));
    expect(decoded.single.businessActionId.moduleTag, 'res-iss');

    expect(
      InstitutionRoleStorageCodec.decodeRolePermissions(
        Uint8List.fromList([8, ...permission, ...permission]),
      ),
      isNull,
    );
  });

  test('机构管理员按账户、CID、姓、名严格解码', () {
    final value = Uint8List.fromList([
      ...utf8.encode('CGOV'),
      8,
      ...List.filled(32, 1),
      0, // 空公民 CID（统一 Admin 恒带 cid，Compact(0)）
      ...bytes('张'),
      ...bytes('三'),
      ...List.filled(32, 2),
      0, // 空公民 CID
      ...bytes('管理'),
      ...bytes('员'),
    ]);
    final decoded = InstitutionRoleStorageCodec.decodeAdmins(value)!;
    expect(decoded.institutionCode, 'CGOV');
    expect(decoded.admins, hasLength(2));
    expect(decoded.admins.map((admin) => admin.family_name), ['张', '管理']);
    expect(decoded.admins.map((admin) => admin.given_name), ['三', '员']);
    expect(
      decoded.admins.map((admin) => admin.account_id),
      ['0x${'01' * 32}', '0x${'02' * 32}'],
    );
  });

  test('机构管理员拒绝纯账户、合并姓名和重复账户布局', () {
    final account = List.filled(32, 7);
    expect(
      InstitutionRoleStorageCodec.decodeAdmins(
        Uint8List.fromList([...utf8.encode('CGOV'), 4, ...account]),
      ),
      isNull,
    );
    expect(
      InstitutionRoleStorageCodec.decodeAdmins(
        Uint8List.fromList([
          ...utf8.encode('CGOV'),
          4,
          ...bytes('管理员'),
          ...account,
        ]),
      ),
      isNull,
    );
    expect(
      InstitutionRoleStorageCodec.decodeAdmins(
        Uint8List.fromList([
          ...utf8.encode('CGOV'),
          8,
          ...account,
          ...bytes('管'),
          ...bytes('理'),
          ...account,
          ...bytes('管'),
          ...bytes('员'),
        ]),
      ),
      isNull,
    );
  });

  test('公权机构管理员按账户、公民 CID、姓、名解码并允许空资料', () {
    final value = Uint8List.fromList([
      ...utf8.encode('NRCG'),
      4,
      ...List.filled(32, 3),
      ...bytes('GZ000-CTZN6-198805200-2026'),
      ...bytes(''),
      ...bytes(''),
    ]);
    final decoded = InstitutionRoleStorageCodec.decodeAdmins(value)!;
    expect(decoded.admins.single.account_id, '0x${'03' * 32}');
    expect(
      decoded.admins.single.cid_number,
      'GZ000-CTZN6-198805200-2026',
    );
    expect(decoded.admins.single.family_name, isEmpty);
    expect(decoded.admins.single.given_name, isEmpty);
  });

  test('岗位和任职分别按 entity 布局解码', () {
    final role = InstitutionRoleStorageCodec.decodeRole(Uint8List.fromList([
      ...bytes('CID-1'),
      ...bytes('DIRECTOR'),
      ...bytes('董事'),
      1,
      0,
    ]))!;
    expect(role.roleName, '董事');
    expect(role.termRequired, isTrue);

    final assignment =
        InstitutionRoleStorageCodec.decodeAssignments(Uint8List.fromList([
      4,
      ...bytes('CID-1'),
      ...List.filled(32, 7),
      ...bytes('DIRECTOR'),
      10,
      0,
      0,
      0,
      20,
      0,
      0,
      0,
      1,
      ...bytes('REG-1'),
      0,
    ]))!
            .single;
    expect(assignment.source, InstitutionAssignmentSource.registry);
    expect(assignment.account_id, '07' * 32);
  });

  test('任期窗口包含起止日且无任期岗位只接受零值', () {
    final base = InstitutionAdminAssignment(
      cidNumber: 'CID-1',
      account_id: '07' * 32,
      roleCode: 'ROLE-1',
      termStart: 10,
      termEnd: 20,
      source: InstitutionAssignmentSource.institutionGovernance,
      sourceRef: 'proposal',
      active: true,
      termRequired: true,
    );
    expect(base.isEffectiveOnDay(10), isTrue);
    expect(base.isEffectiveOnDay(20), isTrue);
    expect(base.isEffectiveOnDay(21), isFalse);
    final nonTerm = InstitutionAdminAssignment(
      cidNumber: 'CID-1',
      account_id: '07' * 32,
      roleCode: 'ROLE-2',
      termStart: 0,
      termEnd: 0,
      source: InstitutionAssignmentSource.institutionGovernance,
      sourceRef: 'proposal',
      active: true,
    );
    expect(nonTerm.isEffectiveOnDay(20), isTrue);
  });

  test('岗位身份文本拒绝畸形 UTF-8 和非法 bool', () {
    final malformedCid = Uint8List.fromList([
      4,
      0xff,
      ...bytes('DIRECTOR'),
      ...bytes('董事'),
      1,
      0,
    ]);
    expect(InstitutionRoleStorageCodec.decodeRole(malformedCid), isNull);

    final invalidBool = Uint8List.fromList([
      ...bytes('CID-1'),
      ...bytes('DIRECTOR'),
      ...bytes('董事'),
      2,
      0,
    ]);
    expect(InstitutionRoleStorageCodec.decodeRole(invalidBool), isNull);
  });

  test('机构命名账户关闭动作保留 actor CID 与具体机构账户', () {
    const actorCidNumber = 'GD001-CGOV0-123456789-2026';
    final actorCidBytes = utf8.encode(actorCidNumber);
    final body = <int>[
      ...utf8.encode('pub-mgmt'),
      InstitutionChainService.actionClose,
      ...compact(actorCidBytes.length),
      ...actorCidBytes,
      ...List.filled(32, 0x21),
      ...List.filled(32, 0x22),
      ...List.filled(32, 0x23),
    ];
    final raw = Uint8List.fromList([...compact(body.length), ...body]);
    final decoded = InstitutionChainService().decodeManageProposalData(7, raw);
    expect(decoded, isA<CloseProposalInfo>());
    final close = decoded! as CloseProposalInfo;
    expect(close.actorCidNumber, actorCidNumber);
    expect(close.institutionAccountId, '21' * 32);

    expect(
      InstitutionChainService().decodeManageProposalData(
        7,
        Uint8List.fromList([...raw, 0]),
      ),
      isNull,
    );
  });
}
