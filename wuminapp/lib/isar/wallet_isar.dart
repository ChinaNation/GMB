import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:isar/isar.dart';
import 'package:path_provider/path_provider.dart';

part 'wallet_isar.g.dart';

@collection
class WalletProfileEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late int walletIndex;

  late String walletName;
  late String walletIcon;
  late double balance;

  @Index(unique: true, replace: true)
  late String address;

  @Index(unique: true, replace: true)
  late String pubkeyHex;

  late String alg;
  late int ss58;
  late int createdAtMillis;
  late String source;

  /// 签名模式：`local`（热钱包）或 `external`（冷钱包）。
  late String signMode;

  /// 中文注释：用户拖拽排序后的稳定顺序。
  /// 数值越小越靠前；旧用户首次启动时通过 SharedPreferences flag 一次性
  /// 按 walletIndex 升序填充，保证升级无感（不丢原有顺序）。
  /// 排序时优先按 sortOrder 升序，相同则回退 walletIndex 兜底（保持稳定）。
  int sortOrder = 0;
}

@collection
class WalletSettingsEntity {
  Id id = 0;

  int? activeWalletIndex;
  int updatedAtMillis = 0;
}

@collection
class AdminRoleCacheEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String pubkeyHex;

  late String roleName;

  @Index()
  late int updatedAt;
}

@collection
class ObservedAccountEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String accountId;

  late String orgName;
  late String publicKey;
  late String address;
  double? balance;
  late String source;
}

@collection
class LoginReplayEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String requestId;

  late int expiresAt;
}

@collection
class AppKvEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String key;

  String? stringValue;
  int? intValue;
  bool? boolValue;
}

/// 多签账户本地状态快照。
///
/// 中文注释：`status` 负责 UI 展示，`lastSyncAtMillis` 负责判断是否需要
/// 再次查链；两者都复用 AppKvEntity，避免为 TTL 新增 Isar schema。
class DuoqianLocalStatusSnapshot {
  const DuoqianLocalStatusSnapshot({
    required this.status,
    required this.lastSyncAtMillis,
  });

  final String status;
  final int? lastSyncAtMillis;
}

/// 多签详情页本地持久化快照。
///
/// 中文注释：这不是短期内存缓存，而是详情页首屏可直接使用的本机状态。
/// 链上刷新成功后覆盖写入；链上失败时保留旧值，避免进详情页被 RPC 卡住。
class DuoqianLocalDetailSnapshot {
  const DuoqianLocalDetailSnapshot({
    required this.status,
    required this.adminPubkeys,
    this.threshold,
    this.balanceYuan,
    this.lastChainRefreshAtMillis,
    this.lastBalanceRefreshAtMillis,
    this.updatedAtMillis,
  });

  final String status;
  final List<String> adminPubkeys;
  final int? threshold;
  final double? balanceYuan;
  final int? lastChainRefreshAtMillis;
  final int? lastBalanceRefreshAtMillis;
  final int? updatedAtMillis;

  Map<String, dynamic> toJson() => {
        'status': status,
        'admin_pubkeys': adminPubkeys,
        'threshold': threshold,
        'balance_yuan': balanceYuan,
        'last_chain_refresh_at_millis': lastChainRefreshAtMillis,
        'last_balance_refresh_at_millis': lastBalanceRefreshAtMillis,
        'updated_at_millis': updatedAtMillis,
      };

  static DuoqianLocalDetailSnapshot? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final adminRaw = decoded['admin_pubkeys'];
      final admins = adminRaw is List
          ? adminRaw
              .map((item) => item.toString().toLowerCase())
              .where((item) => item.isNotEmpty)
              .toList(growable: false)
          : const <String>[];
      final status = decoded['status']?.toString();
      if (status == null || status.isEmpty) return null;
      return DuoqianLocalDetailSnapshot(
        status: status,
        adminPubkeys: admins,
        threshold: _toInt(decoded['threshold']),
        balanceYuan: _toDouble(decoded['balance_yuan']),
        lastChainRefreshAtMillis:
            _toInt(decoded['last_chain_refresh_at_millis']),
        lastBalanceRefreshAtMillis:
            _toInt(decoded['last_balance_refresh_at_millis']),
        updatedAtMillis: _toInt(decoded['updated_at_millis']),
      );
    } catch (_) {
      return null;
    }
  }

  static int? _toInt(Object? value) {
    if (value == null) return null;
    if (value is int) return value;
    return int.tryParse(value.toString());
  }

  static double? _toDouble(Object? value) {
    if (value == null) return null;
    if (value is double) return value;
    if (value is int) return value.toDouble();
    return double.tryParse(value.toString());
  }
}

