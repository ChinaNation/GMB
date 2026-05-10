import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:polkadart/scale_codec.dart' show CompactBigIntCodec, ByteOutput;
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/signed_extrinsic_builder.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';

import 'duoqian_manage_models.dart';
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
class DuoqianManageService {
  DuoqianManageService({ChainRpc? chainRpc}) : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

  // ──── 常量 ────

  /// OrganizationManage pallet index（runtime pallet_index=17,机构多签管理）。
  static const _palletIndex = 17;

  /// OrganizationManage::propose_create_institution call_index=5。
  static const _proposeCreateInstitutionCallIndex = 5;

  /// OrganizationManage::propose_close call_index=1(机构关闭)。
  static const _proposeCloseCallIndex = 1;

  /// ProposalData 中的 action 类型前缀。
  /// OrganizationManage(b"org-mgmt") 命名空间:ACTION_CLOSE=2。
  static const actionClose = 2;

  // ──── Extrinsic 提交 ────

  /// 提交机构多签 propose_create_institution extrinsic。
  ///
  /// 参数编码以 `memory/07-ai/unified-protocols.md` 的 P-TX-001 为准：
  /// [0x11][0x05] + sfid_number + institution_name + accounts + admin_count
  ///   + duoqian_admins + threshold + register_nonce + signature
  ///   + province + signer_admin_pubkey。
  Future<({String txHash, int usedNonce})> submitProposeCreateInstitution({
    required String sfidNumber,
    required String institutionName,
    required List<InstitutionInitialAccountInput> accounts,
    required int adminCount,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required String registerNonce,
    required String signatureHex,
    required String province,
    required String signerAdminPubkeyHex,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final callData = buildProposeCreateInstitutionCallData(
      sfidNumber: sfidNumber,
      institutionName: institutionName,
      accounts: accounts,
      adminCount: adminCount,
      adminPubkeys: adminPubkeys,
      threshold: threshold,
      registerNonce: registerNonce,
      signatureHex: signatureHex,
      province: province,
      signerAdminPubkeyHex: signerAdminPubkeyHex,
    );
    return _signAndSubmit(
      callData: callData,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 构造机构创建 call_data。仅用于生产提交与测试逐字节对齐。
  @visibleForTesting
  static Uint8List buildProposeCreateInstitutionCallData({
    required String sfidNumber,
    required String institutionName,
    required List<InstitutionInitialAccountInput> accounts,
    required int adminCount,
    required List<Uint8List> adminPubkeys,
    required int threshold,
    required String registerNonce,
    required String signatureHex,
    required String province,
    required String signerAdminPubkeyHex,
  }) {
    final sfidBytes = Uint8List.fromList(utf8.encode(sfidNumber.trim()));
    final institutionNameBytes =
        Uint8List.fromList(utf8.encode(institutionName.trim()));
    final registerNonceBytes =
        Uint8List.fromList(utf8.encode(registerNonce.trim()));
    final provinceBytes = Uint8List.fromList(utf8.encode(province.trim()));
    final signatureBytes = _hexDecodeFixed(signatureHex,
        expectedLength: 64, fieldName: 'signature');
    final signerAdminPubkey = _hexDecodeFixed(
      signerAdminPubkeyHex,
      expectedLength: 32,
      fieldName: 'signer_admin_pubkey',
    );

    if (sfidBytes.isEmpty || sfidBytes.length > 96) {
      throw ArgumentError('sfid_number 长度需在 1..=96 字节');
    }
    if (institutionNameBytes.isEmpty || institutionNameBytes.length > 128) {
      throw ArgumentError('institution_name 长度需在 1..=128 字节');
    }
    if (accounts.isEmpty) {
      throw ArgumentError('accounts 不可为空');
    }
    if (adminCount < 2 || adminCount != adminPubkeys.length) {
      throw ArgumentError('admin_count 必须 >=2 且等于管理员公钥数量');
    }
    final minThresholdRaw = (adminCount + 1) ~/ 2;
    final minThreshold = minThresholdRaw < 2 ? 2 : minThresholdRaw;
    if (threshold < minThreshold || threshold > adminCount) {
      throw ArgumentError('threshold 范围必须在 $minThreshold..=$adminCount');
    }
    if (registerNonceBytes.isEmpty) {
      throw ArgumentError('register_nonce 不可为空');
    }
    if (provinceBytes.isEmpty) {
      throw ArgumentError('province 不可为空');
    }

    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCreateInstitutionCallIndex);

    // sfid_number: BoundedVec<u8> = Compact<u32> length + bytes
    _writeBoundedBytes(output, sfidBytes);

    // institution_name: BoundedVec<u8>
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

    // admin_count: u32 little-endian
    output.write(_u32ToLeBytesStatic(adminCount));

    // duoqian_admins: BoundedVec<AccountId32> = Compact<u32> length + N × 32 bytes
    output.write(
        CompactBigIntCodec.codec.encode(BigInt.from(adminPubkeys.length)));
    for (final pubkey in adminPubkeys) {
      if (pubkey.length != 32) {
        throw ArgumentError('duoqian_admins 每项必须为 32 字节');
      }
      output.write(pubkey);
    }

    // threshold: u32 little-endian
    output.write(_u32ToLeBytesStatic(threshold));

    // register_nonce / signature / province / signer_admin_pubkey
    _writeBoundedBytes(output, registerNonceBytes);
    _writeBoundedBytes(output, signatureBytes);
    _writeBoundedBytes(output, provinceBytes);
    output.write(signerAdminPubkey);

    return output.toBytes();
  }

  /// 提交机构多签关闭提案。
  ///
  /// 当前链端机构关闭 call 为 OrganizationManage::propose_close。
  Future<({String txHash, int usedNonce})> submitProposeCloseInstitution({
    required String duoqianAddress,
    required String beneficiaryAddress,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    return _submitProposeClose(
      duoqianAddress: duoqianAddress,
      beneficiaryAddress: beneficiaryAddress,
      fromAddress: fromAddress,
      signerPubkey: signerPubkey,
      sign: sign,
    );
  }

  /// 提交 propose_close extrinsic。
  ///
  /// 参数编码：[0x11][0x01] + duoqian_address(32B) + beneficiary(32B)
  Future<({String txHash, int usedNonce})> _submitProposeClose({
    required String duoqianAddress,
    required String beneficiaryAddress,
    required String fromAddress,
    required Uint8List signerPubkey,
    required Future<Uint8List> Function(Uint8List payload) sign,
  }) async {
    final output = ByteOutput();
    output.pushByte(_palletIndex);
    output.pushByte(_proposeCloseCallIndex);

    // duoqian_address: AccountId32 = 32 bytes
    output.write(_hexDecode(duoqianAddress));

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

  /// 翻页查询某 SFID 机构下的全部 (account_name, duoqian_address)。
  ///
  /// 内部:`state_getKeysPaged` prefix = twox128("OrganizationManage")
  ///       || twox128("SfidRegisteredAddress")
  ///       || blake2_128_concat(sfid_number);每个 key 后段:
  ///       blake2_128(account_name)(16B) || account_name 真值(变长 BoundedVec)。
  ///       value = duoqian_address(32B)。
  Future<List<({String accountName, String duoqianAddressHex})>>
      listSfidAccounts(Uint8List sfidNumber) async {
    final palletHash = Hasher.twoxx128.hashString('OrganizationManage');
    final storageHash = Hasher.twoxx128.hashString('SfidRegisteredAddress');
    final sfidKeyHash = _blake2128Concat(sfidNumber);
    final prefix = Uint8List(
      palletHash.length + storageHash.length + sfidKeyHash.length,
    );
    var offset = 0;
    prefix.setAll(offset, palletHash);
    offset += palletHash.length;
    prefix.setAll(offset, storageHash);
    offset += storageHash.length;
    prefix.setAll(offset, sfidKeyHash);

    final results = <({String accountName, String duoqianAddressHex})>[];
    String? startKey;
    const pageSize = 256;
    final prefixHex = '0x${_hexEncode(prefix)}';
    final prefixHexLen = prefixHex.length;

    while (true) {
      final keysRaw =
          await SmoldotClientManager.instance.request('state_getKeysPaged', [
        prefixHex,
        pageSize,
        startKey,
        null,
      ]);
      if (keysRaw is! List || keysRaw.isEmpty) break;
      final keys = keysRaw.cast<String>();

      for (final keyHex in keys) {
        // 双 key:prefix + blake2_128(name)(16B 32 hex) + name 真值(变长)
        // 截掉 prefix(64 hex)和 name 哈希(32 hex)即可得 name 真字节
        if (keyHex.length < prefixHexLen + 32) continue;
        final nameHex = keyHex.substring(prefixHexLen + 32);
        final nameBytes = _hexDecode(nameHex);
        final accountName = _utf8Decode(Uint8List.fromList(nameBytes));

        final value = await _rpc.fetchStorage(keyHex);
        if (value == null || value.length < 32) continue;
        final duoqianAddrHex =
            _hexEncode(Uint8List.fromList(value.sublist(0, 32)));
        results.add((
          accountName: accountName,
          duoqianAddressHex: duoqianAddrHex,
        ));
      }

      if (keys.length < pageSize) break;
      startKey = keys.last;
    }

    return results;
  }

  /// 查询 SFID (sfid_number + account_name) 是否已注册，返回派生的多签地址 hex（null 表示未注册）。
  Future<String?> fetchSfidRegisteredAddress(
      Uint8List sfidNumber, Uint8List accountName) async {
    final key = _buildDoubleMapStorageKey(
      'OrganizationManage',
      'SfidRegisteredAddress',
      sfidNumber,
      accountName,
    );
    final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
    if (data == null || data.length < 32) return null;
    return _hexEncode(Uint8List.fromList(data.sublist(0, 32)));
  }

  /// 通过机构账户地址反查其 SFID 归属和账户名。
  Future<RegisteredInstitutionRef?> fetchRegisteredInstitutionRef(
    String duoqianAddressHex,
  ) async {
    final refKey = DuoqianStorageCodec.addressRegisteredSfidKey(
      duoqianAddressHex,
    );
    final refData = await _rpc.fetchStorage('0x${_hexEncode(refKey)}');
    if (refData == null) return null;
    return DuoqianStorageCodec.decodeRegisteredInstitution(refData);
  }

  /// 查询机构多签账户信息。
  ///
  /// 注册机构账户走 `AddressRegisteredSfid -> InstitutionAccounts`
  /// + `AdminsChange::Subjects`。
  Future<DuoqianAccountInfo?> fetchDuoqianAccount(
      String duoqianAddressHex) async {
    return _fetchInstitutionDuoqianAccount(duoqianAddressHex);
  }

  Future<DuoqianAccountInfo?> _fetchInstitutionDuoqianAccount(
    String duoqianAddressHex,
  ) async {
    final refKey = DuoqianStorageCodec.addressRegisteredSfidKey(
      duoqianAddressHex,
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
    final subjectId = DuoqianStorageCodec.subjectIdFromInstitutionAccountHex(
      duoqianAddressHex,
    );
    final adminKey = DuoqianStorageCodec.adminSubjectKey(subjectId);
    final adminData = await _rpc.fetchStorage('0x${_hexEncode(adminKey)}');
    if (adminData == null) return null;
    final admin = DuoqianStorageCodec.decodeAdminSubject(adminData);
    if (admin == null) return null;
    return DuoqianAccountInfo(
      adminCount: admin.adminCount,
      threshold: admin.threshold,
      adminPubkeys: admin.adminPubkeys,
      status: _statusFromByte(account.statusByte),
    );
  }

  /// 从 ProposalData 解码机构多签管理提案。
  ///
  /// ProposalData 存储为 BoundedVec<u8>，SCALE：Compact<len> + [ACTION_TYPE(1B)] + action.encode()
  /// OrganizationManage ACTION_CLOSE(2): duoqian_address(32B) + beneficiary(32B) + proposer(32B)
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
    // duoqian_address(32) + beneficiary(32) + proposer(32)
    if (data.length != 32 + 32 + 32) return null;
    var offset = 0;

    final duoqianAddress =
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
      duoqianAddress: duoqianAddress,
      beneficiary: beneficiarySs58,
      proposer: proposerSs58,
    );
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

  DuoqianStatus _statusFromByte(int statusByte) {
    return statusByte == 1 ? DuoqianStatus.active : DuoqianStatus.pending;
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

  String _utf8Decode(Uint8List bytes) =>
      utf8.decode(bytes, allowMalformed: true);

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

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(h.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }
}
