import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/signed_extrinsic_builder.dart';

import 'institution_manage_models.dart';
import 'duoqian_storage_codec.dart';

/// 机构多签创建时的初始账户资金条目。
class InstitutionInitialAccountInput {
  const InstitutionInitialAccountInput({
    required this.accountName,
    required this.amountFen,
  });

  /// 账户名称必须使用 SFID `/registration-info.account_names` 返回值。
  final String accountName;

  /// 初始资金,单位为分。
  final BigInt amountFen;
}

// 业务目录 lib/organization-manage/ 只保留 OrganizationManage 机构多签入口；
// 个人多签主业务已经迁移到 lib/personal-manage/。

/// 机构多签管理链上交互服务（对应 OrganizationManage pallet 17）。
///
/// 负责 propose_create_institution / propose_close 等机构提案创建类
/// extrinsic 的编码与提交,以及 SFID 注册状态和机构多签账户 storage 查询。
///
/// Phase 3(2026-04-22): 本 pallet 内部的管理员投票入口已从链端物理删除,
/// 管理员投票一律走 `InternalVote::cast`(22.0),
/// 通过 [InternalVoteService] 或业务 service 的 `submitInternalVote`
/// 统一入口发送。
class InstitutionManageService {
  InstitutionManageService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// OrganizationManage pallet index（runtime pallet_index=17,机构多签管理）。
  static const _palletIndex = 17;

  /// OrganizationManage::propose_create_institution call_index=5。
  static const _proposeCreateInstitutionCallIndex = 5;

  /// OrganizationManage::propose_close call_index=1(机构关闭)。
  static const _proposeCloseCallIndex = 1;

  /// OrganizationManage::InstitutionCreateProposed event_index=4。
  static const _institutionCreateProposedEventIndex = 4;

  /// ProposalData 中的 action 类型前缀。
  /// OrganizationManage(b"org-mgmt") 命名空间:ACTION_CLOSE=2。
  static const actionClose = 2;

  // ──── Extrinsic 提交 ────