/// 个人多签本地生命周期状态。
///
/// 中文注释：链上注销后账户主体可能已经不存在，但用户本机仍要在账户列表
/// 显示“已注销”，直到用户主动点“删除”清空本地数据。这里复用 AppKvEntity，
/// 避免把状态散落到多个页面。
class PersonalDuoqianLocalState {
  static const statusPending = 'pending';
  static const statusActive = 'active';
  static const statusClosed = 'closed';

  static String statusKey(String personalAddressHex) =>
      'personal_duoqian_status:${_normalizeHex(personalAddressHex)}';

  static String detailKey(String personalAddressHex) =>
      'personal_duoqian_detail:${_normalizeHex(personalAddressHex)}';

  static Future<Map<String, String>> readStatuses(
    Isar isar,
    Iterable<String> personalAddressesHex,
  ) async {
    final snapshots = await readStatusSnapshots(isar, personalAddressesHex);
    return snapshots.map((key, value) => MapEntry(key, value.status));
  }

  static Future<Map<String, DuoqianLocalStatusSnapshot>> readStatusSnapshots(
    Isar isar,
    Iterable<String> personalAddressesHex,
  ) async {
    final result = <String, DuoqianLocalStatusSnapshot>{};
    for (final address in personalAddressesHex) {
      final normalized = _normalizeHex(address);
      final entity = await isar.appKvEntitys.getByKey(statusKey(normalized));
      final status = entity?.stringValue;
      if (status != null && status.isNotEmpty) {
        result[normalized] = DuoqianLocalStatusSnapshot(
          status: status,
          lastSyncAtMillis: entity?.intValue,
        );
      }
    }
    return result;
  }

  static Future<DuoqianLocalDetailSnapshot?> readDetail(
    Isar isar,
    String personalAddressHex,
  ) async {
    final entity =
        await isar.appKvEntitys.getByKey(detailKey(personalAddressHex));
    return DuoqianLocalDetailSnapshot.fromJsonString(entity?.stringValue);
  }

  /// 写入个人多签详情快照；调用方必须处在 Isar writeTxn 内。
  static Future<void> putDetailInTxn(
    Isar isar,
    String personalAddressHex,
    DuoqianLocalDetailSnapshot snapshot,
  ) async {
    final key = detailKey(personalAddressHex);
    final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
    entity
      ..key = key
      ..stringValue = jsonEncode(snapshot.toJson())
      ..intValue = snapshot.lastChainRefreshAtMillis ??
          snapshot.updatedAtMillis ??
          DateTime.now().millisecondsSinceEpoch;
    await isar.appKvEntitys.putByKey(entity);
  }

  /// 写入个人多签本地状态；调用方必须处在 Isar writeTxn 内。
  static Future<void> putStatusInTxn(
    Isar isar,
    String personalAddressHex,
    String status,
  ) async {
    final key = statusKey(personalAddressHex);
    final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
    entity
      ..key = key
      ..stringValue = status
      ..intValue = DateTime.now().millisecondsSinceEpoch;
    await isar.appKvEntitys.putByKey(entity);
  }

  /// 删除个人多签本地状态；调用方必须处在 Isar writeTxn 内。
  static Future<void> deleteStatusInTxn(
    Isar isar,
    String personalAddressHex,
  ) async {
    await isar.appKvEntitys
        .where()
        .keyEqualTo(statusKey(personalAddressHex))
        .deleteAll();
  }

  /// 删除个人多签详情快照；调用方必须处在 Isar writeTxn 内。
  static Future<void> deleteDetailInTxn(
    Isar isar,
    String personalAddressHex,
  ) async {
    await isar.appKvEntitys
        .where()
        .keyEqualTo(detailKey(personalAddressHex))
        .deleteAll();
  }

