// 个人多签提案历史聚合服务(req 5)。
//
// 双轨制数据源:
// 1. 链上 `votingengine.ActiveProposalsByInstitution[personal_address || zeros(16)]`
//    返回当前活跃(STATUS_VOTING)的提案 ID 列表。
// 2. 本机 Isar `PersonalDuoqianProposalEntity` 永久保留所有历史快照,覆盖
//    链上 90 天后已清理的终态提案。
//
// `fetchAll` 把两条数据合并去重(以 proposalId 为 key),并把链上活跃提案的
// 最新状态 upsert 到 Isar(同步缓存,防止其他设备发起的提案漏记)。

import 'dart:convert';
import 'dart:typed_data';

import 'package:isar/isar.dart';
import 'package:polkadart/polkadart.dart' show Hasher;

import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/common/proposal/proposal_query_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';

/// 提案动作类型常量(对齐 Isar entity action 字段)。
class PersonalProposalAction {
  static const String create = 'create';
  static const String transfer = 'transfer';
  static const String close = 'close';
}

/// 提案状态字符串(对齐 votingengine 链上枚举,但用人类可读字符串持久化)。
class PersonalProposalStatus {
  static const String voting = 'voting';
  static const String passed = 'passed';
  static const String rejected = 'rejected';
  static const String executed = 'executed';
  static const String executionFailed = 'execution_failed';
}

/// 链上 votingengine status u8 → Isar 字符串。
String mapChainStatus(int? chainStatus) {
  switch (chainStatus) {
    case 0:
      return PersonalProposalStatus.voting;
    case 1:
      return PersonalProposalStatus.passed;
    case 2:
      return PersonalProposalStatus.rejected;
    case 3:
      return PersonalProposalStatus.executed;
    case 4:
      return PersonalProposalStatus.executionFailed;
    default:
      return PersonalProposalStatus.voting;
  }
}

/// 详情页提案列表渲染所需的视图模型。
class PersonalDuoqianProposalView {
  const PersonalDuoqianProposalView({
    required this.proposalId,
    required this.action,
    required this.status,
    required this.yesVotes,
    required this.noVotes,
    required this.createdAtMillis,
    this.finalStatusAtMillis,
    this.snapshot,
  });

  final int proposalId;
  final String action;
  final String status;
  final int yesVotes;
  final int noVotes;
  final int createdAtMillis;
  final int? finalStatusAtMillis;
  final Map<String, dynamic>? snapshot;

  bool get isActive => status == PersonalProposalStatus.voting;
  bool get isFinal => !isActive;
}

class PersonalProposalHistoryService {
  PersonalProposalHistoryService({
    ChainRpc? chainRpc,
    ProposalQueryService? proposalService,
  })  : _rpc = chainRpc ?? ChainRpc(),
        _proposalService = proposalService ?? ProposalQueryService();

  final ChainRpc _rpc;
  final ProposalQueryService _proposalService;

  /// 拉取该多签的全部提案(活跃 + 历史),按 createdAt desc 排序。
  ///
  /// 容错:链上失败仅回退 Isar;Isar 失败仅回退链上;两者都失败返回空列表。
  ///
  /// 状态新鲜度策略(2026-05-03 修):
  /// 1. 链上 ActiveProposalsByInstitution 同步活跃提案到 Isar(防其他设备漏记)
  /// 2. **额外**:遍历 Isar 中本机已知 status='voting' 的 entity,挨个查链上
  ///    Proposals[id] 拿最新 status + tally。这步必须独立于 active 列表,
  ///    因为提案一旦终态(passed/executed/rejected)就从 active 列表移除,
  ///    本机 Isar 永远停在 voting 状态,UI 卡片显示"投票中"是假数据。
  Future<List<PersonalDuoqianProposalView>> fetchAll(
    String personalAddressHex,
  ) async {
    final activeIds = await _safeFetchActiveProposalIds(personalAddressHex);

    // Step 1: 链上活跃提案逐个同步到 Isar(防止其他设备发起的提案在本机无记录)。
    for (final pid in activeIds) {
      await _syncActiveProposalToIsar(personalAddressHex, pid);
    }

    // Step 2: 重查本机 Isar 中 status='voting' 的 entity,即使它们已不在 active 列表
    // (提案终态后就从 active 列表移除,但本机 entity 还停在 voting)。
    await _refreshLocalVotingEntities(personalAddressHex);

    return _readAllFromIsar(personalAddressHex);
  }

