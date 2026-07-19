import 'dart:convert';
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

  test('机构管理员按账户、姓、名严格解码', () {
    final value = Uint8List.fromList([
      ...utf8.encode('CGOV'),
      8,
      ...List.filled(32, 1),
      ...bytes('张'),
      ...bytes('三'),
      ...List.filled(32, 2),
      ...bytes('管理'),
      ...bytes('员'),
    ]);
    final decoded = InstitutionRoleStorageCodec.decodeAdmins(value)!;
    expect(decoded.institutionCode, 'CGOV');
    expect(decoded.admins, hasLength(2));
    expect(decoded.admins.map((admin) => admin.family_name), ['张', '管理']);
    expect(decoded.admins.map((admin) => admin.given_name), ['三', '员']);
    expect(
      decoded.admins.map((admin) => admin.admin_account),
      ['01' * 32, '02' * 32],
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
    expect(assignment.admin_account, '07' * 32);
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
    expect(close.institutionAccount, '21' * 32);

    expect(
      InstitutionChainService().decodeManageProposalData(
        7,
        Uint8List.fromList([...raw, 0]),
      ),
      isNull,
    );
  });
}
