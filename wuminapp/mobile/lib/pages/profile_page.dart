import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/features/observe_accounts/observe_accounts_page.dart';
import 'package:wuminapp_mobile/pages/profile_edit_page.dart';
import 'package:wuminapp_mobile/pages/settings_page.dart';
import 'package:wuminapp_mobile/wallet/capabilities/sfid_binding_service.dart';
import 'package:wuminapp_mobile/services/user_profile_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/wallet_page.dart';

class ProfilePage extends StatefulWidget {
  const ProfilePage({super.key});

  @override
  State<ProfilePage> createState() => _ProfilePageState();
}

class _ProfilePageState extends State<ProfilePage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);
  final SfidBindingService _sfidBindingService = SfidBindingService();
  final UserProfileService _userProfileService = UserProfileService();
  bool _bindingSubmitting = false;
  SfidBindState _bindState =
      const SfidBindState(status: SfidBindStatus.unbound);
  UserProfileState _userProfile = const UserProfileState(nickname: '公民用户');

  @override
  void initState() {
    super.initState();
    _loadState();
  }

  Future<void> _loadState() async {
    final bindFuture = _sfidBindingService.getState();
    final profileFuture = _userProfileService.getState();
    final bindState = await bindFuture;
    final profile = await profileFuture;
    if (!mounted) {
      return;
    }
    setState(() {
      _bindState = bindState;
      _userProfile = profile;
    });
  }

  Future<void> _handleBindIdentity() async {
    if (_bindState.status != SfidBindStatus.unbound || _bindingSubmitting) {
      return;
    }
    final wallet = await Navigator.of(context).push<WalletProfile>(
      MaterialPageRoute(
        builder: (_) => const MyWalletPage(selectForBind: true),
      ),
    );
    if (!mounted || wallet == null) {
      return;
    }
    setState(() {
      _bindingSubmitting = true;
    });
    try {
      final state = await _sfidBindingService.submitBinding(
        wallet.address,
        wallet.pubkeyHex,
      );
      if (!mounted) {
        return;
      }
      setState(() {
        _bindState = state;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('已提交链上绑定请求，等待链侧与SFID确认')),
      );
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('绑定请求发送失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() {
          _bindingSubmitting = false;
        });
      }
    }
  }

  Future<void> _openProfileEdit() async {
    final result = await Navigator.of(context).push<UserProfileState>(
      MaterialPageRoute(
        builder: (_) => ProfileEditPage(initialState: _userProfile),
      ),
    );
    if (!mounted || result == null) {
      return;
    }
    final saved = await _userProfileService.saveState(result);
    if (!mounted) {
      return;
    }
    setState(() {
      _userProfile = saved;
    });
  }

  Widget _buildBindAction() {
    if (_bindState.status == SfidBindStatus.pending) {
      return const SizedBox(
        height: 20,
        child: FilledButton(
          onPressed: null,
          style: ButtonStyle(
            padding: WidgetStatePropertyAll(
              EdgeInsets.symmetric(horizontal: 6),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.schedule, size: 4),
              SizedBox(width: 2),
              Text('待确认', style: TextStyle(fontSize: 9)),
            ],
          ),
        ),
      );
    }
    if (_bindState.status == SfidBindStatus.bound) {
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
        decoration: BoxDecoration(
          color: const Color(0xFFE9F5EF),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              _bindState.walletAddress ?? '',
              style: const TextStyle(
                color: _inkGreen,
                fontSize: 12,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(width: 6),
            const Icon(Icons.verified, size: 13, color: _inkGreen),
          ],
        ),
      );
    }
    return SizedBox(
      height: 20,
      child: FilledButton(
        onPressed: _bindingSubmitting ? null : _handleBindIdentity,
        style: const ButtonStyle(
          padding: WidgetStatePropertyAll(
            EdgeInsets.symmetric(horizontal: 6),
          ),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              _bindingSubmitting
                  ? Icons.hourglass_top_outlined
                  : Icons.verified_user_outlined,
              size: 4,
            ),
            const SizedBox(width: 2),
            Text(
              _bindingSubmitting ? '提交中' : '绑定身份',
              style: const TextStyle(fontSize: 9),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final hasAvatar = _userProfile.avatarPath != null &&
        _userProfile.avatarPath!.trim().isNotEmpty &&
        File(_userProfile.avatarPath!).existsSync();
    return SafeArea(
      child: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const Center(
            child: Text(
              '我的',
              style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700),
            ),
          ),
          const SizedBox(height: 26),
          Container(
            padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
            decoration: BoxDecoration(
              color: const Color(0xFFF7FAF8),
              borderRadius: BorderRadius.circular(14),
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                ClipRRect(
                  borderRadius: BorderRadius.circular(10),
                  child: Container(
                    width: 84,
                    height: 84,
                    color: const Color(0xFFE3EFE8),
                    child: hasAvatar
                        ? Image.file(
                            File(_userProfile.avatarPath!),
                            fit: BoxFit.cover,
                          )
                        : const Icon(Icons.person, color: _inkGreen, size: 38),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Expanded(
                            child: Transform.translate(
                              offset: const Offset(0, -2),
                              child: Text(
                                _userProfile.nickname,
                                style: const TextStyle(
                                  fontSize: 19,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            ),
                          ),
                          Padding(
                            padding: const EdgeInsets.only(top: 1),
                            child: Transform.translate(
                              offset: const Offset(0, -10),
                              child: const Icon(
                                Icons.qr_code_2,
                                size: 21,
                                color: Colors.grey,
                              ),
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 4),
                      Row(
                        crossAxisAlignment: CrossAxisAlignment.center,
                        children: [
                          _buildBindAction(),
                          const Spacer(),
                          InkWell(
                            onTap: _openProfileEdit,
                            borderRadius: BorderRadius.circular(8),
                            child: const SizedBox(
                              width: 28,
                              height: 28,
                              child: Icon(
                                Icons.chevron_right,
                                size: 24,
                                color: Color(0xFF4D4D4D),
                              ),
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 28),
          Card(
            child: ListTile(
              leading: SvgPicture.asset(
                'assets/icons/wallet.svg',
                width: 22,
                height: 22,
                colorFilter: const ColorFilter.mode(_inkGreen, BlendMode.srcIn),
              ),
              title: const Text(
                '钱包',
                style: TextStyle(fontWeight: FontWeight.w700),
              ),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(builder: (_) => const MyWalletPage()),
                );
              },
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(
                Icons.remove_red_eye_outlined,
                color: _inkGreen,
                size: 22,
              ),
              title: const Text(
                '观察账户',
                style: TextStyle(fontWeight: FontWeight.w700),
              ),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(
                    builder: (_) => const ObserveAccountsPage(),
                  ),
                );
              },
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(
                Icons.settings_outlined,
                color: _inkGreen,
                size: 22,
              ),
              title: const Text(
                '设置',
                style: TextStyle(fontWeight: FontWeight.w700),
              ),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(builder: (_) => const SettingsPage()),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
