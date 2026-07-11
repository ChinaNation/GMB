import 'dart:typed_data';

import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/rpc/chain_rpc.dart';

import 'institution_models.dart';
import 'institution_pallet_router.dart';
import 'multisig_storage_codec.dart';

/// 机构链访问只读服务(机构管理 / 生命周期层,公权私权共用)。
///
/// 机构创建/关闭已收归 onchina 控制台 + 冷钱包,本服务**只读**:
/// CID 注册状态、机构多签账户身份/账户/管理员/动态阈值、机构管理提案(关闭)解码。
/// 公权/私权机构 storage 同名(`Institutions`/`InstitutionAccounts`/`AccountRegisteredCid`/
/// `CidRegisteredAccount`)但前缀随 pallet 名变(PublicManage/PrivateManage);给定账户无法
/// 先验公私,故反查 `AccountRegisteredCid` 对 [InstitutionPalletRouter.managePallets] **双查取
/// 首命中**,再把命中 pallet 贯穿后续 cid 键读。个人多签在 personal-manage 独立线。
///
/// 管理员投票一律走 `InternalVote::cast`(经 InternalVoteService),不在本服务。
class InstitutionChainService {
  InstitutionChainService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// ProposalData 中机构管理提案的 action 类型:ACTION_CLOSE=2(关闭机构多签)。
  static const actionClose = 2;

  // ──── 链上查询(只读) ────

  /// 查询 CID (cid_number + account_name) 是否已注册，返回派生的多签账户 hex（null=未注册）。
  ///
  /// `CidRegisteredAccount` 按 cid 落 PublicManage/PrivateManage 两 pallet,无法先验公私,
  /// 故双查取首命中。
  Future<String?> fetchCidRegisteredAccount(
      Uint8List cidNumber, Uint8List accountName) async {
    for (final pallet in InstitutionPalletRouter.managePallets) {
      final key = _buildDoubleMapStorageKey(
        pallet,
        'CidRegisteredAccount',
        cidNumber,
        accountName,
      );
      final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
      if (data != null && data.length >= 32) {
        return _hexEncode(Uint8List.fromList(data.sublist(0, 32)));
      }
    }
    return null;
  }

  /// 批量反查多个机构账户的 CID 归属(`AccountRegisteredCid` 精确整键)。
  ///
  /// 返回以入参账户原样为键的 map;未注册或解码失败的账户值为 null。
  /// 机构多签发现的唯一反查入口(ADR-018 R2:多 key 一律批量,杜绝循环内逐条)。
  /// 公私两 pallet 各有独立 `AccountRegisteredCid`,故对每账户双查取首命中。
  Future<Map<String, RegisteredInstitutionRef?>>
      fetchRegisteredInstitutionRefsBatch(
    Iterable<String> accountHexList, {
    int chunkSize = 100,
  }) async {
    final addresses = accountHexList
        .where((address) => address.isNotEmpty)
        .toSet()
        .toList(growable: false);
    if (addresses.isEmpty) return {};

    final result = <String, RegisteredInstitutionRef?>{
      for (final address in addresses) address: null,
    };
    // 公私两 pallet 顺序双查:已命中的账户不再查下一 pallet。
    for (final pallet in InstitutionPalletRouter.managePallets) {
      final pending =
          addresses.where((a) => result[a] == null).toList(growable: false);
      if (pending.isEmpty) break;
      final storageKeyByAccount = <String, String>{
        for (final address in pending)
          address:
              '0x${_hexEncode(MultisigStorageCodec.accountRegisteredCidKey(address, pallet))}',
      };
      final values = await _rpc.fetchStorageBatchChunked(
        storageKeyByAccount.values.toSet(),
        chunkSize: chunkSize,
      );
      for (final entry in storageKeyByAccount.entries) {
        final data = values[entry.value];
        if (data == null) continue;
        result[entry.key] =
            MultisigStorageCodec.decodeRegisteredInstitution(data);
      }
    }
    return result;
  }

  /// 查询机构多签账户信息。
  ///
  /// 注册机构账户走 `AccountRegisteredCid -> Institutions/InstitutionAccounts`
  /// + 分类管理员模块 `AdminAccounts`(按机构码路由 admins pallet)。
  Future<InstitutionAccountInfo?> fetchAccount(String accountHex) async {
    return _fetchInstitutionAccount(accountHex);
  }

