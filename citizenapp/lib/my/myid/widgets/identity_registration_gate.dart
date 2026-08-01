import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'package:citizenapp/log/app_log.dart';
import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/import_wallet_page.dart';

enum _GateStatus {
  loading,
  registered,
  unregistered,
  queryFailed,
  noWallet,
  bindFailed,
  mnemonicRequired,
}

/// 设备子钥按需绑定器(测试可注入);默认走 [MyIdService.ensureDeviceSubkeyBound]。
typedef DeviceSubkeyBinder = Future<void> Function();

/// CID 注册门:未注册 CID 身份时挡住功能(聊天 / 广场 / 订阅 / 创作者 / 通讯录)并引导
/// 去注册(决策②:访客无匿名可用面)。
///
/// 本门同时是**设备子钥的懒绑定触发点**。子钥只服务这些需 CID 的场景,故不在建钱包时
/// 注册(那时账户还没有 CID、后端也不收),只用钱包和交易的用户永远不需要它。已注册 CID
/// 的用户初次进入本门时才绑定(弹一次生物识别),绑定失败按 [_GateStatus.bindFailed]
/// 拦住功能页并给重试——与身份判定同样 fail-closed,绝不放行。
///
/// 判据单源 = [ResolvedIdentity.isRegistered](占任意 CID 即放行,匿名占号亦放行——是
/// CID 门非投票身份门),走 [IdentityAccountResolver.resolve] **严格链读路径**:链读失败
/// 按 [_GateStatus.queryFailed] 提示重试(fail-closed,绝不放行也绝不误判未注册)。
/// 本门使用严格解析器，避免高频缓存入口在禁止链读且未命中时返回空所造成的歧义。
/// 注册 / 换绑经 [WalletManager.walletsRevision] 广播即自动重判并放行。
class IdentityRegistrationGate extends StatefulWidget {
  const IdentityRegistrationGate({
    super.key,
    required this.featureLabel,
    required this.child,
    this.resolver,
    this.subkeyBinder,
    this.healthListenable,
    this.scaffoldTitle,
  });

  /// 功能名(引导文案用,如「聊天」「广场」)。
  final String featureLabel;

  /// 已注册 CID 时渲染的真功能页。
  final Widget child;

  /// 非空时,把「未放行」态(loading/引导/重试)包进带 [scaffoldTitle] 标题栏 + 自动
  /// 返回键的 Scaffold。用于**被 push 的子页**(通讯录/订阅/创作者):整个 Scaffold 交给
  /// gate 包裹时,未注册态仍保留返回键,且原页 AppBar 上的动作(如扫码加联系人、刷新)
  /// 随真页一起被挡在门后(修 fail-closed 绕过)。tab 页不传,渲染裸门视图。
  final String? scaffoldTitle;

  /// 判据 resolver(测试注入);默认新建走真链读。
  final IdentityAccountResolver? resolver;

  /// 设备子钥按需绑定器(测试注入);默认走 [MyIdService.ensureDeviceSubkeyBound]。
  final DeviceSubkeyBinder? subkeyBinder;

  /// 链健康信号源(测试注入);默认取 [SmoldotClientManager.instance]。用于区分
  /// 「链未就绪(loading 等自愈)」与「已就绪仍读失败(queryFailed)」。
  final ValueListenable<ChainHealthStatus>? healthListenable;

  /// 全局判据覆盖(仅测试):被 gate 包裹的**页面** widget 测试无法逐个透传 resolver,
  /// 设此即让所有 gate 用它放行,不触发真 smoldot 链读。生产恒为 null。
  @visibleForTesting
  static IdentityAccountResolver? debugResolver;

  /// 全局子钥绑定器覆盖(仅测试):同 [debugResolver],避免页面测试真去绑设备子钥
  /// (会读硬件金库、弹生物识别、打后端)。生产恒为 null。
  @visibleForTesting
  static DeviceSubkeyBinder? debugSubkeyBinder;

  @override
  State<IdentityRegistrationGate> createState() =>
      _IdentityRegistrationGateState();
}

class _IdentityRegistrationGateState extends State<IdentityRegistrationGate> {
  late final IdentityAccountResolver _resolver = _createResolver();
  late final DeviceSubkeyBinder _binder = _createSubkeyBinder();
  _GateStatus _status = _GateStatus.loading;

  /// [_GateStatus.mnemonicRequired] 时待恢复的数据根参数，由链上 finalized 绑定取得。
  CidDataRootRequest? _dataRootRequest;

  /// 单调递增的判定代际:仅最新一次 resolve 的结果可落地,旧响应(乱序返回 / 瞬断)
  /// 直接丢弃,防止注册成功后被一次迟到的失败覆盖回 queryFailed。
  int _resolveGeneration = 0;

  late final ValueListenable<ChainHealthStatus> _health =
      widget.healthListenable ??
          SmoldotClientManager.instance.healthStatusListenable;

