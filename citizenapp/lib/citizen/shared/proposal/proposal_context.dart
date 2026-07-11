import 'package:flutter/foundation.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/admin_activation_service.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/services/institution_admin_service.dart';
import 'package:citizenapp/citizen/shared/institution_info.dart';
import 'package:citizenapp/citizen/institution/governance_registry.dart';
import 'package:citizenapp/isar/app_isar.dart';
import 'package:citizenapp/votingengine/internal-vote/internal_vote_query_service.dart';
import 'package:citizenapp/citizen/proposal/runtime-upgrade/runtime_upgrade_service.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 用户与某个提案的关系上下文。
///
/// 所有提案详情页统一使用该类判断：
/// - 用户是否为该提案对应机构的管理员
/// - 用户可用于投票的冷钱包列表
/// - 用户的角色（admin / citizen / viewer）
class ProposalContext {
  const ProposalContext({
    this.institution,
    this.adminWallets = const [],
    this.role = ProposalRole.viewer,
  });

  /// 匹配到的机构（可能为 null，表示无法关联机构）。
  final InstitutionInfo? institution;

  /// 用户在该机构的管理员冷钱包列表。
  final List<WalletProfile> adminWallets;

  /// 用户角色。
  final ProposalRole role;

  /// 是否有机构上下文（能进行管理员操作）。
  bool get hasInstitutionContext => institution != null;

  /// 是否为管理员。
  bool get isAdmin => role == ProposalRole.admin;

  /// 是否为公民（有链上公民身份）。
  bool get isCitizen =>
      role == ProposalRole.citizen || role == ProposalRole.admin;

  /// 是否有导入的管理员钱包。
  bool get hasAdminWallets => adminWallets.isNotEmpty;
}

/// 用户角色。
enum ProposalRole {
  /// 管理员：有冷钱包匹配到链上管理员列表。
  admin,

  /// 公民：有链上公民身份的钱包（暂未实现，预留）。
  citizen,

  /// 查看者：无关联钱包。
  viewer,
}

/// 提案上下文解析器。
///
/// 统一处理"用户与某提案的关系"解析逻辑，所有入口（治理列表、机构页面）
/// 共用同一套代码，避免重复且保证一致性。
class ProposalContextResolver {
  ProposalContextResolver({
    InstitutionAdminService? adminService,
    WalletManager? walletManager,
    ActivationService? activationService,
  })  : _adminService = adminService ?? InstitutionAdminService(),
        _walletManager = walletManager ?? WalletManager(),
        _activationService = activationService ?? ActivationService();

  final InstitutionAdminService _adminService;
  final WalletManager _walletManager;
  final ActivationService _activationService;

  /// 已缓存的钱包列表（同一次会话内复用）。
  List<WalletProfile>? _wallets;

  /// 静态缓存：用户已确认为管理员的机构 cidNumber 集合。
  ///
  /// 当用户浏览某个机构详情页后，如果匹配到管理员身份，
  /// 该机构 cidNumber 会被加入此集合。机构列表页据此渲染绿色卡片和排序。
  static final Set<String> _institutionAdminIds = {};

  /// 记录某机构的管理员状态。
  static void markInstitutionAdmin(String cidNumber) {
    _institutionAdminIds.add(cidNumber);
  }

  /// 查询某机构是否已确认为管理员。
  static bool isInstitutionAdmin(String cidNumber) {
    return _institutionAdminIds.contains(cidNumber);
  }

  /// 获取所有已确认的管理员机构 cidNumber 集合。
  static Set<String> get institutionAdminIds =>
      Set.unmodifiable(_institutionAdminIds);

  /// 清除管理员机构缓存（如钱包被删除时）。
  static void clearInstitutionAdminCache() {
    _institutionAdminIds.clear();
  }