  /// 批量查询机构多签账户状态。
  ///
  /// 机构多签需要先从账户反查 CID 与账户名(双查命中 pallet)，再按命中 pallet
  /// 读取账户主体和机构主体，最后按机构码读管理员主体，所以必须分阶段批量读取。
  Future<Map<String, InstitutionAccountInfo?>> fetchAccountsBatch(
    Iterable<String> accountHexList, {
    int chunkSize = 100,
  }) async {
    final addresses = accountHexList
        .map(_normalizeHex)
        .where((address) => address.isNotEmpty)
        .toSet()
        .toList(growable: false);
    if (addresses.isEmpty) return {};

    final result = <String, InstitutionAccountInfo?>{};
    // 反查 CID 归属 + 命中 pallet(公私双查;命中即定 pallet,贯穿后续 cid 键读)。
    final refByAddress = <String, RegisteredInstitutionRef>{};
    final palletByAddress = <String, String>{};
    for (final pallet in InstitutionPalletRouter.managePallets) {
      final pending = addresses
          .where((a) => !refByAddress.containsKey(a))
          .toList(growable: false);
      if (pending.isEmpty) break;
      final refKeyByAccount = <String, String>{
        for (final address in pending)
          address:
              '0x${_hexEncode(MultisigStorageCodec.accountRegisteredCidKey(address, pallet))}',
      };
      final refValues = await _rpc.fetchStorageBatchChunked(
        refKeyByAccount.values,
        chunkSize: chunkSize,
      );
      for (final address in pending) {
        final refData = refValues[refKeyByAccount[address]];
        if (refData == null) continue;
        final ref = MultisigStorageCodec.decodeRegisteredInstitution(refData);
        if (ref == null) continue;
        refByAddress[address] = ref;
        palletByAddress[address] = pallet;
      }
    }
    for (final address in addresses) {
      if (!refByAddress.containsKey(address)) result[address] = null;
    }

    final accountKeyByAccount = <String, String>{};
    final institutionKeyByAccount = <String, String>{};
    final adminKeyByAccount = <String, String>{};
    final accountIdByAccount = <String, Uint8List>{};
    final secondRoundKeys = <String>[];
    for (final entry in refByAddress.entries) {
      final pallet = palletByAddress[entry.key]!;
      final accountId = MultisigStorageCodec.accountIdFromAccountHex(entry.key);
      final accountKey =
          '0x${_hexEncode(MultisigStorageCodec.institutionAccountKey(
        entry.value.cidNumber,
        entry.value.accountName,
        pallet,
      ))}';
      final institutionKey =
          '0x${_hexEncode(MultisigStorageCodec.institutionKey(entry.value.cidNumber, pallet))}';
      accountIdByAccount[entry.key] = accountId;
      accountKeyByAccount[entry.key] = accountKey;
      institutionKeyByAccount[entry.key] = institutionKey;
      secondRoundKeys
        ..add(accountKey)
        ..add(institutionKey);
    }

    final secondRoundValues = await _rpc.fetchStorageBatchChunked(
      secondRoundKeys,
      chunkSize: chunkSize,
    );
    final accountByAccount = <String, InstitutionAccountSnapshot>{};
    final institutionByAccount = <String, AdminSnapshot>{};
    for (final address in refByAddress.keys) {
      final accountData = secondRoundValues[accountKeyByAccount[address]];
      final institutionData =
          secondRoundValues[institutionKeyByAccount[address]];
      if (accountData == null || institutionData == null) {
        result[address] = null;
        continue;
      }
      final account =
          MultisigStorageCodec.decodeInstitutionAccount(accountData);
      final institution =
          MultisigStorageCodec.decodeInstitutionInfo(institutionData);
      if (account == null || institution == null) {
        result[address] = null;
        continue;
      }
      accountByAccount[address] = account;
      institutionByAccount[address] = institution;
    }

    final adminRoundKeys = <String>[];
    for (final entry in institutionByAccount.entries) {
      final adminKey = '0x${_hexEncode(MultisigStorageCodec.adminAccountKey(
        institutionCode: entry.value.institutionCode,
        accountId: accountIdByAccount[entry.key]!,
      ))}';
      adminKeyByAccount[entry.key] = adminKey;
      adminRoundKeys.add(adminKey);
    }

    final adminValues = await _rpc.fetchStorageBatchChunked(
      adminRoundKeys,
      chunkSize: chunkSize,
    );
    final adminByAccount = <String, AdminSnapshot>{};
    for (final address in institutionByAccount.keys) {
      final adminData = adminValues[adminKeyByAccount[address]];
      if (adminData == null) {
        result[address] = null;
        continue;
      }
      final admin = MultisigStorageCodec.decodeAdminAccount(adminData);
      if (admin == null) {
        result[address] = null;
        continue;
      }
      adminByAccount[address] = admin;
    }

    final activeThresholdKeyByAccount = <String, String>{};
    for (final entry in adminByAccount.entries) {
      activeThresholdKeyByAccount[entry.key] =
          '0x${_hexEncode(MultisigStorageCodec.dynamicThresholdKey(
        storageName: 'ActiveDynamicThresholds',
        institutionCode: entry.value.institutionCode,
        accountId: accountIdByAccount[entry.key]!,
      ))}';
    }
    final activeThresholdValues = await _rpc.fetchStorageBatchChunked(
      activeThresholdKeyByAccount.values,
      chunkSize: chunkSize,
    );

    final thresholdByAccount = <String, int?>{};
    final pendingThresholdKeyByAccount = <String, String>{};
    for (final entry in activeThresholdKeyByAccount.entries) {
      final threshold = MultisigStorageCodec.decodeDynamicThreshold(
        activeThresholdValues[entry.value],
      );
      thresholdByAccount[entry.key] = threshold;
      if (threshold == null) {
        final admin = adminByAccount[entry.key]!;
        pendingThresholdKeyByAccount[entry.key] =
            '0x${_hexEncode(MultisigStorageCodec.dynamicThresholdKey(
          storageName: 'PendingDynamicThresholds',
          institutionCode: admin.institutionCode,
          accountId: accountIdByAccount[entry.key]!,
        ))}';
      }
    }

    if (pendingThresholdKeyByAccount.isNotEmpty) {
      final pendingThresholdValues = await _rpc.fetchStorageBatchChunked(
        pendingThresholdKeyByAccount.values,
        chunkSize: chunkSize,
      );
      for (final entry in pendingThresholdKeyByAccount.entries) {
        thresholdByAccount[entry.key] =
            MultisigStorageCodec.decodeDynamicThreshold(
          pendingThresholdValues[entry.value],
        );
      }
    }

    for (final address in addresses) {
      final account = accountByAccount[address];
      final admin = adminByAccount[address];
      if (account == null || admin == null) continue;
      result[address] = InstitutionAccountInfo(
        adminsLen: admin.adminsLen,
        threshold: thresholdByAccount[address],
        admins: admin.admins,
        status: _statusFromByte(account.statusByte),
      );
    }

    return result;
  }

  Future<InstitutionAccountInfo?> _fetchInstitutionAccount(
    String accountHex,
  ) async {
    // 反查 AccountRegisteredCid:公私双查,命中即定 pallet,贯穿后续 cid 键读。
    RegisteredInstitutionRef? ref;
    String? managePallet;
    for (final pallet in InstitutionPalletRouter.managePallets) {
      final refKey =
          MultisigStorageCodec.accountRegisteredCidKey(accountHex, pallet);
      final refData = await _rpc.fetchStorage('0x${_hexEncode(refKey)}');
      if (refData == null) continue;
      final decoded = MultisigStorageCodec.decodeRegisteredInstitution(refData);
      if (decoded != null) {
        ref = decoded;
        managePallet = pallet;
        break;
      }
    }
    if (ref == null || managePallet == null) return null;

    final accountKey = MultisigStorageCodec.institutionAccountKey(
      ref.cidNumber,
      ref.accountName,
      managePallet,
    );
    final accountData = await _rpc.fetchStorage('0x${_hexEncode(accountKey)}');
    if (accountData == null) return null;
    final account = MultisigStorageCodec.decodeInstitutionAccount(accountData);
    if (account == null) return null;
    final institutionKey =
        MultisigStorageCodec.institutionKey(ref.cidNumber, managePallet);
    final institutionData =
        await _rpc.fetchStorage('0x${_hexEncode(institutionKey)}');
    if (institutionData == null) return null;
    final institution =
        MultisigStorageCodec.decodeInstitutionInfo(institutionData);
    if (institution == null) return null;
    final accountId = MultisigStorageCodec.accountIdFromAccountHex(accountHex);
    final adminKey = MultisigStorageCodec.adminAccountKey(
      institutionCode: institution.institutionCode,
      accountId: accountId,
    );
    final adminData = await _rpc.fetchStorage('0x${_hexEncode(adminKey)}');
    if (adminData == null) return null;
    final admin = MultisigStorageCodec.decodeAdminAccount(adminData);
    if (admin == null) return null;
    final threshold = await _fetchInstitutionDynamicThreshold(
      institutionCode: admin.institutionCode,
      accountId: accountId,
    );
    return InstitutionAccountInfo(
      adminsLen: admin.adminsLen,
      threshold: threshold,
      admins: admin.admins,
      status: _statusFromByte(account.statusByte),
    );
  }

  Future<int?> _fetchInstitutionDynamicThreshold({
    required String institutionCode,
    required Uint8List accountId,
  }) async {
    for (final storageName in const [
      'ActiveDynamicThresholds',
      'PendingDynamicThresholds',
    ]) {
      final key = MultisigStorageCodec.dynamicThresholdKey(
        storageName: storageName,
        institutionCode: institutionCode,
        accountId: accountId,
      );
      final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
      final threshold = MultisigStorageCodec.decodeDynamicThreshold(data);
      if (threshold != null) return threshold;
    }
    return null;
  }

  /// 从 ProposalData 解码机构多签管理(关闭)提案,供提案列表/详情只读展示。
  ///
  /// ProposalData = BoundedVec<u8>(Compact<len> + bytes);机构管理提案以 MODULE_TAG 前缀认领,
  /// 公权=`pub-mgmt`、私权=`pri-mgmt`(取代旧 `org-mgmt`),其后 ACTION_CLOSE(2):
  /// account(32) + beneficiary(32) + proposer(32)。解码失败返回 null。
  /// 个人多签提案解码在 `PersonalManageService`。
  static const _publicManageTag = [
    0x70, 0x75, 0x62, 0x2d, 0x6d, 0x67, 0x6d, 0x74, // "pub-mgmt"
  ];
  static const _privateManageTag = [
    0x70, 0x72, 0x69, 0x2d, 0x6d, 0x67, 0x6d, 0x74, // "pri-mgmt"
  ];

  Object? decodeManageProposalData(int proposalId, Uint8List raw) {
    try {
      var offset = 0;

      // BoundedVec<u8> 外层：Compact<len> + bytes
      final (vecLen, lenBytes) = _decodeCompact(raw, offset);
      offset += lenBytes;
      if (offset + vecLen > raw.length) return null;
      final data = raw.sublist(offset, offset + vecLen);

      // 公权/私权机构管理提案各自 MODULE_TAG(8 字节)。
      const tagLen = 8;
      final tag = _startsWith(data, _publicManageTag) ||
          _startsWith(data, _privateManageTag);
      if (!tag) return null;
      final actionType = data[tagLen];
      final payload = data.sublist(tagLen + 1);
      if (actionType == actionClose) {
        return _decodeCloseAction(proposalId, payload);
      }
      return null;
    } catch (_) {
      return null;
    }
  }

  CloseProposalInfo? _decodeCloseAction(
      int proposalId, Uint8List data) {
    // account(32) + beneficiary(32) + proposer(32)
    if (data.length != 32 + 32 + 32) return null;
    var offset = 0;

    final account =
        _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32)));
    offset += 32;

    final beneficiaryBytes = data.sublist(offset, offset + 32);
    final beneficiarySs58 =
        Keyring().encodeAddress(Uint8List.fromList(beneficiaryBytes), 2027);
    offset += 32;

    final proposerBytes = data.sublist(offset, offset + 32);
    final proposerSs58 =
        Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);

    return CloseProposalInfo(
      proposalId: proposalId,
      account: account,
      beneficiary: beneficiarySs58,
      proposer: proposerSs58,
    );
  }

  // ──── 内部：storage key 构造 ────

  /// StorageDoubleMap key: twox128(pallet) + twox128(storage)
  ///   + blake2_128_concat(key1) + blake2_128_concat(key2)
  Uint8List _buildDoubleMapStorageKey(
    String palletName,
    String storageName,
    Uint8List key1Data,
    Uint8List key2Data,
  ) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString(storageName);
    final key1Hash = _blake2128Concat(key1Data);
    final key2Hash = _blake2128Concat(key2Data);

    final result = Uint8List(palletHash.length +
        storageHash.length +
        key1Hash.length +
        key2Hash.length);
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, key1Hash);
    offset += key1Hash.length;
    result.setAll(offset, key2Hash);
    return result;
  }

  Uint8List _blake2128Concat(Uint8List data) {
    final hash = Hasher.blake2b128.hash(data);
    final result = Uint8List(hash.length + data.length);
    result.setAll(0, hash);
    result.setAll(hash.length, data);
    return result;
  }

  // ──── 内部：解码工具 ────

  InstitutionStatus _statusFromByte(int statusByte) {
    return statusByte == 1
        ? InstitutionStatus.active
        : InstitutionStatus.pending;
  }

  bool _startsWith(Uint8List data, List<int> prefix) {
    if (data.length < prefix.length + 1) return false;
    for (var i = 0; i < prefix.length; i++) {
      if (data[i] != prefix[i]) return false;
    }
    return true;
  }

  (int, int) _decodeCompact(Uint8List data, int offset) {
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) {
      return (first >> 2, 1);
    } else if (mode == 1) {
      final val = (data[offset] | (data[offset + 1] << 8)) >> 2;
      return (val, 2);
    } else if (mode == 2) {
      final val = (data[offset] |
              (data[offset + 1] << 8) |
              (data[offset + 2] << 16) |
              (data[offset + 3] << 24)) >>
          2;
      return (val, 4);
    } else {
      final lenBytes = (first >> 2) + 4;
      var val = 0;
      for (var i = lenBytes - 1; i >= 0; i--) {
        val = (val << 8) | data[offset + 1 + i];
      }
      return (val, 1 + lenBytes);
    }
  }

  static String _hexEncode(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  static String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }
}
