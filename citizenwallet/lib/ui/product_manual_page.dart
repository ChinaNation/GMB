import 'package:flutter/material.dart';

import 'app_theme.dart';

/// 公民钱包产品手册。第一部分用密钥地图解释单向派生与公开边界。
class ProductManualPage extends StatelessWidget {
  const ProductManualPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('产品手册'),
        centerTitle: true,
        actions: const [
          Padding(
            padding: EdgeInsets.only(right: 16),
            child: Icon(Icons.bookmark_border_rounded),
          ),
        ],
      ),
      body: ListView(
        key: const Key('product-manual-scroll'),
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 22),
        children: const [
          _ManualHero(),
          SizedBox(height: 16),
          _KeyMap(),
          SizedBox(height: 16),
          _BackupNotice(),
        ],
      ),
    );
  }
}

class _ManualHero extends StatelessWidget {
  const _ManualHero();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        gradient: const LinearGradient(
          colors: [Color(0xFF17243A), Color(0xFF101B2D)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ),
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        border: Border.all(color: const Color(0xFF29486D)),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withAlpha(70),
            blurRadius: 18,
            offset: const Offset(0, 8),
          ),
        ],
      ),
      child: const Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            '密钥地图 · 第 1 部分',
            style: TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 14,
              fontWeight: FontWeight.w600,
            ),
          ),
          SizedBox(height: 10),
          Text(
            '一个钱包，五层关系',
            style: TextStyle(
              color: AppTheme.textPrimary,
              fontSize: 24,
              height: 1.2,
              fontWeight: FontWeight.w800,
            ),
          ),
          SizedBox(height: 6),
          Text(
            '从可手抄的备份，到可公开的地址',
            style: TextStyle(
              color: AppTheme.textSecondary,
              fontSize: 14,
              height: 1.45,
            ),
          ),
          SizedBox(height: 12),
          Divider(color: AppTheme.border),
          SizedBox(height: 10),
          Row(
            children: [
              Icon(
                Icons.shield_outlined,
                color: AppTheme.accent,
                size: 22,
              ),
              SizedBox(width: 10),
              Expanded(
                child: Text(
                  '箭头只表示单向派生，不能反推',
                  style: TextStyle(
                    color: AppTheme.textSecondary,
                    fontSize: 14,
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _KeyMap extends StatelessWidget {
  const _KeyMap();

  static const _steps = <_KeyStep>[
    _KeyStep(
      title: '助记词',
      subtitle: '钱包根备份',
      icon: Icons.menu_book_rounded,
      secret: true,
      left: true,
    ),
    _KeyStep(
      title: '种子',
      subtitle: '32 字节主种子',
      icon: Icons.spa_outlined,
      secret: true,
      left: false,
    ),
    _KeyStep(
      title: '私钥',
      subtitle: '账户独立硬派生',
      detail: '//0   //1   //2   …',
      icon: Icons.key_rounded,
      secret: true,
      left: true,
    ),
    _KeyStep(
      title: '公钥 / AccountId',
      subtitle: 'sr25519 · 签名验证',
      icon: Icons.verified_user_outlined,
      secret: false,
      left: false,
    ),
    _KeyStep(
      title: 'SS58 地址',
      subtitle: '展示与扫码编码',
      icon: Icons.qr_code_2_rounded,
      secret: false,
      left: true,
    ),
  ];

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        Positioned.fill(
          child: Center(
            child: Container(
              width: 2,
              margin: const EdgeInsets.symmetric(vertical: 44),
              decoration: BoxDecoration(
                gradient: const LinearGradient(
                  colors: [
                    Colors.transparent,
                    AppTheme.primaryLight,
                    AppTheme.accent,
                    Colors.transparent,
                  ],
                  begin: Alignment.topCenter,
                  end: Alignment.bottomCenter,
                ),
                boxShadow: [
                  BoxShadow(
                    color: AppTheme.primary.withAlpha(140),
                    blurRadius: 10,
                    spreadRadius: 2,
                  ),
                ],
              ),
            ),
          ),
        ),
        Column(
          children: [
            for (var i = 0; i < _steps.length; i++) ...[
              _KeyStepRow(step: _steps[i]),
              if (i != _steps.length - 1) const SizedBox(height: 10),
            ],
          ],
        ),
      ],
    );
  }
}

class _KeyStepRow extends StatelessWidget {
  const _KeyStepRow({required this.step});

  final _KeyStep step;

  @override
  Widget build(BuildContext context) {
    final accent = step.secret ? AppTheme.gold : AppTheme.accent;
    final card = _StepCard(step: step, accent: accent);
    final badge = _BoundaryBadge(secret: step.secret);
    final connector = SizedBox(
      width: 28,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          if (!step.left)
            Icon(Icons.arrow_forward_rounded, size: 16, color: accent),
          Container(
            width: 10,
            height: 10,
            decoration: BoxDecoration(
              color: AppTheme.primaryLight,
              shape: BoxShape.circle,
              border: Border.all(color: Colors.white, width: 2),
              boxShadow: [
                BoxShadow(
                  color: AppTheme.primary.withAlpha(180),
                  blurRadius: 10,
                ),
              ],
            ),
          ),
          if (step.left)
            Icon(Icons.arrow_back_rounded, size: 16, color: accent),
        ],
      ),
    );

    return SizedBox(
      height: step.detail == null ? 84 : 132,
      child: Row(
        children: step.left
            ? [
                Expanded(child: Row(children: [badge, card])),
                connector,
                const Expanded(child: SizedBox()),
              ]
            : [
                const Expanded(child: SizedBox()),
                connector,
                Expanded(child: Row(children: [card, badge])),
              ],
      ),
    );
  }
}

