import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:flutter/foundation.dart';
import 'package:isar_community/isar.dart';
import 'package:path_provider/path_provider.dart';

part 'app_isar.g.dart';

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

  /// 用户拖拽排序后的稳定顺序。
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
class AdminGroupCacheEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String pubkeyHex;

  late String adminGroupName;

  @Index()
  late int updatedAt;
}

@collection
class ObservedAccountEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String accountId;

  late String accountLabel;
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
/// `status` 负责 UI 展示，`lastSyncAtMillis` 负责判断是否需要
/// 再次查链；两者都复用 AppKvEntity，避免为 TTL 新增 Isar schema。
class MultisigLocalStatusSnapshot {
  const MultisigLocalStatusSnapshot({
    required this.status,
    required this.lastSyncAtMillis,
  });

  final String status;
  final int? lastSyncAtMillis;
}

/// 多签详情页本地持久化快照。
///
/// 这不是短期内存缓存，而是详情页首屏可直接使用的本机状态。
/// 链上刷新成功后覆盖写入；链上失败时保留旧值，避免进详情页被 RPC 卡住。
class MultisigLocalDetailSnapshot {
  const MultisigLocalDetailSnapshot({
    required this.status,
    required this.admins,
    this.threshold,
    this.balanceYuan,
    this.lastChainRefreshAtMillis,
    this.lastBalanceRefreshAtMillis,
    this.updatedAtMillis,
  });

  final String status;
  final List<String> admins;
  final int? threshold;
  final double? balanceYuan;
  final int? lastChainRefreshAtMillis;
  final int? lastBalanceRefreshAtMillis;
  final int? updatedAtMillis;

  Map<String, dynamic> toJson() => {
        'status': status,
        'admins': admins,
        'threshold': threshold,
        'balance_yuan': balanceYuan,
        'last_chain_refresh_at_millis': lastChainRefreshAtMillis,
        'last_balance_refresh_at_millis': lastBalanceRefreshAtMillis,
        'updated_at_millis': updatedAtMillis,
      };

