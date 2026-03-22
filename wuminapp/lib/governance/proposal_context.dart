import 'dart:convert';
import 'dart:typed_data';

import '../wallet/core/wallet_manager.dart';
import 'institution_admin_service.dart';
import 'institution_data.dart';
import 'runtime_upgrade_service.dart';
import 'transfer_proposal_service.dart';

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

  /// 是否为公民（有 SFID 绑定）。
  bool get isCitizen =>
      role == ProposalRole.citizen || role == ProposalRole.admin;

  /// 是否有导入的管理员钱包。
  bool get hasAdminWallets => adminWallets.isNotEmpty;
}

/// 用户角色。
enum ProposalRole {
  /// 管理员：有冷钱包匹配到链上管理员列表。
  admin,

  /// 公民：有 SFID 绑定的钱包（暂未实现，预留）。
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
  })  : _adminService = adminService ?? InstitutionAdminService(),
        _walletManager = walletManager ?? WalletManager();

  final InstitutionAdminService _adminService;
  final WalletManager _walletManager;

  /// 已缓存的钱包列表（同一次会话内复用）。
  List<WalletProfile>? _wallets;

  /// 静态缓存：用户已确认为管理员的机构 shenfenId 集合。
  ///
  /// 当用户浏览某个机构详情页后，如果匹配到管理员身份，
  /// 该机构 shenfenId 会被加入此集合。机构列表页据此渲染绿色卡片和排序。
  static final Set<String> _adminInstitutionIds = {};

  /// 记录某机构的管理员状态。
  static void markAdminInstitution(String shenfenId) {
    _adminInstitutionIds.add(shenfenId);
  }

  /// 查询某机构是否已确认为管理员。
  static bool isAdminInstitution(String shenfenId) {
    return _adminInstitutionIds.contains(shenfenId);
  }

  /// 获取所有已确认的管理员机构 shenfenId 集合。
  static Set<String> get adminInstitutionIds =>
      Set.unmodifiable(_adminInstitutionIds);

  /// 清除管理员机构缓存（如钱包被删除时）。
  static void clearAdminInstitutionCache() {
    _adminInstitutionIds.clear();
  }

  /// 解析用户与指定提案的关系。
  ///
  /// [institutionBytes] 提案的 48 字节机构标识（可为 null）。
  /// [knownInstitution] 如果调用方已知机构信息（如从机构页面进入），直接传入跳过反查。
  Future<ProposalContext> resolve({
    List<int>? institutionBytes,
    InstitutionInfo? knownInstitution,
  }) async {
    final wallets = await _getWallets();
    final coldWallets = wallets.where((w) => w.isColdWallet).toList();

    // 1. 确定机构
    InstitutionInfo? institution = knownInstitution;

    if (institution == null && institutionBytes != null) {
      institution = findInstitutionByPalletId(institutionBytes);
    }

    // 2. 如果仍然没有机构（如联合投票 institutionBytes 反查失败），
    //    遍历用户所有冷钱包，逐个查链上管理员列表反向匹配。
    if (institution == null && coldWallets.isNotEmpty) {
      institution = await _reverseMatchInstitution(coldWallets);
    }

    // 3. 匹配管理员钱包
    if (institution == null) {
      return const ProposalContext();
    }

    List<String> admins;
    try {
      admins = await _adminService.fetchAdmins(institution.shenfenId);
    } catch (_) {
      admins = const [];
    }

    final matchedWallets = <WalletProfile>[];
    for (final w in coldWallets) {
      final pk = _normalize(w.pubkeyHex);
      if (admins.contains(pk)) {
        matchedWallets.add(w);
      }
    }

    return ProposalContext(
      institution: institution,
      adminWallets: matchedWallets,
      role: matchedWallets.isNotEmpty
          ? ProposalRole.admin
          : ProposalRole.viewer,
    );
  }

  /// 批量解析多个提案的上下文（用于列表页）。
  ///
  /// 返回 Map<提案索引, ProposalContext>，与传入列表一一对应。
  Future<List<ProposalContext>> resolveBatch(
    List<List<int>?> institutionBytesList,
  ) async {
    final wallets = await _getWallets();
    final coldWallets = wallets.where((w) => w.isColdWallet).toList();
    final results = <ProposalContext>[];

    for (final institutionBytes in institutionBytesList) {
      InstitutionInfo? institution;

      if (institutionBytes != null) {
        institution = findInstitutionByPalletId(institutionBytes);
      }

      if (institution == null && coldWallets.isNotEmpty) {
        institution = await _reverseMatchInstitution(coldWallets);
      }

      if (institution == null) {
        results.add(const ProposalContext());
        continue;
      }

      List<String> admins;
      try {
        admins = await _adminService.fetchAdmins(institution.shenfenId);
      } catch (_) {
        admins = const [];
      }

      final matchedWallets = <WalletProfile>[];
      for (final w in coldWallets) {
        final pk = _normalize(w.pubkeyHex);
        if (admins.contains(pk)) {
          matchedWallets.add(w);
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

  // ---------------------------------------------------------------------------
  // 内部方法
  // ---------------------------------------------------------------------------

  Future<List<WalletProfile>> _getWallets() async {
    _wallets ??= await _walletManager.getWallets();
    return _wallets!;
  }

  /// 反向匹配：遍历所有机构，查管理员列表，看用户冷钱包是否在其中。
  Future<InstitutionInfo?> _reverseMatchInstitution(
    List<WalletProfile> coldWallets,
  ) async {
    final allInstitutions = [
      ...kNationalCouncil,
      ...kProvincialCouncils,
      ...kProvincialBanks,
    ];

    final coldPubkeys = coldWallets
        .map((w) => _normalize(w.pubkeyHex))
        .toSet();

    for (final inst in allInstitutions) {
      List<String> admins;
      try {
        admins = await _adminService.fetchAdmins(inst.shenfenId);
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
}

/// 统一投票状态检查器。
///
/// 根据提案类型（kind）和阶段（stage）自动选择正确的链上存储查询，
/// 避免不同入口查错存储导致的状态不一致。
class VoteChecker {
  VoteChecker({
    TransferProposalService? proposalService,
    RuntimeUpgradeService? runtimeService,
  })  : _proposalService = proposalService ?? TransferProposalService(),
        _runtimeService = runtimeService ?? RuntimeUpgradeService();

  final TransferProposalService _proposalService;
  final RuntimeUpgradeService _runtimeService;

  /// 检查某管理员是否已对某提案投票。
  ///
  /// 根据 [kind] 自动选择正确的链上存储：
  /// - kind=0（内部投票）→ InternalVotesByAccount
  /// - kind=1（联合投票）→ JointVotesByAdmin（需要 [institutionBytes]）
  ///
  /// 返回：null=未投票，true=赞成，false=反对。
  Future<bool?> fetchVote({
    required int proposalId,
    required String pubkeyHex,
    required int kind,
    Uint8List? institutionBytes,
  }) async {
    final pk = _normalize(pubkeyHex);
    switch (kind) {
      case 0: // 内部投票
        return _proposalService.fetchAdminVote(proposalId, pk);
      case 1: // 联合投票
        if (institutionBytes == null || institutionBytes.length != 48) {
          return null;
        }
        return _runtimeService.fetchJointAdminVote(
          proposalId,
          institutionBytes,
          pk,
        );
      default:
        return null;
    }
  }

  /// 检查用户的管理员钱包中是否有未投票的。
  ///
  /// 用于列表红点判断，统一替代各处手动查询。
  Future<bool> hasUnvotedWallet({
    required int proposalId,
    required int kind,
    required List<WalletProfile> adminWallets,
    InstitutionInfo? institution,
  }) async {
    if (adminWallets.isEmpty) return false;

    Uint8List? institutionBytes;
    if (kind == 1 && institution != null) {
      institutionBytes = _shenfenIdToFixed48(institution.shenfenId);
    }

    for (final w in adminWallets) {
      final vote = await fetchVote(
        proposalId: proposalId,
        pubkeyHex: w.pubkeyHex,
        kind: kind,
        institutionBytes: institutionBytes,
      );
      if (vote == null) return true; // 有未投票的
    }
    return false;
  }

  /// 将 shenfen_id 编码为固定 48 字节。
  static Uint8List _shenfenIdToFixed48(String shenfenId) {
    final raw = utf8.encode(shenfenId);
    final out = Uint8List(48);
    out.setAll(0, raw);
    return out;
  }

  static String _normalize(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    return clean.toLowerCase();
  }
}