  /// 提交机构多签 propose_create_institution extrinsic。
  ///
  /// 参数编码以 `memory/07-ai/unified-protocols.md` 的 P-TX-001 为准：
  /// [0x11][0x05] + sfid_number + sfid_full_name + accounts + org + admins_len
  ///   + admins + threshold + register_nonce + signature
  ///   + issuer_sfid_number + issuer_main_account + signer_pubkey + scope_*。
  Future<
      ({
        String txHash,
        int usedNonce,
        int proposalId,
        String mainAccountHex,
        String blockHashHex,
      })> submitProposeCreateInstitution({
    required String sfidNumber,
    required String sfidFullName,
    required List<InstitutionInitialAccountInput> accounts,
    required int org,
    required int adminsLen,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required String registerNonce,
    required String signatureHex,
    required String issuerSfidNumber,
    required String issuerMainAccountHex,
    required String signerPubkeyHex,
    required String scopeProvinceName,
    required String scopeCityName,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = buildProposeCreateInstitutionCallData(
      sfidNumber: sfidNumber,
      sfidFullName: sfidFullName,
      accounts: accounts,
      org: org,
      adminsLen: adminsLen,
      adminPubkeys: adminPubkeys,
      threshold: threshold,
      registerNonce: registerNonce,
      signatureHex: signatureHex,
      issuerSfidNumber: issuerSfidNumber,
      issuerMainAccountHex: issuerMainAccountHex,
      signerPubkeyHex: signerPubkeyHex,
      scopeProvinceName: scopeProvinceName,
      scopeCityName: scopeCityName,
    );
    final submitResult = await _signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
    final initialTotalFen = accounts.fold<BigInt>(
      BigInt.zero,
      (sum, item) => sum + item.amountFen,
    );
    final event = await _confirmInstitutionCreateProposedEvent(
      blockHashHex: submitResult.blockHashHex,
      sfidNumber: sfidNumber,
      sfidFullName: sfidFullName,
      accounts: accounts,
      org: org,
      adminsLen: adminsLen,
      adminPubkeys: adminPubkeys,
      threshold: threshold,
      initialTotalFen: initialTotalFen,
      proposerPubkey: signerPubkey,
    );
    return (
      txHash: submitResult.txHash,
      usedNonce: submitResult.usedNonce,
      proposalId: event.proposalId,
      mainAccountHex: event.mainAccountHex,
      blockHashHex: submitResult.blockHashHex,
    );
  }

  /// 构造机构创建 call_data。仅用于生产提交与测试逐字节对齐。
  @visibleForTesting
  static Uint8List buildProposeCreateInstitutionCallData({
    required String sfidNumber,
    required String sfidFullName,
    required List<InstitutionInitialAccountInput> accounts,
    required int org,
    required int adminsLen,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required String registerNonce,
    required String signatureHex,
    required String issuerSfidNumber,
    required String issuerMainAccountHex,
    required String signerPubkeyHex,
    required String scopeProvinceName,
    required String scopeCityName,
  }) {
    final sfidBytes = Uint8List.fromList(utf8.encode(sfidNumber.trim()));
    final institutionNameBytes =
        Uint8List.fromList(utf8.encode(sfidFullName.trim()));
    final registerNonceBytes =
        Uint8List.fromList(utf8.encode(registerNonce.trim()));
    final issuerSfidNumberBytes =
        Uint8List.fromList(utf8.encode(issuerSfidNumber.trim()));
    final scopeProvinceNameBytes =
        Uint8List.fromList(utf8.encode(scopeProvinceName.trim()));
    final scopeCityNameBytes =
        Uint8List.fromList(utf8.encode(scopeCityName.trim()));
    final signatureBytes = _hexDecodeFixed(signatureHex,
        expectedLength: 64, fieldName: 'signature');
    final issuerMainAccount = _hexDecodeFixed(
      issuerMainAccountHex,
      expectedLength: 32,
      fieldName: 'issuer_main_account',
    );
    final signerPubkey = _hexDecodeFixed(
      signerPubkeyHex,
      expectedLength: 32,
      fieldName: 'signer_pubkey',
    );

    if (sfidBytes.isEmpty || sfidBytes.length > 96) {
      throw ArgumentError('sfid_number 长度需在 1..=96 字节');
    }
    if (institutionNameBytes.isEmpty || institutionNameBytes.length > 128) {
      throw ArgumentError('sfid_full_name 长度需在 1..=128 字节');
    }
    if (accounts.isEmpty) {
      throw ArgumentError('accounts 不可为空');
    }
    if (org != 4 && org != 5) {
      throw ArgumentError('机构账户管理员 org 必须为 ORG_PUP 或 ORG_OTH');
    }
    if (adminsLen < 2 || adminsLen != adminPubkeys.length) {
      throw ArgumentError('admins_len 必须 >=2 且等于管理员公钥数量');
    }
    final minThreshold = (adminsLen ~/ 2) + 1;
    if (threshold < minThreshold || threshold > adminsLen) {
      throw ArgumentError('threshold 范围必须在 $minThreshold..=$adminsLen');
    }
    if (registerNonceBytes.isEmpty) {
      throw ArgumentError('register_nonce 不可为空');
    }
    if (issuerSfidNumberBytes.isEmpty) {
      throw ArgumentError('issuer_sfid_number 不可为空');
    }
    if (scopeProvinceNameBytes.isEmpty) {
      throw ArgumentError('scope_province_name 不可为空');
    }

    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCreateInstitutionCallIndex);

    // sfid_number: BoundedVec<u8> = Compact<u32> length + bytes
    _writeBoundedBytes(output, sfidBytes);

    // sfid_full_name: BoundedVec<u8>
    _writeBoundedBytes(output, institutionNameBytes);

    // accounts: BoundedVec<InstitutionInitialAccount> = Compact<N> + N 项。
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(accounts.length)));
    for (final account in accounts) {
      final accountNameBytes =
          Uint8List.fromList(utf8.encode(account.accountName.trim()));
      if (accountNameBytes.isEmpty || accountNameBytes.length > 128) {
        throw ArgumentError('account_name 长度需在 1..=128 字节');
      }
      if (account.amountFen <= BigInt.zero) {
        throw ArgumentError('account.amount_fen 必须大于 0');
      }
      _writeBoundedBytes(output, accountNameBytes);
      output.write(_u128ToLeBytesStatic(account.amountFen));
    }

    // org: u8。机构账户只能使用 ORG_PUP(4) 或 ORG_OTH(5)。
    output.pushByte(org);

    // admins_len: u32 little-endian
    output.write(_u32ToLeBytesStatic(adminsLen));

    // admins: BoundedVec<AccountId32> = Compact<u32> length + N × 32 bytes
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(adminPubkeys.length)));
    for (final pubkey in adminPubkeys) {
      if (pubkey.length != 32) {
        throw ArgumentError('admins 每项必须为 32 字节');
      }
      output.write(pubkey);
    }

    // threshold: u32 little-endian
    output.write(_u32ToLeBytesStatic(threshold));

    // register_nonce / signature / issuer_sfid_number / issuer_main_account / signer_pubkey / scope_*
    _writeBoundedBytes(output, registerNonceBytes);
    _writeBoundedBytes(output, signatureBytes);
    _writeBoundedBytes(output, issuerSfidNumberBytes);
    output.write(issuerMainAccount);
    output.write(signerPubkey);
    _writeBoundedBytes(output, scopeProvinceNameBytes);
    _writeBoundedBytes(output, scopeCityNameBytes);

    return output.toBytes();
  }

  /// 提交机构多签关闭提案。
  ///
  /// 当前链端机构关闭 call 为 OrganizationManage::propose_close。
  Future<({String txHash, int usedNonce})> submitProposeCloseInstitution({
    required String duoqianAccount,
    required String beneficiaryAddress,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    return _submitProposeClose(
      duoqianAccount: duoqianAccount,
      beneficiaryAddress: beneficiaryAddress,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 提交 propose_close extrinsic。
  ///
  /// 参数编码：[0x11][0x01] + duoqian_account(32B) + beneficiary(32B)
  Future<({String txHash, int usedNonce})> _submitProposeClose({
    required String duoqianAccount,
    required String beneficiaryAddress,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCloseCallIndex);

    // duoqian_account: AccountId32 = 32 bytes
    output.write(_hexDecode(duoqianAccount));

    // beneficiary: AccountId32 = 32 bytes
    final beneficiaryId = Keyring().decodeAddress(beneficiaryAddress);
    output.write(beneficiaryId);

    return _signAndSubmit(
      callData: output.toBytes(),
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  // 投票动作已迁移到 `InternalVoteService`（Phase 3, pallet=9 call=0）。

  // ──── 链上查询 ────

  /// 查询 SFID (sfid_number + account_name) 是否已注册，返回派生的多签账户 hex（null 表示未注册）。
  Future<String?> fetchSfidRegisteredAccount(
      Uint8List sfidNumber, Uint8List accountName) async {
    final key = _buildDoubleMapStorageKey(
      'OrganizationManage',
      'SfidRegisteredAccount',
      sfidNumber,
      accountName,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 32) return null;
    return _hexEncode(Uint8List.fromList(data.sublist(0, 32)));
  }

  /// 批量反查多个机构账户的 SFID 归属(`AccountRegisteredSfid` 精确整键)。
  ///
  /// 返回以入参账户原样为键的 map;未注册或解码失败的账户值为 null。
  /// 机构多签发现的唯一反查入口(ADR-018 R2:多 key 一律批量,杜绝循环内逐条)。
  Future<Map<String, RegisteredInstitutionRef?>>
      fetchRegisteredInstitutionRefsBatch(
    Iterable<String> duoqianAccountHexList, {
    int chunkSize = 100,
  }) async {
    final addresses = duoqianAccountHexList
        .where((address) => address.isNotEmpty)
        .toSet()
        .toList(growable: false);
    if (addresses.isEmpty) return {};

    final storageKeyByAccount = <String, String>{
      for (final address in addresses)
        address:
            '0x${_hexEncode(DuoqianStorageCodec.accountRegisteredSfidKey(address))}',
    };

    final values = await _rpc.fetchStorageBatchChunked(
      storageKeyByAccount.values.toSet(),
      chunkSize: chunkSize,
    );

    final result = <String, RegisteredInstitutionRef?>{};
    for (final entry in storageKeyByAccount.entries) {
      final data = values[entry.value];
      result[entry.key] = data == null
          ? null
          : DuoqianStorageCodec.decodeRegisteredInstitution(data);
    }
    return result;
  }

  /// 查询机构多签账户信息。
  ///
  /// 注册机构账户走 `AccountRegisteredSfid -> InstitutionAccounts`
  /// + `AdminsChange::AdminAccounts`。
  Future<InstitutionAccountInfo?> fetchDuoqianAccount(
      String duoqianAccountHex) async {
    return _fetchInstitutionDuoqianAccount(duoqianAccountHex);
  }

  /// 批量查询机构多签账户状态。
  ///
  /// 中文注释：机构多签需要先从账户反查 SFID 与账户名，再读取账户主体和
  /// 管理员主体，所以必须分阶段批量读取，不能简单逐个调用详情查询。
  Future<Map<String, InstitutionAccountInfo?>> fetchDuoqianAccountsBatch(
    Iterable<String> duoqianAccountHexList, {
    int chunkSize = 100,
  }) async {
    final addresses = duoqianAccountHexList
        .map(_normalizeHex)
        .where((address) => address.isNotEmpty)
        .toSet()
        .toList(growable: false);
    if (addresses.isEmpty) return {};

    final result = <String, InstitutionAccountInfo?>{};
    final refKeyByAccount = <String, String>{};
    for (final address in addresses) {
      refKeyByAccount[address] =
          '0x${_hexEncode(DuoqianStorageCodec.accountRegisteredSfidKey(address))}';
    }

    final refValues = await _rpc.fetchStorageBatchChunked(
      refKeyByAccount.values,
      chunkSize: chunkSize,
    );
    final refByAddress = <String, RegisteredInstitutionRef>{};
    for (final address in addresses) {
      final refData = refValues[refKeyByAccount[address]];
      if (refData == null) {
        result[address] = null;
        continue;
      }
      final ref = DuoqianStorageCodec.decodeRegisteredInstitution(refData);
      if (ref == null) {
        result[address] = null;
        continue;
      }
      refByAddress[address] = ref;
    }

    final accountKeyByAccount = <String, String>{};
    final adminKeyByAccount = <String, String>{};
    final accountIdByAccount = <String, Uint8List>{};
    final secondRoundKeys = <String>[];
    for (final entry in refByAddress.entries) {
      final accountId = DuoqianStorageCodec.accountIdFromAccountHex(
        entry.key,
      );
      final accountKey =
          '0x${_hexEncode(DuoqianStorageCodec.institutionAccountKey(
        entry.value.sfidNumber,
        entry.value.accountName,
      ))}';
      final adminKey =
          '0x${_hexEncode(DuoqianStorageCodec.adminAccountKey(accountId))}';
      accountIdByAccount[entry.key] = accountId;
      accountKeyByAccount[entry.key] = accountKey;
      adminKeyByAccount[entry.key] = adminKey;
      secondRoundKeys
        ..add(accountKey)
        ..add(adminKey);
    }

    final secondRoundValues = await _rpc.fetchStorageBatchChunked(
      secondRoundKeys,
      chunkSize: chunkSize,
    );
    final accountByAccount = <String, InstitutionAccountSnapshot>{};
    final adminByAccount = <String, AdminSnapshot>{};
    for (final address in refByAddress.keys) {
      final accountData = secondRoundValues[accountKeyByAccount[address]];
      final adminData = secondRoundValues[adminKeyByAccount[address]];
      if (accountData == null || adminData == null) {
        result[address] = null;
        continue;
      }
      final account = DuoqianStorageCodec.decodeInstitutionAccount(accountData);
      final admin = DuoqianStorageCodec.decodeAdminAccount(adminData);
      if (account == null || admin == null) {
        result[address] = null;
        continue;
      }
      accountByAccount[address] = account;
      adminByAccount[address] = admin;
    }

    final activeThresholdKeyByAccount = <String, String>{};
    for (final entry in adminByAccount.entries) {
      activeThresholdKeyByAccount[entry.key] =
          '0x${_hexEncode(DuoqianStorageCodec.dynamicThresholdKey(
        storageName: 'ActiveDynamicThresholds',
        org: entry.value.org,
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
      final threshold = DuoqianStorageCodec.decodeDynamicThreshold(
        activeThresholdValues[entry.value],
      );
      thresholdByAccount[entry.key] = threshold;
      if (threshold == null) {
        final admin = adminByAccount[entry.key]!;
        pendingThresholdKeyByAccount[entry.key] =
            '0x${_hexEncode(DuoqianStorageCodec.dynamicThresholdKey(
          storageName: 'PendingDynamicThresholds',
          org: admin.org,
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
            DuoqianStorageCodec.decodeDynamicThreshold(
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
        adminPubkeys: admin.adminPubkeys,
        status: _statusFromByte(account.statusByte),
      );
    }

    return result;
  }

  Future<InstitutionAccountInfo?> _fetchInstitutionDuoqianAccount(
    String duoqianAccountHex,
  ) async {
    final refKey = DuoqianStorageCodec.accountRegisteredSfidKey(
      duoqianAccountHex,
    );
    final refData = await _rpc.fetchStorage('0x${_hexEncode(refKey)}');
    if (refData == null) return null;
    final ref = DuoqianStorageCodec.decodeRegisteredInstitution(refData);
    if (ref == null) return null;

    final accountKey = DuoqianStorageCodec.institutionAccountKey(
      ref.sfidNumber,
      ref.accountName,
    );
    final accountData = await _rpc.fetchStorage('0x${_hexEncode(accountKey)}');
    if (accountData == null) return null;
    final account = DuoqianStorageCodec.decodeInstitutionAccount(accountData);
    if (account == null) return null;
    final accountId = DuoqianStorageCodec.accountIdFromAccountHex(
      duoqianAccountHex,
    );
    final adminKey = DuoqianStorageCodec.adminAccountKey(accountId);
    final adminData = await _rpc.fetchStorage('0x${_hexEncode(adminKey)}');
    if (adminData == null) return null;
    final admin = DuoqianStorageCodec.decodeAdminAccount(adminData);
    if (admin == null) return null;
    final threshold = await _fetchInstitutionDynamicThreshold(
      org: admin.org,
      accountId: accountId,
    );
    return InstitutionAccountInfo(
      adminsLen: admin.adminsLen,
      threshold: threshold,
      adminPubkeys: admin.adminPubkeys,
      status: _statusFromByte(account.statusByte),
    );
  }

  Future<int?> _fetchInstitutionDynamicThreshold({
    required int org,
    required Uint8List accountId,
  }) async {
    for (final storageName in const [
      'ActiveDynamicThresholds',
      'PendingDynamicThresholds',
    ]) {
      final key = DuoqianStorageCodec.dynamicThresholdKey(
        storageName: storageName,
        org: org,
        accountId: accountId,
      );
      final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
      final threshold = DuoqianStorageCodec.decodeDynamicThreshold(data);
      if (threshold != null) return threshold;
    }
    return null;
  }

  /// 从 ProposalData 解码机构多签管理提案。
  ///
  /// ProposalData 存储为 BoundedVec<u8>，SCALE：Compact<len> + [ACTION_TYPE(1B)] + action.encode()
  /// OrganizationManage ACTION_CLOSE(2): duoqian_account(32B) + beneficiary(32B) + proposer(32B)
  ///
  /// 返回 CloseDuoqianProposalInfo，解码失败返回 null。
  /// PersonalManage 提案解码已经迁移到 `PersonalManageService`。
  static const _orgModuleTag = [
    0x6f,
    0x72,
    0x67,
    0x2d,
    0x6d,
    0x67,
    0x6d,
    0x74
  ]; // "org-mgmt"

  Object? decodeManageProposalData(int proposalId, Uint8List raw) {
    try {
      var offset = 0;

      // BoundedVec<u8> 外层：Compact<len> + bytes
      final (vecLen, lenBytes) = _decodeCompact(raw, offset);
      offset += lenBytes;
      if (offset + vecLen > raw.length) return null;
      final data = raw.sublist(offset, offset + vecLen);

      if (!_startsWith(data, _orgModuleTag)) return null;
      final actionType = data[_orgModuleTag.length];
      final payload = data.sublist(_orgModuleTag.length + 1);
      if (actionType == actionClose) {
        return _decodeCloseAction(proposalId, payload);
      }
      return null;
    } catch (_) {
      return null;
    }
  }

  CloseDuoqianProposalInfo? _decodeCloseAction(int proposalId, Uint8List data) {
    // duoqian_account(32) + beneficiary(32) + proposer(32)
    if (data.length != 32 + 32 + 32) return null;
    var offset = 0;

    final duoqianAccount =
        _hexEncode(Uint8List.fromList(data.sublist(offset, offset + 32)));
    offset += 32;

    final beneficiaryBytes = data.sublist(offset, offset + 32);
    final beneficiarySs58 =
        Keyring().encodeAddress(Uint8List.fromList(beneficiaryBytes), 2027);
    offset += 32;

    final proposerBytes = data.sublist(offset, offset + 32);
    final proposerSs58 =
        Keyring().encodeAddress(Uint8List.fromList(proposerBytes), 2027);

    return CloseDuoqianProposalInfo(
      proposalId: proposalId,
      duoqianAccount: duoqianAccount,
      beneficiary: beneficiarySs58,
      proposer: proposerSs58,
    );
  }

  Future<({int proposalId, String mainAccountHex})>
      _confirmInstitutionCreateProposedEvent({
    required String blockHashHex,
    required String sfidNumber,
    required String sfidFullName,
    required List<InstitutionInitialAccountInput> accounts,
    required int org,
    required int adminsLen,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required BigInt initialTotalFen,
    required Uint8List proposerPubkey,
  }) async {
    final events = await _rpc.fetchSystemEventsAtBlock(blockHashHex);
    if (events == null || events.isEmpty) {
      throw StateError('交易已入块，但未读取到 System.Events，不能确认机构多签创建提案');
    }
    final failure = _rpc.findExtrinsicFailureInEvents(events);
    if (failure != null) {
      throw StateError(failure.description);
    }
    final found = _findInstitutionCreateProposedEvent(
      events,
      sfidNumber: sfidNumber,
      sfidFullName: sfidFullName,
      accounts: accounts,
      org: org,
      adminsLen: adminsLen,
      adminPubkeys: adminPubkeys,
      threshold: threshold,
      initialTotalFen: initialTotalFen,
      proposerPubkey: proposerPubkey,
    );
    if (found == null) {
      throw StateError(
        '交易已入块，但未确认 OrganizationManage.InstitutionCreateProposed，也未检测到链上失败事件，请检查当前区块事件',
      );
    }
    return found;
  }

  ({int proposalId, String mainAccountHex})?
      _findInstitutionCreateProposedEvent(
    Uint8List data, {
    required String sfidNumber,
    required String sfidFullName,
    required List<InstitutionInitialAccountInput> accounts,
    required int org,
    required int adminsLen,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required BigInt initialTotalFen,
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
            eventIndex == _institutionCreateProposedEventIndex) {
          final decoded = _decodeInstitutionCreateProposedEvent(
            data,
            offset,
            sfidNumber: sfidNumber,
            sfidFullName: sfidFullName,
            accounts: accounts,
            org: org,
            adminsLen: adminsLen,
            adminPubkeys: adminPubkeys,
            threshold: threshold,
            initialTotalFen: initialTotalFen,
            proposerPubkey: proposerPubkey,
          );
          if (decoded != null) return decoded;
        }
      } catch (_) {
        // 中文注释：System.Events 逐字节扫描时会遇到其他事件，失败继续向后找。
      }
    }
    return null;
  }

  ({int proposalId, String mainAccountHex})?
      _decodeInstitutionCreateProposedEvent(
    Uint8List data,
    int offset, {
    required String sfidNumber,
    required String sfidFullName,
    required List<InstitutionInitialAccountInput> accounts,
    required int org,
    required int adminsLen,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required BigInt initialTotalFen,
    required Uint8List proposerPubkey,
  }) {
    try {
      var pos = offset;
      if (pos + 8 > data.length) return null;
      final proposalId = _readU64Le(data, pos);
      pos += 8;

      final sfidRead = _readCompactBytes(data, pos);
      if (sfidRead == null) return null;
      pos = sfidRead.nextOffset;
      final nameRead = _readCompactBytes(data, pos);
      if (nameRead == null) return null;
      pos = nameRead.nextOffset;
      if (pos + 32 + 32 > data.length) return null;
      final mainAccount = Uint8List.fromList(data.sublist(pos, pos + 32));
      pos += 32;
      final proposer = Uint8List.fromList(data.sublist(pos, pos + 32));
      pos += 32;

      final (accountsLen, accountsLenBytes) = _decodeCompact(data, pos);
      pos += accountsLenBytes;
      if (accountsLen != accounts.length) return null;
      for (var i = 0; i < accountsLen; i++) {
        final accountNameRead = _readCompactBytes(data, pos);
        if (accountNameRead == null) return null;
        pos = accountNameRead.nextOffset;
        if (pos + 32 + 16 + 1 > data.length) return null;
        pos += 32; // account
        final eventAmount = _readU128Le(data.sublist(pos, pos + 16));
        pos += 16;
        pos += 1; // is_default
        final expectedName =
            Uint8List.fromList(utf8.encode(accounts[i].accountName.trim()));
        if (!_bytesEqual(accountNameRead.bytes, expectedName) ||
            eventAmount != accounts[i].amountFen) {
          return null;
        }
      }

      final (adminsLen, adminsLenBytes) = _decodeCompact(data, pos);
      pos += adminsLenBytes;
      if (adminsLen < 0 || pos + adminsLen * 32 > data.length) return null;
      final eventAdmins = <Uint8List>[];
      for (var i = 0; i < adminsLen; i++) {
        eventAdmins.add(Uint8List.fromList(data.sublist(pos, pos + 32)));
        pos += 32;
      }
      if (pos + 1 + 4 + 4 + 16 + 16 > data.length) return null;
      final eventOrg = data[pos];
      pos += 1;
      final eventAdminsLen = _readU32Le(data, pos);
      pos += 4;
      final eventThreshold = _readU32Le(data, pos);
      pos += 4;
      final eventInitialTotal = _readU128Le(data.sublist(pos, pos + 16));

      final expectedSfid = Uint8List.fromList(utf8.encode(sfidNumber.trim()));
      final expectedName = Uint8List.fromList(utf8.encode(sfidFullName.trim()));
      final matches = _bytesEqual(sfidRead.bytes, expectedSfid) &&
          _bytesEqual(nameRead.bytes, expectedName) &&
          _bytesEqual(proposer, proposerPubkey) &&
          eventOrg == org &&
          eventAdminsLen == adminsLen &&
          eventThreshold == threshold &&
          eventInitialTotal == initialTotalFen &&
          _adminListsEqual(eventAdmins, adminPubkeys);
      if (!matches) return null;
      return (
        proposalId: proposalId,
        mainAccountHex: _hexEncode(mainAccount),
      );
    } catch (_) {
      return null;
    }
  }

  // ──── 内部：签名提交 ────

  Future<({String txHash, int usedNonce})> _signAndSubmit({
    required Uint8List callData,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    return SignedExtrinsicBuilder(
      chainRpc: _rpc,
      logLabel: 'OrganizationManage',
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
      logLabel: 'OrganizationManage',
    ).signAndSubmitInBlock(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  // ──── 内部：storage key 构造 ────

  /// StorageDoubleMap key: twox128(pallet) + twox128(storage) + blake2_128_concat(key1) + blake2_128_concat(key2)
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

  // ──── 内部：编码工具 ────

  InstitutionStatus _statusFromByte(int statusByte) {
    return statusByte == 1
        ? InstitutionStatus.active
        : InstitutionStatus.pending;
  }

  static void _writeBoundedBytes(ByteOutput output, Uint8List bytes) {
    output.write(CompactBigIntCodec.codec.encode(BigInt.from(bytes.length)));
    output.write(bytes);
  }

  static Uint8List _u32ToLeBytesStatic(int value) {
    final bytes = Uint8List(4);
    final bd = ByteData.sublistView(bytes);
    bd.setUint32(0, value, Endian.little);
    return bytes;
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

  static Uint8List _hexDecodeFixed(
    String hex, {
    required int expectedLength,
    required String fieldName,
  }) {
    final raw = hex.trim();
    final h = raw.startsWith('0x') ? raw.substring(2) : raw;
    if (h.length != expectedLength * 2 ||
        !RegExp(r'^[0-9a-fA-F]+$').hasMatch(h)) {
      throw ArgumentError('$fieldName 必须为 $expectedLength 字节 hex');
    }
    final result = Uint8List(expectedLength);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
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

  int _readU64Le(Uint8List data, int offset) {
    var value = 0;
    for (var i = 7; i >= 0; i--) {
      value = (value << 8) | data[offset + i];
    }
    return value;
  }

  int _readU32Le(Uint8List data, int offset) {
    return data[offset] |
        (data[offset + 1] << 8) |
        (data[offset + 2] << 16) |
        (data[offset + 3] << 24);
  }

  BigInt _readU128Le(Uint8List bytes) {
    var value = BigInt.zero;
    for (var i = bytes.length - 1; i >= 0; i--) {
      value = (value << 8) | BigInt.from(bytes[i]);
    }
    return value;
  }

  ({Uint8List bytes, int nextOffset})? _readCompactBytes(
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

  bool _adminListsEqual(
    List<Uint8List> left,
    List<Uint8List> right,
  ) {
    if (left.length != right.length) return false;
    for (var i = 0; i < left.length; i++) {
      if (!_bytesEqual(left[i], right[i])) return false;
    }
    return true;
  }

  bool _bytesEqual(Uint8List left, Uint8List right) {
    if (left.length != right.length) return false;
    for (var i = 0; i < left.length; i++) {
      if (left[i] != right[i]) return false;
    }
    return true;
  }

  static String _hexEncode(Uint8List bytes) {
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