  /// 遍历本机 Isar 中所有 status='voting' 的 entity,查链上 `Proposals[id]` 拿最新状态。
  /// 链上若已终态(passed/executed/rejected/execution_failed),upsert 为终态;
  /// 链上仍 voting 则只刷新 yesVotes/noVotes;链上 storage 不存在(已被 90 天清理)
  /// 也只刷新 vote tally(取现有值)— 不强制覆盖为终态,等本机其他渠道写入历史。
  Future<void> _refreshLocalVotingEntities(String personalAddressHex) async {
    try {
      final isar = await WalletIsar.instance.db();
      final votingEntities = await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(personalAddressHex)
          .statusEqualTo(PersonalProposalStatus.voting)
          .findAll();

      for (final e in votingEntities) {
        try {
          final chainStatus =
              await _proposalService.fetchProposalStatus(e.proposalId);
          if (chainStatus == null) continue; // 链上不存在,跳过
          final tally = await _proposalService.fetchVoteTally(e.proposalId);
          await recordOrUpdate(
            personalAddressHex: personalAddressHex,
            proposalId: e.proposalId,
            action: e.action,
            status: mapChainStatus(chainStatus),
            yesVotes: tally.yes,
            noVotes: tally.no,
          );
        } catch (_) {
          // 单条刷新失败不阻断其他 entity
        }
      }
    } catch (_) {
      // 整个刷新失败也不阻断主流程
    }
  }

  /// 写入或更新 Isar 提案 entity。
  ///
  /// [snapshot] 若非空将以 JSON 形式持久化(转账金额 / beneficiary / account_name 等)。
  Future<void> recordOrUpdate({
    required String personalAddressHex,
    required int proposalId,
    required String action,
    required String status,
    required int yesVotes,
    required int noVotes,
    Map<String, dynamic>? snapshot,
  }) async {
    final isar = await WalletIsar.instance.db();
    final now = DateTime.now().millisecondsSinceEpoch;

    await isar.writeTxn(() async {
      final existing = await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(personalAddressHex)
          .proposalIdEqualTo(proposalId)
          .findFirst();

      final isFinal = status != PersonalProposalStatus.voting;

      final entity = existing ?? PersonalDuoqianProposalEntity()
        ..personalAddress = personalAddressHex
        ..proposalId = proposalId
        ..createdAtMillis = existing?.createdAtMillis ?? now;

      entity.action = action;
      entity.status = status;
      entity.yesVotes = yesVotes;
      entity.noVotes = noVotes;
      entity.finalStatusAtMillis =
          isFinal ? (existing?.finalStatusAtMillis ?? now) : null;
      if (snapshot != null) {
        entity.snapshotJson = jsonEncode(snapshot);
      } else if (existing != null) {
        entity.snapshotJson = existing.snapshotJson;
      }

      await isar.personalDuoqianProposalEntitys.put(entity);
    });
  }

  // ──── 内部 ─────────────────────────────────────────────

  Future<List<int>> _safeFetchActiveProposalIds(
    String personalAddressHex,
  ) async {
    try {
      final subjectId = _personalAddressToSubjectId(personalAddressHex);
      final key = _buildStorageKey(
        'VotingEngine',
        'ActiveProposalsByInstitution',
        subjectId,
      );
      final data = await _rpc.fetchStorage('0x${_hexEncode(key)}');
      if (data == null || data.isEmpty) return const [];
      final (count, lenSize) = _decodeCompact(data, 0);
      final ids = <int>[];
      var offset = lenSize;
      for (var i = 0; i < count && offset + 8 <= data.length; i++) {
        ids.add(_decodeU64(data.sublist(offset, offset + 8)));
        offset += 8;
      }
      return ids;
    } catch (_) {
      return const [];
    }
  }

  Future<void> _syncActiveProposalToIsar(
    String personalAddressHex,
    int proposalId,
  ) async {
    try {
      final chainStatus =
          await _proposalService.fetchProposalStatus(proposalId);
      final tally = await _proposalService.fetchVoteTally(proposalId);
      final statusStr = mapChainStatus(chainStatus);

      // 已知该 entity 时保留 action / snapshot,首次发现则归类 unknown(本机无快照)。
      final isar = await WalletIsar.instance.db();
      final existing = await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(personalAddressHex)
          .proposalIdEqualTo(proposalId)
          .findFirst();

      await recordOrUpdate(
        personalAddressHex: personalAddressHex,
        proposalId: proposalId,
        action: existing?.action ?? PersonalProposalAction.create,
        status: statusStr,
        yesVotes: tally.yes,
        noVotes: tally.no,
      );
    } catch (_) {
      // 单条同步失败不阻断其他提案同步。
    }
  }