  static String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }
}

/// 机构多签本地生命周期状态。
///
/// 中文注释：机构多签链上关闭后也继续留在本机账户列表，显示“已注销”；
/// 用户主动点详情页右上角“删除”时，才清理本机机构账户记录。
class InstitutionDuoqianLocalState {
  static const statusPending = 'pending';
  static const statusActive = 'active';
  static const statusClosed = 'closed';

  static String statusKey(String duoqianAddressHex) =>
      'institution_duoqian_status:${_normalizeHex(duoqianAddressHex)}';

  static String detailKey(String duoqianAddressHex) =>
      'institution_duoqian_detail:${_normalizeHex(duoqianAddressHex)}';

  static Future<Map<String, String>> readStatuses(
    Isar isar,
    Iterable<String> duoqianAddressesHex,
  ) async {
    final snapshots = await readStatusSnapshots(isar, duoqianAddressesHex);
    return snapshots.map((key, value) => MapEntry(key, value.status));
  }

  static Future<Map<String, DuoqianLocalStatusSnapshot>> readStatusSnapshots(
    Isar isar,
    Iterable<String> duoqianAddressesHex,
  ) async {
    final result = <String, DuoqianLocalStatusSnapshot>{};
    for (final address in duoqianAddressesHex) {
      final normalized = _normalizeHex(address);
      final entity = await isar.appKvEntitys.getByKey(statusKey(normalized));
      final status = entity?.stringValue;
      if (status != null && status.isNotEmpty) {
        result[normalized] = DuoqianLocalStatusSnapshot(
          status: status,
          lastSyncAtMillis: entity?.intValue,
        );
      }
    }
    return result;
  }

  static Future<DuoqianLocalDetailSnapshot?> readDetail(
    Isar isar,
    String duoqianAddressHex,
  ) async {
    final entity =
        await isar.appKvEntitys.getByKey(detailKey(duoqianAddressHex));
    return DuoqianLocalDetailSnapshot.fromJsonString(entity?.stringValue);
  }

  /// 写入机构多签详情快照；调用方必须处在 Isar writeTxn 内。
  static Future<void> putDetailInTxn(
    Isar isar,
    String duoqianAddressHex,
    DuoqianLocalDetailSnapshot snapshot,
  ) async {
    final key = detailKey(duoqianAddressHex);
    final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
    entity
      ..key = key
      ..stringValue = jsonEncode(snapshot.toJson())
      ..intValue = snapshot.lastChainRefreshAtMillis ??
          snapshot.updatedAtMillis ??
          DateTime.now().millisecondsSinceEpoch;
    await isar.appKvEntitys.putByKey(entity);
  }

  /// 写入机构多签本地状态；调用方必须处在 Isar writeTxn 内。
  static Future<void> putStatusInTxn(
    Isar isar,
    String duoqianAddressHex,
    String status,
  ) async {
    final key = statusKey(duoqianAddressHex);
    final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
    entity
      ..key = key
      ..stringValue = status
      ..intValue = DateTime.now().millisecondsSinceEpoch;
    await isar.appKvEntitys.putByKey(entity);
  }

  /// 删除机构多签本地状态；调用方必须处在 Isar writeTxn 内。
  static Future<void> deleteStatusInTxn(
    Isar isar,
    String duoqianAddressHex,
  ) async {
    await isar.appKvEntitys
        .where()
        .keyEqualTo(statusKey(duoqianAddressHex))
        .deleteAll();
  }

  /// 删除机构多签详情快照；调用方必须处在 Isar writeTxn 内。
  static Future<void> deleteDetailInTxn(
    Isar isar,
    String duoqianAddressHex,
  ) async {
    await isar.appKvEntitys
        .where()
        .keyEqualTo(detailKey(duoqianAddressHex))
        .deleteAll();
  }

  static String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }
}

/// 用户创建的个人多签账户（本地持久化）。
@collection
class PersonalDuoqianEntity {
  Id id = Isar.autoIncrement;

  /// 多签地址公钥 hex（32 字节，不含 0x 前缀）。
  @Index(unique: true, replace: true)
  late String duoqianAddress;

  /// 多签账户名称。
  late String name;

