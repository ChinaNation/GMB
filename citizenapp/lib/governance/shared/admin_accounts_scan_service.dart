// 共享:`AdminsChange::AdminAccounts` 全表单次扫描(机构多签 + 个人多签共用)。
//
// 背景(ADR-018 §九):机构多签与个人多签发现都依赖同一张
// `AdminsChange::AdminAccounts` 反向索引。历史上两个发现服务各自全表扫一遍,
// 同一张表扫两次纯属浪费。本服务把"翻页 getKeysPaged + 批量 fetchStorageBatch
// + 解码 + 提取账户地址"收敛为一次扫描,产出已解码条目;各业务模块按 kind/org
// 客户端过滤,不再各自扫链。
//
// 扫描走轻节点 smoldot 的**短前缀整表**(prefix = twox128(pallet) || twox128(storage),
// 无嵌长 K1),ADR-018 §一确认轻节点可用。

import 'package:flutter/foundation.dart';
import 'package:polkadart/polkadart.dart' show Hasher;
import 'package:citizenapp/governance/shared/admin_account_storage_codec.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';

/// 单条已解码的 AdminAccount 记录(地址 + 过滤所需字段)。
@immutable
class ScannedAdminAccount {
  const ScannedAdminAccount({
    required this.addrHex,
    required this.org,
    required this.kind,
    required this.adminPubkeysHex,
  });

  /// 账户地址小写 hex(无 0x),由 storage key 末 32B 提取。
  final String addrHex;

  /// 治理 org 标识(0=NRC,1=PRC,2=PRB,3=个人多签,4=PUP,5=OTH)。
  final int org;

  /// 管理员账户类型(0=Builtin,1=Personal,2=InstitutionAccount)。
  final int kind;

  /// 管理员公钥小写 hex 列表(无 0x,32B = 64 hex)。
  final List<String> adminPubkeysHex;
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

/// `AdminsChange::AdminAccounts` 单次全表扫描服务(机构/个人多签共用)。
class AdminAccountsScanService {
  AdminAccountsScanService({ChainRpc? chainRpc})
      : _rpc = chainRpc ?? ChainRpc();

  final ChainRpc _rpc;

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
    final prefixHex = _adminAccountsPrefixHex();
    final allKeys = <String>[];
    String? startKey;
    var partialFailure = false;

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
          org: decoded.org,
          kind: decoded.kind,
          adminPubkeysHex: decoded.adminPubkeysHex,
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

  /// 纯函数:从扫描结果里筛出"我的"账户(指定 kind,可选 org 白名单,
  /// 且管理员集合含本地任一钱包公钥)。供机构/个人多签模块复用,便于单测。
  static List<ScannedAdminAccount> filterMine(
    AdminAccountsScanResult scan, {
    required Set<String> myPubkeysHex,
    required int kind,
    Set<int>? orgWhitelist,
  }) {
    return scan.accounts
        .where((a) =>
            a.kind == kind &&
            (orgWhitelist == null || orgWhitelist.contains(a.org)) &&
            a.adminPubkeysHex.any(myPubkeysHex.contains))
        .toList(growable: false);
  }

  /// `AdminsChange::AdminAccounts` 双 prefix(twox128 || twox128)的 hex 形式。
  String _adminAccountsPrefixHex() {
    final palletHash = Hasher.twoxx128.hashString('AdminsChange');
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
