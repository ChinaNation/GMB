import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/citizen/institution/institution_role_models.dart';
import 'package:citizenapp/citizen/institution/institution_role_storage_codec.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  List<int> bytes(String text) =>
      [(utf8.encode(text).length << 2), ...utf8.encode(text)];

  test('机构管理员账户只解码钱包集合', () {
    final value = Uint8List.fromList([
      ...bytes('CID-1'),
      ...utf8.encode('CGOV'),
      8,
      ...List.filled(32, 1),
      ...List.filled(32, 2),
      1,
    ]);
    final decoded = InstitutionRoleStorageCodec.decodeAdminAccount(value)!;
    expect(decoded.cidNumber, 'CID-1');
    expect(decoded.admins, hasLength(2));
    expect(decoded.isActive, isTrue);
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
    expect(assignment.adminAccount, '07' * 32);
  });
}