  /// 生产恒用真链读 resolver;仅非 release 构建允许测试注入的 [debugResolver] 生效
  /// (物理上杜绝 release 误读全局静态钩子导致的全 App 门禁失效)。
  IdentityAccountResolver _createResolver() {
    final injected = widget.resolver;
    if (injected != null) return injected;
    if (!kReleaseMode) {
      final debug = IdentityRegistrationGate.debugResolver;
      if (debug != null) return debug;
    }
    return IdentityAccountResolver();
  }

  /// 同 [_createResolver]:生产恒用真绑定器,仅非 release 允许测试注入。
  DeviceSubkeyBinder _createSubkeyBinder() {
    final injected = widget.subkeyBinder;
    if (injected != null) return injected;
    if (!kReleaseMode) {
      final debug = IdentityRegistrationGate.debugSubkeyBinder;
      if (debug != null) return debug;
    }
    return MyIdService().ensureDeviceSubkeyBound;
  }

  @override
  void initState() {
    super.initState();
    // 注册 / 换绑 / 切钱包广播 → 自动重判(注册成功即放行,跨 tab 一致)。
    WalletManager.walletsRevision.addListener(_reresolve);
    // 链就绪(smoldot operational)→ 若门尚未判定成功,自动重判(修冷启动:落地页
    // 启动时链未就绪,不再永久卡在「读取失败」需手动重试)。
    _health.addListener(_onChainHealthChanged);
    _reresolve();
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_reresolve);
    _health.removeListener(_onChainHealthChanged);
    super.dispose();
  }

  void _onChainHealthChanged() {
    if (_health.value != ChainHealthStatus.operational) return;
    // 只在「尚未判定成功」态自愈;已 registered/unregistered 由 walletsRevision 驱动。
    // bindFailed 不自愈:绑定要弹生物识别,只能由用户点「重试」显式发起。
    if (_status == _GateStatus.loading || _status == _GateStatus.queryFailed) {
      _reresolve();
    }
  }

  Future<void> _reresolve() async {
    final generation = ++_resolveGeneration;
    _GateStatus next;
    try {
      final resolved = await _resolver.resolve();
      if (resolved == null) {
        next = _GateStatus.noWallet;
      } else if (!resolved.isRegistered) {
        next = _GateStatus.unregistered;
      } else {
        // 已有 CID → 放行前确保本设备子钥已绑到当前身份账户(懒绑定,首次会弹一次
        // 生物识别;已绑则直接返回不弹)。绑定失败不放行。
        try {
          await _binder();
          next = _GateStatus.registered;
        } on CidDataRootMnemonicRequiredException catch (error) {
          // 本机没有该 CID 的数据根，且此前绑定账户的私钥也不在本机（新设备、
          // 注册局代办换绑）。服务端从不持有密钥，只能由用户补录助记词重新派生。
          AppLog.d('identity gate needs mnemonic: $error');
          _dataRootRequest = CidDataRootRequest(
            cidNumber: error.cidNumber,
            bindingRevision: resolved.snapshot?.bindingRevision ?? 0,
            accountId: error.accountId,
          );
          next = _GateStatus.mnemonicRequired;
        } on Object catch (error) {
          AppLog.d('identity gate subkey bind failed: $error');
          next = _GateStatus.bindFailed;
        }
      }
    } catch (_) {
      // 链**未就绪**(冷启动 smoldot 未 operational)→ 停在 loading 等就绪自动重判,
      // 不冒充「未注册」也不弹「读取失败」;链**已就绪仍失败**→ queryFailed(可手动重试)。
      // 二者都 fail-closed:绝不放行。
      next = _health.value == ChainHealthStatus.operational
          ? _GateStatus.queryFailed
          : _GateStatus.loading;
    }
    // 仅最新代际、仍挂载时落地。
    if (!mounted || generation != _resolveGeneration) return;
    setState(() => _status = next);
  }

  /// 补录助记词恢复数据密钥：复用既有导入页，导入成功后在同一份助记词还在手上时
  /// 顺手派生本 CID 的数据根（见 [ImportWalletPage.dataRootRequest]），返回即重判。
  Future<void> _importMnemonic() async {
    final request = _dataRootRequest;
    if (request == null) return _retry();
    await Navigator.of(context).push<bool>(MaterialPageRoute(
      builder: (_) => ImportWalletPage(dataRootRequest: request),
    ));
    if (!mounted) return;
    await _retry();
  }

  Future<void> _retry() async {
    if (mounted) setState(() => _status = _GateStatus.loading);
    await _reresolve();
  }

  Future<void> _openRegister() async {
    // 跳唯一注册落地页 [MyIdPage](注册编排单源,DRY);返回后重判,注册成功亦经广播触发。
    await Navigator.of(context).push(
      MaterialPageRoute<void>(builder: (_) => const MyIdPage()),
    );
    await _reresolve();
  }

  @override
  Widget build(BuildContext context) {
    if (_status == _GateStatus.registered) return widget.child;

    // 未注册引导标题:聊天用「注册后开始聊天」,其余功能用「注册后使用{功能名}」(用户定稿文案)。
    final featureLabel = widget.featureLabel;
    final registerTitle =
        featureLabel == '聊天' ? '注册后开始聊天' : '注册后使用$featureLabel';

    final Widget view = switch (_status) {
      _GateStatus.loading => const Center(child: CircularProgressIndicator()),
      _GateStatus.unregistered => _GateView(
          icon: Icons.badge_outlined,
          title: registerTitle,
          bannerTitle: '需要注册身份',
          bannerBody: '注册后可使用广场、聊天、通讯录等功能，未注册用户只能使用钱包、交易等功能。',
          actionLabel: '注册',
          actionIcon: Icons.how_to_reg_outlined,
          onAction: _openRegister,
        ),
      _GateStatus.noWallet => _GateView(
          icon: Icons.account_balance_wallet_outlined,
          title: '需要热钱包',
          body: '请先创建或导入热钱包,再注册身份使用$featureLabel。',
          bannerTitle: '尚无热钱包',
          bannerBody: '钱包是注册身份与鉴权的前提。',
          actionLabel: '注册',
          actionIcon: Icons.how_to_reg_outlined,
          onAction: _openRegister,
        ),
      _GateStatus.queryFailed => _GateView(
          icon: Icons.cloud_off_outlined,
          title: '身份读取失败',
          body: '暂时无法确认你的身份状态,请检查网络后重试。',
          bannerTitle: '链上身份读取失败',
          bannerBody: '为保护你的账户,读取失败时不会放行,请重试。',
          actionLabel: '重试',
          actionIcon: Icons.refresh,
          onAction: _retry,
        ),
      _GateStatus.mnemonicRequired => _GateView(
          icon: Icons.key_outlined,
          title: '需要导入你的钱包助记词',
          body: '本设备上没有你的数据密钥。$featureLabel的数据只由你自己的钱包加密，'
              '服务端不持有任何密钥，只能由你导入助记词后在本机重新生成。',
          bannerTitle: '本设备缺少数据密钥',
          bannerBody: '导入助记词即可恢复;钱包与交易功能不受影响。',
          actionLabel: '导入助记词',
          actionIcon: Icons.download_outlined,
          onAction: _importMnemonic,
        ),
      _GateStatus.bindFailed => _GateView(
          icon: Icons.phonelink_lock_outlined,
          title: '设备绑定未完成',
          body: '使用$featureLabel需要先把本设备绑定到你的身份,请重试并通过验证。',
          bannerTitle: '本设备尚未绑定',
          bannerBody: '绑定失败时不会放行;钱包与交易功能不受影响。',
          actionLabel: '重试',
          actionIcon: Icons.refresh,
          onAction: _retry,
        ),
      // registered 已在上面短路返回,此分支不可达。
      _GateStatus.registered => widget.child,
    };

    final title = widget.scaffoldTitle;
    if (title == null) return view;
    // 被 push 的子页:未放行态保留标题栏 + 返回键,避免用户被困在无返回的门视图。
    return Scaffold(
      appBar: AppBar(title: Text(title)),
      body: view,
    );
  }
}

