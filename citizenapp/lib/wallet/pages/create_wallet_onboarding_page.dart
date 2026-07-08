import 'package:flutter/material.dart';
import 'package:local_auth/local_auth.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/pages/create_wallet_flow.dart';

/// 首启强制创建钱包页。
///
/// 公民 App 用户的唯一账户是钱包账户，发消息、发动态、发起交易都依赖热钱包
/// 签名，因此本页只允许创建 12/24 个助记词的热钱包：无返回、无导入、无冷
/// 钱包入口。创建成功后经 [onCreated] 通知 WalletGate 放行主界面。
class CreateWalletOnboardingPage extends StatefulWidget {
  const CreateWalletOnboardingPage({
    super.key,
    required this.onCreated,
    this.deviceSecureProbe,
  });

  /// 创建成功回调（WalletGate 收到后翻转到主界面）。
  final VoidCallback onCreated;

  /// 系统锁屏可用性探测，测试注入用；默认走 local_auth 的 isDeviceSupported。
  final Future<bool> Function()? deviceSecureProbe;

  @override
  State<CreateWalletOnboardingPage> createState() =>
      _CreateWalletOnboardingPageState();
}

class _CreateWalletOnboardingPageState extends State<CreateWalletOnboardingPage>
    with WidgetsBindingObserver {
  /// null = 检测中；createWallet 前置要求系统锁屏已开启，未开启时禁用创建。
  bool? _deviceSecure;
  bool _creating = false;
  int _wordCount = 12;
  String? _error;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    _probeDeviceSecure();
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    // 用户去系统设置开完锁屏回到前台，自动复检。
    if (state == AppLifecycleState.resumed) {
      _probeDeviceSecure();
    }
  }

  Future<void> _probeDeviceSecure() async {
    bool secure;
    try {
      final probe = widget.deviceSecureProbe ??
          LocalAuthentication().isDeviceSupported;
      secure = await probe();
    } catch (_) {
      // 探测不可用按未开锁屏处理（fail-closed），与 createWallet 的前置一致。
      secure = false;
    }
    if (!mounted) return;
    setState(() => _deviceSecure = secure);
  }

  Future<void> _create() async {
    setState(() {
      _creating = true;
      _error = null;
    });
    try {
      await runCreateWalletFlow(context, wordCount: _wordCount);
      if (!mounted) return;
      widget.onCreated();
    } catch (e, st) {
      debugPrint('onboarding wallet create failed: $e\n$st');
      if (!mounted) return;
      setState(() => _error = walletOperationErrorMessage(e));
      // 创建失败常见原因是锁屏状态变化，顺手复检刷新警示卡。
      _probeDeviceSecure();
    } finally {
      if (mounted) {
        setState(() => _creating = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final canCreate = _deviceSecure == true && !_creating;
    return PopScope(
      canPop: false,
      child: Scaffold(
        backgroundColor: AppTheme.scaffoldBg,
        body: SafeArea(
          child: Center(
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxWidth: 420),
              child: ListView(
                padding: const EdgeInsets.fromLTRB(24, 40, 24, 24),
                children: [
                  Center(
                    child: Container(
                      width: 56,
                      height: 56,
                      decoration: BoxDecoration(
                        gradient: AppTheme.primaryGradient,
                        borderRadius: BorderRadius.circular(14),
                      ),
                      child: const Icon(
                        Icons.account_balance_wallet_outlined,
                        color: Colors.white,
                        size: 26,
                      ),
                    ),
                  ),
                  const SizedBox(height: 16),
                  const Text(
                    '创建你的公民钱包',
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: 20,
                      fontWeight: FontWeight.w700,
                      color: AppTheme.textPrimary,
                    ),
                  ),
                  const SizedBox(height: 8),
                  const Text(
                    '钱包账户是你在公民 App 的唯一账户，发消息、发动态、发起交易都由它签名',
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: 13,
                      height: 1.6,
                      color: AppTheme.textSecondary,
                    ),
                  ),
                  const SizedBox(height: 24),
                  if (_deviceSecure == false) ...[
                    _buildDeviceLockWarning(),
                    const SizedBox(height: 16),
                  ],
                  const Text(
                    '助记词长度',
                    style: TextStyle(
                      fontSize: 12,
                      color: AppTheme.textTertiary,
                    ),
                  ),
                  const SizedBox(height: 8),
                  _WordCountCard(
                    wordCount: 12,
                    subtitle: '128 位熵 · 标准安全强度',
                    recommended: true,
                    selected: _wordCount == 12,
                    onTap: () => setState(() => _wordCount = 12),
                  ),
                  const SizedBox(height: 10),
                  _WordCountCard(
                    wordCount: 24,
                    subtitle: '256 位熵 · 安全性更高',
                    recommended: false,
                    selected: _wordCount == 24,
                    onTap: () => setState(() => _wordCount = 24),
                  ),
                  const SizedBox(height: 20),
                  const _SecurityNoteRow(
                    icon: Icons.lock_outline,
                    text: '助记词加密存储在本机，可在钱包详情中查看',
                  ),
                  const SizedBox(height: 8),
                  const _SecurityNoteRow(
                    icon: Icons.edit_outlined,
                    text: '请手抄备份——这是恢复钱包的唯一凭证',
                  ),
                  const SizedBox(height: 8),
                  const _SecurityNoteRow(
                    icon: Icons.visibility_off_outlined,
                    text: '展示助记词时禁止截屏，不支持复制',
                  ),
                  const SizedBox(height: 24),
                  if (_error != null) ...[
                    Text(
                      _error!,
                      style: const TextStyle(
                        fontSize: 12,
                        color: AppTheme.danger,
                      ),
                    ),
                    const SizedBox(height: 8),
                  ],
                  SizedBox(
                    height: 48,
                    child: FilledButton(
                      onPressed: canCreate ? _create : null,
                      child: Text(_creating ? '创建中…' : '创建热钱包'),
                    ),
                  ),
                  const SizedBox(height: 10),
                  Text(
                    _deviceSecure == false ? '开启系统锁屏后可创建' : '创建完成后进入公民广场',
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      fontSize: 11,
                      color: AppTheme.textTertiary,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildDeviceLockWarning() {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: AppTheme.bannerDecoration(AppTheme.warning),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(
                Icons.warning_amber_rounded,
                size: 18,
                color: AppTheme.warning,
              ),
              SizedBox(width: 8),
              Expanded(
                child: Text(
                  '未检测到系统锁屏',
                  style: TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.textPrimary,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          const Padding(
            padding: EdgeInsets.only(left: 26),
            child: Text(
              '钱包密钥依赖系统锁屏保护。请先在系统设置中开启屏幕锁定'
              '（数字密码、图案或生物识别），再返回创建。',
              style: TextStyle(
                fontSize: 12,
                height: 1.55,
                color: AppTheme.textSecondary,
              ),
            ),
          ),
          const SizedBox(height: 10),
          Padding(
            padding: const EdgeInsets.only(left: 26),
            child: OutlinedButton(
              onPressed: _probeDeviceSecure,
              child: const Text('重新检测'),
            ),
          ),
        ],
      ),
    );
  }
}

class _WordCountCard extends StatelessWidget {
  const _WordCountCard({
    required this.wordCount,
    required this.subtitle,
    required this.recommended,
    required this.selected,
    required this.onTap,
  });

  final int wordCount;
  final String subtitle;
  final bool recommended;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: AppTheme.surfaceWhite,
      borderRadius: BorderRadius.circular(AppTheme.radiusMd),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(AppTheme.radiusMd),
            border: Border.all(
              color: selected ? AppTheme.primary : AppTheme.border,
              width: selected ? 2 : 1,
            ),
          ),
          child: Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Text(
                          '$wordCount 个助记词',
                          style: TextStyle(
                            fontSize: 15,
                            fontWeight: FontWeight.w600,
                            color: selected
                                ? AppTheme.primaryDark
                                : AppTheme.textPrimary,
                          ),
                        ),
                        if (recommended) ...[
                          const SizedBox(width: 6),
                          Container(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 7,
                              vertical: 1,
                            ),
                            decoration: BoxDecoration(
                              color: AppTheme.primary,
                              borderRadius: BorderRadius.circular(8),
                            ),
                            child: const Text(
                              '推荐',
                              style: TextStyle(
                                fontSize: 10.5,
                                fontWeight: FontWeight.w500,
                                color: Colors.white,
                              ),
                            ),
                          ),
                        ],
                      ],
                    ),
                    const SizedBox(height: 2),
                    Text(
                      subtitle,
                      style: TextStyle(
                        fontSize: 11.5,
                        color: selected
                            ? AppTheme.primary
                            : AppTheme.textSecondary,
                      ),
                    ),
                  ],
                ),
              ),
              Icon(
                selected ? Icons.check_circle : Icons.circle_outlined,
                size: 20,
                color: selected ? AppTheme.primary : AppTheme.textTertiary,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _SecurityNoteRow extends StatelessWidget {
  const _SecurityNoteRow({required this.icon, required this.text});

  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(icon, size: 15, color: AppTheme.primary),
        const SizedBox(width: 8),
        Expanded(
          child: Text(
            text,
            style: const TextStyle(
              fontSize: 11.5,
              height: 1.5,
              color: AppTheme.textSecondary,
            ),
          ),
        ),
      ],
    );
  }
}
