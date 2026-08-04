import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/my/creator/creator_money.dart'
    show fenToYuanMoneyLabel;
import 'package:citizenapp/my/membership/membership_detail_page.dart';
import 'package:citizenapp/my/membership/subscription_service.dart';
import 'package:citizenapp/rpc/subscription_rpc.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 会员三档固定顺序（与价格升序一致，ADR-036，与身份彻底解耦）：
/// 自由 freedom < 民主 democracy < 薪火 spark。
const List<String> _tierOrder = ['freedom', 'democracy', 'spark'];

/// 「会员」页：三档订阅卡前后层叠，当前会员档卡在最上层，另两档退到下层露边等候选；
/// 左右滑动 / 点击把候选档换到最上层。任意身份可订任意档（无身份门槛）。
///
/// 订阅 / 取消全部在 App 内完成：热钱包上链热签（生物识别）→ 链上 square-post
/// 订阅授权 → confirm 刷新镜像。价格由链上 `PlatformPrice[level]` 单源读取（公民币）。
class MembershipPage extends StatefulWidget {
  const MembershipPage({
    super.key,
    SquareChainService? chainService,
    SquareSessionProvider? sessionProvider,
    SubscriptionService? subscriptionService,
  })  : _chainService = chainService,
        _sessionProvider = sessionProvider,
        _subscriptionService = subscriptionService;

  final SquareChainService? _chainService;
  final SquareSessionProvider? _sessionProvider;
  final SubscriptionService? _subscriptionService;

  @override
  State<MembershipPage> createState() => _MembershipPageState();
}