/// 门禁引导视图(照 `CreatorGateView` 样式):图标 + 标题 + 说明 + 警示条 + 主操作按钮。
class _GateView extends StatelessWidget {
  const _GateView({
    required this.icon,
    required this.title,
    this.body,
    required this.bannerTitle,
    required this.bannerBody,
    required this.actionLabel,
    required this.actionIcon,
    required this.onAction,
  });

  final IconData icon;
  final String title;

  /// 标题下方说明小字;为空则不渲染(未注册态按用户要求删除该行)。
  final String? body;
  final String bannerTitle;
  final String bannerBody;
  final String actionLabel;
  final IconData actionIcon;
  final Future<void> Function() onAction;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 28),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 64,
              height: 64,
              alignment: Alignment.center,
              decoration: BoxDecoration(
                color: AppTheme.primary.withAlpha(24),
                borderRadius: BorderRadius.circular(AppTheme.radiusLg),
              ),
              child: Icon(icon, size: 32, color: AppTheme.primary),
            ),
            const SizedBox(height: 14),
            Text(
              title,
              textAlign: TextAlign.center,
              style: const TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
            if (body != null && body!.isNotEmpty) ...[
              const SizedBox(height: 8),
              Text(
                body!,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  fontSize: 13,
                  height: 1.6,
                  color: AppTheme.textSecondary,
                ),
              ),
            ],
            const SizedBox(height: 20),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(14),
              decoration: AppTheme.bannerDecoration(AppTheme.warning),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Icon(Icons.lock_outline,
                      size: 18, color: AppTheme.warning),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          bannerTitle,
                          style: const TextStyle(
                            fontSize: 13,
                            fontWeight: FontWeight.w600,
                            color: AppTheme.textPrimary,
                          ),
                        ),
                        const SizedBox(height: 2),
                        Text(
                          bannerBody,
                          style: const TextStyle(
                            fontSize: 12,
                            height: 1.5,
                            color: AppTheme.textSecondary,
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: onAction,
              icon: Icon(actionIcon, size: 19),
              label: Text(actionLabel),
            ),
          ],
        ),
      ),
    );
  }
}
