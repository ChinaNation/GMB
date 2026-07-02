import 'dart:convert';
import 'dart:typed_data';

import 'package:isar_community/isar.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/shared/proposal/proposal_models.dart';
import 'package:citizenapp/isar/wallet_isar.dart';
import 'package:citizenapp/my/util/amount_format.dart';
import 'package:citizenapp/transaction/multisig-transfer/multisig_transfer_models.dart';

/// 提案列表本地持久化读库。
///
/// 中文注释：这里复用 Isar 的 AppKvEntity 保存“展示摘要”和“机构列表索引”，
/// 只服务治理机构详情页和公民-提案列表展示；投票、执行、提交前仍必须重查链上真值。
class ProposalLocalStore {
  ProposalLocalStore._();

  static final ProposalLocalStore instance = ProposalLocalStore._();

  static const Duration institutionIndexTtl = Duration(minutes: 5);

  static const String _summaryPrefix = 'governance.proposal.summary.';
  static const String _institutionIndexPrefix =
      'governance.proposal.index.institution.';

  String institutionIndexKey(String cidNumber) =>
      '$_institutionIndexPrefix$cidNumber';

  Future<ProposalLocalIndex?> readInstitutionIndex(String cidNumber) {
    return _readIndex(institutionIndexKey(cidNumber));
  }

  Future<bool> isInstitutionIndexFresh(String cidNumber) async {
    final index = await readInstitutionIndex(cidNumber);
    return index != null && index.isFresh(institutionIndexTtl);
  }

  Future<List<LocalProposalSummary>> readInstitutionSummaries(
    String cidNumber,
  ) async {
    final index = await readInstitutionIndex(cidNumber);
    if (index == null || index.ids.isEmpty) return const [];
    return readSummariesForIds(index.ids);
  }

  Future<List<LocalProposalSummary>> readSummariesForIds(List<int> ids) {
    if (ids.isEmpty) return Future.value(const []);
    return WalletIsar.instance.read((isar) async {
      final result = <LocalProposalSummary>[];
      for (final id in ids) {
        final entity = await isar.appKvEntitys.getByKey('$_summaryPrefix$id');
        final summary = LocalProposalSummary.fromJsonString(
          entity?.stringValue,
        );
        if (summary != null) {
          result.add(summary);
        }
      }
      return result;
    });
  }

  Future<void> putInstitutionIndex(String cidNumber, List<int> ids) {
    return _putIndex(institutionIndexKey(cidNumber), ids);
  }

  Future<void> upsertSummaries(List<LocalProposalSummary> summaries) async {
    if (summaries.isEmpty) return;
    await WalletIsar.instance.writeTxn((isar) async {
      for (final summary in summaries) {
        final key = '$_summaryPrefix${summary.proposalId}';
        final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
        entity
          ..key = key
          ..stringValue = jsonEncode(summary.toJson())
          ..intValue = summary.updatedAtMillis
          ..boolValue = null;
        await isar.appKvEntitys.putByKey(entity);
      }
    });
  }

  Future<void> clearAllForTest() async {
    await WalletIsar.instance.writeTxn((isar) async {
      final rows = await isar.appKvEntitys
          .filter()
          .keyStartsWith('governance.proposal.')
          .findAll();
      await isar.appKvEntitys.deleteAll(rows.map((row) => row.id).toList());
    });
  }

  Future<ProposalLocalIndex?> _readIndex(String key) {
    return WalletIsar.instance.read((isar) async {
      final entity = await isar.appKvEntitys.getByKey(key);
      return ProposalLocalIndex.fromJsonString(
        entity?.stringValue,
        fallbackSyncedAtMillis: entity?.intValue,
      );
    });
  }

  Future<void> _putIndex(String key, List<int> ids) async {
    final now = DateTime.now().millisecondsSinceEpoch;
    final index = ProposalLocalIndex(ids: ids, syncedAtMillis: now);
    await WalletIsar.instance.writeTxn((isar) async {
      final entity = await isar.appKvEntitys.getByKey(key) ?? AppKvEntity();
      entity
        ..key = key
        ..stringValue = jsonEncode(index.toJson())
        ..intValue = now
        ..boolValue = null;
      await isar.appKvEntitys.putByKey(entity);
    });
  }
}

class ProposalLocalIndex {
  const ProposalLocalIndex({
    required this.ids,
    required this.syncedAtMillis,
  });

  final List<int> ids;
  final int syncedAtMillis;

  bool isFresh(Duration ttl) {
    final age = DateTime.now().millisecondsSinceEpoch - syncedAtMillis;
    return age >= 0 && age < ttl.inMilliseconds;
  }

  Map<String, dynamic> toJson() => {
        'ids': ids,
        'synced_at_millis': syncedAtMillis,
      };

