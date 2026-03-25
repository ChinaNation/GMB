import 'dart:convert';
import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;

import '../rpc/chain_rpc.dart';
import 'institution_data.dart';

class InstitutionAdminState {
  const InstitutionAdminState({
    required this.admins,
    this.threshold,
  });

  final List<String> admins;
  final int? threshold;
}

/// 查询链上 `AdminsOriginGov.CurrentAdmins` 存储，
/// 判断指定公钥是否为某机构管理员。
class InstitutionAdminService {
  InstitutionAdminService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// 内存缓存：institutionIdentity → 管理员状态。
  final Map<String, InstitutionAdminState> _cache = {};

  /// 查询指定机构的管理员公钥列表。
  ///
  /// 返回值为不含 0x 前缀的小写 hex 公钥列表。
  /// 链上不存在该机构时返回空列表。
  Future<List<String>> fetchAdmins(String shenfenId) async {
    final state = await _fetchState(shenfenId);
    return state.admins;
  }

  /// 查询机构当前内部投票阈值。
  ///
  /// 静态治理机构返回 null，由页面使用本地常量。
  /// 注册型机构从 `DuoqianAccounts.threshold` 动态读取。
  Future<int?> fetchThreshold(String shenfenId) async {
    final state = await _fetchState(shenfenId);
    return state.threshold;
  }

  /// 判断 [pubkeyHex] 是否为 [shenfenId] 机构的管理员。
  ///
  /// [pubkeyHex] 可含或不含 0x 前缀。
  Future<bool> isAdmin(String pubkeyHex, String shenfenId) async {
    final normalized = _normalize(pubkeyHex);
    final admins = await fetchAdmins(shenfenId);
    return admins.contains(normalized);
  }

  /// 清除缓存（如管理员更换后需刷新）。
  void clearCache([String? shenfenId]) {
    if (shenfenId != null) {
      _cache.remove(shenfenId);
    } else {
      _cache.clear();
    }
  }

  // ---------------------------------------------------------------------------
  // Storage key 构造
  // ---------------------------------------------------------------------------

  Future<InstitutionAdminState> _fetchState(String shenfenId) async {
    final cached = _cache[shenfenId];
    if (cached != null) return cached;

    final duoqianAddress = registeredDuoqianAddressFromIdentity(shenfenId);
    final state = duoqianAddress == null
        ? await _fetchGovernanceAdmins(shenfenId)
        : await _fetchRegisteredDuoqianState(duoqianAddress);
    _cache[shenfenId] = state;
    return state;
  }

  Future<InstitutionAdminState> _fetchGovernanceAdmins(String shenfenId) async {
    final storageKey = _buildCurrentAdminsKey(shenfenId);
    final keyHex = '0x${_hexEncode(storageKey)}';
    final data = await _rpc.fetchStorage(keyHex);
    if (data == null) {
      return const InstitutionAdminState(admins: []);
    }
    return InstitutionAdminState(admins: _decodeAdminList(data));
  }

  Future<InstitutionAdminState> _fetchRegisteredDuoqianState(
    String duoqianAddress,
  ) async {
    final storageKey = _buildDuoqianAccountsKey(duoqianAddress);
    final keyHex = '0x${_hexEncode(storageKey)}';
    final data = await _rpc.fetchStorage(keyHex);
    if (data == null || data.isEmpty) {
      return const InstitutionAdminState(admins: []);
    }
    return _decodeDuoqianAccountState(data);
  }

  /// 构造 `AdminsOriginGov::CurrentAdmins(institution_id)` 的 storage key。
  ///
  /// 格式：twox_128("AdminsOriginGov") + twox_128("CurrentAdmins")
  ///        + blake2_128(institution_48bytes) + institution_48bytes
  Uint8List _buildCurrentAdminsKey(String shenfenId) {
    final institutionId = _shenfenIdToFixed48(shenfenId);
    final palletHash = Hasher.twoxx128.hashString('AdminsOriginGov');
    final storageHash = Hasher.twoxx128.hashString('CurrentAdmins');
    final blake2Hash = Hasher.blake2b128.hash(institutionId);

    final key = Uint8List(
      palletHash.length +
          storageHash.length +
          blake2Hash.length +
          institutionId.length,
    );
    var offset = 0;
    key.setAll(offset, palletHash);
    offset += palletHash.length;
    key.setAll(offset, storageHash);
    offset += storageHash.length;
    key.setAll(offset, blake2Hash);
    offset += blake2Hash.length;
    key.setAll(offset, institutionId);
    return key;
  }