  Future<List<PersonalDuoqianProposalView>> _readAllFromIsar(
    String personalAddressHex,
  ) async {
    try {
      final isar = await WalletIsar.instance.db();
      final entities = await isar.personalDuoqianProposalEntitys
          .filter()
          .personalAddressEqualTo(personalAddressHex)
          .sortByCreatedAtMillisDesc()
          .findAll();
      return entities.map(_entityToView).toList(growable: false);
    } catch (_) {
      return const [];
    }
  }

  PersonalDuoqianProposalView _entityToView(
    PersonalDuoqianProposalEntity e,
  ) {
    Map<String, dynamic>? snapshot;
    if (e.snapshotJson != null && e.snapshotJson!.isNotEmpty) {
      try {
        snapshot = jsonDecode(e.snapshotJson!) as Map<String, dynamic>;
      } catch (_) {
        snapshot = null;
      }
    }
    return PersonalDuoqianProposalView(
      proposalId: e.proposalId,
      action: e.action,
      status: e.status,
      yesVotes: e.yesVotes,
      noVotes: e.noVotes,
      createdAtMillis: e.createdAtMillis,
      finalStatusAtMillis: e.finalStatusAtMillis,
      snapshot: snapshot,
    );
  }

  // ──── 编码 / 哈希工具(对齐 votingengine storage key) ────

  /// 个人多签 SubjectId = byte[0]=0x03 PersonalDuoqian + byte[1..33]=address(32B) + byte[33..48]=zeros(15B)。
  ///
  /// D 阶段(SubjectKind 协议统一,2026-05-06)起,链端协议规范见 ADR-010;
  /// 对齐 Rust [primitives::derive::subject_id_from_account] 实现。
  Uint8List _personalAddressToSubjectId(String addressHex) {
    final addr = _hexDecode(addressHex);
    final id = Uint8List(48);
    id[0] = 0x03; // SubjectKind::PersonalDuoqian
    final copy = addr.length < 32 ? addr.length : 32;
    id.setRange(1, 1 + copy, addr);
    return id;
  }

  Uint8List _buildStorageKey(
    String pallet,
    String storage,
    Uint8List keyData,
  ) {
    final palletHash = Hasher.twoxx128.hashString(pallet);
    final storageHash = Hasher.twoxx128.hashString(storage);
    final keyHash = Hasher.blake2b128.hash(keyData);
    final result = Uint8List(palletHash.length +
        storageHash.length +
        keyHash.length +
        keyData.length);
    var offset = 0;
    result.setAll(offset, palletHash);
    offset += palletHash.length;
    result.setAll(offset, storageHash);
    offset += storageHash.length;
    result.setAll(offset, keyHash);
    offset += keyHash.length;
    result.setAll(offset, keyData);
    return result;
  }

  Uint8List _hexDecode(String hex) {
    final h = hex.startsWith('0x') ? hex.substring(2) : hex;
    final bytes = Uint8List(h.length ~/ 2);
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = int.parse(h.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return bytes;
  }

  String _hexEncode(Uint8List bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  (int, int) _decodeCompact(Uint8List data, int offset) {
    final mode = data[offset] & 0x03;
    if (mode == 0) return (data[offset] >> 2, 1);
    if (mode == 1) {
      return ((data[offset] >> 2) | (data[offset + 1] << 6), 2);
    }
    if (mode == 2) {
      return (
        (data[offset] >> 2) |
            (data[offset + 1] << 6) |
            (data[offset + 2] << 14) |
            (data[offset + 3] << 22),
        4
      );
    }
    final len = ((data[offset] >> 2) + 4) & 0xFF;
    var value = 0;
    for (var i = 0; i < len; i++) {
      value |= data[offset + 1 + i] << (i * 8);
    }
    return (value, len + 1);
  }

  int _decodeU64(Uint8List data) {
    final bd = ByteData.sublistView(data);
    return bd.getUint64(0, Endian.little);
  }
}