class _MembershipPageState extends State<MembershipPage>
    with SingleTickerProviderStateMixin {
  late final SquareChainService _chainService =
      widget._chainService ?? SquareChainService();
  late final SquareSessionProvider _sessionProvider =
      widget._sessionProvider ?? SquareSessionProvider.instance;
  late final SubscriptionService _subscriptionService =
      widget._subscriptionService ?? SubscriptionService();
  late final AnimationController _snapController;
  Animation<double>? _snapAnim;

  static const _subscriptionCacheTtl = Duration(minutes: 5);
  static const _pricesCacheTtl = Duration(minutes: 30);

  /// 首屏永远使用 App 内置三档静态定义立即渲染；联网只在后台替换动态字段。
  bool _refreshing = false;

  /// 订阅 / 取消上链进行中：期间禁用按钮、显示按钮内进度圈。
  bool _busy = false;
  _MembershipViewData _data = const _MembershipViewData(
    accountId: '',
    cidNumber: '',
    state: _staticMembershipState,
    prices: <String, int>{},
    subscriptionReady: false,
  );

  /// 连续层叠位置（0..卡数-1）；整数=某卡在最上层，拖动时为小数。
  double _page = 0;
  int _cardCount = 0;

  @override
  void initState() {
    super.initState();
    _snapController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 320),
    )..addListener(() {
        final anim = _snapAnim;
        if (anim != null && mounted) setState(() => _page = anim.value);
      });
    unawaited(_load());
  }

  @override
  void dispose() {
    _snapController.dispose();
    super.dispose();
  }

  Future<void> _load({bool forceRefresh = false}) async {
    if (_refreshing) return;
    setState(() {
      _refreshing = true;
    });
    Object? refreshError;
    try {
      final session = await _sessionProvider.ensureSession();
      if (session == null) {
        return;
      }

      final accountId = session.accountId;
      final cidNumber = session.cidNumber;
      final cached = await _subscriptionService.readDisplaySnapshot(cidNumber);
      if (cached != null && mounted) {
        _applyViewData(
          _MembershipViewData(
            accountId: accountId,
            cidNumber: cidNumber,
            state: _withStaticPlans(cached.state),
            prices: cached.prices,
            subscriptionReady: cached.subscriptionFetchedAtMs > 0,
          ),
        );
      } else if (mounted) {
        _applyViewData(
          _MembershipViewData(
            accountId: accountId,
            cidNumber: cidNumber,
            state: _staticMembershipState,
            prices: const <String, int>{},
            subscriptionReady: false,
          ),
        );
      }

      final nowMs = DateTime.now().millisecondsSinceEpoch;
      final refreshSubscription = forceRefresh ||
          cached == null ||
          !cached.subscriptionIsFresh(nowMs, _subscriptionCacheTtl);
      final refreshPrices = forceRefresh ||
          cached == null ||
          !cached.pricesAreFresh(nowMs, _pricesCacheTtl);
      if (!refreshSubscription && !refreshPrices) return;

      SquareMembershipState? refreshedState;
      Map<String, int>? refreshedPrices;
      final refreshes = <Future<void>>[];
      if (refreshSubscription) {
        refreshes.add(() async {
          try {
            final snapshot =
                await _subscriptionService.fetchFinalizedState(cidNumber);
            refreshedState =
                _stateFromFinalized(_staticMembershipState, snapshot);
          } on Object catch (error) {
            refreshError ??= error;
          }
        }());
      }
      if (refreshPrices) {
        refreshes.add(() async {
          try {
            refreshedPrices = await _chainService.fetchAllPlatformPrices(
              forceFresh: forceRefresh,
            );
          } on Object catch (error) {
            refreshError ??= error;
          }
        }());
      }
      await Future.wait(refreshes);

      final state = refreshedState ??
          (cached == null ? _staticMembershipState : cached.state);
      final prices = refreshedPrices ?? cached?.prices ?? const <String, int>{};
      final updated = MembershipDisplaySnapshot(
        state: state,
        prices: prices,
        subscriptionFetchedAtMs: refreshedState == null
            ? cached?.subscriptionFetchedAtMs ?? 0
            : nowMs,
        pricesFetchedAtMs:
            refreshedPrices == null ? cached?.pricesFetchedAtMs ?? 0 : nowMs,
      );
      if (refreshedState != null || refreshedPrices != null) {
        try {
          await _subscriptionService.writeDisplaySnapshot(cidNumber, updated);
        } on Object {
          // 本地缓存失败不影响当前内存态，也不能让静态首屏退回加载页。
        }
      }
      if (!mounted) return;
      _applyViewData(
        _MembershipViewData(
          accountId: accountId,
          cidNumber: cidNumber,
          state: _withStaticPlans(state),
          prices: prices,
          subscriptionReady: updated.subscriptionFetchedAtMs > 0,
        ),
      );
    } on Object catch (error) {
      refreshError = error;
    } finally {
      if (mounted) setState(() => _refreshing = false);
    }
    if (forceRefresh && refreshError != null && mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('会员动态数据刷新失败：$refreshError')),
      );
    }
  }

  void _applyViewData(_MembershipViewData data) {
    if (!mounted) return;
    final accountChanged = _data.accountId != data.accountId;
    final membershipChanged =
        _data.state.membershipLevel != data.state.membershipLevel;
    final defaultIndex = _tierIndexOfLevel(data.state.membershipLevel);
    setState(() {
      _data = data;
      if (accountChanged || membershipChanged || _cardCount == 0) {
        _page = defaultIndex.toDouble();
      }
    });
  }

  /// App 内订阅 / 取消：据当前订阅态决定动作 → 上链热签（生物识别）→ confirm → 刷新。
  /// 失败弹 SnackBar（文案单源自 [SubscriptionException]）。
  Future<void> _handleAction(String level) async {
    if (_busy) return;
    if (_data.accountId.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先在「我的 → 我的钱包」创建热钱包')),
      );
      return;
    }
    setState(() => _busy = true);
    try {
      // 动权前绕过展示缓存，重新核验 finalized 订阅态和链上价格。
      final snapshot =
          await _subscriptionService.fetchFinalizedState(_data.cidNumber);
      final state = _stateFromFinalized(_staticMembershipState, snapshot);
      final action = _actionFor(state, level);
      final prices = action == _SubscribeAction.cancel
          ? _data.prices
          : await _chainService.fetchAllPlatformPrices(forceFresh: true);
      final priceFen = prices[level];
      if (action != _SubscribeAction.cancel && priceFen == null) {
        throw const SubscriptionException('链上会员价格尚未就绪，请稍后重试');
      }
      if (action == _SubscribeAction.cancel) {
        await _subscriptionService.cancel();
      } else if (action == _SubscribeAction.change) {
        await _subscriptionService.changePlan(level, priceFen!);
      } else {
        await _subscriptionService.subscribe(level, priceFen!);
      }
      if (!mounted) return;
      await _load(forceRefresh: true);
    } on SubscriptionException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(e.message)));
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  /// 打开该档「会员详情页」（完整权益只读展示 + 订阅入口，订阅动作仍走本页 [_handleAction]）。
  void _openDetail(
    SquareMembershipPlan plan,
    int? priceFen,
    SquareMembershipState state,
  ) {
    final action = _actionFor(state, plan.membershipLevel);
    final noWallet = _data.accountId.isEmpty;
    final stateNotReady = !_data.subscriptionReady;
    final label = noWallet
        ? '请先创建热钱包'
        : stateNotReady
            ? '会员状态同步中'
            : action != _SubscribeAction.cancel && priceFen == null
                ? '链上价格未就绪'
                : _actionLabel(action);
    final enabled = !noWallet &&
        !stateNotReady &&
        (action == _SubscribeAction.cancel || priceFen != null);
    Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => MembershipDetailPage(
          plan: plan,
          priceFen: priceFen,
          actionLabel: label,
          subscribeEnabled: enabled,
          onSubscribe: () => _handleAction(plan.membershipLevel),
        ),
      ),
    );
  }

  void _animateToPage(int target) {
    final clamped = target.clamp(0, (_cardCount - 1).clamp(0, 99)).toDouble();
    _snapAnim = Tween<double>(begin: _page, end: clamped).animate(
      CurvedAnimation(parent: _snapController, curve: Curves.easeOutCubic),
    );
    _snapController.forward(from: 0);
  }

  void _onDragUpdate(DragUpdateDetails details, double dragUnit) {
    setState(() {
      _page = (_page - details.primaryDelta! / dragUnit)
          .clamp(0.0, (_cardCount - 1).toDouble());
    });
  }

  void _onDragEnd(DragEndDetails details) {
    final velocity = details.primaryVelocity ?? 0;
    final int target;
    if (velocity.abs() > 320) {
      target = velocity < 0 ? _page.ceil() : _page.floor();
    } else {
      target = _page.round();
    }
    _animateToPage(target);
  }

  @override
  Widget build(BuildContext context) {
    // 三张套餐卡是本地静态界面，页面查看本身不属于动权操作，不能被身份链读门禁替换成
    // 全屏加载。会员、价格与钱包状态由 [_load] 在卡片已经出现后后台补齐；真正订阅、
    // 换档或取消时仍由 [SubscriptionService] 重新核验热钱包、CID 与 finalized 链真值。
    return Scaffold(
      appBar: AppBar(
        title: const Text('会员｜订阅'),
        centerTitle: true,
        actions: [
          IconButton(
            tooltip: '刷新',
            onPressed: _refreshing ? null : () => _load(forceRefresh: true),
            // 自动刷新只临时禁用按钮，不用转圈替换页面或工具栏；三张卡始终保留。
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    final data = _data;
    final state = data.state;
    // 三档订阅卡（自由 / 民主 / 薪火）始终来自 App 内置静态定义。
    final plans = _orderedPlans(state.plans);
    _cardCount = plans.length;

    final size = MediaQuery.of(context).size;
    final cardWidth = (size.width * 0.8).clamp(280.0, 360.0);
    final bandHeight = (size.height * 0.60).clamp(430.0, 540.0);
    final peek = cardWidth * 0.40;
    final frontIndex = _page.round().clamp(0, plans.length - 1);
    final activeColor = _tierColor(plans[frontIndex].membershipLevel);

    // 绘制顺序：离最上层越远越先画（在下层），当前卡最后画（压在最上层）。
    final drawOrder = List<int>.generate(plans.length, (i) => i)
      ..sort((a, b) => (b - _page).abs().compareTo((a - _page).abs()));

    return Column(
      children: [
        if (state.hasSubscriptionWindow) _ActiveMembershipBanner(state: state),
        Expanded(
          child: GestureDetector(
            key: const ValueKey('membership-tier-stack-gesture'),
            behavior: HitTestBehavior.opaque,
            onHorizontalDragStart: (_) => _snapController.stop(),
            onHorizontalDragUpdate: (d) => _onDragUpdate(d, cardWidth),
            onHorizontalDragEnd: _onDragEnd,
            child: Center(
              child: SizedBox(
                height: bandHeight,
                width: double.infinity,
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    for (final index in drawOrder)
                      _buildStackedCard(
                        index: index,
                        plan: plans[index],
                        state: state,
                        priceFen: data.prices[plans[index].membershipLevel],
                        canSubscribe:
                            data.accountId.isNotEmpty && data.subscriptionReady,
                        unavailableLabel: data.accountId.isEmpty && !_refreshing
                            ? '请先创建热钱包'
                            : '会员状态同步中',
                        cardWidth: cardWidth,
                        cardHeight: bandHeight,
                        peek: peek,
                      ),
                  ],
                ),
              ),
            ),
          ),
        ),
        _PageDots(
          count: plans.length,
          activeIndex: frontIndex,
          activeColor: activeColor,
        ),
        const SizedBox(height: 10),
        const Text(
          '左右滑动切换会员档',
          style: TextStyle(color: AppTheme.textTertiary, fontSize: 12),
        ),
        const SizedBox(height: 20),
      ],
    );
  }

  Widget _buildStackedCard({
    required int index,
    required SquareMembershipPlan plan,
    required SquareMembershipState state,
    required int? priceFen,
    required bool canSubscribe,
    required String unavailableLabel,
    required double cardWidth,
    required double cardHeight,
    required double peek,
  }) {
    final off = index - _page;
    final absOff = off.abs();
    final scale = (1.0 - 0.16 * absOff).clamp(0.68, 1.0);
    final opacity = (1.0 - 0.55 * absOff).clamp(0.0, 1.0);
    final isFront = absOff < 0.5;

    final visual = Transform.translate(
      offset: Offset(off * peek, 0),
      child: Transform.scale(
        scale: scale,
        child: Opacity(
          opacity: opacity,
          child: SizedBox(
            width: cardWidth,
            height: cardHeight,
            child: _MembershipTierCard(
              plan: plan,
              state: state,
              priceFen: priceFen,
              canSubscribe: canSubscribe,
              unavailableLabel: unavailableLabel,
              busy: _busy,
              onTapAction: () => _handleAction(plan.membershipLevel),
              onViewDetail: () => _openDetail(plan, priceFen, state),
              elevated: isFront,
            ),
          ),
        ),
      ),
    );

    if (isFront) {
      // 当前卡在最上层，内部按钮可点。
      return KeyedSubtree(
        key: const ValueKey('membership-front-card'),
        child: visual,
      );
    }
    // 下层候选卡：点击整卡切到最上层（拦掉内部按钮点击）。
    return IgnorePointer(
      ignoring: opacity < 0.05,
      child: GestureDetector(
        onTap: () => _animateToPage(index),
        child: IgnorePointer(child: visual),
      ),
    );
  }
}

class _MembershipViewData {
  const _MembershipViewData({
    required this.accountId,
    required this.cidNumber,
    required this.state,
    required this.prices,
    required this.subscriptionReady,
  });

  final String accountId;
  final String cidNumber;
  final SquareMembershipState state;

  /// 各档链上月价（分，公民币）；缺档表示链上未设该档价，卡片显示占位「—」。
  final Map<String, int> prices;

  /// 至少成功读取过一次 finalized 订阅态；未就绪时只展示卡片，禁止动权入口。
  final bool subscriptionReady;
}

/// 单张会员档卡（ADR-036）：一张卡 = 一个订阅档（自由/民主/薪火）。档色顶带 + 大字档名
/// + 会员权益（聊天文件上限 / 动态 / 文章）+ 公民币月价 + 订阅按钮。无任何身份字段。
class _MembershipTierCard extends StatelessWidget {
  const _MembershipTierCard({
    required this.plan,
    required this.state,
    required this.priceFen,
    required this.canSubscribe,
    required this.unavailableLabel,
    required this.busy,
    required this.onTapAction,
    required this.onViewDetail,
    this.elevated = true,
  });

  final SquareMembershipPlan plan;
  final SquareMembershipState state;

  /// 本档链上月价（分）；null=链上未设该档价，价签显示占位「—」。
  final int? priceFen;

  /// 是否已有可执行订阅动作的默认热钱包会话。
  final bool canSubscribe;

  final String unavailableLabel;

  /// 订阅 / 取消上链进行中：禁用按钮并显示进度圈。
  final bool busy;

  final VoidCallback onTapAction;

  /// 点击「查看详细权益」进入该档会员详情页。
  final VoidCallback onViewDetail;

  /// 是否在最上层（决定投影强度）。
  final bool elevated;

  @override
  Widget build(BuildContext context) {
    final level = plan.membershipLevel;
    final tierColor = _tierColor(level);
    final onTier = _onTierColor(level);
    // 当前所购且有效的档：高亮边框 + 顶带「当前会员」标记。
    final isCurrentTier =
        state.subscriptionActive && state.membershipLevel == level;
    final action = _actionFor(state, level);
    final actionLabel = !canSubscribe
        ? unavailableLabel
        : action != _SubscribeAction.cancel && priceFen == null
            ? '链上价格未就绪'
            : _actionLabel(action);

    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        border: Border.all(
          color: isCurrentTier ? tierColor : AppTheme.border,
          width: isCurrentTier ? 2 : 1,
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: elevated ? 0.22 : 0.08),
            blurRadius: elevated ? 24 : 8,
            offset: Offset(0, elevated ? 12 : 4),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildHeader(level, tierColor, onTier, isCurrentTier, priceFen),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 16, 16, 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: SingleChildScrollView(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          _SectionLabel(
                            icon: Icons.workspace_premium_outlined,
                            text: '会员权益',
                            color: tierColor,
                          ),
                          const SizedBox(height: 10),
                          // 卡片提炼短行（每条 1 行）；完整权益见「查看详细权益」详情页。
                          _ParamLine(
                            icon: Icons.attach_file_outlined,
                            color: tierColor,
                            text:
                                '聊天文件 每个 ${plan.chatFileSizeLabel}${plan.supportsLargeFileRelay ? ' · 大文件中转' : ''}',
                          ),
                          const SizedBox(height: 8),
                          _ParamLine(
                            icon: Icons.videocam_outlined,
                            color: tierColor,
                            text:
                                '动态视频 ${plan.dynamicVideoDurationLabel} · ${plan.dynamicVideoQualityLabel}',
                          ),
                          const SizedBox(height: 8),
                          _ParamLine(
                            icon: Icons.photo_outlined,
                            color: tierColor,
                            text:
                                '动态图片 ${plan.dynamicMaxImages} 张 · ${plan.dynamicImageQualityLabel}',
                          ),
                          const SizedBox(height: 8),
                          _ParamLine(
                            icon: Icons.article_outlined,
                            color: tierColor,
                            text:
                                '文章 ${_wanLabel(plan.articleBodyMaxChars)} · ${plan.articleMaxImages} 图',
                          ),
                          const SizedBox(height: 8),
                          _ParamLine(
                            icon: Icons.cloud_upload_outlined,
                            color: tierColor,
                            text:
                                '每月 图 ${_thousands(plan.monthlyImages)} · 视频 ${plan.monthlyVideoDurationLabel}',
                          ),
                          const SizedBox(height: 8),
                          _ParamLine(
                            icon: Icons.auto_awesome_outlined,
                            color: tierColor,
                            text: '整体画质 ${plan.dynamicImageQualityLabel}',
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 8),
                  // 查看详细权益：整行可点，进入该档会员详情页。
                  InkWell(
                    onTap: onViewDetail,
                    child: Padding(
                      padding: const EdgeInsets.symmetric(vertical: 8),
                      child: Row(
                        children: [
                          Text(
                            '查看详细权益',
                            style: TextStyle(
                              color: tierColor,
                              fontSize: 13,
                              fontWeight: FontWeight.w600,
                            ),
                          ),
                          const Spacer(),
                          Icon(Icons.chevron_right, size: 18, color: tierColor),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  SizedBox(
                    width: double.infinity,
                    child: _SubscribeButton(
                      label: actionLabel,
                      color: tierColor,
                      busy: busy,
                      action: action,
                      enabled: canSubscribe &&
                          (action == _SubscribeAction.cancel ||
                              priceFen != null),
                      onTap: onTapAction,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader(String level, Color tierColor, Color onTier,
      bool isCurrentTier, int? priceFen) {
    // 自由金底使用规范允许的主文字色；民主蓝/薪火红深色底使用纯白标志。
    final gmbMarkColor =
        level == 'freedom' ? AppTheme.textPrimary : Colors.white;
    return Container(
      color: tierColor,
      padding: const EdgeInsets.fromLTRB(16, 16, 16, 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  '会员订阅',
                  style: TextStyle(
                    color: onTier.withValues(alpha: 0.82),
                    fontSize: 12,
                    fontWeight: FontWeight.w500,
                    letterSpacing: 0.4,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  plan.displayName,
                  style: TextStyle(
                    color: onTier,
                    fontSize: 22,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                if (isCurrentTier) ...[
                  const SizedBox(height: 10),
                  Container(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                    decoration: BoxDecoration(
                      color: onTier.withAlpha(38),
                      borderRadius: BorderRadius.circular(999),
                    ),
                    child: Text(
                      '当前会员',
                      style: TextStyle(
                        color: onTier,
                        fontSize: 11,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
          const SizedBox(width: 12),
          // 价格移至右上角：千分号 + 两位小数 + 元/月（链上单源，读不到显示「—」）。
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  // 标志只辅助识别，价格文本继续承担金额和无障碍语义。
                  ExcludeSemantics(
                    child: Image.asset(
                      'assets/icons/gmb-mark.png',
                      key: ValueKey('membership-price-gmb-mark-$level'),
                      width: 18,
                      height: 18,
                      color: gmbMarkColor,
                      colorBlendMode: BlendMode.srcIn,
                      filterQuality: FilterQuality.high,
                    ),
                  ),
                  const SizedBox(width: 6),
                  Text(
                    priceFen == null ? '—' : fenToYuanMoneyLabel(priceFen),
                    style: TextStyle(
                      color: onTier,
                      fontSize: 17,
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                ],
              ),
              if (priceFen != null)
                Text(
                  '/月',
                  style: TextStyle(
                    color: onTier.withValues(alpha: 0.82),
                    fontSize: 11,
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }
}

/// 整数千分号：1500 → "1,500"。
String _thousands(int n) =>
    n.toString().replaceAllMapped(RegExp(r'\B(?=(\d{3})+$)'), (_) => ',');

/// 字数「万」简写：20000 → "2 万字"；非整万回退千分号原值。
String _wanLabel(int chars) => chars >= 10000 && chars % 10000 == 0
    ? '${chars ~/ 10000} 万字'
    : '${_thousands(chars)} 字';

/// 同一业务操作只签一次：新订阅、取消当前订阅、换到另一档分别提交一笔链上交易。
enum _SubscribeAction { subscribe, change, cancel }

_SubscribeAction _actionFor(SquareMembershipState state, String level) {
  if (state.subscriptionStatus == 'active') {
    return state.membershipLevel == level
        ? _SubscribeAction.cancel
        : _SubscribeAction.change;
  }
  if (state.subscriptionStatus == 'cancelled' &&
      state.membershipLevel != level) {
    return _SubscribeAction.change;
  }
  return _SubscribeAction.subscribe;
}

String _actionLabel(_SubscribeAction action) => switch (action) {
      _SubscribeAction.subscribe => '订阅',
      _SubscribeAction.change => '更换为此档',
      _SubscribeAction.cancel => '取消订阅',
    };

class _SubscribeButton extends StatelessWidget {
  const _SubscribeButton({
    required this.label,
    required this.color,
    required this.busy,
    required this.action,
    required this.enabled,
    required this.onTap,
  });

  final String label;
  final Color color;
  final bool busy;

  final _SubscribeAction action;

  /// 链上价格未就绪时禁止发起订阅；取消既有订阅不依赖当前价格。
  final bool enabled;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return FilledButton.icon(
      onPressed: busy || !enabled ? null : onTap,
      icon: busy
          ? const SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                color: Colors.white,
              ),
            )
          : Icon(
              switch (action) {
                _SubscribeAction.subscribe => Icons.workspace_premium_outlined,
                _SubscribeAction.change => Icons.swap_horiz,
                _SubscribeAction.cancel => Icons.cancel_outlined,
              },
              size: 16,
            ),
      label: Text(label),
      style: FilledButton.styleFrom(
        backgroundColor: color,
        foregroundColor: Colors.white,
        minimumSize: const Size.fromHeight(46),
        textStyle: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700),
      ),
    );
  }
}

/// 用 finalized 真态覆盖 Cloudflare 镜像状态，只保留 Worker 下发的展示套餐定义。
SquareMembershipState _stateFromFinalized(
  SquareMembershipState mirror,
  FinalizedSubscriptionSnapshot snapshot,
) {
  final chain = snapshot.state;
  if (chain == null) {
    return SquareMembershipState(
      active: false,
      paidUntil: 0,
      plans: mirror.plans,
    );
  }
  if (chain.plan.kind != 'platform' || chain.plan.membershipLevel == null) {
    throw const FormatException('平台会员读取到了非平台订阅计划');
  }
  final effective = chain.isEffectiveAt(snapshot.chainNowMs);
  return SquareMembershipState(
    active: effective,
    paidUntil: chain.paidUntil,
    membershipLevel: chain.plan.membershipLevel,
    subscriptionStatus: chain.status,
    subscriptionActive: effective,
    lastChargedAt: chain.lastChargedAt,
    plans: mirror.plans,
  );
}

/// 卡内分区小标题（会员权益）。
class _SectionLabel extends StatelessWidget {
  const _SectionLabel({
    required this.icon,
    required this.text,
    required this.color,
  });

  final IconData icon;
  final String text;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Icon(icon, size: 15, color: color),
        const SizedBox(width: 6),
        Text(
          text,
          style: TextStyle(
            color: color,
            fontSize: 12,
            fontWeight: FontWeight.w700,
            letterSpacing: 0.3,
          ),
        ),
      ],
    );
  }
}

class _ParamLine extends StatelessWidget {
  const _ParamLine({
    required this.icon,
    required this.color,
    required this.text,
  });

  final IconData icon;
  final Color color;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(top: 1),
          child: Icon(icon, size: 16, color: color),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: Text(
            text,
            style: const TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 13,
              height: 1.5,
            ),
          ),
        ),
      ],
    );
  }
}

class _PageDots extends StatelessWidget {
  const _PageDots({
    required this.count,
    required this.activeIndex,
    required this.activeColor,
  });

  final int count;
  final int activeIndex;
  final Color activeColor;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: List.generate(count, (index) {
        final active = index == activeIndex;
        return AnimatedContainer(
          duration: const Duration(milliseconds: 220),
          margin: const EdgeInsets.symmetric(horizontal: 3),
          width: active ? 18 : 6,
          height: 6,
          decoration: BoxDecoration(
            color: active ? activeColor : AppTheme.border,
            borderRadius: BorderRadius.circular(999),
          ),
        );
      }),
    );
  }
}

/// 订阅起止横幅（ADR-034 段4）：展示当前有效会员的档位、续费态与订阅起止日期。
/// 会员操作（订阅 / 取消）已在 App 内卡片按钮完成，横幅只读展示。
class _ActiveMembershipBanner extends StatelessWidget {
  const _ActiveMembershipBanner({required this.state});

  final SquareMembershipState state;

  @override
  Widget build(BuildContext context) {
    final plan = state.planForLevel(state.membershipLevel);
    final name = plan?.displayName ?? '会员';
    // 用户订阅授权未取消时，runtime 按链上真实公历到期时间自动扣款。
    final route = switch (state.subscriptionStatus) {
      'cancelled' => '已取消 · 到期终止',
      'terminated' => '扣款失败 · 订阅已终止',
      'suspended' => '已挂起 · 待重新签名或充值',
      'creatorPaused' => '创作者暂停 · 恢复后自动续',
      _ => '链上到期自动续费',
    };
    final window =
        '订阅 ${_formatYmd(state.lastChargedAt)} ~ ${_formatYmd(state.paidUntil)}';
    return Container(
      margin: const EdgeInsets.fromLTRB(16, 12, 16, 0),
      padding: const EdgeInsets.all(12),
      decoration: AppTheme.bannerDecoration(AppTheme.info),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.event_available, size: 18, color: AppTheme.info),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  '$name · $route',
                  style: const TextStyle(
                    color: AppTheme.info,
                    fontSize: 12,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  window,
                  style: const TextStyle(
                    color: AppTheme.textSecondary,
                    fontSize: 12,
                    height: 1.4,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// 毫秒时间戳格式化为本地 YYYY-MM-DD。
String _formatYmd(int ms) {
  final d = DateTime.fromMillisecondsSinceEpoch(ms).toLocal();
  final mm = d.month.toString().padLeft(2, '0');
  final dd = d.day.toString().padLeft(2, '0');
  return '${d.year}-$mm-$dd';
}

const int _mib = 1024 * 1024;

/// 三档 App 内置套餐：进入页面第一帧直接使用，与 Worker 权益参数对齐（ADR-036）。
/// 价格不进入静态套餐，仍以链上 `PlatformPrice` 为单一真源。
const List<SquareMembershipPlan> _fallbackMembershipPlans = [
  SquareMembershipPlan(
    membershipLevel: 'freedom',
    displayName: '自由会员',
    chatFileMaxBytes: 10 * _mib,
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'sd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'sd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 60,
    dynamicMaxVideoBytes: 40 * _mib,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 20000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'sd',
    articleMaxImages: 50,
    monthlyImages: 300,
    monthlyVideoSeconds: 1800,
    activeUploads: 1,
  ),
  SquareMembershipPlan(
    membershipLevel: 'democracy',
    displayName: '民主会员',
    chatFileMaxBytes: 100 * _mib,
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'hd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'hd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 1800,
    dynamicMaxVideoBytes: 1536 * _mib,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 30000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'hd',
    articleMaxImages: 100,
    monthlyImages: 1500,
    monthlyVideoSeconds: 10800,
    activeUploads: 2,
  ),
  SquareMembershipPlan(
    membershipLevel: 'spark',
    displayName: '薪火会员',
    chatFileMaxBytes: 5120 * _mib,
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'hd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'hd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 10800,
    dynamicMaxVideoBytes: 8192 * _mib,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 30000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'hd',
    articleMaxImages: 100,
    monthlyImages: 5000,
    monthlyVideoSeconds: 108000,
    activeUploads: 3,
  ),
];

const SquareMembershipState _staticMembershipState = SquareMembershipState(
  active: false,
  paidUntil: 0,
  plans: _fallbackMembershipPlans,
);

/// 给缓存或 finalized 订阅态补入 App 内置套餐；动态快照不持久化静态权益字段。
SquareMembershipState _withStaticPlans(SquareMembershipState state) {
  return SquareMembershipState(
    active: state.active,
    paidUntil: state.paidUntil,
    membershipLevel: state.membershipLevel,
    subscriptionStatus: state.subscriptionStatus,
    subscriptionActive: state.subscriptionActive,
    lastChargedAt: state.lastChargedAt,
    plans: _fallbackMembershipPlans,
  );
}

/// 按固定档序（自由 / 民主 / 薪火）取套餐。
List<SquareMembershipPlan> _orderedPlans(List<SquareMembershipPlan> plans) {
  SquareMembershipPlan planFor(String level) {
    for (final plan in plans) {
      if (plan.membershipLevel == level) return plan;
    }
    return _fallbackMembershipPlans
        .firstWhere((p) => p.membershipLevel == level);
  }

  return _tierOrder.map(planFor).toList();
}

Color _tierColor(String level) => switch (level) {
      'spark' => AppTheme.identityCandidate,
      'democracy' => AppTheme.identityVoting,
      _ => AppTheme.identityVisitor,
    };

/// 顶带/价格标签前景色：自由金底用深棕保证对比度，民主蓝/薪火红底用白字。
Color _onTierColor(String level) =>
    level == 'freedom' ? const Color(0xFF4A3000) : Colors.white;

/// 会员档在固定档序中的下标；未知/无订阅归 0（自由）。
int _tierIndexOfLevel(String? level) {
  final index = _tierOrder.indexOf(level ?? '');
  return index < 0 ? 0 : index;
}