  /// 构造 `DuoqianManagePow::DuoqianAccounts(duoqian_address)` 的 storage key。
  Uint8List _buildDuoqianAccountsKey(String duoqianAddress) {
    final address = Uint8List.fromList(_hexDecode(duoqianAddress));
    final palletHash = Hasher.twoxx128.hashString('DuoqianManagePow');
    final storageHash = Hasher.twoxx128.hashString('DuoqianAccounts');
    final blake2Hash = Hasher.blake2b128.hash(address);

    final key = Uint8List(
      palletHash.length + storageHash.length + blake2Hash.length + address.length,
    );
    var offset = 0;
    key.setAll(offset, palletHash);
    offset += palletHash.length;
    key.setAll(offset, storageHash);
    offset += storageHash.length;
    key.setAll(offset, blake2Hash);
    offset += blake2Hash.length;
    key.setAll(offset, address);
    return key;
  }

  /// 将 shenfen_id 编码为固定 48 字节（与 Rust `shenfen_id_to_fixed48` 一致）。
  Uint8List _shenfenIdToFixed48(String shenfenId) {
    final raw = utf8.encode(shenfenId);
    if (raw.isEmpty || raw.length > 48) {
      throw ArgumentError('shenfenId 长度必须在 1..48 字节，实际: ${raw.length}');
    }
    final out = Uint8List(48);
    out.setAll(0, raw);
    return out;
  }

  // ---------------------------------------------------------------------------
  // SCALE 解码
  // ---------------------------------------------------------------------------

  /// 解码 SCALE 编码的 `BoundedVec<AccountId32, MaxAdminsPerInstitution>`。
  ///
  /// BoundedVec 在链上编码与普通 Vec 一致：Compact<u32> 长度 + N 个 32 字节元素。
  List<String> _decodeAdminList(Uint8List data) {
    if (data.isEmpty) return const [];

    var offset = 0;
    // 读取 Compact<u32> 长度
    final (count, bytesRead) = _readCompactU32(data, offset);
    offset += bytesRead;

    final admins = <String>[];
    for (var i = 0; i < count; i++) {
      if (offset + 32 > data.length) break;
      final pubkey = data.sublist(offset, offset + 32);
      admins.add(_hexEncode(pubkey));
      offset += 32;
    }
    return admins;
  }

  /// 解码 `DuoqianAccount` 的前半段，提取 threshold 与管理员列表。
  ///
  /// 结构顺序：
  /// - admin_count: u32
  /// - threshold: u32
  /// - duoqian_admins: BoundedVec<AccountId32>
  InstitutionAdminState _decodeDuoqianAccountState(Uint8List data) {
    if (data.length < 8) {
      return const InstitutionAdminState(admins: []);
    }

    var offset = 0;
    offset += 4; // 跳过 admin_count
    final threshold = _decodeU32(data, offset);
    offset += 4;

    final (count, lenSize) = _readCompactU32(data, offset);
    offset += lenSize;

    final admins = <String>[];
    for (var i = 0; i < count; i++) {
      if (offset + 32 > data.length) break;
      admins.add(_hexEncode(data.sublist(offset, offset + 32)));
      offset += 32;
    }

    return InstitutionAdminState(admins: admins, threshold: threshold);
  }

  /// 读取 SCALE Compact<u32>，返回值和消耗的字节数。
  (int value, int bytesRead) _readCompactU32(Uint8List data, int offset) {
    final first = data[offset];
    final mode = first & 0x03;
    switch (mode) {
      case 0: // single byte
        return (first >> 2, 1);
      case 1: // two bytes
        final value = ((data[offset + 1] << 8) | first) >> 2;
        return (value, 2);
      case 2: // four bytes
        final value = ((data[offset + 3] << 24) |
                (data[offset + 2] << 16) |
                (data[offset + 1] << 8) |
                first) >>
            2;
        return (value, 4);
      default:
        throw const FormatException('Compact<u32> big-integer 模式暂不支持');
    }
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

  static Uint8List _hexDecode(String hex) {
    final clean = _normalize(hex);
    final out = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < out.length; i++) {
      out[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return out;
  }

  static int _decodeU32(Uint8List data, int offset) {
    return ByteData.sublistView(data).getUint32(offset, Endian.little);
  }
}
