import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:image_picker/image_picker.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/user/observe_accounts.dart';
import 'package:wuminapp_mobile/login/pages/settings_page.dart';
import 'package:wuminapp_mobile/wallet/capabilities/sfid_binding_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/wallet_page.dart';

class UserProfileState {
  const UserProfileState({
    required this.nickname,
    this.avatarPath,
  });

  final String nickname;
  final String? avatarPath;
}

class UserProfileService {
  static const _kNickname = 'user.profile.nickname';
  static const _kAvatarPath = 'user.profile.avatar_path';

  Future<UserProfileState> getState() async {
    final prefs = await SharedPreferences.getInstance();
    return UserProfileState(
      nickname: prefs.getString(_kNickname) ?? '公民用户',
      avatarPath: prefs.getString(_kAvatarPath),
    );
  }

  Future<UserProfileState> saveState(UserProfileState state) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kNickname, state.nickname.trim());
    if (state.avatarPath == null || state.avatarPath!.trim().isEmpty) {
      await prefs.remove(_kAvatarPath);
    } else {
      await prefs.setString(_kAvatarPath, state.avatarPath!.trim());
    }
    return getState();
  }
}

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

  Future<void> _openUserQr() async {
    final payload = jsonEncode({
      'type': 'WUMINAPP_USER_V1',
      'nickname': _userProfile.nickname,
      'avatar_path': _userProfile.avatarPath ?? '',
      'sfid_bind_status': _bindState.status.name,
      'wallet_address': _bindState.walletAddress ?? '',
    });
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => UserQrPage(
          nickname: _userProfile.nickname,
          avatarPath: _userProfile.avatarPath,
          qrPayload: payload,
        ),
      ),
    );
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
                _SquareAvatar(path: _userProfile.avatarPath, size: 84),
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
                              child: InkWell(
                                onTap: _openUserQr,
                                borderRadius: BorderRadius.circular(8),
                                child: const Padding(
                                  padding: EdgeInsets.all(2),
                                  child: Icon(
                                    Icons.qr_code_2,
                                    size: 21,
                                    color: Colors.grey,
                                  ),
                                ),
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

class ProfileEditPage extends StatefulWidget {
  const ProfileEditPage({
    super.key,
    required this.initialState,
  });

  final UserProfileState initialState;

  @override
  State<ProfileEditPage> createState() => _ProfileEditPageState();
}

class _ProfileEditPageState extends State<ProfileEditPage> {
  final ImagePicker _imagePicker = ImagePicker();
  late final TextEditingController _nicknameController;
  String? _avatarPath;

  @override
  void initState() {
    super.initState();
    _nicknameController =
        TextEditingController(text: widget.initialState.nickname);
    _avatarPath = widget.initialState.avatarPath;
  }

  @override
  void dispose() {
    _nicknameController.dispose();
    super.dispose();
  }

  Future<void> _pickAvatarFromGallery() async {
    try {
      final picked = await _imagePicker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 1024,
        maxHeight: 1024,
      );
      if (picked == null || !mounted) {
        return;
      }
      setState(() {
        _avatarPath = picked.path;
      });
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('访问相册失败：$e')),
      );
    }
  }

  void _save() {
    final nickname = _nicknameController.text.trim();
    if (nickname.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('用户昵称不能为空')),
      );
      return;
    }
    Navigator.of(context).pop(
      UserProfileState(
        nickname: nickname,
        avatarPath: _avatarPath,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('修改用户资料'),
        centerTitle: true,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                _SquareAvatar(path: _avatarPath, size: 84),
                const SizedBox(width: 14),
                OutlinedButton(
                  onPressed: _pickAvatarFromGallery,
                  child: const Text('设置头像'),
                ),
              ],
            ),
            const SizedBox(height: 18),
            TextField(
              controller: _nicknameController,
              decoration: const InputDecoration(
                labelText: '用户昵称',
                hintText: '请输入用户昵称',
                border: OutlineInputBorder(),
              ),
            ),
            const SizedBox(height: 18),
            FilledButton(
              onPressed: _save,
              child: const Text('保存'),
            ),
          ],
        ),
      ),
    );
  }
}

class UserQrPage extends StatelessWidget {
  const UserQrPage({
    super.key,
    required this.nickname,
    required this.avatarPath,
    required this.qrPayload,
  });

  final String nickname;
  final String? avatarPath;
  final String qrPayload;

  Future<void> _copyPayload(BuildContext context) async {
    await Clipboard.setData(ClipboardData(text: qrPayload));
    if (!context.mounted) {
      return;
    }
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('二维码内容已复制')),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('我的二维码'),
        centerTitle: true,
      ),
      body: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          children: [
            Row(
              children: [
                _SquareAvatar(path: avatarPath, size: 52),
                const SizedBox(width: 10),
                Expanded(
                  child: Text(
                    nickname,
                    style: const TextStyle(
                      fontSize: 18,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 22),
            Expanded(
              child: Center(
                child: Container(
                  padding: const EdgeInsets.all(16),
                  decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(14),
                    border: Border.all(color: const Color(0xFFE5ECE8)),
                  ),
                  child: QrImageView(
                    data: qrPayload,
                    version: QrVersions.auto,
                    size: 260,
                    backgroundColor: Colors.white,
                  ),
                ),
              ),
            ),
            SizedBox(
              width: double.infinity,
              child: OutlinedButton.icon(
                onPressed: () => _copyPayload(context),
                icon: const Icon(Icons.copy),
                label: const Text('复制二维码内容'),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SquareAvatar extends StatelessWidget {
  const _SquareAvatar({required this.path, required this.size});

  final String? path;
  final double size;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final validImage = hasImage && File(path!).existsSync();
    return ClipRRect(
      borderRadius: BorderRadius.circular(10),
      child: Container(
        width: size,
        height: size,
        color: const Color(0xFFE3EFE8),
        child: validImage
            ? Image.file(
                File(path!),
                fit: BoxFit.cover,
              )
            : const Icon(
                Icons.person,
                size: 40,
                color: Color(0xFF0B3D2E),
              ),
      ),
    );
  }
}
