import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/citizen/institution/institution_role_models.dart';
import 'package:citizenapp/citizen/institution/institution_role_storage_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_account_service.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';

class InstitutionAdminState {
  const InstitutionAdminState({
    required this.admins,
    this.assignments = const [],
    this.threshold,
  });

  final List<String> admins;

  /// 机构岗位任职；个人多签不使用本字段。
  final List<InstitutionAdminAssignment> assignments;
  final int? threshold;
}

/// 管理员查询门面。
///
/// 调用方必须传入明确的 `AdminAccountIdentity`，不再把
/// `cidNumber` 当作个人多签、机构账户和治理机构共用的模糊参数。
class InstitutionAdminService {
  InstitutionAdminService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc(),
        _accountService = AdminAccountService(chainRpc: chainRpc);

  final ChainRpc _rpc;
  final AdminAccountService _accountService;

  Future<List<String>> fetchAdmins(AdminAccountIdentity identity) {
    return _accountService.fetchAdmins(identity);
  }

  /// 从 entity 唯一真源读取机构岗位和有效任职，并与 admins 钱包集合交叉校验。
  Future<List<InstitutionAdminAssignment>> fetchAssignments(
    AdminAccountIdentity identity,
    String cidNumber,
  ) async {
    if (identity.type == AdminAccountIdentityType.personalAccount) {
      throw ArgumentError('个人多签不属于机构岗位模型');
    }
    final adminState = await _accountService.fetchByIdentity(identity);
    if (adminState == null || !adminState.isActive) return const [];
    final adminSet = adminState.admins.toSet();
    // 非法人机构码本身不能推断公权/私权，必须按链上管理员类型路由。
    final entityPallet = identity.kind == 1 ? 'PrivateManage' : 'PublicManage';
    final roleValues =
        await _scanValues(entityPallet, 'InstitutionRoles', cidNumber);
    final assignmentValues = await _scanValues(
        entityPallet, 'InstitutionRoleAssignments', cidNumber);
    final roles = <String, InstitutionRole>{};
    for (final value in roleValues) {
      final role = InstitutionRoleStorageCodec.decodeRole(value);
      if (role != null &&
          role.cidNumber == cidNumber &&
          role.status == InstitutionRoleStatus.active) {
        roles[role.roleCode] = role;
      }
    }
    final out = <InstitutionAdminAssignment>[];
    for (final value in assignmentValues) {
      final decoded = InstitutionRoleStorageCodec.decodeAssignments(value);
      if (decoded == null) continue;
      for (final assignment in decoded) {
        final role = roles[assignment.roleCode];
        if (assignment.cidNumber == cidNumber &&
            assignment.active &&
            role != null &&
            adminSet.contains(assignment.adminAccount)) {
          out.add(assignment.withRole(role));
        }
      }
    }
    final covered = out.map((assignment) => assignment.adminAccount).toSet();
    if (!covered.containsAll(adminSet)) {
      throw StateError('机构管理员钱包缺少有效岗位任职');
    }
    return out;
  }

  Future<int?> fetchThreshold(AdminAccountIdentity identity) {
    return _accountService.fetchThreshold(identity);
  }

  Future<bool> isAdmin(String pubkeyHex, AdminAccountIdentity identity) {
    return _accountService.isAdmin(pubkeyHex, identity);
  }

  Future<InstitutionAdminState> fetchState(
    AdminAccountIdentity identity, {
    String? cidNumber,
  }) async {
    final account = await _accountService.fetchByIdentity(identity);
    final assignments = cidNumber == null
        ? const <InstitutionAdminAssignment>[]
        : await fetchAssignments(identity, cidNumber);
    return InstitutionAdminState(
      admins: account?.admins ?? const [],
      assignments: assignments,
      threshold: account?.threshold,
    );
  }

  void clearCache([AdminAccountIdentity? identity]) {
    _accountService.clearCache(identity);
  }

  Future<List<Uint8List>> _scanValues(
      String pallet, String storage, String cidNumber) async {
    final prefix =
        _doubleMapFirstKeyPrefix(pallet, storage, utf8.encode(cidNumber));
    final prefixHex = '0x${_hex(prefix)}';
    final keys = <String>[];
    String? startKey;
    while (true) {
      final page = await SmoldotClientManager.instance.getKeysPagedFinalized(
        prefixHex,
        count: 256,
        startKey: startKey,
      );
      if (page.isEmpty) break;
      keys.addAll(page);
      if (page.length < 256) break;
      startKey = page.last;
    }
    if (keys.isEmpty) return const [];
    final values = await _rpc.fetchStorageBatchChunked(keys);
    return keys
        .map((key) => values[key])
        .whereType<Uint8List>()
        .toList(growable: false);
  }

  Uint8List _doubleMapFirstKeyPrefix(
      String pallet, String storage, List<int> cidBytes) {
    final encodedCid =
        Uint8List.fromList([(cidBytes.length << 2), ...cidBytes]);
    final parts = [
      Hasher.twoxx128.hashString(pallet),
      Hasher.twoxx128.hashString(storage),
      AdminAccountIdCodec.blake2128Concat(encodedCid),
    ];
    return Uint8List.fromList(
        parts.expand((part) => part).toList(growable: false));
  }

  String _hex(Iterable<int> bytes) =>
      bytes.map((byte) => byte.toRadixString(16).padLeft(2, '0')).join();
}
