import 'package:flutter/material.dart';
import 'package:citizenapp/security/app_permission_bootstrap.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 首次启动权限说明入口。
///
/// 中文注释：该页只解释并申请通知权限；网络权限由系统安装时自动授予，
/// 相机和相册等敏感权限仍在用户进入扫码、选图、保存二维码时由对应功能申请。
class AppPermissionGate extends StatefulWidget {
  const AppPermissionGate({
    super.key,
    required this.child,
  });

  final Widget child;

  @override
  State<AppPermissionGate> createState() => _AppPermissionGateState();
}

class _AppPermissionGateState extends State<AppPermissionGate> {
  bool _loading = true;
  bool _showGuide = false;
  bool _requesting = false;

  @override
  void initState() {
    super.initState();
    _loadGuideState();
  }

  Future<void> _loadGuideState() async {
    final shouldShow = await AppPermissionBootstrap.shouldShowGuide();
    if (!mounted) return;
    setState(() {
      _showGuide = shouldShow;
      _loading = false;
    });
  }

  Future<void> _continue({required bool requestNotification}) async {
    if (_requesting) return;
    setState(() => _requesting = true);
    if (requestNotification) {
      await AppPermissionBootstrap.requestNotificationPermission();
    }
    await AppPermissionBootstrap.markGuideSeen();
    if (!mounted) return;
    setState(() {
      _showGuide = false;
      _requesting = false;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (_loading) {
      return const Scaffold(
        body: Center(
          child: CircularProgressIndicator(
            strokeWidth: 2.5,
            color: AppTheme.primary,
          ),
        ),
      );
    }

    if (!_showGuide) {
      return widget.child;
    }

    return Scaffold(
      backgroundColor: AppTheme.surfaceWhite,
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(24, 32, 24, 24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Container(
                width: 56,
                height: 56,
                decoration: BoxDecoration(
                  gradient: AppTheme.primaryGradient,
                  borderRadius: BorderRadius.circular(16),
                ),
                child: const Icon(
                  Icons.notifications_active_outlined,
                  color: Colors.white,
                  size: 28,
                ),
              ),
              const SizedBox(height: 28),
              const Text(
                '权限设置',
                style: TextStyle(
                  color: AppTheme.textPrimary,
                  fontSize: 24,
                  fontWeight: FontWeight.w700,
                ),
              ),
              const SizedBox(height: 12),
              const Text(
                '网络权限用于链同步和版本更新，系统会自动授予，不会弹窗。通知权限用于后续交易、投票和更新提醒；相机与相册会在扫码、选图或保存二维码时再申请。',
                style: TextStyle(
                  color: AppTheme.textSecondary,
                  fontSize: 15,
                  height: 1.55,
                ),
              ),
              const SizedBox(height: 24),
              const _PermissionRow(
                icon: Icons.public_rounded,
                title: '网络',
                body: '安装时自动授予，用于轻节点和更新检查。',
              ),
              const SizedBox(height: 14),
              const _PermissionRow(
                icon: Icons.notifications_none_rounded,
                title: '通知',
                body: '现在可授权；拒绝后仍可正常进入应用。',
              ),
              const SizedBox(height: 14),
              const _PermissionRow(
                icon: Icons.photo_camera_outlined,
                title: '相机与相册',
                body: '扫码、选图、保存二维码时按功能申请。',
              ),
              const Spacer(),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  onPressed: _requesting
                      ? null
                      : () => _continue(requestNotification: true),
                  child: _requesting
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        )
                      : const Text('开启通知并继续'),
                ),
              ),
              const SizedBox(height: 10),
              SizedBox(
                width: double.infinity,
                child: TextButton(
                  onPressed: _requesting
                      ? null
                      : () => _continue(requestNotification: false),
                  child: const Text('稍后再说'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _PermissionRow extends StatelessWidget {
  const _PermissionRow({
    required this.icon,
    required this.title,
    required this.body,
  });

  final IconData icon;
  final String title;
  final String body;

  @override
  Widget build(BuildContext context) {
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          width: 38,
          height: 38,
          decoration: BoxDecoration(
            color: AppTheme.primary.withAlpha(20),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Icon(icon, color: AppTheme.primary, size: 20),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: const TextStyle(
                  color: AppTheme.textPrimary,
                  fontSize: 15,
                  fontWeight: FontWeight.w700,
                ),
              ),
              const SizedBox(height: 2),
              Text(
                body,
                style: const TextStyle(
                  color: AppTheme.textSecondary,
                  fontSize: 13,
                  height: 1.35,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
