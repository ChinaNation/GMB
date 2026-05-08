import 'dart:typed_data';

import 'package:wuminapp_mobile/duoqian/shared/duoqian_storage_codec.dart';
import 'package:wuminapp_mobile/institution/institution_data.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

class InstitutionAdminState {
  const InstitutionAdminState({
    required this.admins,
    this.threshold,
  });

  final List<String> admins;
  final int? threshold;
}

/// 查询链上 `AdminsChange::Subjects` 存储，判断指定公钥是否为某机构管理员。
class InstitutionAdminService {
  InstitutionAdminService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// 内存缓存：institutionIdentity → 管理员状态。
  final Map<String, InstitutionAdminState> _cache = {};

  /// 查询指定机构的管理员公钥列表。
  ///
  /// 返回值为不含 0x 前缀的小写 hex 公钥列表。链上不存在该机构时返回空列表。
  Future<List<String>> fetchAdmins(String sfidNumber) async {
    final state = await _fetchState(sfidNumber);
    return state.admins;
  }

  /// 查询机构当前内部投票阈值。
  ///
  /// 内置机构、SFID 注册机构、个人多签都统一从 `AdminsChange::Subjects`
  /// 的 subject 管理员状态读取。
  Future<int?> fetchThreshold(String sfidNumber) async {
    final state = await _fetchState(sfidNumber);
    return state.threshold;
  }

  /// 判断 [pubkeyHex] 是否为 [sfidNumber] 机构的管理员。
  ///
  /// [pubkeyHex] 可含或不含 0x 前缀。
  Future<bool> isAdmin(String pubkeyHex, String sfidNumber) async {
    final normalized = _normalize(pubkeyHex);
    final admins = await fetchAdmins(sfidNumber);
    return admins.contains(normalized);
  }

  /// 清除缓存（如管理员更换后需刷新）。
  void clearCache([String? sfidNumber]) {
    if (sfidNumber != null) {
      _cache.remove(sfidNumber);
    } else {
      _cache.clear();
    }
  }

  // ---------------------------------------------------------------------------
  // Storage 查询路由
  // ---------------------------------------------------------------------------

  /// "personal:" 前缀，用于个人多签 sfidNumber。
  static const String _personalPrefix = 'personal:';

  Future<InstitutionAdminState> _fetchState(String sfidNumber) async {
    final cached = _cache[sfidNumber];
    if (cached != null) return cached;

    InstitutionAdminState state;
    final duoqianAddress = registeredDuoqianAddressFromIdentity(sfidNumber);
    if (duoqianAddress != null) {
      state = await _fetchInstitutionAccountAdmins(duoqianAddress);
    } else if (sfidNumber.startsWith(_personalPrefix)) {
      final hex = sfidNumber.substring(_personalPrefix.length);
      final normalized = hex.startsWith('0x') ? hex.substring(2) : hex;
      state = normalized.length == 64
          ? await _fetchPersonalDuoqianAdmins(normalized)
          : const InstitutionAdminState(admins: []);
    } else {
      state = await _fetchAdminSubject(
        DuoqianStorageCodec.subjectIdFromBuiltin(sfidNumber),
      );
    }

    _cache[sfidNumber] = state;
    return state;
  }

  Future<InstitutionAdminState> _fetchInstitutionAccountAdmins(
    String duoqianAddress,
  ) async {
    // 中文注释：注册机构账户的管理员主体是 0x05 InstitutionAccount；
    // AddressRegisteredSfid 只用于确认该地址确实属于 SFID 机构账户。
    final refKey = DuoqianStorageCodec.addressRegisteredSfidKey(
      duoqianAddress,
    );
    final refData = await _rpc.fetchStorage('0x${_hexEncode(refKey)}');
    if (refData == null) {
      return const InstitutionAdminState(admins: []);
    }
    final ref = DuoqianStorageCodec.decodeRegisteredInstitution(refData);
    if (ref == null) {
      return const InstitutionAdminState(admins: []);
    }
    return _fetchAdminSubject(
      DuoqianStorageCodec.subjectIdFromInstitutionAccountHex(duoqianAddress),
    );
  }

  Future<InstitutionAdminState> _fetchPersonalDuoqianAdmins(
    String personalAddress,
  ) {
    return _fetchAdminSubject(
      DuoqianStorageCodec.subjectIdFromAccountHex(personalAddress),
    );
  }

  Future<InstitutionAdminState> _fetchAdminSubject(Uint8List subjectId) async {
    final storageKey = DuoqianStorageCodec.adminSubjectKey(subjectId);
    final data = await _rpc.fetchStorage('0x${_hexEncode(storageKey)}');
    if (data == null) {
      return const InstitutionAdminState(admins: []);
    }
    final decoded = DuoqianStorageCodec.decodeAdminSubject(data);
    if (decoded == null) {
      return const InstitutionAdminState(admins: []);
    }
    return InstitutionAdminState(
      admins: decoded.adminPubkeys,
      threshold: decoded.threshold,
    );
  }

  // ---------------------------------------------------------------------------
  // 工具
  // ---------------------------------------------------------------------------

  static String _normalize(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return clean.toLowerCase();
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
