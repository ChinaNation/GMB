import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:citizenapp/log/app_log.dart';
import 'package:citizenapp/citizen/public/data/admin_division_store.dart';
import 'package:citizenapp/citizen/public/data/area_path_formatter.dart';
import 'package:citizenapp/citizen/public/data/isar_admin_division_store.dart';
import 'package:citizenapp/citizen/public/data/public_provinces.dart';
import 'package:citizenapp/citizen/cid/cid_generator.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';
import 'package:citizenapp/rpc/citizen_identity_rpc.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/my/user/contact_service.dart';

import 'identity_account_resolver.dart';
import 'identity_badge_snapshot_store.dart';
import 'identity_rebind_revoker.dart';
import 'identity_synced_account_store.dart';

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
    UserContactService? contactService,
    IdentitySyncedAccountStore? syncedAccountStore,
    IdentityRebindRevoker? rebindRevoker,
    DateTime Function()? nowProvider,
    int Function()? cidYearProvider,
  })  : _walletManager = walletManager ?? WalletManager(),
        _divisionStore = divisionStore ?? IsarAdminDivisionStore(),
        _badgeSnapshotStore =
            badgeSnapshotStore ?? IdentityBadgeSnapshotStore(),
        _contactService =
            contactService ?? UserContactService(walletManager: walletManager),
        _syncedAccountStore =
            syncedAccountStore ?? IdentitySyncedAccountStore(),
        _rebindRevoker = rebindRevoker ??
            IdentityRebindRevoker(walletManager: walletManager),
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
        _chainRpc = chainRpc ?? ChainRpc(),
        _nowProvider = nowProvider ?? _beijingNow,
        _cidYearProvider = cidYearProvider ?? _utcYear;

  final WalletManager _walletManager;
  final AdminDivisionStore _divisionStore;
  final IdentityBadgeSnapshotStore _badgeSnapshotStore;
  final IdentityAccountResolver _identityResolver;
  final CitizenIdentityRpc _identityRpc;
  final UserContactService _contactService;
  final IdentitySyncedAccountStore _syncedAccountStore;
  final IdentityRebindRevoker _rebindRevoker;
  final ChainRpc _chainRpc;

  /// 换绑本地重建的并发去重:同一时刻只允许一次真正执行,其余调用等待同一 Future
  /// (rebindCidTo 直触发与 getState 对账触发可能并发,防重复触发互相 409 伪失败)。
  Future<void>? _migrationInflight;

  /// 子钥懒绑定的并发去重:五处门禁可能同时挂载,只允许一次真正执行。
  Future<void>? _subkeyBindInflight;

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

    // 对账式换绑重建:链上身份账户与「本地已同步账户」标记不一致即补齐迁移(死契约
    // [[cid-rebind-subkeys-must-auto-migrate]],替代脆弱的一次性意图,消除崩溃窗口)。
    await _reconcileIdentityRebuild(resolved);

    final identityAccountId = resolved.accountId;
    final chainIdentity = resolved.snapshot;

    if (chainIdentity == null) {
      await _persistBadgeSnapshot(identityAccountId, 'visitor');
      return const MyIdState(tier: MyIdTier.visitor);
    }

    if (chainIdentity.isAnonymous) {
      // 匿名已注册:访客卡 + 展示 CID;徽章仍访客色(决策:不新增卡/色)。
      await _persistBadgeSnapshot(identityAccountId, 'visitor');
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
      identityAccountId,
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
    return cid;
  }

  /// 把当前身份 CID [cidNumber] 换绑到另一本地账户 [newAccountId]。
  ///
  /// 旧账户 = **当前 CID 绑定账户**(经 [IdentityAccountResolver] 解析,可为任意 `//n`,
  /// 非恒账户0),对 `(cid,new)` 授权;新账户自签提交 `self_rebind_cid_account`(旧、新各
  /// 触发一次生物识别)。换绑成功后 CID 归新账户、广播身份绑定变化,常驻页/身份页跟随。
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
    // 先尝试补齐上一笔已经换绑成功的安全清理；若上一笔仍未完成且不是同一三元组，
    // 必须 fail-closed，禁止 A→B→C 连续换绑让旧授权失去精确目标。
    await _rebindRevoker.retryPendingCleanup(
      cidNumber: cidNumber,
      currentAccountId: oldAccountId,
    );
    await _rebindRevoker.ensureCanStartRebind(
      cidNumber: cidNumber,
      oldAccountId: oldAccountId,
      newAccountId: newAccount.accountId,
    );
    await _identityRpc.selfRebindCidAccount(
      cidNumber: cidNumber,
      newAccountId: newAccount.accountId,
      oldAccountId: oldAccountId,
      newFromSs58Address: newAccount.ss58Address,
      // 旧账户签名完成、extrinsic 提交前先持久化安全 outbox；finalized 后即使进程
      // 退出，下次身份对账仍能用新账户会话重放同一授权完成清理。
      onOldAuthorizationReady: (oldAccountSignature) =>
          _rebindRevoker.stagePendingCleanup(
        cidNumber: cidNumber,
        oldAccountId: oldAccountId,
        newAccountId: newAccount.accountId,
        oldAccountSignature: oldAccountSignature,
      ),
    );
    // 链上换绑已 Finalized:把设备子钥/会话/通讯录**自动更换到新账户**(死契约
    // [[cid-rebind-subkeys-must-auto-migrate]])。功能重建仍靠「链上真值(new) != 已同步
    // 标记(old)」对账续跑；独立安全 outbox 只负责旧账户吊销授权，二者不能互相冒充完成。
    await _runRebindMigration(
      cidNumber,
      oldAccountId,
      newAccount.accountId,
    );
  }

  /// 对账式换绑重建触发:比较链上身份账户 [resolved] 与「本地已同步账户」标记,不一致即
  /// 补齐迁移。首个基线(标记为空)只登记不迁移(账户0 的凭证在钱包创建时已就绪)。
  ///
  /// 覆盖两类情形:①换绑 Finalized 后本地重建曾中断(含 Finalized 与本地写入间崩溃);
  /// ②注册时把 CID 绑到非账户0 的 `//n`(身份账户 != 基线账户0),据此补齐 `//n` 凭证。
  Future<void> _reconcileIdentityRebuild(ResolvedIdentity resolved) async {
    final current = resolved.accountId;
    final String? synced;
    try {
      synced = await _syncedAccountStore.read();
    } catch (e) {
      AppLog.d('identity synced marker read failed: $e');
      return;
    }
    if (synced == null) {
      // 尚无基线 = 本设备子钥还没绑过任何身份账户(懒绑定下建钱包不注册子钥)。
      //
      // **绝不能在这里写基线**:写了就等于谎称凭证已就绪,`ensureDeviceSubkeyBound`
      // 会直接短路返回,用户进广场时子钥其实没绑。标记的写入唯一交给真正完成绑定的
      // [ensureDeviceSubkeyBound] 与换绑迁移收尾,此处只重试待清理项。
      await _retryPendingRebindCleanup(resolved);
      return;
    }
    if (synced == current) {
      // 功能凭证已同步不代表安全吊销完成；独立 outbox 必须继续重试，不能再被同步
      // 标记短路后永久遗忘。
      await _retryPendingRebindCleanup(resolved);
      return;
    }
    final cidNumber = resolved.snapshot?.cidNumber;
    if (cidNumber == null) return;
    try {
      await _runRebindMigration(cidNumber, synced, current);
    } catch (e) {
      // 仍失败(网络等)则保留旧标记,下次 getState 再对账补齐,绝不阻断身份页展示。
      AppLog.d('identity rebind reconcile deferred: $e');
    }
  }

  /// 注册身份前的自付能力测算:返回门槛与该账户当前余额(均为**分**)。
  ///
  /// 门槛 = 链上 `OnchainMinFee + ExistentialDeposit`,两个数都现取自链上 metadata——
  /// 交易费常量真源恒在区块链常量库,App 侧不留任何副本。余额走 `forceFresh` 绕开块内
  /// 缓存,否则会拿到充值前的旧值,把刚充完钱的用户又踢回充值页。
  ///
  /// 链读失败**不吞**:上抛给调用方 fail-closed 处理,绝不静默当成「余额不足」或「充足」。
  Future<({BigInt requiredFen, BigInt balanceFen})> fetchRegistrationAffordability(
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

  /// 确保本设备 P-256 子钥已绑定到当前身份账户;未绑则绑(触发一次生物识别)。
  ///
  /// **懒绑定**:子钥只服务广场 / 聊天 / 通讯录 / 创作者 / 会员这些需 CID 的场景,故不在
  /// 建钱包时注册(那时账户还没有 CID,后端也不收),而由 `IdentityRegistrationGate` 在用户
  /// 初次进入上述页面时调用本方法。只用钱包和交易的用户永远不会走到这里。
  ///
  /// 判据是 [IdentitySyncedAccountStore] —— 「本地鉴权凭证已同步到的身份账户」。标记与
  /// 当前身份账户一致即认为已绑,直接返回,不弹窗;不一致(含首次为空)才真正绑定并推进标记。
  /// 绑定本身是后端幂等 upsert,重复调用安全。
  ///
  /// 并发去重:五处 gate 可能同时挂载,用 [_subkeyBindInflight] 保证同一时刻只跑一次,
  /// 其余调用等同一个 Future。失败向上抛,由 gate 按 fail-closed 拦住功能页并给重试。
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
    final identityAccountId = resolved.accountId;
    final synced = await _syncedAccountStore.read();
    if (synced == identityAccountId) return;
    await _walletManager.bindDeviceSubkeyToAccountId(identityAccountId);
    await _writeSyncedMarker(identityAccountId);
  }

  /// 供 App 生命周期 / 测试显式触发一次对账;自行解析当前身份账户。
  Future<void> reconcileIdentityRebuild() async {
    final ResolvedIdentity? resolved;
    try {
      resolved = await _identityResolver.resolve();
    } catch (e) {
      AppLog.d('identity reconcile resolve failed: $e');
      return;
    }
    if (resolved == null) return;
    await _reconcileIdentityRebuild(resolved);
  }

  Future<void> _retryPendingRebindCleanup(ResolvedIdentity resolved) async {
    final cidNumber = resolved.snapshot?.cidNumber;
    if (cidNumber == null) return;
    try {
      await _rebindRevoker.retryPendingCleanup(
        cidNumber: cidNumber,
        currentAccountId: resolved.accountId,
      );
    } catch (e) {
      // 待清理记录仍保留，下次生命周期/身份读取继续重试；不把已 finalized 的身份
      // 降级成失败页，也绝不把异常当作清理成功。
      AppLog.d('identity rebind revoke retry deferred: $e');
    }
  }

  /// 执行换绑后的本地重建(全步幂等,重复调用安全,并发去重)。
  ///
  /// rebindCidTo 直触发与 getState 对账触发可能并发,用 [_migrationInflight] 去重,
  /// 同一时刻只真正执行一次,其余等待同一 Future(防重复触发互相 409 伪失败)。
  Future<void> _runRebindMigration(
    String cidNumber,
    String oldAccountId,
    String newAccountId,
  ) {
    final inflight = _migrationInflight;
    if (inflight != null) return inflight;
    final future = _doRunRebindMigration(
      cidNumber,
      oldAccountId,
      newAccountId,
    ).whenComplete(() => _migrationInflight = null);
    _migrationInflight = future;
    return future;
  }

  /// 本地重建实体步骤:
  /// 1. 设备子钥归属切新账户(新账户云会话登录的**硬前置**,须最先且必成功);
  /// 2. 通讯录迁到新账户并用新账户 child 密钥重加密上云(上云失败落待办,不阻断);
  /// 3. 本地静止态数据密钥(LDK)重 wrap 到新账户;
  /// 4. 新账户会话携旧账户换绑授权执行账户级吊销；失败保留安全 outbox 独立重试;
  /// 5. 广播身份绑定变化——**在通讯录迁移完成之后**:避免其它页在新账户通讯录尚空/
  ///    迁移进行中就翻到新账户,读到空/错乱,或与迁移的整份快照发生读-改-写覆盖丢数据;
  /// 6. 更新「已同步账户」标记为新账户(功能对账基线推进；安全 outbox 仍以服务端确认
  ///    为唯一完成条件)。
  Future<void> _doRunRebindMigration(
    String cidNumber,
    String oldAccountId,
    String newAccountId,
  ) async {
    await _walletManager.bindDeviceSubkeyToAccountId(newAccountId);
    await _contactService.migrateContactsToNewIdentity(
      oldAccountId,
      newAccountId,
    );
    // 步骤 3:LDK 重 wrap。顺序上必须**在通讯录迁移之后**——迁移要读旧账户的本地
    // 密文 KV,而重 wrap 会删掉旧账户的 LDK wrap;倒过来迁移就读不出旧数据了。
    // 也必须**在广播身份变化之前**:广播后各页会按新账户读本地密文,若此时新账户
    // 还没有 LDK wrap,`ensureLocalDataKeyForAccountId` 会新生成一把,导致聊天/
    // MLS/附件/通讯录已落盘密文全部不可解密。
    // 不吞异常:失败必须让整个迁移重试(死契约 cid-rebind-subkeys-must-auto-migrate)。
    await _walletManager.rewrapLocalDataKeyForRebind(
      oldAccountId: oldAccountId,
      newAccountId: newAccountId,
    );
    // 用新账户会话代吊销旧账户。网络失败不阻断已经 finalized 的身份切换，但安全
    // outbox 绝不删除；即使同步标记推进，后续 getState 仍独立重试，直至 Worker 确认。
    try {
      await _rebindRevoker.retryPendingCleanup(
        cidNumber: cidNumber,
        currentAccountId: newAccountId,
      );
    } catch (e) {
      AppLog.d('identity rebind revoke old account deferred: $e');
    }
    WalletManager.notifyIdentityBindingChanged();
    await _writeSyncedMarker(newAccountId);
  }

  Future<void> _writeSyncedMarker(String accountId) async {
    try {
      await _syncedAccountStore.write(accountId);
    } catch (e) {
      AppLog.d('identity synced marker write failed: $e');
    }
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

  /// 写身份账户的身份徽章快照,供非链页面(个人页/广场)展示,不作权限依据。
  Future<void> _persistBadgeSnapshot(String accountId, String level) async {
    try {
      await _badgeSnapshotStore.write(
        accountId: accountId,
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