  /// 解析用户与指定提案的关系。
  ///
  /// [institutionBytes] Proposal.account_context 的 AccountId32（可为 null）。
  /// [knownInstitution] 如果调用方已知机构信息（如从机构页面进入），直接传入跳过反查。
  Future<ProposalContext> resolve({
    List<int>? institutionBytes,
    String? internalCode,
    InstitutionInfo? knownInstitution,
  }) async {
    final wallets = await _getWallets();
    final coldWallets = wallets.where((w) => w.isColdWallet).toList();

    // 1. 确定机构
    InstitutionInfo? institution = knownInstitution;

    if (institution == null && institutionBytes != null) {
      institution = findInstitutionByAccountId(institutionBytes,
          adminAccountCode: internalCode);
    }

    // 2. 如果仍然没有机构（如 account_context 反查失败），
    //    遍历用户所有冷钱包，逐个查链上管理员列表反向匹配。
    if (institution == null && coldWallets.isNotEmpty) {
      institution = await _reverseMatchInstitution(coldWallets);
    }

    // 3. 匹配管理员钱包（通过激活状态判断）
    if (institution == null) {
      return const ProposalContext();
    }

    late final AdminAccountIdentity identity;
    try {
      identity = AdminAccountIdentity.fromInstitution(institution);
      if (institution.isRegisteredAccount) {
        final threshold = await _adminService.fetchThreshold(
          identity,
        );
        if (threshold != null) {
          institution = institution.copyWith(
            internalThresholdOverride: threshold,
          );
        }
      }
    } catch (_) {
      return ProposalContext(institution: institution);
    }

    // 仅通过激活记录匹配管理员钱包
    final activatedAdmins = await _activationService
        .getActivatedAdmins(identity)
        .catchError((_) => <ActivatedAdmin>[]);

    final matchedWallets = <WalletProfile>[];

    // 已激活的管理员 → 在钱包列表中找到对应钱包
    for (final activated in activatedAdmins) {
      WalletProfile? wallet;
      for (final w in wallets) {
        if (_normalize(w.pubkeyHex) == activated.pubkeyHex) {
          wallet = w;
          break;
        }
      }
      if (wallet != null &&
          !matchedWallets.any(
            (w) => _normalize(w.pubkeyHex) == activated.pubkeyHex,
          )) {
        matchedWallets.add(wallet);
      }
    }

    return ProposalContext(
      institution: institution,
      adminWallets: matchedWallets,
      role:
          matchedWallets.isNotEmpty ? ProposalRole.admin : ProposalRole.viewer,
    );
  }

  /// 批量解析多个提案的上下文（用于列表页）。
  ///
  /// 返回 Map<提案索引, ProposalContext>，与传入列表一一对应。
  Future<List<ProposalContext>> resolveBatch(
      List<List<int>?> institutionBytesList,
      {List<String?>? internalCodeList,
      Map<String, InstitutionInfo> knownInstitutionsByAccountHex =
          const {}}) async {
    final wallets = await _getWallets();
    final coldWallets = wallets.where((w) => w.isColdWallet).toList();
    final results = <ProposalContext>[];
    final knownInstitutions = {
      for (final entry in knownInstitutionsByAccountHex.entries)
        _normalize(entry.key): entry.value,
    };

    for (var i = 0; i < institutionBytesList.length; i++) {
      final institutionBytes = institutionBytesList[i];
      final internalCode =
          internalCodeList != null && i < internalCodeList.length
              ? internalCodeList[i]
              : null;
      InstitutionInfo? institution;

      if (institutionBytes != null) {
        institution = knownInstitutions[_hexFromBytes(institutionBytes)] ??
            findInstitutionByAccountId(institutionBytes,
                adminAccountCode: internalCode);
      }

      if (institution == null && coldWallets.isNotEmpty) {
        institution = await _reverseMatchInstitution(coldWallets);
      }

      if (institution == null) {
        results.add(const ProposalContext());
        continue;
      }

      late final AdminAccountIdentity identity;
      try {
        identity = AdminAccountIdentity.fromInstitution(institution);
        if (institution.isRegisteredAccount) {
          final threshold = await _adminService.fetchThreshold(
            identity,
          );
          if (threshold != null) {
            institution = institution.copyWith(
              internalThresholdOverride: threshold,
            );
          }
        }
      } catch (_) {
        results.add(ProposalContext(institution: institution));
        continue;
      }

      // 仅通过激活记录匹配管理员钱包
      final activatedAdmins = await _activationService
          .getActivatedAdmins(identity)
          .catchError((_) => <ActivatedAdmin>[]);

      final matchedWallets = <WalletProfile>[];
      for (final activated in activatedAdmins) {
        for (final w in wallets) {
          if (_normalize(w.pubkeyHex) == activated.pubkeyHex &&
              !matchedWallets.any(
                (m) => _normalize(m.pubkeyHex) == activated.pubkeyHex,
              )) {
            matchedWallets.add(w);
          }
        }
      }

      results.add(ProposalContext(
        institution: institution,
        adminWallets: matchedWallets,
        role: matchedWallets.isNotEmpty
            ? ProposalRole.admin
            : ProposalRole.viewer,
      ));
    }

    return results;
  }

  /// 清除钱包缓存（钱包列表变化时调用）。
  void clearWalletCache() => _wallets = null;
  // 内部方法
  Future<List<WalletProfile>> _getWallets() async {
    try {
      _wallets ??= await _walletManager.getWallets();
    } catch (e, st) {
      // 治理页的链上内容不能因为本地钱包库短暂繁忙而整体加载失败。
      if (!WalletIsar.instance.isBusyError(e)) {
        debugPrint('[ProposalContext] local wallet load failed: $e\n$st');
      }
      _wallets = const [];
    }
    return _wallets!;
  }