  /// 创建人 SS58 地址。
  late String creatorAddress;

  /// 添加时间戳（毫秒）。
  @Index()
  late int addedAtMillis;

  /// true = 通过反向索引发现的(钱包账户作为 admin 命中);false = 本机创建/手动添加。
  /// 反向校验只会删除 true 的 entity,false 的永不被自动清理。
  @Index()
  bool discoveredViaAdmin = false;

  /// 本钱包持有的 admin 公钥列表(快照,UI 显示"我作为 N 位管理员之一参与")。
  /// 仅在 discoveredViaAdmin=true 时填充;false 时空列表。
  List<String> matchedAdminPubkeys = const [];
}

/// 个人多签提案历史快照（本地持久化）。
///
/// 链上 votingengine 90 天后清理终态提案 (REJECTED/EXECUTED/EXECUTION_FAILED)，
/// wuminapp 端必须在本地永久保留历史，详情页提案列表才能在历史段始终可见。
///
/// 写入时机:
/// 1. 本机发起提案后(propose_create_personal / propose_transfer / propose_close)
/// 2. 详情页打开时同步链上活跃提案最新状态（upsert）
/// 3. 本机投票后刷新该提案的 status / yesVotes / noVotes
@collection
class PersonalDuoqianProposalEntity {
  Id id = Isar.autoIncrement;

  /// 多签地址公钥 hex(32 字节,不含 0x 前缀)。复合索引 key 之一。
  @Index(composite: [CompositeIndex('proposalId')], unique: true, replace: true)
  late String personalAddress;

  /// 链上提案 ID。
  late int proposalId;

  /// 提案动作:'create' / 'transfer' / 'close'。
  @Index()
  late String action;

  /// 提案状态最新快照:'voting' / 'passed' / 'rejected' / 'executed' / 'execution_failed'。
  @Index()
  late String status;

  /// 投票计数 yes(每次刷新链上状态时同步)。
  late int yesVotes;

  /// 投票计数 no。
  late int noVotes;

  /// 提案首次记录时间(本机发起或首次发现)。
  @Index()
  late int createdAtMillis;

  /// 终态时间:rejected / executed / execution_failed 时写入;voting 期间为 null。
  int? finalStatusAtMillis;

  /// 业务快照(JSON 字符串):转账金额 / 关闭 beneficiary / 创建账户名等,便于扩展。
  String? snapshotJson;
}

/// 用户添加的多签机构（本地持久化）。
@collection
class DuoqianInstitutionEntity {
  Id id = Isar.autoIncrement;

  /// 多签地址公钥 hex（32 字节，不含 0x 前缀），唯一标识。
  @Index(unique: true, replace: true)
  late String duoqianAddress;

  /// SFID 标识（UTF-8 字符串）。
  late String sfidNumber;

  /// 机构账户管理员更换 org：4=公权机构账户，5=其他机构账户。
  int? adminSubjectOrg;

  /// 机构名称（链上升级前暂用 sfidNumber 代替）。
  late String name;

  /// 添加时间戳（毫秒），用于排序。
  @Index()
  late int addedAtMillis;

  /// true = 通过反向索引发现的(钱包账户作为 admin 命中);false = 本机创建/手动添加。
  /// 反向校验只会删除 true 的 entity,false 的永不被自动清理。
  @Index()
  bool discoveredViaAdmin = false;

  /// 本钱包持有的 admin 公钥列表(快照,UI 显示"我作为 N 位管理员之一参与")。
  /// 仅在 discoveredViaAdmin=true 时填充;false 时空列表。
  List<String> matchedAdminPubkeys = const [];
}

/// 本地钱包余额变化流水（持久化存储，去中心化设计，不依赖 SFID 服务器）。
@collection
class LocalTxEntity {
  Id id = Isar.autoIncrement;

  /// 单条钱包流水唯一键。
  ///
  /// 中文注释：钱包账户由 walletPubkeyHex 唯一，流水记录由 recordKey 唯一。
  /// 区块事件记录使用 `walletPubkeyHex:blockHash:eventIndex`，本机提交记录
  /// 使用 `walletPubkeyHex:pending:txHash`，避免把 txHash 误当成单条流水唯一性。
  @Index(unique: true, replace: true)
  late String recordKey;

