import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/identity_badge.dart';

/// 官网会员订阅页（订阅 / 取消订阅 / 续订会员均在官网完成，App 只负责拉起浏览器）。
///
/// 默认打开已部署的官网会员页；官网主域不同时，构建期用
/// `--dart-define=MEMBERSHIP_URL=...` 覆盖即可。
class MembershipSiteConfig {
  const MembershipSiteConfig._();

  static const _defineName = 'MEMBERSHIP_URL';
  static const _configured = String.fromEnvironment(_defineName);
  static const _prodUrl = 'https://www.crcfrcn.com/membership';

  static String get membershipUrl {
    final value = _configured.trim();
    return value.isNotEmpty ? value : _prodUrl;
  }
}

/// 三档固定顺序：访客 0 < 投票 1 < 竞选 2。
const List<String> _tierOrder = ['visitor', 'voting', 'candidate'];

/// 「身份 ｜ 会员」页：三档身份/会员卡前后层叠，命中身份档卡在最上层，
/// 另两档退到下层露边等候选；左右滑动 / 点击把候选档换到最上层。
class MembershipPage extends StatefulWidget {
  const MembershipPage({
    super.key,
    SquareApiClient? apiClient,
    SquareSessionProvider? sessionProvider,
  })  : _apiClient = apiClient,
        _sessionProvider = sessionProvider;

  final SquareApiClient? _apiClient;
  final SquareSessionProvider? _sessionProvider;

  @override
  State<MembershipPage> createState() => _MembershipPageState();
}

class _MembershipPageState extends State<MembershipPage>
    with SingleTickerProviderStateMixin {
  late final SquareApiClient _apiClient =
      widget._apiClient ?? SquareApiClient();
  late final SquareSessionProvider _sessionProvider =
      widget._sessionProvider ?? SquareSessionProvider.instance;
  late final AnimationController _snapController;
  Animation<double>? _snapAnim;

  bool _loading = true;
  Object? _error;
  _MembershipViewData? _data;

  /// 连续层叠位置（0..卡数-1）；整数=某卡在最上层，拖动时为小数。
  double _page = 0;
  int _cardCount = 0;

  /// 访客卡内会员档切换：0=自由(freedom) / 1=民主(democracy)，默认自由。
  int _visitorPlanIndex = 0;

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
    _load();
  }

  @override
  void dispose() {
    _snapController.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final session = await _sessionProvider.ensureSession();
      final _MembershipViewData data;
      if (session == null) {
        data = const _MembershipViewData(ownerAccount: '', state: null);
      } else {
        final state = await _apiClient.fetchMembership(session);
        data = _MembershipViewData(
          ownerAccount: session.ownerAccount,
          state: state,
        );
      }
      if (!mounted) return;
      final defaultIndex =
          data.state == null ? 0 : _tierIndex(data.state!.identityLevel);
      setState(() {
        _data = data;
        _page = defaultIndex.toDouble();
        _loading = false;
      });
    } on Exception catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e;
        _loading = false;
      });
    }
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
    return Scaffold(
      appBar: AppBar(
        title: const Text('身份 ｜ 会员'),
        centerTitle: true,
        actions: [
          IconButton(
            tooltip: '刷新',
            onPressed: _loading ? null : _load,
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_loading) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_error != null) {
      return _MembershipMessage(
        title: '会员状态加载失败',
        message: '$_error',
        onRetry: _load,
      );
    }
    final data = _data;
    if (data == null || data.state == null) {
      return _MembershipMessage(
        title: '暂无默认热钱包',
        message: '创建默认热钱包后即可显示身份与会员状态。',
        onRetry: _load,
      );
    }
    final state = data.state!;
    // 3 张身份卡，各含 1~2 档会员套餐（访客含自由/民主两档）。
    final tierPlans = _plansByTier(state.plans);
    _cardCount = tierPlans.length;

    final size = MediaQuery.of(context).size;
    final cardWidth = (size.width * 0.8).clamp(280.0, 360.0);
    final bandHeight = (size.height * 0.63).clamp(460.0, 550.0);
    final peek = cardWidth * 0.40;
    final frontIndex = _page.round().clamp(0, tierPlans.length - 1);
    final activeColor = _tierColor(_tierOrder[frontIndex]);

    // 绘制顺序：离最上层越远越先画（在下层），命中卡最后画（压在最上层）。
    final drawOrder = List<int>.generate(tierPlans.length, (i) => i)
      ..sort((a, b) => (b - _page).abs().compareTo((a - _page).abs()));

    return Column(
      children: [
        if (state.frozen) _FrozenMembershipBanner(state: state),
        // 冻结时只显示冻结横幅（权益已停），不再叠加"有效订阅"横幅制造矛盾信息。
        if (state.hasSubscriptionWindow && !state.frozen)
          _ActiveMembershipBanner(state: state),
        Expanded(
          child: GestureDetector(
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
                        identityTier: _tierOrder[index],
                        tierPlans: tierPlans[index],
                        state: state,
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
          count: tierPlans.length,
          activeIndex: frontIndex,
          activeColor: activeColor,
        ),
        const SizedBox(height: 10),
        const Text(
          '左右滑动把候选档换到最上层 · 已停在你的身份卡',
          style: TextStyle(color: AppTheme.textTertiary, fontSize: 12),
        ),
        const SizedBox(height: 20),
      ],
    );
  }

  Widget _buildStackedCard({
    required int index,
    required String identityTier,
    required List<SquareMembershipPlan> tierPlans,
    required SquareMembershipState state,
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
            child: _IdentityMembershipCard(
              identityTier: identityTier,
              tierPlans: tierPlans,
              state: state,
              selectedPlanIndex:
                  identityTier == 'visitor' ? _visitorPlanIndex : 0,
              onSelectPlan: (i) => setState(() => _visitorPlanIndex = i),
              elevated: isFront,
            ),
          ),
        ),
      ),
    );

    if (isFront) {
      // 命中卡在最上层，内部按钮可点。
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
    required this.ownerAccount,
    required this.state,
  });

  final String ownerAccount;
  final SquareMembershipState? state;
}