class _StepCard extends StatelessWidget {
  const _StepCard({required this.step, required this.accent});

  final _KeyStep step;
  final Color accent;

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Container(
        height: double.infinity,
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: const Color(0xFF142036),
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          border: Border.all(color: const Color(0xFF2A466D)),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withAlpha(55),
              blurRadius: 10,
              offset: const Offset(0, 5),
            ),
          ],
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Row(
              children: [
                Container(
                  width: 32,
                  height: 32,
                  decoration: BoxDecoration(
                    color: accent.withAlpha(12),
                    borderRadius: BorderRadius.circular(9),
                    border: Border.all(color: accent.withAlpha(130)),
                  ),
                  child: Icon(step.icon, color: accent, size: 18),
                ),
                const SizedBox(width: 7),
                Expanded(
                  child: Text(
                    step.title,
                    maxLines: 2,
                    style: const TextStyle(
                      color: AppTheme.textPrimary,
                      fontSize: 13,
                      height: 1.2,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              step.subtitle,
              maxLines: 2,
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontSize: 10.5,
                height: 1.3,
              ),
            ),
            if (step.detail != null) ...[
              const SizedBox(height: 5),
              Container(
                width: double.infinity,
                padding: const EdgeInsets.symmetric(
                  horizontal: 8,
                  vertical: 4,
                ),
                decoration: BoxDecoration(
                  color: AppTheme.scaffoldBg.withAlpha(150),
                  borderRadius: BorderRadius.circular(7),
                  border: Border.all(color: AppTheme.border),
                ),
                child: Text(
                  step.detail!,
                  style: const TextStyle(
                    color: AppTheme.primaryLight,
                    fontSize: 11,
                    fontFamily: 'monospace',
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _BoundaryBadge extends StatelessWidget {
  const _BoundaryBadge({required this.secret});

  final bool secret;

  @override
  Widget build(BuildContext context) {
    final color = secret ? AppTheme.gold : AppTheme.accent;
    return SizedBox(
      width: 38,
      child: Text(
        secret ? '必须\n保密' : '可以\n公开',
        textAlign: TextAlign.center,
        style: TextStyle(
          color: color,
          height: 1.45,
          fontSize: 11,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}

class _BackupNotice extends StatelessWidget {
  const _BackupNotice();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: const Color(0xFF13243A),
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        border: Border.all(color: const Color(0xFF29486D)),
      ),
      child: const Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(Icons.info_outline_rounded, color: AppTheme.accent, size: 26),
          SizedBox(width: 10),
          Expanded(
            child: Text(
              '备份助记词，就能重新派生全部账户；\n备份地址不能恢复钱包。',
              style: TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 13,
                height: 1.55,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _KeyStep {
  const _KeyStep({
    required this.title,
    required this.subtitle,
    required this.icon,
    required this.secret,
    required this.left,
    this.detail,
  });

  final String title;
  final String subtitle;
  final String? detail;
  final IconData icon;
  final bool secret;
  final bool left;
}