  /// 所属钱包地址（SS58）。
  @Index()
  late String walletAddress;

  /// 所属钱包公钥 hex（32 字节，不含 0x）。
  @Index()
  late String walletPubkeyHex;

  /// 业务类型：transfer / fee / reward / interest / issuance / burn / duoqian_transfer。
  late String type;

  /// 该钱包实际余额变化（分），带正负号；正数=增加，负数=减少。
  ///
  /// 中文注释：Dart int 在不同平台上不适合承载链上 u128，统一用十进制字符串保存。
  late String amountDeltaFen;

  /// 转账本金（分），不带正负号。
  String? transferAmountFen;

  /// 手续费（分），不带正负号；只有本钱包支付手续费时记录。
  String? feeFen;

  /// 对方地址；余额增加时是来源，余额减少时是去向。
  String? counterpartyAddress;

  String? fromAddress;
  String? toAddress;

  /// 状态：pending=已提交 / inBlock=已出块 / finalized=已确认 / failed=失败。
  late String status;

  /// 记录来源：local_submit / chain_event / resync。
  late String source;

  /// 链上交易哈希。
  String? txHash;

  /// 链上区块号。
  int? blockNumber;

  /// 链上区块哈希。
  String? blockHash;

  /// 区块内事件序号。
  int? eventIndex;

  /// 事件所属 extrinsic 序号（如果 phase 为 ApplyExtrinsic）。
  int? extrinsicIndex;

  /// 提交交易时使用的 nonce（只用于详情辅助展示，不作为流水确认真源）。
  int? usedNonce;

  /// 本地创建时间（毫秒时间戳）。
  @Index()
  late int createdAtMillis;

  /// 最终确认时间（毫秒时间戳）。
  int? confirmedAtMillis;

  /// 失败原因。
  String? failureReason;
}

/// 钱包交易记录本机同步游标。
///
/// 中文注释：wuminapp 不扫描导入前历史。游标只记录该钱包进入本机后，
/// 本机已经同步到哪个 finalized 区块，离线重开时只补这之后的缺口。
@collection
class WalletTxSyncCursorEntity {
  Id id = Isar.autoIncrement;

  late String walletAddress;

  @Index(unique: true, replace: true)
  late String walletPubkeyHex;

  late int trackingStartBlock;
  late int lastSyncedBlock;
  late int createdAtMillis;
  late int updatedAtMillis;
}

class WalletIsar {
  WalletIsar._();

  static final WalletIsar instance = WalletIsar._();

  Isar? _isar;
  Future<Isar>? _opening;
  Future<void> _operationTail = Future<void>.value();
  bool _operationActive = false;
  Future<void>? _testCoreInit;

  static const List<Duration> _busyRetryDelays = [
    Duration(milliseconds: 80),
    Duration(milliseconds: 160),
    Duration(milliseconds: 320),
    Duration(milliseconds: 640),
    Duration(milliseconds: 1200),
    Duration(milliseconds: 2400),
    Duration(milliseconds: 3600),
    Duration(milliseconds: 5000),
  ];

  /// 中文注释：给低优先级后台任务判断是否让路；前台读写仍应直接排队执行。
  bool get hasActiveOperation => _operationActive;

  /// 中文注释：业务调度层用它识别 MDBX 短暂繁忙，并选择跳过低优先级后台任务。
  bool isBusyError(Object error) => _isBusyError(error);

  Future<Isar> db() async {
    final current = _isar;
    if (current != null && current.isOpen) {
      return current;
    }

    final opening = _opening;
    if (opening != null) {
      return opening;
    }

    final task = _openAndMigrate();
    _opening = task;
    try {
      final opened = await task;
      _isar = opened;
      return opened;
    } finally {
      _opening = null;
    }
  }

  /// 中文注释：低端 Android 的 MDBX 在读写窗口重叠时可能短暂返回 EAGAIN。
  /// 对这类 busy 错误做小间隔重试，避免交易流水同步或余额刷新把瞬时竞争暴露给 UI。
  Future<T> runWithBusyRetry<T>(Future<T> Function() action) async {
    for (var attempt = 0; attempt <= _busyRetryDelays.length; attempt++) {
      try {
        return await action();
      } catch (error) {
        if (!_isBusyError(error) || attempt == _busyRetryDelays.length) {
          rethrow;
        }
        await Future<void>.delayed(_busyRetryDelays[attempt]);
      }
    }
    throw StateError('unreachable');
  }