/// 单张身份卡：一张卡 = 一个身份档（访客/投票/竞选）。访客档内含自由/民主两档
/// 会员套餐（[tierPlans] 长度 2，可切换）；投票/竞选各一档。档色顶带 + 右上角徽章
/// + 链上公开身份字段 + 选中套餐的会员权益/价格/订阅按钮。
class _IdentityMembershipCard extends StatelessWidget {
  const _IdentityMembershipCard({
    required this.identityTier,
    required this.tierPlans,
    required this.state,
    required this.selectedPlanIndex,
    required this.onSelectPlan,
    this.elevated = true,
  });

  /// 身份档：'visitor' / 'voting' / 'candidate'（决定卡色、档名、身份字段清单）。
  final String identityTier;

  /// 该身份档下可订阅套餐（访客 2 档，其余 1 档），已按价升序（自由在前）。
  final List<SquareMembershipPlan> tierPlans;

  final SquareMembershipState state;

  /// 当前展示的套餐下标（仅访客档有自由/民主切换）。
  final int selectedPlanIndex;
  final ValueChanged<int> onSelectPlan;

  /// 是否在最上层（决定投影强度，让命中/候选卡「浮」在下层卡之上）。
  final bool elevated;

  @override
  Widget build(BuildContext context) {
    final tier = identityTier;
    final tierColor = _tierColor(tier);
    final onTier = _onTierColor(tier);
    final userTier = _normalizeIdentity(state.identityLevel);
    final isUserCard = tier == userTier;
    final hasToggle = tierPlans.length > 1;
    final selIndex = selectedPlanIndex.clamp(0, tierPlans.length - 1);
    final plan = tierPlans[selIndex];
    final action = _actionFor(state, plan.membershipLevel);
    // 精确匹配：仅本人身份档可订阅（禁止降档/越级），其余置灰。
    final canSubscribe = isUserCard;

    // 徽章：底色恒为该身份档色；只有「你的身份卡」按真实会员态显示勾/小人。
    final badgeStyle = identityBadgeStyle(
      identityLevel: tier,
      membershipLevel: isUserCard ? state.membershipLevel : null,
      membershipActive: isUserCard && state.active,
    );

    return Container(
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        border: Border.all(
          color: isUserCard ? tierColor : AppTheme.border,
          width: isUserCard ? 2 : 1,
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
          _buildHeader(tier, tierColor, onTier, isUserCard, badgeStyle),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 14, 16, 16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  // 身份 + 权益占中段弹性空间，过长自动可滚，价格/按钮吸底；
                  // 三档卡外框同尺寸，内容多少不影响卡片大小。
                  Expanded(
                    child: SingleChildScrollView(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          _SectionLabel(
                            icon: Icons.fingerprint,
                            text: '链上公开的身份信息',
                            color: tierColor,
                          ),
                          const SizedBox(height: 8),
                          ..._buildIdentityRows(tier, tierColor),
                          const SizedBox(height: 14),
                          _SectionLabel(
                            icon: Icons.workspace_premium_outlined,
                            text: '会员权益',
                            color: tierColor,
                          ),
                          const SizedBox(height: 8),
                          _ParamLine(
                            icon: Icons.dynamic_feed_outlined,
                            color: tierColor,
                            text: plan.dynamicLabel,
                          ),
                          const SizedBox(height: 8),
                          _ParamLine(
                            icon: Icons.article_outlined,
                            color: tierColor,
                            text: plan.articleLabel,
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  // 访客档：自由/民主切换（默认自由，点击切换价格/权益/按钮）。
                  if (hasToggle) ...[
                    _PlanToggle(
                      plans: tierPlans,
                      selectedIndex: selIndex,
                      color: tierColor,
                      onSelect: onSelectPlan,
                    ),
                    const SizedBox(height: 10),
                  ],
                  // 价格统一：档色填充标签。
                  Container(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                    decoration: BoxDecoration(
                      color: tierColor,
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Text(
                      plan.priceLabel,
                      style: TextStyle(
                        color: onTier,
                        fontSize: 16,
                        fontWeight: FontWeight.w800,
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  SizedBox(
                    width: double.infinity,
                    child: _SubscribeButton(
                      label: _actionLabel(action),
                      color: tierColor,
                      onTap: canSubscribe
                          ? () => _openMembershipSite(context)
                          : null,
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

  Widget _buildHeader(
    String tier,
    Color tierColor,
    Color onTier,
    bool isUserCard,
    IdentityBadgeStyle? badgeStyle,
  ) {
    return Container(
      color: tierColor,
      padding: const EdgeInsets.fromLTRB(16, 14, 16, 14),
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                '身份 · ${_identityName(tier)}',
                style: TextStyle(
                  color: onTier.withValues(alpha: 0.82),
                  fontSize: 12,
                  fontWeight: FontWeight.w500,
                  letterSpacing: 0.4,
                ),
              ),
              const SizedBox(height: 4),
              Text(
                _cardName(tier),
                style: TextStyle(
                  color: onTier,
                  fontSize: 19,
                  fontWeight: FontWeight.w700,
                ),
              ),
              if (isUserCard) ...[
                const SizedBox(height: 10),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: onTier.withAlpha(38),
                    borderRadius: BorderRadius.circular(999),
                  ),
                  child: Text(
                    '你的身份',
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
          if (badgeStyle != null)
            Positioned(
              top: 0,
              right: 0,
              // 半透明白圆底：徽章扇贝与顶带同为档色，加此背景才能「浮」出来。
              child: Container(
                padding: const EdgeInsets.all(5),
                decoration: BoxDecoration(
                  color: Colors.white.withValues(alpha: 0.85),
                  shape: BoxShape.circle,
                  boxShadow: [
                    BoxShadow(
                      color: Colors.black.withValues(alpha: 0.10),
                      blurRadius: 6,
                      offset: const Offset(0, 2),
                    ),
                  ],
                ),
                child: IdentityBadge(
                  style: badgeStyle,
                  size: 34,
                  tooltip: identityBadgeLabel(
                    identityLevel: tier,
                    checked: badgeStyle.checked,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }

  Future<void> _openMembershipSite(BuildContext context) async {
    final uri = Uri.tryParse(MembershipSiteConfig.membershipUrl);
    var ok = false;
    if (uri != null) {
      try {
        ok = await launchUrl(uri, mode: LaunchMode.externalApplication);
      } on Exception catch (_) {
        ok = false;
      }
    }
    if (!ok && context.mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('无法打开官网，请稍后再试')),
      );
    }
  }
}

/// 订阅按钮三态：未订阅→订阅 / 自动续费中→取消订阅 / 已取消未到期→续订会员。
enum _SubscribeAction { subscribe, cancel, resume }

_SubscribeAction _actionFor(SquareMembershipState state, String tier) {
  final isActiveTier =
      state.subscriptionActive && state.membershipLevel == tier;
  if (!isActiveTier) return _SubscribeAction.subscribe;
  return state.cancelAtPeriodEnd
      ? _SubscribeAction.resume
      : _SubscribeAction.cancel;
}

String _actionLabel(_SubscribeAction action) => switch (action) {
      _SubscribeAction.subscribe => '订阅',
      _SubscribeAction.cancel => '取消订阅',
      _SubscribeAction.resume => '续订会员',
    };

class _SubscribeButton extends StatelessWidget {
  const _SubscribeButton({
    required this.label,
    required this.color,
    required this.onTap,
  });

  final String label;
  final Color color;

  /// null=非本人身份档，按钮置灰不可点（精确匹配，禁止降档/越级）。
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final enabled = onTap != null;
    return FilledButton.icon(
      onPressed: onTap,
      icon: Icon(enabled ? Icons.open_in_new : Icons.lock_outline, size: 16),
      label: Text(enabled ? label : '仅本档身份可订阅'),
      style: FilledButton.styleFrom(
        backgroundColor: color,
        foregroundColor: Colors.white,
        disabledBackgroundColor: AppTheme.surfaceElevated,
        disabledForegroundColor: AppTheme.textTertiary,
        minimumSize: const Size.fromHeight(46),
        textStyle: const TextStyle(fontSize: 14, fontWeight: FontWeight.w700),
      ),
    );
  }
}

/// 访客卡自由/民主分段切换：选中段填档色，点击切换展示套餐。
class _PlanToggle extends StatelessWidget {
  const _PlanToggle({
    required this.plans,
    required this.selectedIndex,
    required this.color,
    required this.onSelect,
  });

  final List<SquareMembershipPlan> plans;
  final int selectedIndex;
  final Color color;
  final ValueChanged<int> onSelect;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(3),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(10),
      ),
      child: Row(
        children: [
          for (var i = 0; i < plans.length; i++)
            Expanded(
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => onSelect(i),
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 180),
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: i == selectedIndex ? color : Colors.transparent,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Text(
                    plans[i].displayName,
                    style: TextStyle(
                      color: i == selectedIndex
                          ? _onTierColor(plans[i].requiredIdentityLevel)
                          : AppTheme.textSecondary,
                      fontSize: 13,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// 卡内分区小标题（链上身份信息 / 会员权益）。
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

/// 链上公开身份字段行（通用字段名，非任何用户真实值）。
class _IdentityFieldRow extends StatelessWidget {
  const _IdentityFieldRow({
    required this.icon,
    required this.text,
    required this.color,
  });

  final IconData icon;
  final String text;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        children: [
          Icon(icon, size: 16, color: color),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              text,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 13,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 各档链上公开身份字段清单（通用模板）：访客=没有链上身份；投票=CID/居住选区/
/// 有效期；竞选=在投票基础上再公开姓名/性别/出生地/出生日期。全为字段名，不读个人数据。
List<Widget> _buildIdentityRows(String tier, Color color) {
  if (tier == 'visitor') {
    return const [_AnonymousBlock()];
  }
  final List<(IconData, String)> fields = switch (tier) {
    'candidate' => const [
        (Icons.badge_outlined, '公民身份 CID 号'),
        (Icons.place_outlined, '居住选区'),
        (Icons.event_available_outlined, '身份有效期'),
        (Icons.person_outline, '真实姓名'),
        (Icons.wc_outlined, '性别'),
        (Icons.location_city_outlined, '出生地'),
        (Icons.cake_outlined, '出生日期'),
      ],
    _ => const [
        (Icons.badge_outlined, '公民身份 CID 号'),
        (Icons.place_outlined, '居住选区'),
        (Icons.event_available_outlined, '投票身份有效期'),
      ],
  };
  return [
    for (final field in fields)
      _IdentityFieldRow(icon: field.$1, text: field.$2, color: color),
  ];
}

/// 访客身份区：「完全匿名」大字 + 「没有链上身份」小字。
class _AnonymousBlock extends StatelessWidget {
  const _AnonymousBlock();

  @override
  Widget build(BuildContext context) {
    return const Row(
      children: [
        Icon(Icons.person_off_outlined,
            size: 24, color: AppTheme.textSecondary),
        SizedBox(width: 10),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              '完全匿名',
              style: TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 18,
                fontWeight: FontWeight.w700,
              ),
            ),
            SizedBox(height: 2),
            Text(
              '没有链上身份',
              style: TextStyle(
                color: AppTheme.textTertiary,
                fontSize: 11,
              ),
            ),
          ],
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

/// 冻结横幅（ADR-033 规则5）：链上身份与会员档位不匹配 → 权益已冻结、已暂停收款，
/// 提示到官网换档到与身份匹配的会员档解冻（各身份卡的订阅按钮即换档入口）。
class _FrozenMembershipBanner extends StatelessWidget {
  const _FrozenMembershipBanner({required this.state});

  final SquareMembershipState state;

  @override
  Widget build(BuildContext context) {
    final message = (state.inactiveMessage?.isNotEmpty ?? false)
        ? state.inactiveMessage!
        : '链上身份已变更，会员权益已冻结，请到官网换档到与身份匹配的会员档。';
    return Container(
      margin: const EdgeInsets.fromLTRB(16, 12, 16, 0),
      padding: const EdgeInsets.all(12),
      decoration: AppTheme.bannerDecoration(AppTheme.danger),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Icon(Icons.ac_unit, size: 18, color: AppTheme.danger),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              message,
              style: const TextStyle(
                color: AppTheme.danger,
                fontSize: 12,
                height: 1.5,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 订阅起止横幅（ADR-034 段4）：展示当前有效会员的档位、支付路线与订阅起止日期。
/// 会员操作（订阅 / 取消 / 换档）在官网完成，App 只读展示。
class _ActiveMembershipBanner extends StatelessWidget {
  const _ActiveMembershipBanner({required this.state});

  final SquareMembershipState state;

  @override
  Widget build(BuildContext context) {
    final plan = state.planForLevel(state.membershipLevel);
    final name = plan?.displayName ?? '会员';
    // 路线标签直白点出续费行为：预付到期失效、卡已取消到期终止、卡在续则自动续费。
    final route = state.isPrepaid
        ? '预付 · 到期失效'
        : state.cancelAtPeriodEnd
            ? '已取消 · 到期终止'
            : '自动续费';
    final window =
        '订阅 ${_formatYmd(state.currentPeriodStart)} ~ ${_formatYmd(state.expiresAt)}';
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

class _MembershipMessage extends StatelessWidget {
  const _MembershipMessage({
    required this.title,
    required this.message,
    required this.onRetry,
  });

  final String title;
  final String message;
  final Future<void> Function() onRetry;

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              title,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontWeight: FontWeight.w700,
                fontSize: 18,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              message,
              textAlign: TextAlign.center,
              style: const TextStyle(color: AppTheme.textSecondary),
            ),
            const SizedBox(height: 16),
            OutlinedButton(
              onPressed: onRetry,
              child: const Text('刷新'),
            ),
          ],
        ),
      ),
    );
  }
}

/// 四档兜底套餐：Worker 未下发 plans 时使用，与官网和 Worker 参数对齐。
const List<SquareMembershipPlan> _fallbackMembershipPlans = [
  SquareMembershipPlan(
    membershipLevel: 'freedom',
    displayName: '自由会员',
    priceUsdMonthly: '2.99',
    requiredIdentityLevel: 'visitor',
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'sd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'sd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 60,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 20000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'sd',
    articleMaxImages: 50,
  ),
  // 民主会员：访客身份的 $9.99 高权益档，权益对齐投票公民会员。
  SquareMembershipPlan(
    membershipLevel: 'democracy',
    displayName: '民主会员',
    priceUsdMonthly: '9.99',
    requiredIdentityLevel: 'visitor',
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'hd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'hd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 1800,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 30000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'hd',
    articleMaxImages: 100,
  ),
  SquareMembershipPlan(
    membershipLevel: 'voting',
    displayName: '投票公民会员',
    priceUsdMonthly: '9.99',
    requiredIdentityLevel: 'voting',
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'hd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'hd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 1800,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 30000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'hd',
    articleMaxImages: 100,
  ),
  SquareMembershipPlan(
    membershipLevel: 'candidate',
    displayName: '竞选公民会员',
    priceUsdMonthly: '99.99',
    requiredIdentityLevel: 'candidate',
    dynamicTextMaxChars: 300,
    dynamicImageQuality: 'hd',
    dynamicMaxImages: 9,
    dynamicVideoQuality: 'hd',
    dynamicMaxVideos: 1,
    dynamicMaxVideoSeconds: 10800,
    articleTitleMinChars: 10,
    articleTitleMaxChars: 50,
    articleBodyMaxChars: 30000,
    articleCoverQuality: 'hd',
    articleImageQuality: 'hd',
    articleMaxImages: 100,
  ),
];

/// 按身份档聚合套餐（3 张卡）：每档取 required_identity_level 匹配的套餐，档内按价
/// 升序（自由在前）。访客档 → [自由 freedom, 民主 democracy]；投票/竞选各一档。
/// worker 缺档用兜底补齐。
List<List<SquareMembershipPlan>> _plansByTier(
    List<SquareMembershipPlan> plans) {
  return _tierOrder.map((tier) {
    final tierPlans =
        plans.where((plan) => plan.requiredIdentityLevel == tier).toList();
    final resolved = tierPlans.isNotEmpty
        ? tierPlans
        : _fallbackMembershipPlans
            .where((plan) => plan.requiredIdentityLevel == tier)
            .toList();
    resolved.sort((a, b) => (double.tryParse(a.priceUsdMonthly) ?? 0)
        .compareTo(double.tryParse(b.priceUsdMonthly) ?? 0));
    return resolved;
  }).toList();
}

Color _tierColor(String tier) => switch (tier) {
      'candidate' => AppTheme.identityCandidate,
      'voting' => AppTheme.identityVoting,
      _ => AppTheme.identityVisitor,
    };

/// 顶带/价格标签前景色：访客金底用深棕保证对比度，投票蓝/竞选红底用白字。
Color _onTierColor(String tier) =>
    tier == 'visitor' ? const Color(0xFF4A3000) : Colors.white;

String _cardName(String tier) => switch (tier) {
      'candidate' => '公民轻节点 · 竞选',
      'voting' => '公民轻节点 · 投票',
      _ => '访客轻节点',
    };

String _identityName(String tier) => switch (tier) {
      'candidate' => '竞选公民',
      'voting' => '投票公民',
      _ => '访客',
    };

int _rank(String? tier) => switch (tier) {
      'candidate' => 2,
      'voting' => 1,
      _ => 0,
    };

String _normalizeIdentity(String? level) => switch (level) {
      'candidate' => 'candidate',
      'voting' => 'voting',
      _ => 'visitor',
    };

int _tierIndex(String? level) => _rank(_normalizeIdentity(level));
