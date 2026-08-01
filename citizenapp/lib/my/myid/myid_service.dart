import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/foundation.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/log/app_log.dart';
import 'package:citizenapp/citizen/public/data/admin_division_store.dart';
import 'package:citizenapp/citizen/public/data/area_path_formatter.dart';
import 'package:citizenapp/citizen/public/data/isar_admin_division_store.dart';
import 'package:citizenapp/citizen/public/data/public_provinces.dart';
import 'package:citizenapp/citizen/cid/cid_generator.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/citizen_identity_rpc.dart';
import 'package:citizenapp/security/local_data_key.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

import 'identity_account_resolver.dart';
import 'identity_badge_snapshot_store.dart';

/// 身份页身份档。
///
/// 身份钥匙 = **CID 绑定的钱包账户**(经 [IdentityAccountResolver] 解析:优先账户0、
/// 未命中查子账户;见 memory `citizenapp-cid-identity-master-key`)。身份主键是 CID 号,
/// 绑定账户切换(换绑)即跟随;链上一人一 CID 一账户一身份,故无多身份冲突。
enum MyIdTier {
  /// 访客(默认匿名)。含两子态,均用同一张访客卡、同一枚访客徽章:
  /// - **纯访客**:账户从未占号(`cidNumber` 为空)。
  /// - **匿名已注册**:账户自助占了一个 CID 并双向绑定,但链上无 `VotingIdentityByCid`
  ///   (`cidNumber` 非空,见 [MyIdState.isAnonymousRegistered])。仅在第 1 卡展示该 CID 号,
  ///   不升级徽章色/卡片(投票/竞选身份只能经注册局线下升级)。
  visitor,

  /// 钱包与永久 CID 双向绑定闭环完整，且存在 `VotingIdentityByCid`。
  voting,

  /// 在投票公民之上再有 `CandidateIdentityByCid` 对应的竞选身份。
  candidate,
}

/// 护照有效期/生命周期状态(仅公民档有意义;`queryFailed` 为链读失败兜底)。
enum MyIdStatus { normal, notYetValid, expired, revoked, queryFailed }

/// 身份页只读链上状态(身份账户维度)。
class MyIdState {
  const MyIdState({
    required this.tier,
    this.status,
    this.votingAccountId,
    this.cidNumber,
    this.residenceDistrict,
    this.passportValidFrom,
    this.passportValidUntil,
    this.familyName,
    this.givenName,
    this.citizenSexLabel,
    this.birthDistrict,
    this.citizenBirthDate,
    this.errorMessage,
  });

  final MyIdTier tier;

  /// 公民档的护照状态;访客为 null,链读失败为 [MyIdStatus.queryFailed]。
  final MyIdStatus? status;

  /// 链上投票绑定账户 = CID 绑定账户地址(SS58)。访客不显示,为 null。
  final String? votingAccountId;
  final String? cidNumber;

  /// 预 join 的居住选区「省·市·镇」(service 层查字典拼好,UI 直接展示)。
  final String? residenceDistrict;

  /// YYYY-MM-DD,来自链上 YYYYMMDD 整数。
  final String? passportValidFrom;
  final String? passportValidUntil;

  // ── 竞选公民专属公开字段 ──
  final String? familyName;
  final String? givenName;

  /// 男/女。
  final String? citizenSexLabel;
  final String? birthDistrict;

  /// 出生日期 YYYY-MM-DD,来自链上竞选身份 birth_date(YYYYMMDD 整数)。
  final String? citizenBirthDate;

  final String? errorMessage;

  bool get isCitizen => tier == MyIdTier.voting || tier == MyIdTier.candidate;

  /// 访客卡的「已占匿名 CID」子态:访客档 + 已绑定 CID 号(无投票身份)。
  /// UI 据此在第 1 卡展示 CID 号、把右上按钮从「注册」切成「更换」。
  bool get isAnonymousRegistered =>
      tier == MyIdTier.visitor && (cidNumber?.trim().isNotEmpty ?? false);

  /// 徽章分色信号:visitor/voting/candidate,与 [IdentityBadgeSnapshotStore] 契约一致。
  String get identityLevel => switch (tier) {
        MyIdTier.candidate => 'candidate',
        MyIdTier.voting => 'voting',
        MyIdTier.visitor => 'visitor',
      };
}