  static ProposalLocalIndex? fromJsonString(
    String? raw, {
    int? fallbackSyncedAtMillis,
  }) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final rawIds = decoded['ids'];
      if (rawIds is! List) return null;
      final ids = rawIds
          .map((item) => _toInt(item))
          .whereType<int>()
          .toList(growable: false);
      final syncedAt =
          _toInt(decoded['synced_at_millis']) ?? fallbackSyncedAtMillis;
      if (syncedAt == null) return null;
      return ProposalLocalIndex(ids: ids, syncedAtMillis: syncedAt);
    } catch (_) {
      return null;
    }
  }
}

class LocalProposalSummary {
  const LocalProposalSummary({
    required this.proposalId,
    required this.kind,
    required this.stage,
    required this.status,
    required this.title,
    required this.subtitle,
    required this.listSubtitle,
    required this.iconKind,
    required this.updatedAtMillis,
    this.internalCode,
    this.institutionBytesHex,
    this.subjectCidNumbers = const [],
    this.displayYear,
    this.displaySeqInYear,
    this.institutionCidNumber,
    this.cidFullName,
  });

  final int proposalId;
  final int kind;
  final int stage;
  final int status;
  final String? internalCode;
  final String? institutionBytesHex;
  final List<String> subjectCidNumbers;
  final int? displayYear;
  final int? displaySeqInYear;
  final String? institutionCidNumber;
  final String? cidFullName;
  final String title;
  final String subtitle;
  final String listSubtitle;
  final String iconKind;
  final int updatedAtMillis;

  ProposalDisplayMeta? get displayMeta {
    final year = displayYear;
    final seq = displaySeqInYear;
    if (year == null || seq == null) return null;
    return ProposalDisplayMeta(year: year, seqInYear: seq);
  }

  String get displayId => formatProposalId(displayMeta);

  ProposalMeta get meta => ProposalMeta(
        proposalId: proposalId,
        kind: kind,
        stage: stage,
        status: status,
        internalCode: internalCode,
        institutionBytes: _hexToBytes(institutionBytesHex),
        subjectCidNumbers: subjectCidNumbers,
        displayMeta: displayMeta,
      );

  static LocalProposalSummary fromProposal(
    ProposalWithDetail proposal, {
    InstitutionInfo? institution,
    int? nowMillis,
  }) {
    final meta = proposal.meta;
    final displayId = formatProposalId(meta.displayMeta);
    final status = _statusLabel(meta.status);
    final transfer =
        proposal.businessDetails[MultisigTransferProposalDetailKeys.transfer];
    final safetyFund =
        proposal.businessDetails[MultisigTransferProposalDetailKeys.safetyFund];
    final sweep =
        proposal.businessDetails[MultisigTransferProposalDetailKeys.sweep];

    String title;
    String subtitle;
    String listSubtitle;
    String iconKind;

    if (transfer is TransferProposalInfo) {
      title = '转账提案 $displayId';
      listSubtitle =
          '转账 ${AmountFormat.format(transfer.amountYuan, symbol: '')} 元';
      subtitle = '$listSubtitle · $status';
      iconKind = 'transfer';
    } else if (safetyFund is SafetyFundProposalInfo) {
      title = '安全基金转账 $displayId';
      listSubtitle =
          '安全基金转账 ${AmountFormat.format(safetyFund.amountYuan, symbol: '')} 元';
      subtitle = '$listSubtitle · $status';
      iconKind = 'safety_fund';
    } else if (sweep is SweepProposalInfo) {
      title = '手续费划转 $displayId';
      listSubtitle =
          '手续费划转 ${AmountFormat.format(sweep.amountYuan, symbol: '')} 元';
      subtitle = '$listSubtitle · $status';
      iconKind = 'sweep';
    } else if (proposal.createMultisigDetail != null) {
      title = '创建多签 $displayId';
      listSubtitle = '创建个人多签';
      subtitle = '创建个人多签账户 · $status';
      iconKind = 'create_multisig';
    } else if (proposal.closeMultisigDetail != null) {
      title = '关闭多签 $displayId';
      listSubtitle = '关闭多签';
      subtitle = '关闭多签账户 · $status';
      iconKind = 'close_multisig';
    } else if (proposal.runtimeUpgradeDetail != null) {
      title = '协议升级 $displayId';
      listSubtitle = '协议升级';
      subtitle = '协议升级 · $status';
      iconKind = 'runtime_upgrade';
    } else if (proposal.resolutionIssuanceSummary != null) {
      title = '联合投票提案 $displayId';
      listSubtitle = '决议发行';
      subtitle = '决议发行 · $status';
      iconKind = 'resolution_issuance';
    } else if (proposal.resolutionDestroySummary != null) {
      title = '联合投票提案 $displayId';
      listSubtitle = '决议销毁';
      subtitle = '决议销毁 · $status';
      iconKind = 'resolution_destroy';
    } else if (meta.kind == 1) {
      title = '联合投票提案 $displayId';
      listSubtitle = '联合投票提案';
      subtitle = '联合投票 · $status';
      iconKind = 'joint';
    } else {
      title = '提案 $displayId';
      listSubtitle = '提案 ${_kindLabel(meta.kind)}';
      subtitle = '提案事件 · $status';
      iconKind = 'proposal';
    }

    return LocalProposalSummary(
      proposalId: meta.proposalId,
      kind: meta.kind,
      stage: meta.stage,
      status: meta.status,
      internalCode: meta.internalCode,
      institutionBytesHex: _bytesToHex(meta.institutionBytes),
      subjectCidNumbers: meta.subjectCidNumbers,
      displayYear: meta.displayMeta?.year,
      displaySeqInYear: meta.displayMeta?.seqInYear,
      institutionCidNumber: institution?.cidNumber,
      cidFullName: institution?.cidFullName,
      title: title,
      subtitle: subtitle,
      listSubtitle: listSubtitle,
      iconKind: iconKind,
      updatedAtMillis: nowMillis ?? DateTime.now().millisecondsSinceEpoch,
    );
  }