  Future<T> _enqueue<T>(Future<T> Function() action) {
    final previous = _operationTail;
    final completer = Completer<T>();
    _operationTail = completer.future.then<void>(
      (_) {},
      onError: (_) {},
    );

    () async {
      try {
        await previous.catchError((_) {});
        _operationActive = true;
        final result = await runWithBusyRetry(action);
        completer.complete(result);
      } catch (error, stackTrace) {
        completer.completeError(error, stackTrace);
      } finally {
        _operationActive = false;
      }
    }();

    return completer.future;
  }

  Future<T> read<T>(Future<T> Function(Isar isar) action) {
    return _enqueue(() async {
      final isar = await db();
      return action(isar);
    });
  }

  bool _isBusyError(Object error) {
    final raw = error.toString().toLowerCase();
    return raw.contains('mdbxerror (11)') ||
        raw.contains('try again') ||
        raw.contains('active transaction');
  }

  /// 中文注释：全 App 共用的 Isar 写事务入口。
  ///
  /// Android 低端机上交易流水同步、余额刷新、多签扫描和钱包导入可能同时读写库，
  /// MDBX 会返回 `MdbxError(11): Try again`。所有业务读写统一排队到这里，
  /// 保证同一进程内任何时刻只有一个 Isar 业务操作在运行。
  Future<T> writeTxn<T>(Future<T> Function(Isar isar) action) {
    return _enqueue(() async {
      final isar = await db();
      return isar.writeTxn<T>(() => action(isar));
    });
  }

  Future<Isar> _openAndMigrate() async {
    await ensureTestCoreInitialized();

    // 中文注释：先检查是否已有同名实例打开（Isar 不允许重复打开同名数据库）。
    // 如果已有实例但 schema 不完整（缺少新增的 collection），关闭后重新打开。
    final existing = Isar.getInstance('wuminapp_wallet');
    if (existing != null && existing.isOpen) {
      try {
        // 尝试访问 LocalTxEntity collection，如果成功说明 schema 完整。
        existing.localTxEntitys;
        return existing;
      } catch (_) {
        // schema 不完整，关闭旧实例后重新打开
        await existing.close();
      }
    }

    final dir = await _resolveDirectory();
    final schemas = [
      WalletProfileEntitySchema,
      WalletSettingsEntitySchema,
      AdminRoleCacheEntitySchema,
      ObservedAccountEntitySchema,
      LoginReplayEntitySchema,
      AppKvEntitySchema,
      DuoqianInstitutionEntitySchema,
      PersonalDuoqianEntitySchema,
      PersonalDuoqianProposalEntitySchema,
      LocalTxEntitySchema,
      WalletTxSyncCursorEntitySchema,
    ];
    final isar =
        await Isar.open(schemas, name: 'wuminapp_wallet', directory: dir);
    await WalletIsarMigration.ensureMigrated(isar);
    return isar;
  }

  Future<void> ensureTestCoreInitialized() async {
    if (!_isFlutterTest()) {
      return;
    }

    final inflight = _testCoreInit;
    if (inflight != null) {
      return inflight;
    }

    final task = _initTestCoreInternal();
    _testCoreInit = task;
    try {
      await task;
    } finally {
      _testCoreInit = null;
    }
  }

  Future<void> _initTestCoreInternal() async {
    final localPath = _resolveLocalIsarCorePath();
    if (localPath == null) {
      throw StateError(
        'Flutter test 模式未找到 Isar Core 动态库，请先执行 flutter pub get。',
      );
    }
    await Isar.initializeIsarCore(
      libraries: <Abi, String>{Abi.current(): localPath},
    );
  }

  Future<void> resetForTest() async {
    if (!_isFlutterTest()) {
      return;
    }
    final current = _isar;
    if (current != null && current.isOpen) {
      await current.close(deleteFromDisk: true);
    }
    _isar = null;
    _opening = null;
    _operationTail = Future<void>.value();
    _operationActive = false;
  }