  static MultisigLocalDetailSnapshot? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final adminRaw = decoded['admins'];
      final admins = adminRaw is List
          ? adminRaw
              .map((item) => item.toString().toLowerCase())
              .where((item) => item.isNotEmpty)
              .toList(growable: false)
          : const <String>[];
      final status = decoded['status']?.toString();
      if (status == null || status.isEmpty) return null;
      return MultisigLocalDetailSnapshot(
        status: status,
        admins: admins,
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
/// 链上注销后账户主体可能已经不存在，但用户本机仍要在账户列表
/// 显示“已注销”，直到用户主动点“删除”清空本地数据。这里复用 AppKvEntity，
/// 避免把状态散落到多个页面。
class PersonalMultisigLocalState {
  static const statusPending = 'pending';
  static const statusActive = 'active';
  static const statusClosed = 'closed';

  static String statusKey(String personalAccountHex) =>
      'personal_account_status:${_normalizeHex(personalAccountHex)}';

  static String detailKey(String personalAccountHex) =>
      'personal_account_detail:${_normalizeHex(personalAccountHex)}';

  static Future<Map<String, String>> readStatuses(
    Isar isar,
    Iterable<String> personalAccountsHex,
  ) async {
    final snapshots = await readStatusSnapshots(isar, personalAccountsHex);
    return snapshots.map((key, value) => MapEntry(key, value.status));
  }

  static Future<Map<String, MultisigLocalStatusSnapshot>> readStatusSnapshots(
    Isar isar,
    Iterable<String> personalAccountsHex,
  ) async {
    final result = <String, MultisigLocalStatusSnapshot>{};
    for (final address in personalAccountsHex) {
      final normalized = _normalizeHex(address);
      final entity = await isar.appKvEntitys.getByKey(statusKey(normalized));
      final status = entity?.stringValue;
      if (status != null && status.isNotEmpty) {
        result[normalized] = MultisigLocalStatusSnapshot(
          status: status,
          lastSyncAtMillis: entity?.intValue,
        );
      }
    }
    return result;
  }

  static Future<MultisigLocalDetailSnapshot?> readDetail(
    Isar isar,
    String personalAccountHex,
  ) async {
    final entity =
        await isar.appKvEntitys.getByKey(detailKey(personalAccountHex));
    return MultisigLocalDetailSnapshot.fromJsonString(entity?.stringValue);
  }

  /// 写入个人多签详情快照；调用方必须处在 Isar writeTxn 内。
  static Future<void> putDetailInTxn(
    Isar isar,
    String personalAccountHex,
    MultisigLocalDetailSnapshot snapshot,
  ) async {
    final key = detailKey(personalAccountHex);
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
    String personalAccountHex,
    String status,
  ) async {
    final key = statusKey(personalAccountHex);
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
    String personalAccountHex,
  ) async {
    await isar.appKvEntitys
        .where()
        .keyEqualTo(statusKey(personalAccountHex))
        .deleteAll();
  }

  /// 删除个人多签详情快照；调用方必须处在 Isar writeTxn 内。
  static Future<void> deleteDetailInTxn(
    Isar isar,
    String personalAccountHex,
  ) async {
    await isar.appKvEntitys
        .where()
        .keyEqualTo(detailKey(personalAccountHex))
        .deleteAll();
  }

  static String _normalizeHex(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    return h.toLowerCase();
  }
}

/// 用户创建的个人多签账户（本地持久化）。
@collection
class PersonalAccountEntity {
  Id id = Isar.autoIncrement;

  /// 多签账户公钥 hex（32 字节，不含 0x 前缀）。
  @Index(unique: true, replace: true)
  late String account;

  /// 个人多签账户名称。
  late String accountName;

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
/// citizenapp 端必须在本地永久保留历史，详情页提案列表才能在历史段始终可见。
///
/// 写入时机:
/// 1. 本机发起提案后(propose_create_personal / propose_transfer / propose_close)
/// 2. 详情页打开时同步链上活跃提案最新状态（upsert）
/// 3. 本机投票后刷新该提案的 status / yesVotes / noVotes
@collection
class PersonalAccountProposalEntity {
  Id id = Isar.autoIncrement;

  /// 多签账户公钥 hex(32 字节,不含 0x 前缀)。复合索引 key 之一。
  @Index(composite: [CompositeIndex('proposalId')], unique: true, replace: true)
  late String personalAccount;

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
class InstitutionEntity {
  Id id = Isar.autoIncrement;

  /// 多签账户公钥 hex（32 字节，不含 0x 前缀），唯一标识。
  @Index(unique: true, replace: true)
  late String account;

  /// CID 标识（UTF-8 字符串）。
  late String cidNumber;

  /// 机构账户管理员更换 institution_code：如 CGOV/SFGQ/UNIN。
  String? adminAccountCode;

  /// 本地机构多签账户显示名,不是机构全称/简称;后者只允许使用 cidFullName/cidShortName。
  late String accountName;

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

/// 行政区字典实体(ADR-021 行政区唯一真源)。
///
/// 派生自 china.sqlite(经 `assets/admin_divisions/` 静态数据包),是
/// citizenapp 端行政区名字的**唯一真源**——机构记录只存 code,显示名一律按
/// (level, scopeKey, code) 查本表 join 得到。别处零独立存行政区名字。
///
/// 镇 code 全国不唯一(LN 三市都有镇 001),故唯一键必须带完整层级前缀:
/// [divisionKey] = `"<level>|<pcode>|<ccode>|<tcode>"`,缺级留空。
@collection
class AdminDivisionEntity {
  Id id = Isar.autoIncrement;

  /// 复合唯一键:`"<level>|<pcode>|<ccode>|<tcode>"`(缺级留空)。
  /// 例:省=`"province|LN||"`、市=`"city|LN|001|"`、镇=`"town|LN|001|005"`。
  @Index(unique: true, replace: true)
  late String divisionKey;

  /// 层级:`province` / `city` / `town`。
  @Index()
  late String level;

  /// 该层级自身的 code(省 code / 市 code / 镇 code)。
  late String code;

  /// 父定位键:province 空、city=pcode、town=`"<pcode>|<ccode>"`。
  /// 用于「某省的全部市」「某省某市的全部镇」按父批量取。
  @Index()
  late String scopeKey;

  /// 行政区名字(来自 china.sqlite)。
  late String divisionName;

  /// 字典版本戳(来自 manifest version;排错/增量比对用)。
  String? dictVersion;
}

/// 公权机构目录本地完整缓存(ADR-018 §九 混合模式)。
///
/// 数据来自发布期数据包(基线)+ CID 公开接口增量同步;UI 永远读本表,
/// 省/市/机构导航零链读零现查。主/费账户本地派生不入库,仅自定义账户名(op_tag=0x06)
/// 入 [customAccountNames](绝大多数机构为空)。
@collection
class PublicInstitutionEntity {
  Id id = Isar.autoIncrement;

  /// 机构身份 ID(cid_number),全局唯一。
  @Index(unique: true, replace: true)
  late String cidNumber;

  late String cidFullName;
  String? cidShortName;
  late String status;

  /// 所属省 code(行政区唯一真源键;名字由 [AdminDivisionEntity] 字典 join,
  /// 见 ADR-021)。左侧省导航分组键。
  @Index()
  late String provinceCode;

  /// 所属市 code(市列表分组键;名字走字典 join)。
  @Index()
  late String cityCode;

  /// 所属镇 code(可空,空串表示机构只定位到市级)。
  String townCode = '';

  /// 机构类型机构码(CID 单一真源,如 PRS/PGV/CGOV/CREG 等)。
  /// 索引:五子 tab(治理/立法)按机构码过滤目录(ADR-028 P2);Isar 打开自动建索引。
  @Index()
  late String institutionCode;

  String? parentCidNumber;
  bool? hasLegalPersonality;

  /// 法定代表人姓名(公开目录字段,来自 CID subjects.legal_representative_name);无则 null → 留空。
  String? legalRepresentativeName;

  /// 法定代表人唯一公民 CID；与姓名、账户同时存在或同时为空。
  String? legalRepresentativeCidNumber;

  /// 法定代表人唯一钱包账户。
  String? legalRepresentativeAccount;
  late int accountCount;

  /// 自定义账户名(op_tag=0x06);主/费可本地派生不入库。空占绝大多数。
  List<String> customAccountNames = const [];

  /// 该省目录同步版本戳(增量比对/排错用)。
  String? catalogVersion;

  late int updatedAtMillis;
}

/// 公权机构订阅("关注"分组)。
///
/// 按钱包公钥隔离的纯本地决策表;[subscriptionKey] = `pubkeyHex|cidNumber`
/// 复合唯一,只有订阅的机构才纳入动态刷新集(详情页卡C),目录浏览不依赖本表。
@collection
class PublicInstitutionSubscriptionEntity {
  Id id = Isar.autoIncrement;

  /// 复合唯一键:`walletPubkeyHex|cidNumber`。
  @Index(unique: true, replace: true)
  late String subscriptionKey;

  /// 订阅者钱包公钥(查"我的关注"用)。
  @Index()
  late String walletPubkeyHex;

  late String cidNumber;
  late int subscribedAtMillis;
}

/// Chat 会话本地索引。
///
/// 聊天明文只允许在公民手机本地保存；Cloudflare 瞬时转发与近场 transport
/// 只承载密文 envelope。本表负责会话列表首屏，不参与链上状态。
@collection
class ChatConversationEntity {
  Id id = Isar.autoIncrement;

  /// 会话 ID，对应 MLS group id。
  @Index(unique: true, replace: true)
  late String conversationId;

  /// 本机钱包聊天账户。
  @Index()
  late String ownerAccount;

  /// 对方钱包聊天账户。
  @Index()
  late String peerAccount;

  late String title;
  late String lastMessage;

  @Index()
  late int lastUpdatedAtMillis;

  late int unreadCount;
  late String lastDeliveryState;
}

/// Chat 消息本地记录。
///
/// `envelopeBytesHex` 保存完整 GMB_CHAT_V1 Protobuf bytes，便于重试
/// 和排查；`plaintext` 只写手机本地库，绝不上传 Cloudflare 或近场 transport。
@collection
class ChatMessageEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String envelopeId;

  @Index()
  late String conversationId;

  @Index()
  late String ownerAccount;

  late String direction;
  late String senderAccount;
  late String recipientAccount;
  late String senderDeviceId;
  late String messageKind;
  late String mlsMessageKind;
  late String deliveryState;
  String? plaintext;
  late String envelopeBytesHex;

  @Index()
  late int createdAtMillis;
}

/// Chat 出站队列。
///
/// 投递失败时只重试完整 envelope bytes，不重新加密，避免破坏
/// MLS 会话状态和消息顺序。
@collection
class ChatOutboundQueueEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String envelopeId;

  @Index()
  late String conversationId;

  late String recipientAccount;
  late String envelopeBytesHex;
  late String deliveryState;
  late int attemptCount;
  String? lastError;

  @Index()
  late int updatedAtMillis;
}

/// Chat 待设备投递的媒体字节(离线补发用)。
///
/// 媒体字节走 WebRTC 设备直连;对方离线时字节发不出,控制消息已入队,发送方在此
/// 登记"待投递"并把字节留在本机缓存。对方上线(peer_ready)时**由 conversationId /
/// attachmentId / fileName 用当前 Documents 目录重算缓存路径**重发字节(不持久化
/// 绝对路径,避免容器 UUID 变更后误判丢失);WebRTC ack 到达后删除本行。App 重启后
/// 仍在,恢复即补发。
@collection
class ChatOutgoingMediaEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String attachmentId;

  @Index()
  late String recipientAccount;

  late String conversationId;
  late String fileName;
  late String contentType;
  late int byteSize;
  late int createdAtMillis;
}

/// Chat 待处理入站 envelope。
///
/// application 早于 Welcome 到达时先落这里；处理 Welcome 后再
/// 重放同会话 pending，避免因为网络乱序丢消息。
@collection
class ChatPendingInboundEntity {
  Id id = Isar.autoIncrement;

  @Index(unique: true, replace: true)
  late String envelopeId;

  @Index()
  late String conversationId;

  late String envelopeBytesHex;
  late String reason;

  @Index()
  late int createdAtMillis;
}

/// Chat 路由缓存记录。
///
/// Chat 路由缓存只保存在公民手机本地，用于把联系人钱包地址映射到
/// OpenMLS 设备和近场提示；用户联系人仍以“我的通讯录”为准。
@collection
class ChatRouteCacheEntity {
  Id id = Isar.autoIncrement;

  /// 路由唯一键，当前等于钱包聊天账户。
  @Index(unique: true, replace: true)
  late String routeId;

  /// 对方钱包聊天账户，也是公民币收款账户。
  @Index(unique: true, replace: true)
  late String peerAccount;

  /// Chat 路由显示名，只用于联系人路由列表，不承载机构全称或简称。
  late String routeDisplayName;
  late String deviceId;
  late String devicePublicKeyHex;
  late String safetyNumber;
  String? nearbyPeerHint;
  String? note;
  late int createdAtMillis;
  late int updatedAtMillis;
}

/// 本地钱包余额变化流水（持久化存储，去中心化设计，不依赖 CID 服务器）。
@collection
class LocalTxEntity {
  Id id = Isar.autoIncrement;

  /// 单条钱包流水唯一键。
  ///
  /// 钱包账户由 walletPubkeyHex 唯一，流水记录由 recordKey 唯一。
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

  /// 业务类型：transfer / fee / reward / interest / issuance / burn / multisig_transfer。
  late String type;

  /// 该钱包实际余额变化（分），带正负号；正数=增加，负数=减少。
  ///
  /// Dart int 在不同平台上不适合承载链上 u128，统一用十进制字符串保存。
  late String amountDeltaFen;

  /// 转账本金（分），不带正负号。
  String? transferAmountFen;

  /// 手续费（分），不带正负号；只有本钱包支付手续费时记录。
  String? feeFen;

  /// 对方地址；余额增加时是来源，余额减少时是去向。
  String? counterpartyAddress;

  String? fromAddress;
  String? toAddress;

  /// 转账备注，来自 OnchainTransaction::TransferWithRemark 事件或本机提交草稿。
  String? remark;

  /// 状态(ADR-017)：pending=已提交 / finalized=已确认 / failed=失败；
  /// inBlock 为交易提交 watch 的临时进度态(豁免区)，非 finalized 流水终态。
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
/// citizenapp 不扫描导入前历史。游标只记录该钱包进入本机后，
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

  /// 仅测试用:覆盖测试期 Isar 目录。让每个测试文件(独立 isolate)用唯一临时目录,
  /// 从物理上根除跨文件共享同一磁盘库导致的并发锁竞争与残留污染。生产恒为 null。
  static String? debugTestDirectoryOverride;

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

  /// 给低优先级后台任务判断是否让路；前台读写仍应直接排队执行。
  bool get hasActiveOperation => _operationActive;

  /// 业务调度层用它识别 MDBX 短暂繁忙，并选择跳过低优先级后台任务。
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

  /// 低端 Android 的 MDBX 在读写窗口重叠时可能短暂返回 EAGAIN。
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

  /// 全 App 共用的 Isar 写事务入口。
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

    // 先检查是否已有同名实例打开（Isar 不允许重复打开同名数据库）。
    // 如果已有实例但 schema 不完整（缺少新增的 collection），关闭后重新打开。
    final existing = Isar.getInstance('citizenapp');
    if (existing != null && existing.isOpen) {
      try {
        // 尝试访问 LocalTxEntity collection，如果成功说明 schema 完整。
        existing.localTxEntitys;
        existing.chatConversationEntitys;
        existing.chatRouteCacheEntitys;
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
      AdminGroupCacheEntitySchema,
      ObservedAccountEntitySchema,
      LoginReplayEntitySchema,
      AppKvEntitySchema,
      InstitutionEntitySchema,
      PersonalAccountEntitySchema,
      PersonalAccountProposalEntitySchema,
      AdminDivisionEntitySchema,
      PublicInstitutionEntitySchema,
      PublicInstitutionSubscriptionEntitySchema,
      ChatConversationEntitySchema,
      ChatMessageEntitySchema,
      ChatOutboundQueueEntitySchema,
      ChatOutgoingMediaEntitySchema,
      ChatPendingInboundEntitySchema,
      ChatRouteCacheEntitySchema,
      LocalTxEntitySchema,
      WalletTxSyncCursorEntitySchema,
    ];
    final isar = await Isar.open(schemas, name: 'citizenapp', directory: dir);
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

  /// 应用锁触发清空数据时使用，同样走队列等待前序读写结束。
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
      final current = _isar ?? Isar.getInstance('citizenapp');
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
      return debugTestDirectoryOverride ?? Directory.systemTemp.path;
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
            .startsWith('isar_community_flutter_libs-'))
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
  static const int currentSchemaVersion = 8;

  static Future<void> ensureMigrated(Isar isar) async {
    await _ensureSettingsRow(isar);
    final version = await _schemaVersion(isar);
    if (version >= currentSchemaVersion) {
      return;
    }
    await isar.writeTxn(() async {
      if (version < 3) {
        // 三段交易状态上线前的本机流水可能已经把 best block
        // 误标为 finalized，且 newHeads/finalized 去重规则不完整。旧记录
        // 不作为账本真源，升级到 v3 时直接清空，从当前本机时刻重新记录。
        await isar.localTxEntitys.clear();
        await isar.walletTxSyncCursorEntitys.clear();
      }
      if (version < 4) {
        // MultisigInstitutionEntity 改名为 InstitutionEntity（collection
        // 名变更），旧 collection 数据仅为本地缓存，丢弃后由反向索引重新发现即可，
        // 无需数据迁移。这里清空新 collection 兜底，避免新旧叠加出现脏数据。
        await isar.institutionEntitys.clear();
      }
      if (version < 7) {
        // ADR-021 行政区唯一真源:公权机构目录从「存行政区名字」
        // 改为「只存 province/city/town code」，名字由 AdminDivisionEntity 字典
        // join。公权目录是只读派生数据(无用户数据),旧 name-keyed 行直接清空,
        // 首启从 assets 数据包(已带 code)全量重灌;字典表同步清空待 bundle 重灌。
        await isar.publicInstitutionEntitys.clear();
        await isar.adminDivisionEntitys.clear();
      }
      if (version < 8) {
        // 2026-06-23:修复公权机构市卡片显示 001;`AdminDivisionEntity`
        // 的 `divisionName`(市名)字段在 bf187d53「统一命名修复」才加入,该提交之前的
        // build 灌进 Isar 的字典 divisionName 为空 → cityNameMap 查到空名 → 市名回退
        // code(001)。而省版本游标(storedProvVers)判定"内容没变"会跳过重灌
        // (ensureSynced reconciled=0),旧空名永久残留,覆盖安装不清 Isar 也救不了。
        // 强制清空字典表:ensureSynced 见 divisionCount=0(hasData=false)即全量重灌带
        // 市名的新数据。与 [[feedback-dto-field-rename-bump-cache-version]] 同根
        // (改数据结构必 bump 强制刷新版本)。bundle 自身的省级 ver 增量仍照常工作。
        await isar.adminDivisionEntitys.clear();
      }
      // schema version 以 key 为唯一真源；重复迁移时必须原地更新，
      // 不能新建同 key 行，否则 Isar 唯一索引会报错。
      final entity =
          await isar.appKvEntitys.getByKey(_kSchemaVersion) ?? AppKvEntity();
      entity
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