class MyIdService {
  MyIdService({
    WalletManager? walletManager,
    ChainRpc? chainRpc,
    AdminDivisionStore? divisionStore,
    IdentityBadgeSnapshotStore? badgeSnapshotStore,
    IdentityAccountResolver? identityResolver,
    CitizenIdentityRpc? identityRpc,
    SquareApiClient? squareApiClient,
    DateTime Function()? nowProvider,
    int Function()? cidYearProvider,
  })  : _walletManager = walletManager ?? WalletManager(),
        _divisionStore = divisionStore ?? IsarAdminDivisionStore(),
        _badgeSnapshotStore =
            badgeSnapshotStore ?? IdentityBadgeSnapshotStore(),
        _identityResolver = identityResolver ??
            IdentityAccountResolver(
              walletManager: walletManager,
              chainRpc: chainRpc,
            ),
        _identityRpc = identityRpc ??
            CitizenIdentityRpc(
              chainRpc: chainRpc,
              walletManager: walletManager,
            ),
        _squareApiClient = squareApiClient ?? SquareApiClient(),
        _chainRpc = chainRpc ?? ChainRpc(),
        _nowProvider = nowProvider ?? _beijingNow,
        _cidYearProvider = cidYearProvider ?? _utcYear;

  final WalletManager _walletManager;
  final AdminDivisionStore _divisionStore;
  final IdentityBadgeSnapshotStore _badgeSnapshotStore;
  final IdentityAccountResolver _identityResolver;
  final CitizenIdentityRpc _identityRpc;
  final SquareApiClient _squareApiClient;

  /// 子钥懒绑定的并发去重:五处门禁可能同时挂载,只允许一次真正执行。
  Future<void>? _subkeyBindInflight;

  final ChainRpc _chainRpc;

  final DateTime Function() _nowProvider;
  final int Function() _cidYearProvider;

  /// 链上护照有效期窗口按 UTC+8 判定(与 runtime `can_vote` 口径一致),
  /// 避免本机时区在跨日边界把"今天"算差一天。
  static DateTime _beijingNow() =>
      DateTime.now().toUtc().add(const Duration(hours: 8));

  /// 自助占号的 CID 年份取 **UTC 当前年**(与 CID 生成金标口径一致,不随本机时区漂移)。
  static int _utcYear() => DateTime.now().toUtc().year;

  /// 读取当前身份账户(CID 绑定账户)的身份状态。
  ///
  /// 身份主键 = CID 号;先经 [IdentityAccountResolver] 解析出 CID 绑定的钱包账户
  /// (优先账户0,未命中查子账户,全未命中回退账户0=未注册),再取其链上 finalized
  /// 身份闭环:CID Active、CID↔账户双向绑定、`VotingIdentityByCid`(+`Candidate` 才竞选)。
  Future<MyIdState> getState() async {
    ResolvedIdentity? resolved;
    try {
      resolved = await _identityResolver.resolve();
    } catch (e) {
      AppLog.d('myid identity resolve failed: $e');
      // 链读失败不静默降级访客、不覆盖徽章快照,交由 UI 提示重试。
      return const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '链上身份读取失败',
      );
    }
    if (resolved == null) {
      // 无热钱包 → 无身份账户 → 访客(引导创建钱包)。
      return const MyIdState(tier: MyIdTier.visitor, errorMessage: '请先创建钱包');
    }

    final identityAccountId = resolved.accountId;
    final chainIdentity = resolved.snapshot;

    if (chainIdentity == null) {
      return const MyIdState(tier: MyIdTier.visitor);
    }

    if (chainIdentity.isAnonymous) {
      // 匿名已注册:访客卡 + 展示 CID;徽章仍访客色(决策:不新增卡/色)。
      await _persistBadgeSnapshot(chainIdentity.cidNumber, 'visitor');
      return MyIdState(
        tier: MyIdTier.visitor,
        votingAccountId: identityAccountId,
        cidNumber: chainIdentity.cidNumber,
      );
    }

