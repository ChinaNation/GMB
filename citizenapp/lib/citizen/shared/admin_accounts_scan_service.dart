// 分类管理员模块 `AdminAccounts` 链上扫描。
//
// 管理员身份和个人多签发现都依赖链上 `AdminAccounts` 反向索引。本服务把
// "翻页 getKeysPaged + 批量 fetchStorageBatch + 解码 + 提取账户"收敛为一次扫描,
// 产出已解码条目。调用方必须显式选择要扫描的分类管理员 pallet，个人多签只扫
// `PersonalAdmins`，钱包管理员标签则扫描公权、私权和个人三类。
//
// 扫描走轻节点 smoldot 的**短前缀整表**(prefix = twox128(pallet) || twox128(storage),
// 无嵌长 K1)。

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/citizen/shared/admin_account_storage_codec.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';

/// 单条已解码的 AdminAccount 记录(地址 + 过滤所需字段)。
@immutable
class ScannedAdminAccount {
  const ScannedAdminAccount({
    required this.addrHex,
    required this.institutionCode,
    required this.kind,
    required this.adminsHex,
  });

  /// 账户小写 hex(无 0x),由 storage key 末 32B 提取。
  final String addrHex;

  /// 4 字节机构码字符串（"NRC"/"PRC"/"PRB"/"PMUL"/"CGOV" 等）。
  final String institutionCode;

  /// 管理员账户类型(0=Public,1=Private,2=Personal)。
  final int kind;

  /// 管理员公钥小写 hex 列表(无 0x,32B = 64 hex)。
  final List<String> adminsHex;
}

/// 一次全表扫描的结果。
@immutable
class AdminAccountsScanResult {
  const AdminAccountsScanResult({
    required this.accounts,
    required this.totalKeys,
    required this.partialFailure,
  });

  /// 全部成功解码的条目(未过滤)。
  final List<ScannedAdminAccount> accounts;

  /// 扫描到的 storage key 总数(含未能解码者),用于统计。
  final int totalKeys;

  /// 翻页或批量读取过程中出现过失败(结果可能不完整)。
  final bool partialFailure;

  static const empty = AdminAccountsScanResult(
    accounts: <ScannedAdminAccount>[],
    totalKeys: 0,
    partialFailure: false,
  );
}

/// 分类管理员 `AdminAccounts` 单次扫描服务。
class AdminAccountsScanService {
  AdminAccountsScanService({
    ChainRpc? chainRpc,
    this.palletNames = const ['PersonalAdmins'],
  }) : _rpc = chainRpc ?? ChainRpc() {
    if (palletNames.isEmpty) {
      throw ArgumentError.value(palletNames, 'palletNames', '不能为空');
    }
  }

  final ChainRpc _rpc;

  /// 本次扫描的分类管理员 pallet；只允许当前链上三张管理员表。
  final List<String> palletNames;

  static const Set<String> _allowedPalletNames = {
    'PublicAdmins',
    'PrivateAdmins',
    'PersonalAdmins',
  };

  /// 翻页大小。
  static const _pageSize = 256;

  /// 单批 storage 读取上限(防 RPC 超时)。
  static const _batchSize = 100;

  /// 全表扫描 AdminAccounts,返回全部已解码条目。
  ///
  /// [onProgress] 进度回调:(已扫描 key 数, 已知总数或 null, 已解码条目数)。
  Future<AdminAccountsScanResult> scanAll({
    void Function(int scanned, int? total, int decoded)? onProgress,
  }) async {
    final allKeys = <String>[];
    var partialFailure = false;

    for (final prefixHex in _adminAccountsPrefixHexList()) {
      String? startKey;
      while (true) {
        List<String> page;
        try {
          page = await SmoldotClientManager.instance.getKeysPagedFinalized(
            prefixHex,
            count: _pageSize,
            startKey: startKey,
          );
        } catch (e) {
          debugPrint('[AdminAccountsScan] getKeysPaged 失败: $e');
          partialFailure = true;
          break;
        }
        if (page.isEmpty) break;
        allKeys.addAll(page);
        onProgress?.call(allKeys.length, null, 0);
        if (page.length < _pageSize) break;
        startKey = page.last;
      }
    }

    final accounts = <ScannedAdminAccount>[];
    for (var start = 0; start < allKeys.length; start += _batchSize) {
      final end = (start + _batchSize).clamp(0, allKeys.length);
      final batchKeys = allKeys.sublist(start, end);

      Map<String, Uint8List?> values;
      try {
        values = await _rpc.fetchStorageBatch(batchKeys);
      } catch (e) {
        debugPrint('[AdminAccountsScan] fetchStorageBatch 失败: $e');
        partialFailure = true;
        continue;
      }

      for (final keyHex in batchKeys) {
        final value = values[keyHex];
        if (value == null) continue;
        final decoded = AdminAccountStorageCodec.tryDecode(value);
        if (decoded == null) continue;
        final accountId = AdminAccountStorageCodec.extractAccountIdFromKey(
          _hexDecode(keyHex),
        );
        if (accountId == null) continue;
        final addr =
            AdminAccountStorageCodec.accountHexFromAccountId(accountId);
        if (addr == null) continue;
        accounts.add(ScannedAdminAccount(
          addrHex: addr,
          institutionCode: decoded.institutionCode,
          kind: decoded.kind,
          adminsHex: decoded.adminsHex,
        ));
      }
      onProgress?.call(allKeys.length, allKeys.length, accounts.length);
    }

    return AdminAccountsScanResult(
      accounts: accounts,
      totalKeys: allKeys.length,
      partialFailure: partialFailure,
    );
  }

  /// 纯函数:从扫描结果里筛出"我的"账户(指定 kind,可选机构码白名单,
  /// 且管理员集合含本地任一钱包公钥)。供个人多签发现复用,便于单测。
  static List<ScannedAdminAccount> filterMine(
    AdminAccountsScanResult scan, {
    required Set<String> myPubkeysHex,
    int? kind,
    Set<int>? kinds,
    Set<String>? codeWhitelist,
  }) {
    final acceptedKinds = kinds ?? (kind == null ? const <int>{} : {kind});
    return scan.accounts
        .where(
          (a) =>
              acceptedKinds.contains(a.kind) &&
              (codeWhitelist == null ||
                  codeWhitelist.contains(a.institutionCode)) &&
              a.adminsHex.any(myPubkeysHex.contains),
        )
        .toList(growable: false);
  }

  /// `PersonalAdmins.AdminAccounts` 双 prefix(twox128 || twox128)的 hex 形式。
  List<String> _adminAccountsPrefixHexList() {
    final invalid =
        palletNames.where((name) => !_allowedPalletNames.contains(name));
    if (invalid.isNotEmpty) {
      throw ArgumentError.value(
        invalid.join(','),
        'palletNames',
        '包含未知分类管理员 pallet',
      );
    }
    return palletNames
        .toSet()
        .map(_adminAccountsPrefixHex)
        .toList(growable: false);
  }

  String _adminAccountsPrefixHex(String palletName) {
    final palletHash = Hasher.twoxx128.hashString(palletName);
    final storageHash = Hasher.twoxx128.hashString('AdminAccounts');
    final prefix = Uint8List(palletHash.length + storageHash.length);
    prefix.setAll(0, palletHash);
    prefix.setAll(palletHash.length, storageHash);
    return '0x${_hexEncode(prefix)}';
  }

  static Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final bytes = Uint8List(h.length ~/ 2);
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return bytes;
  }

  static String _hexEncode(Uint8List bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
}