  /// 反向匹配：遍历所有机构，查管理员列表，看用户冷钱包是否在其中。
  Future<InstitutionInfo?> _reverseMatchInstitution(
    List<WalletProfile> coldWallets,
  ) async {
    final allInstitutions = [
      ...kNrc,
      ...kPrcs,
      ...kProvincialBanks,
    ];

    final coldPubkeys = coldWallets.map((w) => _normalize(w.pubkeyHex)).toSet();

    for (final inst in allInstitutions) {
      List<String> admins;
      try {
        admins = await _adminService.fetchAdmins(
          AdminAccountIdentity.fromInstitution(inst),
        );
      } catch (_) {
        continue;
      }
      for (final admin in admins) {
        if (coldPubkeys.contains(admin)) {
          return inst;
        }
      }
    }

    return null;
  }

  static String _normalize(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return clean.toLowerCase();
  }

  static String _hexFromBytes(List<int> bytes) {
    final buffer = StringBuffer();
    for (final byte in bytes) {
      buffer.write(byte.toRadixString(16).padLeft(2, '0'));
    }
    return buffer.toString();
  }
}

/// 统一投票状态检查器。
///
/// 根据提案类型（kind）和阶段（stage）自动选择正确的链上存储查询，
/// 避免不同入口查错存储导致的状态不一致。
class VoteChecker {
  VoteChecker({
    InternalVoteQueryService? internalVoteService,
    RuntimeUpgradeService? runtimeService,
  })  : _internalVoteService =
            internalVoteService ?? InternalVoteQueryService(),
        _runtimeService = runtimeService ?? RuntimeUpgradeService();

  final InternalVoteQueryService _internalVoteService;
  final RuntimeUpgradeService _runtimeService;

  /// 跨提案批量计算"哪些提案存在本机未投票的管理员钱包"。
  ///
  /// (ADR-018 R2):列表页原来按提案逐个查投票(P 个提案 = P 次往返);
  /// 这里把同类提案(内部/联合)的投票 key 各自一次性拼齐批量读取,P 次往返
  /// 降为最多 2 次。只统计 status==0(投票中)且本机有管理员钱包的提案。
  Future<Set<int>> proposalsNeedingVote(List<VoteCheckTarget> targets) async {
    final active =
        targets.where((t) => t.status == 0 && t.adminWallets.isNotEmpty);

    final internalByPid = <int, List<String>>{};
    final jointByPid =
        <int, ({Uint8List institutionAccountId, List<String> pubkeysHex})>{};
    for (final t in active) {
      final pubkeys = t.adminWallets.map((w) => w.pubkeyHex).toList();
      if (t.kind == 0) {
        internalByPid[t.proposalId] = pubkeys;
      } else if (t.kind == 1 && t.institution != null) {
        final inst = Uint8List.fromList(institutionIdentityToAccountId(
          t.institution!.cidNumber,
          mainAccount: t.institution!.mainAccount,
        ));
        if (inst.length == 32) {
          jointByPid[t.proposalId] =
              (institutionAccountId: inst, pubkeysHex: pubkeys);
        }
      }
    }

    final needs = <int>{};
    if (internalByPid.isNotEmpty) {
      final votes =
          await _internalVoteService.fetchAdminVotesForProposals(internalByPid);
      internalByPid.forEach((pid, pubkeys) {
        final m = votes[pid] ?? const <String, bool?>{};
        if (pubkeys.any((pk) => m[_normalize(pk)] == null)) needs.add(pid);
      });
    }
    if (jointByPid.isNotEmpty) {
      final votes =
          await _runtimeService.fetchJointAdminVotesForProposals(jointByPid);
      jointByPid.forEach((pid, v) {
        final m = votes[pid] ?? const <String, bool?>{};
        if (v.pubkeysHex.any((pk) => m[_normalize(pk)] == null)) needs.add(pid);
      });
    }
    return needs;
  }

  static String _normalize(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return clean.toLowerCase();
  }
}

/// [VoteChecker.proposalsNeedingVote] 的输入项:一个提案的投票判定所需上下文。
class VoteCheckTarget {
  const VoteCheckTarget({
    required this.proposalId,
    required this.kind,
    required this.status,
    required this.adminWallets,
    this.institution,
  });

  final int proposalId;
  final int kind;
  final int status;
  final List<WalletProfile> adminWallets;
  final InstitutionInfo? institution;
}