  /// 中文注释：应用锁触发清空数据时使用，同样走队列等待前序读写结束。
  Future<void> closeAndDeleteFromDisk() {
    return _enqueue(() async {
      final opening = _opening;
      if (opening != null) {
        try {
          await opening;
        } catch (_) {
          // 打开失败时继续尝试关闭已注册实例。
        }
      }
      final current = _isar ?? Isar.getInstance('wuminapp_wallet');
      if (current != null && current.isOpen) {
        await current.close(deleteFromDisk: true);
      }
      _isar = null;
      _opening = null;
    });
  }

  Future<String> _resolveDirectory() async {
    if (kIsWeb) {
      return '.';
    }
    if (_isFlutterTest()) {
      return Directory.systemTemp.path;
    }
    final appDir = await getApplicationSupportDirectory();
    return appDir.path;
  }

  bool _isFlutterTest() {
    return Platform.environment.containsKey('FLUTTER_TEST');
  }

  String? _resolveLocalIsarCorePath() {
    final fromEnv = Platform.environment['ISAR_CORE_LIB_PATH'];
    if (fromEnv != null && fromEnv.trim().isNotEmpty) {
      final file = File(fromEnv.trim());
      if (file.existsSync()) {
        return file.path;
      }
    }

    final home = Platform.environment['HOME'];
    if (home == null || home.isEmpty) {
      return null;
    }

    final hosted = Directory('$home/.pub-cache/hosted/pub.dev');
    if (!hosted.existsSync()) {
      return null;
    }

    final candidates = hosted
        .listSync(followLinks: false)
        .whereType<Directory>()
        .where((dir) => dir.path
            .split(Platform.pathSeparator)
            .last
            .startsWith('isar_flutter_libs-'))
        .toList(growable: false)
      ..sort((a, b) => b.path.compareTo(a.path));

    final relative = switch (Abi.current()) {
      Abi.macosArm64 || Abi.macosX64 => 'macos/libisar.dylib',
      Abi.linuxX64 => 'linux/libisar.so',
      Abi.windowsArm64 || Abi.windowsX64 => 'windows/isar.dll',
      _ => null,
    };
    if (relative == null) {
      return null;
    }

    for (final dir in candidates) {
      final path = '${dir.path}/$relative';
      if (File(path).existsSync()) {
        return path;
      }
    }
    return null;
  }
}

class WalletIsarMigration {
  static const String _kSchemaVersion = 'wallet.data.schema.version';

  /// 当前 schema 版本。开发阶段直接覆盖，不做增量迁移。
  static const int currentSchemaVersion = 3;

  static Future<void> ensureMigrated(Isar isar) async {
    await _ensureSettingsRow(isar);
    final version = await _schemaVersion(isar);
    if (version >= currentSchemaVersion) {
      return;
    }
    await isar.writeTxn(() async {
      if (version < 3) {
        // 中文注释：三段交易状态上线前的本机流水可能已经把 best block
        // 误标为 finalized，且 newHeads/finalized 去重规则不完整。旧记录
        // 不作为账本真源，升级到 v3 时直接清空，从当前本机时刻重新记录。
        await isar.localTxEntitys.clear();
        await isar.walletTxSyncCursorEntitys.clear();
      }
      final entity = AppKvEntity()
        ..key = _kSchemaVersion
        ..intValue = currentSchemaVersion
        ..boolValue = null
        ..stringValue = null;
      await isar.appKvEntitys.put(entity);
    });
  }

  static Future<int> _schemaVersion(Isar isar) async {
    final row = await isar.appKvEntitys
        .filter()
        .keyEqualTo(_kSchemaVersion)
        .findFirst();
    return row?.intValue ?? 0;
  }

  static Future<void> _ensureSettingsRow(Isar isar) async {
    final settings = await isar.walletSettingsEntitys.get(0);
    if (settings != null) {
      return;
    }
    await isar.writeTxn(() async {
      await isar.walletSettingsEntitys.put(
        WalletSettingsEntity()
          ..id = 0
          ..updatedAtMillis = DateTime.now().millisecondsSinceEpoch,
      );
    });
  }
}