    final voting = _decodeVotingIdentity(chainIdentity.votingIdentity!);
    if (voting == null) {
      // 有记录但解不开 = 数据异常,不静默当访客。
      return const MyIdState(
        tier: MyIdTier.visitor,
        status: MyIdStatus.queryFailed,
        errorMessage: '身份数据解析失败',
      );
    }

    final status = _deriveStatus(voting);
    final residence = await _resolveDistrict(
      voting.resProvince,
      voting.resCity,
      voting.resTown,
    );

    final candidateRaw = chainIdentity.candidateIdentity;
    final candidate =
        candidateRaw == null ? null : _decodeCandidateIdentity(candidateRaw);
    final tier = candidate != null ? MyIdTier.candidate : MyIdTier.voting;
    await _persistBadgeSnapshot(
      chainIdentity.cidNumber,
      tier == MyIdTier.candidate ? 'candidate' : 'voting',
    );

    final birth = candidate == null
        ? null
        : await _resolveDistrict(
            candidate.birthProvince,
            candidate.birthCity,
            candidate.birthTown,
          );

    return MyIdState(
      tier: tier,
      status: status,
      votingAccountId: identityAccountId,
      cidNumber: chainIdentity.cidNumber,
      residenceDistrict: residence,
      passportValidFrom: _formatDateInt(voting.passportValidFrom),
      passportValidUntil: _formatDateInt(voting.passportValidUntil),
      familyName: candidate?.familyName,
      givenName: candidate?.givenName,
      citizenSexLabel:
          candidate == null ? null : (candidate.sex == 1 ? '女' : '男'),
      birthDistrict: birth,
      citizenBirthDate:
          candidate == null ? null : _formatDateInt(candidate.birthDate),
    );
  }

  /// 自助占一个匿名 CID,把它绑定到用户所选的钱包账户 [bindAccountId](null = 账户0),
  /// 返回占用的 CID 号。
  ///
  /// 身份主键 = CID 号;绑定账户是用户自选的鉴权凭证(可任意 `//n`,私钥泄漏可换绑)。
  /// [institution] 取 [kCidInstitutionCitizen](公民)/ [kCidInstitutionResident](居民)。
  /// 年份取 UTC 当前年;CID = f(绑定 accountId, institution, year),撞号由链端 registry
  /// 兜底吸收。提交经 `self_occupy_cid` 由绑定账户自签自付费(触发一次生物识别);成功后
  /// 广播身份绑定变化,常驻页重读身份。
  Future<String> registerAnonymousCid({
    required String institution,
    String? bindAccountId,
  }) async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) {
      throw const WalletAuthException('无热钱包,请先创建钱包');
    }
    // 绑定账户:默认账户0(=当前热钱包),或用户所选的本地账户。
    final String resolvedBindAccountId;
    final String bindSs58;
    if (bindAccountId == null || bindAccountId == wallet.accountId) {
      resolvedBindAccountId = wallet.accountId;
      bindSs58 = wallet.ss58Address;
    } else {
      final account = await _walletManager.getAccountByAccountId(bindAccountId);
      if (account == null) {
        throw const WalletAuthException('绑定账户不存在');
      }
      resolvedBindAccountId = account.accountId;
      bindSs58 = account.ss58Address;
    }
    final cid = generateCitizenCid(
      accountId: resolvedBindAccountId,
      institution: institution,
      year: _cidYearProvider(),
    );
    await _identityRpc.selfOccupyCid(
      cidNumber: cid,
      accountId: resolvedBindAccountId,
      fromSs58Address: bindSs58,
    );
    WalletManager.notifyIdentityBindingChanged();
    final finalized = await _requireFinalizedBinding(
      cidNumber: cid,
      accountId: resolvedBindAccountId,
    );
    await _ensureBindingReady(finalized);
    return cid;
  }

  /// 把当前身份 CID [cidNumber] 换绑到另一本地账户 [newAccountId]。
  ///
  /// 旧账户 = **当前 CID 绑定账户**(经 [IdentityAccountResolver] 解析,可为任意 `//n`,
  /// 非恒账户0),对含创世、当前绑定、revision 与 expires_at 的授权载荷签名;新账户自签
  /// 提交 `self_rebind_cid_account_id`(旧、新各触发一次生物识别)。换绑成功后 CID 归
  /// 新账户、广播身份绑定变化,常驻页/身份页跟随。
  /// 仅**匿名 CID** 可自助换绑;投票/竞选链端强制走注册局(`CivicRebindRequiresRegistrar`)。
  Future<void> rebindCidTo({
    required String cidNumber,
    required String newAccountId,
  }) async {
    final resolved = await _identityResolver.resolve();
    if (resolved == null || !resolved.isRegistered) {
      throw const WalletAuthException('当前无已注册身份,无法换绑');
    }
    final oldAccountId = resolved.accountId;
    final resolvedCidNumber = resolved.snapshot?.cidNumber;
    if (resolvedCidNumber == null || resolvedCidNumber != cidNumber) {
      throw const WalletAuthException('当前链上身份与待换绑 CID 不一致');
    }
    final newAccount = await _walletManager.getAccountByAccountId(newAccountId);
    if (newAccount == null) {
      throw const WalletAuthException('目标账户不存在');
    }
    if (newAccount.accountId == oldAccountId) {
      throw const WalletAuthException('目标账户与当前身份账户相同');
    }
    await _identityRpc.selfRebindCidAccount(
      cidNumber: cidNumber,
      newAccountId: newAccount.accountId,
      oldAccountId: oldAccountId,
      newFromSs58Address: newAccount.ss58Address,
    );
    // 当前账户授权已由 runtime 交易验证。finalized 后进入第二阶段：新账户独立接管
    // CID 数据根与设备子钥，不读取或要求此前账户、此前私钥或此前设备参与。
    final finalized = await _requireFinalizedBinding(
      cidNumber: cidNumber,
      accountId: newAccount.accountId,
    );
    await _ensureBindingReady(finalized);
  }

  /// 注册身份前的自付能力测算:返回门槛与该账户当前余额(均为**分**)。
  ///
  /// 门槛 = 链上 `OnchainMinFee + ExistentialDeposit`,两个数都现取自链上 metadata——
  /// 交易费常量真源恒在区块链常量库,App 侧不留任何副本。余额走 `forceFresh` 绕开块内
  /// 缓存,否则会拿到充值前的旧值,把刚充完钱的用户又踢回充值页。
  ///
  /// 链读失败**不吞**:上抛给调用方 fail-closed 处理,绝不静默当成「余额不足」或「充足」。
  Future<({BigInt requiredFen, BigInt balanceFen})>
      fetchRegistrationAffordability(
    String bindAccountId,
  ) async {
    final requiredFen = await _chainRpc.fetchMinSelfPayBalanceFen();
    final balanceYuan =
        await _chainRpc.fetchFinalizedBalance(bindAccountId, forceFresh: true);
    return (
      requiredFen: requiredFen,
      balanceFen: BigInt.from((balanceYuan * 100).round()),
    );
  }

  /// 确保当前 finalized 绑定在本机就位：CID 数据根 + 本设备 P-256 子钥。
  ///
  /// **懒执行**：两者只服务广场 / 聊天 / 通讯录 / 创作者 / 会员这些需 CID 的场景，
  /// 故不在建钱包时做（那时账户还没有 CID），而由 `IdentityRegistrationGate` 在用户
  /// 初次进入上述页面时调用本方法。只用钱包和交易的用户永远不会走到这里。
  ///
  /// 数据根属于 CID 且不从钱包派生。本机没有当前绑定包装时，由 finalized 当前账户
  /// 签一次性恢复挑战取得同一数据根；子钥绑定会再触发一次当前账户验证。
  ///
  /// 并发去重:五处门禁可能同时挂载,用 [_subkeyBindInflight] 保证同一时刻只跑一次,
  /// 其余调用等同一个 Future。失败向上抛,由门禁按 fail-closed 拦住功能页并给重试。
  Future<void> ensureDeviceSubkeyBound() {
    final inflight = _subkeyBindInflight;
    if (inflight != null) return inflight;
    final future = _doEnsureDeviceSubkeyBound()
        .whenComplete(() => _subkeyBindInflight = null);
    _subkeyBindInflight = future;
    return future;
  }

  Future<void> _doEnsureDeviceSubkeyBound() async {
    final resolved = await _identityResolver.resolve();
    if (resolved == null || !resolved.isRegistered) {
      // 无热钱包或尚未占到 CID:后端不收未绑 CID 账户的子钥,此处不做无谓尝试。
      throw const WalletAuthException('当前无已注册身份,无法绑定设备');
    }
    await _ensureBindingReady(resolved);
  }

  /// 全步幂等：数据根已激活到同一三元组、子钥已登记时都不重复执行。
  /// 正确顺序是恢复授权 → 新账户包装读回 → 用途子钥落地/旧本地包装清理 →
  /// 新设备子钥登记/旧服务端凭证清理。
  Future<void> _ensureBindingReady(ResolvedIdentity resolved) async {
    final snapshot = resolved.snapshot;
    if (snapshot == null) {
      throw const WalletAuthException('当前无已注册身份，无法完成设备绑定');
    }
    try {
      await _walletManager.ensureCidDataRootReady(
        cidNumber: snapshot.cidNumber,
        bindingRevision: snapshot.bindingRevision,
        accountId: resolved.accountId,
      );
    } on CidDataRootRecoveryRequiredException {
      final genesisHash = await _chainRpc.fetchGenesisHash();
      if (genesisHash.length != 32) {
        throw const WalletAuthException('创世哈希无效，禁止恢复 CID 数据根');
      }
      final grant = await _squareApiClient.takeoverCidDataRoot(
        cidNumber: snapshot.cidNumber,
        bindingRevision: snapshot.bindingRevision,
        accountId: resolved.accountId,
        expectedGenesisHash: '0x${_bytesToHex(genesisHash)}',
        signAction: (message) async => _signatureHex(
          await _walletManager.signForAccountId(resolved.accountId, message),
        ),
      );
      try {
        await _walletManager.installCidDataRootForCurrentBinding(
          cidNumber: grant.cidNumber,
          bindingRevision: grant.bindingRevision,
          accountId: grant.accountId,
          dataRoot: CidDataRoot(grant.dataRoot),
          dataRootHash: grant.dataRootHash,
        );
      } finally {
        grant.dataRoot.fillRange(0, grant.dataRoot.length, 0);
      }
    }
    await _walletManager.bindDeviceSubkeyToCurrentBinding(
      cidNumber: snapshot.cidNumber,
      bindingRevision: snapshot.bindingRevision,
      accountId: resolved.accountId,
    );
    WalletManager.notifyIdentityBindingChanged();
  }

  static String _signatureHex(Uint8List signature) =>
      '0x${_bytesToHex(signature)}';

  static String _bytesToHex(List<int> bytes) {
    const alphabet = '0123456789abcdef';
    final output = StringBuffer();
    for (final byte in bytes) {
      output
        ..write(alphabet[(byte >> 4) & 0x0f])
        ..write(alphabet[byte & 0x0f]);
    }
    return output.toString();
  }

  Future<ResolvedIdentity> _requireFinalizedBinding({
    required String cidNumber,
    required String accountId,
  }) async {
    final resolved = await _identityResolver.resolve();
    final snapshot = resolved?.snapshot;
    if (resolved == null ||
        snapshot == null ||
        snapshot.cidNumber != cidNumber ||
        resolved.accountId != accountId) {
      throw const WalletAuthException('finalized CID 当前绑定与预期不一致');
    }
    return resolved;
  }

  /// 列出可作换绑目标的本地账户(当前身份账户以外的全部账户)。
  Future<List<Account>> listRebindTargets() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) return const <Account>[];
    final resolved = await _identityResolver.resolve();
    final currentIdentityAccountId = resolved?.accountId ?? wallet.accountId;
    final accounts = await _walletManager.getAccounts(wallet.accountId);
    return accounts
        .where((account) => account.accountId != currentIdentityAccountId)
        .toList(growable: false);
  }

  /// 列出注册 CID 时可选的绑定账户(当前热钱包下全部本地账户,含账户0)。
  Future<List<Account>> listBindableAccounts() async {
    final wallet = await _walletManager.getDefaultWallet();
    if (wallet == null) return const <Account>[];
    return _walletManager.getAccounts(wallet.accountId);
  }

  /// 把三段行政区码预 join 成「省·市·镇」展示串;省码空则返回空串。
  ///
  /// [formatAreaPath] 内部字典缺失会回退显 code(绝不崩、绝不空);再包一层
  /// 兜底防字典异常,避免选区展示阻断整卡。
  Future<String> _resolveDistrict(
    String province,
    String city,
    String town,
  ) async {
    if (province.isEmpty) return '';
    try {
      return await formatAreaPath(
        _divisionStore,
        provinceName: provinceDisplayNameByCode(province),
        provinceCode: province,
        cityCode: city,
        townCode: town,
      );
    } catch (e) {
      AppLog.d('myid area path resolve failed: $e');
      return [provinceDisplayNameByCode(province), city, town]
          .where((s) => s.isNotEmpty)
          .join(' · ');
    }
  }

  /// 写永久 CID 的身份徽章快照，供非链页面（个人页/广场）展示，不作权限依据。
  Future<void> _persistBadgeSnapshot(String cidNumber, String level) async {
    try {
      await _badgeSnapshotStore.write(
        cidNumber: cidNumber,
        identityLevel: level,
      );
    } catch (e) {
      // 快照只服务展示,写失败不改变本次真实链查询结果。
      AppLog.d('myid badge snapshot save failed: $e');
    }
  }

  MyIdStatus _deriveStatus(_VotingIdentity identity) {
    if (identity.citizenStatus == _CitizenStatus.revoked) {
      return MyIdStatus.revoked;
    }
    final today = _dateInt(_nowProvider());
    if (today < identity.passportValidFrom) return MyIdStatus.notYetValid;
    if (today > identity.passportValidUntil) return MyIdStatus.expired;
    return MyIdStatus.normal;
  }

  /// 解码链上 `VotingIdentity<BlockNumber>`,字段序与
  /// `citizenchain/runtime/misc/citizen-identity/src/lib.rs` 逐字节一致:
  /// valid_from(u32) + valid_until(u32) + status(u8) + residence_省/市/镇码
  /// + updated_at(u32)。永久 CID 只存在于 storage key，不在值中重复保存。
  _VotingIdentity? _decodeVotingIdentity(Uint8List data) {
    try {
      var offset = 0;
      if (offset + 4 + 4 + 1 > data.length) return null;
      final validFrom = _readU32Le(data, offset);
      offset += 4;
      final validUntil = _readU32Le(data, offset);
      offset += 4;
      if (!_isValidDateInt(validFrom) || !_isValidDateInt(validUntil)) {
        return null;
      }
      final status = switch (data[offset]) {
        0 => _CitizenStatus.normal,
        1 => _CitizenStatus.revoked,
        _ => null,
      };
      if (status == null) return null;
      offset += 1;
      // 居住 3 码允许空(区划码可能只到市;空段绝不能把整条身份误判为不存在)。
      final prov = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = prov.nextOffset;
      final city = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = city.nextOffset;
      final town = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = town.nextOffset;
      // updated_at(BlockNumber=u32):只校验尾部存在,展示不使用。
      if (offset + 4 > data.length) return null;
      return _VotingIdentity(
        passportValidFrom: validFrom,
        passportValidUntil: validUntil,
        citizenStatus: status,
        resProvince: prov.value,
        resCity: city.value,
        resTown: town.value,
      );
    } catch (_) {
      return null;
    }
  }

  /// 解码链上 `CandidateIdentity<BlockNumber>`(增量存储,不含 voting 基础字段):
  /// birth_省/市/镇码 + family_name + given_name + citizen_sex(u8,0男1女)
  /// + birth_date(u32 YYYYMMDD) + updated_at(u32)。
  _CandidateIdentity? _decodeCandidateIdentity(Uint8List data) {
    try {
      var offset = 0;
      final prov = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = prov.nextOffset;
      final city = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = city.nextOffset;
      final town = _readUtf8VecAllowEmpty(data, offset, maxLen: 16);
      offset = town.nextOffset;
      final familyName = _readUtf8VecAllowEmpty(data, offset, maxLen: 128);
      offset = familyName.nextOffset;
      final givenName = _readUtf8VecAllowEmpty(data, offset, maxLen: 128);
      offset = givenName.nextOffset;
      if (offset + 1 > data.length) return null;
      final sex = data[offset];
      offset += 1;
      if (sex != 0 && sex != 1) return null;
      // birth_date(u32 YYYYMMDD) + 尾部 updated_at(u32)。
      if (offset + 4 > data.length) return null;
      final birthDate = _readU32Le(data, offset);
      offset += 4;
      if (!_isValidDateInt(birthDate)) return null;
      if (offset + 4 > data.length) return null;
      return _CandidateIdentity(
        birthProvince: prov.value,
        birthCity: city.value,
        birthTown: town.value,
        familyName: familyName.value,
        givenName: givenName.value,
        sex: sex,
        birthDate: birthDate,
      );
    } catch (_) {
      return null;
    }
  }

  /// 读 `BoundedVec<u8>`,允许空(长度 0 返回空串)。区划码/姓名用。
  static ({String value, int nextOffset}) _readUtf8VecAllowEmpty(
    Uint8List data,
    int offset, {
    required int maxLen,
  }) {
    final (length, lengthSize) = _readCompactU32(data, offset);
    final start = offset + lengthSize;
    final end = start + length;
    if (length < 0 || length > maxLen || end > data.length) {
      throw const FormatException('BoundedVec 长度不合法');
    }
    if (length == 0) return (value: '', nextOffset: start);
    return (
      value: utf8.decode(data.sublist(start, end), allowMalformed: false),
      nextOffset: end,
    );
  }

  static (int, int) _readCompactU32(Uint8List data, int offset) {
    if (offset >= data.length) {
      throw const FormatException('Compact<u32> offset 越界');
    }
    final first = data[offset];
    final mode = first & 0x03;
    if (mode == 0) return (first >> 2, 1);
    if (mode == 1) {
      if (offset + 1 >= data.length) {
        throw const FormatException('Compact<u32> mode1 长度不足');
      }
      return ((first >> 2) | (data[offset + 1] << 6), 2);
    }
    if (mode == 2) {
      if (offset + 3 >= data.length) {
        throw const FormatException('Compact<u32> mode2 长度不足');
      }
      return (
        (first >> 2) |
            (data[offset + 1] << 6) |
            (data[offset + 2] << 14) |
            (data[offset + 3] << 22),
        4,
      );
    }
    throw const FormatException('Compact<u32> big-integer 模式暂不支持');
  }

  static int _readU32Le(Uint8List data, int offset) {
    return data[offset] |
        (data[offset + 1] << 8) |
        (data[offset + 2] << 16) |
        (data[offset + 3] << 24);
  }

  static bool _isValidDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value ~/ 100) % 100;
    final day = value % 100;
    return year >= 1900 &&
        year <= 9999 &&
        month >= 1 &&
        month <= 12 &&
        day >= 1 &&
        day <= 31;
  }

  static int _dateInt(DateTime value) =>
      value.year * 10000 + value.month * 100 + value.day;

  static String _formatDateInt(int value) {
    final year = value ~/ 10000;
    final month = (value ~/ 100) % 100;
    final day = value % 100;
    return '${year.toString().padLeft(4, '0')}-'
        '${month.toString().padLeft(2, '0')}-'
        '${day.toString().padLeft(2, '0')}';
  }
}

class _VotingIdentity {
  const _VotingIdentity({
    required this.passportValidFrom,
    required this.passportValidUntil,
    required this.citizenStatus,
    required this.resProvince,
    required this.resCity,
    required this.resTown,
  });

  final int passportValidFrom;
  final int passportValidUntil;
  final _CitizenStatus citizenStatus;
  final String resProvince;
  final String resCity;
  final String resTown;
}

class _CandidateIdentity {
  const _CandidateIdentity({
    required this.birthProvince,
    required this.birthCity,
    required this.birthTown,
    required this.familyName,
    required this.givenName,
    required this.sex,
    required this.birthDate,
  });

  final String birthProvince;
  final String birthCity;
  final String birthTown;
  final String familyName;
  final String givenName;
  final int sex;

  /// 出生日期(YYYYMMDD 整数),竞选身份专属。
  final int birthDate;
}

enum _CitizenStatus { normal, revoked }
