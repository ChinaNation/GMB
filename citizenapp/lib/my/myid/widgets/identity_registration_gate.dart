import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import 'package:citizenapp/my/myid/identity_account_resolver.dart';
import 'package:citizenapp/my/myid/myid_page.dart';
import 'package:citizenapp/rpc/smoldot_client.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

enum _GateStatus { loading, registered, unregistered, queryFailed, noWallet }

/// CID 注册门:未注册 CID 身份时挡住功能(聊天 / 广场 / 订阅 / 创作者 / 通讯录)并引导
/// 去注册(决策②:访客无匿名可用面)。
///
/// 判据单源 = [ResolvedIdentity.isRegistered](占任意 CID 即放行,匿名占号亦放行——是
/// CID 门非投票身份门),走 [IdentityAccountResolver.resolve] **严格链读路径**:链读失败
/// 按 [_GateStatus.queryFailed] 提示重试(fail-closed,绝不放行也绝不误判未注册),**不用**
/// `IdentityAccountCache`(其链读失败乐观回退账户0 会把"未知"误当"未注册")。注册 / 换绑经
/// [WalletManager.walletsRevision] 广播即自动重判并放行。
class IdentityRegistrationGate extends StatefulWidget {
  const IdentityRegistrationGate({
    super.key,
    required this.featureLabel,
    required this.child,
    this.resolver,
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

  /// 链健康信号源(测试注入);默认取 [SmoldotClientManager.instance]。用于区分
  /// 「链未就绪(loading 等自愈)」与「已就绪仍读失败(queryFailed)」。
  final ValueListenable<ChainHealthStatus>? healthListenable;

  /// 全局判据覆盖(仅测试):被 gate 包裹的**页面** widget 测试无法逐个透传 resolver,
  /// 设此即让所有 gate 用它放行,不触发真 smoldot 链读。生产恒为 null。
  @visibleForTesting
  static IdentityAccountResolver? debugResolver;

  @override
  State<IdentityRegistrationGate> createState() =>
      _IdentityRegistrationGateState();
}

class _IdentityRegistrationGateState extends State<IdentityRegistrationGate> {
  late final IdentityAccountResolver _resolver = _createResolver();
  _GateStatus _status = _GateStatus.loading;

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
      } else {
        next = resolved.isRegistered
            ? _GateStatus.registered
            : _GateStatus.unregistered;
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

    final Widget view = switch (_status) {
      _GateStatus.loading => const Center(child: CircularProgressIndicator()),
      _GateStatus.unregistered => _GateView(
          icon: Icons.badge_outlined,
          title: '注册身份后使用${widget.featureLabel}',
          body: '${widget.featureLabel}需要先注册你的公民身份(CID)。注册后即可使用。',
          bannerTitle: '需要已注册身份',
          bannerBody: '身份是你在链上的唯一主键;注册后聊天、广场、通讯录等功能对你开放。',
          actionLabel: '去注册身份',
          actionIcon: Icons.how_to_reg_outlined,
          onAction: _openRegister,
        ),
      _GateStatus.noWallet => _GateView(
          icon: Icons.account_balance_wallet_outlined,
          title: '需要热钱包',
          body: '请先创建或导入热钱包,再注册身份使用${widget.featureLabel}。',
          bannerTitle: '尚无热钱包',
          bannerBody: '钱包是注册身份与鉴权的前提。',
          actionLabel: '去注册身份',
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
    required this.body,
    required this.bannerTitle,
    required this.bannerBody,
    required this.actionLabel,
    required this.actionIcon,
    required this.onAction,
  });

  final IconData icon;
  final String title;
  final String body;
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
            const SizedBox(height: 8),
            Text(
              body,
              textAlign: TextAlign.center,
              style: const TextStyle(
                fontSize: 13,
                height: 1.6,
                color: AppTheme.textSecondary,
              ),
            ),
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