  Map<String, dynamic> toJson() => {
        'proposal_id': proposalId,
        'kind': kind,
        'stage': stage,
        'status': status,
        'internal_code': internalCode,
        'institution_bytes_hex': institutionBytesHex,
        'subject_cid_numbers': subjectCidNumbers,
        'display_year': displayYear,
        'display_seq_in_year': displaySeqInYear,
        'institution_cid_number': institutionCidNumber,
        'cid_full_name': cidFullName,
        'title': title,
        'subtitle': subtitle,
        'list_subtitle': listSubtitle,
        'icon_kind': iconKind,
        'updated_at_millis': updatedAtMillis,
      };

  static LocalProposalSummary? fromJsonString(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    try {
      final decoded = jsonDecode(raw);
      if (decoded is! Map<String, dynamic>) return null;
      final proposalId = _toInt(decoded['proposal_id']);
      final kind = _toInt(decoded['kind']);
      final stage = _toInt(decoded['stage']);
      final status = _toInt(decoded['status']);
      final title = decoded['title']?.toString();
      final subtitle = decoded['subtitle']?.toString();
      final listSubtitle = decoded['list_subtitle']?.toString();
      final iconKind = decoded['icon_kind']?.toString();
      final updatedAt = _toInt(decoded['updated_at_millis']);
      if (proposalId == null ||
          kind == null ||
          stage == null ||
          status == null ||
          title == null ||
          subtitle == null ||
          listSubtitle == null ||
          iconKind == null ||
          updatedAt == null) {
        return null;
      }
      return LocalProposalSummary(
        proposalId: proposalId,
        kind: kind,
        stage: stage,
        status: status,
        internalCode: _toNullableString(decoded['internal_code']),
        institutionBytesHex:
            _toNullableString(decoded['institution_bytes_hex']),
        subjectCidNumbers: _toStringList(decoded['subject_cid_numbers']),
        displayYear: _toInt(decoded['display_year']),
        displaySeqInYear: _toInt(decoded['display_seq_in_year']),
        institutionCidNumber:
            _toNullableString(decoded['institution_cid_number']),
        cidFullName: _toNullableString(decoded['cid_full_name']),
        title: title,
        subtitle: subtitle,
        listSubtitle: listSubtitle,
        iconKind: iconKind,
        updatedAtMillis: updatedAt,
      );
    } catch (_) {
      return null;
    }
  }
}

String _statusLabel(int status) {
  switch (status) {
    case 0:
      return '投票中';
    case 1:
      return '已通过';
    case 2:
      return '已拒绝';
    case 3:
      return '已执行';
    case 4:
      return '执行失败';
    default:
      return '未知';
  }
}

String _kindLabel(int kind) {
  switch (kind) {
    case 0:
      return '内部投票';
    case 1:
      return '联合投票';
    default:
      return '';
  }
}

String? _bytesToHex(Uint8List? bytes) {
  if (bytes == null) return null;
  final buffer = StringBuffer();
  for (final byte in bytes) {
    buffer.write(byte.toRadixString(16).padLeft(2, '0'));
  }
  return buffer.toString();
}

Uint8List? _hexToBytes(String? hex) {
  if (hex == null || hex.isEmpty) return null;
  final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
  if (clean.length.isOdd) return null;
  final result = Uint8List(clean.length ~/ 2);
  for (var i = 0; i < result.length; i++) {
    final value = int.tryParse(
      clean.substring(i * 2, i * 2 + 2),
      radix: 16,
    );
    if (value == null) return null;
    result[i] = value;
  }
  return result;
}

int? _toInt(Object? value) {
  if (value == null) return null;
  if (value is int) return value;
  return int.tryParse(value.toString());
}

String? _toNullableString(Object? value) {
  final text = value?.toString();
  if (text == null || text.isEmpty || text == 'null') return null;
  return text;
}

List<String> _toStringList(Object? value) {
  if (value is! List) return const [];
  return value
      .map((item) => item?.toString())
      .whereType<String>()
      .where((item) => item.isNotEmpty)
      .toList(growable: false);
}
