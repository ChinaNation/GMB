import 'package:flutter/foundation.dart';
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';

import 'personal_manage_models.dart';
import 'personal_manage_storage_codec.dart';

/// PersonalAdmins 链上交互服务。
///
/// 只负责个人多签的创建、关闭、查询和 PersonalAdmins ProposalData 解码；
/// 机构多签链访问由 `citizen/institution` 的 InstitutionChainService 处理。
class PersonalManageService {
  PersonalManageService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  /// PersonalAdmins pallet index(runtime pallet_index=7)。
  static const _palletIndex = 7;

  /// PersonalAdmins::propose_create call_index=0。
  static const _proposeCreateCallIndex = 0;

  /// PersonalAdmins::propose_close call_index=1。
  static const _proposeCloseCallIndex = 1;

  /// PersonalAdmins 个人账户创建成功事件 event_index=0。
  static const _personalAccountProposedEventIndex = 0;

  /// PersonalAdmins ProposalData action。
  static const actionCreate = 0;
  static const actionClose = 1;

  static const _moduleTag = [
    0x70,
    0x65,
    0x72,
    0x2d,
    0x6d,
    0x67,
    0x6d,
    0x74
  ]; // "per-mgmt"

  /// 提交 PersonalAdmins::propose_create extrinsic（个人多签，无需 CID）。
  Future<
      ({
        String txHash,
        int usedNonce,
        int proposalId,
        String accountHex,
        String blockHashHex,
      })> submitProposeCreatePersonal({
    required Uint8List accountName,
    required List<Uint8List> admins,
    required int regularThreshold,
    required BigInt amountFen,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = buildProposeCreatePersonalCallData(
      accountName: accountName,
      admins: admins,
      regularThreshold: regularThreshold,
      amountFen: amountFen,
    );
    final submitResult = await _signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
    final event = await _confirmPersonalAccountProposedEvent(
      blockHashHex: submitResult.blockHashHex,
      accountName: accountName,
      admins: admins,
      regularThreshold: regularThreshold,
      amountFen: amountFen,
      proposerPubkey: signerPubkey,
    );
    return (
      txHash: submitResult.txHash,
      usedNonce: submitResult.usedNonce,
      proposalId: event.proposalId,
      accountHex: event.accountHex,
      blockHashHex: submitResult.blockHashHex,
    );
  }

  /// 构造个人多签创建 call_data。用于生产提交与测试逐字节对齐。
  @visibleForTesting
  static Uint8List buildProposeCreatePersonalCallData({
    required Uint8List accountName,
    required List<Uint8List> admins,
    required int regularThreshold,
    required BigInt amountFen,
  }) {
    if (accountName.isEmpty || accountName.length > 128) {
      throw ArgumentError('account_name 长度需在 1..=128 字节');
    }
    if (admins.length < 2 || admins.length > 64) {
      throw ArgumentError('个人多签管理员数量需在 2..=64');
    }
    final minThreshold = minimumRegularThreshold(admins.length);
    if (regularThreshold < minThreshold || regularThreshold > admins.length) {
      throw ArgumentError(
          'regular_threshold 范围必须在 $minThreshold..=${admins.length}');
    }
    final seen = <String>{};
    for (final pubkey in admins) {
      if (pubkey.length != 32) {
        throw ArgumentError('admins 每项必须为 32 字节');
      }
      final hex = _hexEncode(pubkey);
      if (!seen.add(hex)) {
        throw ArgumentError('admins 不允许重复');
      }
    }
    if (amountFen <= BigInt.zero) {
      throw ArgumentError('amount 必须大于 0');
    }

    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCreateCallIndex);

    // account_name: BoundedVec<u8> = Compact<u32> length + bytes
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(accountName.length)));
    output.write(accountName);

    // admins: BoundedVec<AccountId32>
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(admins.length)));
    for (final pubkey in admins) {
      output.write(pubkey);
    }

    // regular_threshold: u32 little-endian。注册提案阈值仍由链端固定为全员通过。
    output.write(_u32ToLeBytesStatic(regularThreshold));

    // amount: u128 little-endian
    output.write(_u128ToLeBytesStatic(amountFen));

    return output.toBytes();
  }

  /// 提交 PersonalAdmins::propose_close extrinsic。
  Future<({String txHash, int usedNonce})> submitProposeClosePersonal({
    required String account,
    required String beneficiaryAddress,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCloseCallIndex);
    output.write(_hexDecode(account));
    final beneficiaryId = Keyring().decodeAddress(beneficiaryAddress);
    output.write(beneficiaryId);
    return _signAndSubmit(
      callData: output.toBytes(),
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 批量反查多个个人多签账户的发起人 / 账户名(`PersonalAccounts` 精确整键)。
  ///
  /// 返回以入参地址原样为键的 map;未注册或解码失败的地址值为 null。
  /// 个人多签发现的唯一反查入口(ADR-018 R2:多 key 一律批量,杜绝循环内逐条)。
  Future<Map<String, ({String creatorAccountHex, String accountName})?>>
      fetchPersonalMetasBatch(
    Iterable<String> personalAccountHexList, {
    int chunkSize = 100,
  }) async {
    final addresses = personalAccountHexList
        .where((address) => address.isNotEmpty)
        .toSet()
        .toList(growable: false);
    if (addresses.isEmpty) return {};

    final storageKeyByAccount = <String, String>{
      for (final address in addresses)
        address:
            '0x${_hexEncode(PersonalManageStorageCodec.personalAccountsKey(address))}',
    };

    final values = await _rpc.fetchStorageBatchChunked(
      storageKeyByAccount.values.toSet(),
      chunkSize: chunkSize,
    );

    final result =
        <String, ({String creatorAccountHex, String accountName})?>{};
    for (final entry in storageKeyByAccount.entries) {
      final data = values[entry.value];
      final meta = data == null
          ? null
          : PersonalManageStorageCodec.decodePersonalAccount(data);
      result[entry.key] = meta == null
          ? null
          : (
              creatorAccountHex: meta.creatorHex,
              accountName:
                  PersonalManageStorageCodec.accountNameText(meta.accountName),
            );
    }
    return result;
  }

  /// 查询个人多签账户信息。
  Future<AccountInfo?> fetchPersonalAccount(
    String accountHex,
  ) async {
    final key = PersonalManageStorageCodec.personalAccountsKey(
      accountHex,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null) return null;
    final personal = PersonalManageStorageCodec.decodePersonalAccount(data);
    if (personal == null) return null;
    final accountId = PersonalManageStorageCodec.accountIdFromAccountHex(
      accountHex,
    );
    final adminKey = PersonalManageStorageCodec.adminAccountKey(accountId);
    final adminData = await _rpc.fetchStorage('0x${_hexEncode(adminKey)}');
    if (adminData == null) return null;
    final admin = PersonalManageStorageCodec.decodeAdminAccount(adminData);
    if (admin == null) return null;
    final threshold = await _fetchPersonalDynamicThreshold(
      institutionCode: admin.institutionCode,
      accountId: accountId,
    );
    return AccountInfo(
      adminsLen: admin.adminsLen,
      threshold: threshold,
      admins: admin.admins,
      status: _statusFromByte(personal.statusByte),
    );
  }

  /// 批量查询个人多签账户状态。
  ///
  /// 多签列表页不能对每个账户逐个调用 [fetchPersonalAccount]。
  /// 这里按 storage 依赖分阶段批量读取：先读账户与管理员主体，再批量读动态阈值。
  Future<Map<String, AccountInfo?>> fetchPersonalAccountsBatch(
    Iterable<String> accountHexList, {
    int chunkSize = 100,
  }) async {
    final addresses = accountHexList
        .map(_normalizeHex)
        .where((address) => address.isNotEmpty)
        .toSet()
        .toList(growable: false);
    if (addresses.isEmpty) return {};

    final personalKeyByAddress = <String, String>{};
    final adminKeyByAccount = <String, String>{};
    final accountIdByAccount = <String, Uint8List>{};
    final firstRoundKeys = <String>[];

    for (final address in addresses) {
      final accountId = PersonalManageStorageCodec.accountIdFromAccountHex(
        address,
      );
      final personalKey =
          '0x${_hexEncode(PersonalManageStorageCodec.personalAccountsKey(address))}';
      final adminKey =
          '0x${_hexEncode(PersonalManageStorageCodec.adminAccountKey(accountId))}';
      accountIdByAccount[address] = accountId;
      personalKeyByAddress[address] = personalKey;
      adminKeyByAccount[address] = adminKey;
      firstRoundKeys
        ..add(personalKey)
        ..add(adminKey);
    }

    final firstRoundValues = await _rpc.fetchStorageBatchChunked(
      firstRoundKeys,
      chunkSize: chunkSize,
    );
    final result = <String, AccountInfo?>{};
    final personalByAddress = <String, PersonalManageAccountSnapshot>{};
    final adminByAccount = <String, PersonalManageAdminSnapshot>{};

    for (final address in addresses) {
      final personalData = firstRoundValues[personalKeyByAddress[address]];
      final adminData = firstRoundValues[adminKeyByAccount[address]];
      if (personalData == null || adminData == null) {
        result[address] = null;
        continue;
      }
      final personal =
          PersonalManageStorageCodec.decodePersonalAccount(personalData);
      final admin = PersonalManageStorageCodec.decodeAdminAccount(adminData);
      if (personal == null || admin == null) {
        result[address] = null;
        continue;
      }
      personalByAddress[address] = personal;
      adminByAccount[address] = admin;
    }

    final activeThresholdKeyByAccount = <String, String>{};
    for (final entry in adminByAccount.entries) {
      final accountId = accountIdByAccount[entry.key]!;
      activeThresholdKeyByAccount[entry.key] =
          '0x${_hexEncode(PersonalManageStorageCodec.dynamicThresholdKey(
        storageName: 'ActiveDynamicThresholds',
        institutionCode: entry.value.institutionCode,
        accountId: accountId,
      ))}';
    }
    final activeThresholdValues = await _rpc.fetchStorageBatchChunked(
      activeThresholdKeyByAccount.values,
      chunkSize: chunkSize,
    );

    final thresholdByAccount = <String, int?>{};
    final pendingThresholdKeyByAccount = <String, String>{};
    for (final entry in activeThresholdKeyByAccount.entries) {
      final threshold = PersonalManageStorageCodec.decodeDynamicThreshold(
        activeThresholdValues[entry.value],
      );
      thresholdByAccount[entry.key] = threshold;
      if (threshold == null) {
        final admin = adminByAccount[entry.key]!;
        pendingThresholdKeyByAccount[entry.key] =
            '0x${_hexEncode(PersonalManageStorageCodec.dynamicThresholdKey(
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
            PersonalManageStorageCodec.decodeDynamicThreshold(
          pendingThresholdValues[entry.value],
        );
      }
    }

    for (final address in addresses) {
      final personal = personalByAddress[address];
      final admin = adminByAccount[address];
      if (personal == null || admin == null) continue;
      result[address] = AccountInfo(
        adminsLen: admin.adminsLen,
        threshold: thresholdByAccount[address],
        admins: admin.admins,
        status: _statusFromByte(personal.statusByte),
      );
    }

    return result;
  }

  Future<int?> _fetchPersonalDynamicThreshold({
    required String institutionCode,
    required Uint8List accountId,
  }) async {
    for (final storageName in const [
      'ActiveDynamicThresholds',
      'PendingDynamicThresholds',
    ]) {
      final key = PersonalManageStorageCodec.dynamicThresholdKey(
        storageName: storageName,
        institutionCode: institutionCode,
        accountId: accountId,
      );
      final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
      final threshold = PersonalManageStorageCodec.decodeDynamicThreshold(data);
      if (threshold != null) return threshold;
    }
    return null;
  }

  /// 从 ProposalData 解码 PersonalAdmins 创建或关闭提案。
  Object? decodePersonalProposalData(int proposalId, Uint8List raw) {
    try {
      var offset = 0;
      final (vecLen, lenBytes) = _decodeCompact(raw, offset);
      offset += lenBytes;
      if (offset + vecLen > raw.length) return null;
      final data = raw.sublist(offset, offset + vecLen);

      if (!_startsWith(data, _moduleTag)) return null;
      final actionType = data[_moduleTag.length];
      final payload = data.sublist(_moduleTag.length + 1);
      if (actionType == actionCreate) {
        return _decodeCreateAction(proposalId, payload);
      }
      if (actionType == actionClose) {
        return _decodeCloseAction(proposalId, payload);
      }
      return null;
    } catch (_) {
      return null;
    }
  }

  CreateMultisigProposalInfo? _decodeCreateAction(
    int proposalId,
    Uint8List data,
  ) {
    if (data.length != 32 + 32 + 16 + 16) return null;
    var offset = 0;

    final account =
        _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32)));
    offset += 32;

    final proposerBytes = data.sublist(offset, offset + 32);
    final proposerSs58 =
        Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);
    offset += 32;

    final amountFen = _readU128Le(data.sublist(offset, offset + 16));
    offset += 16;

    final feeFen = _readU128Le(data.sublist(offset, offset + 16));

    return CreateMultisigProposalInfo(
      proposalId: proposalId,
      account: account,
      proposer: proposerSs58,
      amountFen: amountFen,
      feeFen: feeFen,
    );
  }

  CloseMultisigProposalInfo? _decodeCloseAction(
    int proposalId,
    Uint8List data,
  ) {
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

    return CloseMultisigProposalInfo(
      proposalId: proposalId,
      account: account,
      beneficiary: beneficiarySs58,
      proposer: proposerSs58,
    );
  }

  Future<({int proposalId, String accountHex})>
      _confirmPersonalAccountProposedEvent({
    required String blockHashHex,
    required Uint8List accountName,
    required List<Uint8List> admins,
    required int regularThreshold,
    required BigInt amountFen,
    required Uint8List proposerPubkey,
  }) async {
    final events = await _rpc.fetchSystemEventsAtBlock(blockHashHex);
    if (events == null || events.isEmpty) {
      throw StateError('交易已入块，但未读取到 System.Events，不能确认个人多签创建提案');
    }
    final failure = _rpc.findExtrinsicFailureInEvents(events);
    if (failure != null) {
      throw StateError(failure.description);
    }
    final found = _findPersonalAccountProposedEvent(
      events,
      accountName: accountName,
      admins: admins,
      regularThreshold: regularThreshold,
      amountFen: amountFen,
      proposerPubkey: proposerPubkey,
    );
    if (found == null) {
      throw StateError(
        '交易已入块，但未确认 PersonalAdmins 个人账户创建成功事件，也未检测到链上失败事件，请检查当前区块事件',
      );
    }
    return found;
  }

  ({int proposalId, String accountHex})? _findPersonalAccountProposedEvent(
    Uint8List data, {
    required Uint8List accountName,
    required List<Uint8List> admins,
    required int regularThreshold,
    required BigInt amountFen,
    required Uint8List proposerPubkey,
  }) {
    final (_, countSize) = _decodeCompact(data, 0);
    if (countSize <= 0) return null;
    for (var scanOffset = countSize; scanOffset < data.length; scanOffset++) {
      try {
        var offset = scanOffset;
        final phase = data[offset];
        offset += 1;
        if (phase == 0x00) {
          if (offset + 4 > data.length) continue;
          offset += 4;
        } else if (phase != 0x01 && phase != 0x02) {
          continue;
        }

        if (offset + 2 > data.length) continue;
        final palletIndex = data[offset];
        final eventIndex = data[offset + 1];
        offset += 2;

        if (palletIndex == _palletIndex &&
            eventIndex == _personalAccountProposedEventIndex) {
          final decoded = _decodePersonalAccountProposedEvent(
            data,
            offset,
            accountName: accountName,
            admins: admins,
            regularThreshold: regularThreshold,
            amountFen: amountFen,
            proposerPubkey: proposerPubkey,
          );
          if (decoded != null) return decoded;
        }
      } catch (_) {
        // System.Events 里混有其他 pallet 事件，扫描失败继续尝试后续 offset。
      }
    }
    return null;
  }

  ({int proposalId, String accountHex})? _decodePersonalAccountProposedEvent(
    Uint8List data,
    int offset, {
    required Uint8List accountName,
    required List<Uint8List> admins,
    required int regularThreshold,
    required BigInt amountFen,
    required Uint8List proposerPubkey,
  }) {
    try {
      var pos = offset;
      if (pos + 8 + 32 + 32 > data.length) return null;
      final proposalId = _readU64Le(data, pos);
      pos += 8;
      final account = Uint8List.fromList(data.sublist(pos, pos + 32));
      pos += 32;
      final proposer = Uint8List.fromList(data.sublist(pos, pos + 32));
      pos += 32;
      final nameRead = _readCompactBytes(data, pos);
      if (nameRead == null) return null;
      pos = nameRead.nextOffset;
      final (adminsLen, adminsLenBytes) = _decodeCompact(data, pos);
      pos += adminsLenBytes;
      if (adminsLen < 0 || pos + adminsLen * 32 > data.length) return null;
      final eventAdmins = <Uint8List>[];
      for (var i = 0; i < adminsLen; i++) {
        eventAdmins.add(Uint8List.fromList(data.sublist(pos, pos + 32)));
        pos += 32;
      }
      if (pos + 4 + 4 + 16 + 16 > data.length) return null;
      final eventAdminsLen = _readU32Le(data, pos);
      pos += 4;
      final eventThreshold = _readU32Le(data, pos);
      pos += 4;
      final eventAmount = _readU128Le(data.sublist(pos, pos + 16));

      final matches = _bytesEqual(proposer, proposerPubkey) &&
          _bytesEqual(nameRead.bytes, accountName) &&
          eventAdminsLen == admins.length &&
          eventThreshold == regularThreshold &&
          eventAmount == amountFen &&
          _adminListsEqual(eventAdmins, admins);
      if (!matches) return null;
      return (
        proposalId: proposalId,
        accountHex: _hexEncode(account),
      );
    } catch (_) {
      return null;
    }
  }

  Future<({String txHash, int usedNonce})> _signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'PersonalAdmins',
    ).signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  Future<({String txHash, int usedNonce, String blockHashHex})>
      _signAndSubmitInBlock({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'PersonalAdmins',
    ).signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  static MultisigStatus _statusFromByte(int statusByte) {
    return statusByte == 1 ? MultisigStatus.active : MultisigStatus.pending;
  }

  /// 普通提案最低阈值：必须严格过半。
  static int minimumRegularThreshold(int adminsLen) {
    if (adminsLen < 2) return 2;
    return (adminsLen ~/ 2) + 1;
  }

  static Uint8List _u32ToLeBytesStatic(int value) {
    return Uint8List.fromList([
      value & 0xFF,
      (value >> 8) & 0xFF,
      (value >> 16) & 0xFF,
      (value >> 24) & 0xFF,
    ]);
  }

  static Uint8List _u128ToLeBytesStatic(BigInt value) {
    final bytes = Uint8List(16);
    var v = value;
    for (var i = 0; i < 16; i++) {
      bytes[i] = (v & BigInt.from(0xFF)).toInt();
      v >>= 8;
    }
    return bytes;
  }

  static BigInt _readU128Le(Uint8List bytes) {
    var value = BigInt.zero;
    for (var i = bytes.length - 1; i >= 0; i--) {
      value = (value << 8) | BigInt.from(bytes[i]);
    }
    return value;
  }

  static int _readU64Le(Uint8List data, int offset) {
    var value = 0;
    for (var i = 7; i >= 0; i--) {
      value = (value << 8) | data[offset + i];
    }
    return value;
  }

  static int _readU32Le(Uint8List data, int offset) {
    return data[offset] |
        (data[offset + 1] << 8) |
        (data[offset + 2] << 16) |
        (data[offset + 3] << 24);
  }

  static ({Uint8List bytes, int nextOffset})? _readCompactBytes(
    Uint8List data,
    int offset,
  ) {
    if (offset >= data.length) return null;
    final (length, lengthBytes) = _decodeCompact(data, offset);
    final start = offset + lengthBytes;
    final end = start + length;
    if (length < 0 || start > data.length || end > data.length) return null;
    return (
      bytes: Uint8List.fromList(data.sublist(start, end)),
      nextOffset: end,
    );
  }

  static bool _adminListsEqual(
    List<Uint8List> left,
    List<Uint8List> right,
  ) {
    if (left.length != right.length) return false;
    for (var i = 0; i < left.length; i++) {
      if (!_bytesEqual(left[i], right[i])) return false;
    }
    return true;
  }

  static bool _bytesEqual(Uint8List left, Uint8List right) {
    if (left.length != right.length) return false;
    for (var i = 0; i < left.length; i++) {
      if (left[i] != right[i]) return false;
    }
    return true;
  }

  static bool _startsWith(Uint8List data, List<int> prefix) {
    if (data.length < prefix.length + 1) return false;
    for (var i = 0; i < prefix.length; i++) {
      if (data[i] != prefix[i]) return false;
    }
    return true;
  }

  static (int, int) _decodeCompact(Uint8List data, int offset) {
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

  static String _hexEncode(List<int> bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  static String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
