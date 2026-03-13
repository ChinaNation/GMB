import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:image_picker/image_picker.dart';
import 'package:mobile_scanner/mobile_scanner.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:wuminapp_mobile/login/pages/settings_page.dart';
import 'package:wuminapp_mobile/user/user_service.dart';
import 'package:wuminapp_mobile/wallet/capabilities/sfid_binding_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/wallet_page.dart';

class ProfilePage extends StatefulWidget {
  const ProfilePage({super.key});

  @override
  State<ProfilePage> createState() => _ProfilePageState();
}

class _ProfilePageState extends State<ProfilePage> {
  static const Color _inkGreen = Color(0xFF0B3D2E);
  final ImagePicker _imagePicker = ImagePicker();
  final SfidBindingService _sfidBindingService = SfidBindingService();
  final UserProfileService _userProfileService = UserProfileService();

  bool _bindingSubmitting = false;
  SfidBindState _bindState =
      const SfidBindState(status: SfidBindStatus.unbound);
  UserProfileState _userProfile = const UserProfileState(
    nickname: UserProfileService.defaultNickname,
    nicknameCustomized: false,
  );

  bool get _isQrEnabled {
    return _userProfileService.isNicknameReady(_userProfile) &&
        _bindState.status == SfidBindStatus.bound &&
        _currentAccountPubkey.isNotEmpty;
  }

  String get _currentAccountPubkey {
    return _bindState.walletPubkeyHex?.trim() ?? '';
  }

  String get _userQrPayload {
    return UserQrPayload(
      nickname: _userProfile.nickname,
      accountPubkeyHex: _currentAccountPubkey,
    ).toRawJson();
  }

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

  Future<void> _pickBackgroundImage() async {
    await _pickImage(
      onSaved: (path) => _userProfileService.updateBackgroundPath(path),
      onApplied: (state) {
        _userProfile = state;
      },
      failurePrefix: '设置背景图失败',
    );
  }

  Future<void> _pickAvatarImage() async {
    await _pickImage(
      onSaved: (path) => _userProfileService.updateAvatarPath(path),
      onApplied: (state) {
        _userProfile = state;
      },
      failurePrefix: '设置头像失败',
    );
  }

  Future<void> _pickImage({
    required Future<UserProfileState> Function(String? path) onSaved,
    required void Function(UserProfileState state) onApplied,
    required String failurePrefix,
  }) async {
    try {
      final picked = await _imagePicker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 1600,
        maxHeight: 1600,
      );
      if (picked == null) {
        return;
      }
      final saved = await onSaved(picked.path);
      if (!mounted) {
        return;
      }
      setState(() {
        onApplied(saved);
      });
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('$failurePrefix：$e')),
      );
    }
  }

  Future<void> _editNickname() async {
    final controller = TextEditingController(text: _userProfile.nickname);
    final nickname = await showDialog<String>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('修改昵称'),
          content: TextField(
            controller: controller,
            autofocus: true,
            maxLength: 20,
            decoration: const InputDecoration(
              hintText: '请输入昵称',
              border: OutlineInputBorder(),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('取消'),
            ),
            FilledButton(
              onPressed: () {
                Navigator.of(context).pop(controller.text.trim());
              },
              child: const Text('保存'),
            ),
          ],
        );
      },
    );
    controller.dispose();

    if (!mounted || nickname == null) {
      return;
    }
    if (nickname.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('昵称不能为空')),
      );
      return;
    }

    final saved = await _userProfileService.updateNickname(nickname);
    if (!mounted) {
      return;
    }
    setState(() {
      _userProfile = saved;
    });
  }

  Future<void> _handleBindIdentity() async {
    if (_bindingSubmitting || _bindState.status == SfidBindStatus.pending) {
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
        const SnackBar(content: Text('已将所选公钥提交到 SFID 系统，等待绑定结果')),
      );
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('身份绑定提交失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() {
          _bindingSubmitting = false;
        });
      }
    }
  }

  Future<void> _openUserQr() async {
    if (!_isQrEnabled) {
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => UserQrPage(
          nickname: _userProfile.nickname,
          avatarPath: _userProfile.avatarPath,
          accountPubkeyHex: _currentAccountPubkey,
          qrPayload: _userQrPayload,
        ),
      ),
    );
  }

  Future<void> _openContacts() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => ContactBookPage(
          selfAccountPubkeyHex: _currentAccountPubkey,
        ),
      ),
    );
    await _loadState();
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

  Widget _buildExpandedTapArea({
    required Widget child,
    VoidCallback? onTap,
  }) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(8),
        child: SizedBox(
          height: 36,
          child: Align(
            alignment: Alignment.centerLeft,
            child: IgnorePointer(
              child: child,
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildBindAction() {
    if (_bindState.status == SfidBindStatus.pending) {
      return _buildExpandedTapArea(
        child: const SizedBox(
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
                Text('等待绑定', style: TextStyle(fontSize: 9)),
              ],
            ),
          ),
        ),
      );
    }
    if (_bindState.status == SfidBindStatus.bound) {
      return _buildExpandedTapArea(
        onTap: _handleBindIdentity,
        child: Container(
          width: double.infinity,
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
          decoration: BoxDecoration(
            color: const Color(0xFFE9F5EF),
            borderRadius: BorderRadius.circular(8),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: Text(
                  _currentAccountPubkey,
                  softWrap: true,
                  style: const TextStyle(
                    color: _inkGreen,
                    fontSize: 12,
                    height: 1.4,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
              const SizedBox(width: 6),
              const Padding(
                padding: EdgeInsets.only(top: 2),
                child: Icon(Icons.verified, size: 13, color: _inkGreen),
              ),
            ],
          ),
        ),
      );
    }
    return _buildExpandedTapArea(
      onTap: _bindingSubmitting ? null : _handleBindIdentity,
      child: SizedBox(
        height: 20,
        child: FilledButton(
          onPressed: () {},
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
      ),
    );
  }

  Widget _buildProfileCard() {
    final qrColor = _isQrEnabled ? Colors.white : Colors.white38;
    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          GestureDetector(
            onTap: _pickAvatarImage,
            child: _SquareAvatar(path: _userProfile.avatarPath, size: 84),
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
                        child: GestureDetector(
                          onTap: _editNickname,
                          child: Text(
                            _userProfile.nickname,
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 19,
                              fontWeight: FontWeight.w600,
                              shadows: [
                                Shadow(
                                  color: Color(0x80000000),
                                  blurRadius: 10,
                                  offset: Offset(0, 2),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.only(top: 1),
                      child: Transform.translate(
                        offset: const Offset(0, -10),
                        child: InkWell(
                          onTap: _isQrEnabled ? _openUserQr : null,
                          borderRadius: BorderRadius.circular(8),
                          child: Padding(
                            padding: const EdgeInsets.all(2),
                            child: Icon(
                              Icons.qr_code_2,
                              size: 21,
                              color: qrColor,
                              shadows: const [
                                Shadow(
                                  color: Color(0x80000000),
                                  blurRadius: 10,
                                  offset: Offset(0, 2),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 4),
                Row(
                  crossAxisAlignment: _bindState.status == SfidBindStatus.bound
                      ? CrossAxisAlignment.start
                      : CrossAxisAlignment.center,
                  children: [
                    Expanded(child: _buildBindAction()),
                    const SizedBox(width: 8),
                    InkWell(
                      onTap: _openProfileEdit,
                      borderRadius: BorderRadius.circular(8),
                      child: const SizedBox(
                        width: 40,
                        height: 40,
                        child: Icon(
                          Icons.chevron_right,
                          size: 24,
                          color: Colors.white,
                          shadows: [
                            Shadow(
                              color: Color(0x80000000),
                              blurRadius: 10,
                              offset: Offset(0, 2),
                            ),
                          ],
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
    );
  }

  Widget _buildEntryCard({
    required Widget leading,
    required String title,
    required VoidCallback onTap,
  }) {
    return Card(
      child: ListTile(
        leading: leading,
        title: Text(
          title,
          style: const TextStyle(fontWeight: FontWeight.w700),
        ),
        trailing: const Icon(Icons.chevron_right),
        onTap: onTap,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final topPadding = MediaQuery.of(context).padding.top;
    final headerHeight = topPadding + 260.0;
    return AnnotatedRegion<SystemUiOverlayStyle>(
      value: SystemUiOverlayStyle.light.copyWith(
        statusBarColor: Colors.transparent,
      ),
      child: Scaffold(
        body: ListView(
          padding: EdgeInsets.zero,
          children: [
            SizedBox(
              height: headerHeight,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  GestureDetector(
                    onTap: _pickBackgroundImage,
                    child: _HeaderBackground(
                      path: _userProfile.backgroundPath,
                      height: headerHeight,
                    ),
                  ),
                  Positioned(
                    top: topPadding + 10,
                    left: 0,
                    right: 0,
                    child: const Center(
                      child: Text(
                        '我的',
                        style: TextStyle(
                          color: Colors.white,
                          fontSize: 20,
                          fontWeight: FontWeight.w700,
                          shadows: [
                            Shadow(
                              color: Color(0x66000000),
                              blurRadius: 12,
                              offset: Offset(0, 2),
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                  Positioned(
                    left: 16,
                    right: 16,
                    bottom: 22,
                    child: _buildProfileCard(),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 12),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
                leading: SvgPicture.asset(
                  'assets/icons/contact-round.svg',
                  width: 22,
                  height: 22,
                  colorFilter:
                      const ColorFilter.mode(_inkGreen, BlendMode.srcIn),
                ),
                title: '通讯录',
                onTap: _openContacts,
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
                leading: SvgPicture.asset(
                  'assets/icons/wallet.svg',
                  width: 22,
                  height: 22,
                  colorFilter:
                      const ColorFilter.mode(_inkGreen, BlendMode.srcIn),
                ),
                title: '钱包',
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const MyWalletPage()),
                  );
                },
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: _buildEntryCard(
                leading: const Icon(
                  Icons.settings_outlined,
                  color: _inkGreen,
                  size: 22,
                ),
                title: '设置',
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const SettingsPage()),
                  );
                },
              ),
            ),
            const SizedBox(height: 24),
          ],
        ),
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
      widget.initialState.copyWith(
        nickname: nickname,
        nicknameCustomized: true,
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

class ContactBookPage extends StatefulWidget {
  const ContactBookPage({
    super.key,
    required this.selfAccountPubkeyHex,
  });

  final String selfAccountPubkeyHex;

  @override
  State<ContactBookPage> createState() => _ContactBookPageState();
}

class _ContactBookPageState extends State<ContactBookPage> {
  final UserContactService _userContactService = UserContactService();
  late Future<List<UserContact>> _contactsFuture;

  @override
  void initState() {
    super.initState();
    _contactsFuture = _userContactService.getContacts();
  }

  void _reload() {
    setState(() {
      _contactsFuture = _userContactService.getContacts();
    });
  }

  Future<void> _scanContactQr() async {
    final result = await Navigator.of(context).push<ContactImportResult>(
      MaterialPageRoute(
        builder: (_) => UserContactScannerPage(
          selfAccountPubkeyHex: widget.selfAccountPubkeyHex,
        ),
      ),
    );
    if (!mounted || result == null) {
      return;
    }
    _reload();
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          result.created
              ? '已加入通讯录：${result.contact.displayNickname}'
              : '已更新通讯录：${result.contact.displayNickname}',
        ),
      ),
    );
  }

  Future<void> _renameContact(UserContact contact) async {
    final controller = TextEditingController(text: contact.localNickname ?? '');
    final nextName = await showDialog<String>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('修改通讯录昵称'),
          content: TextField(
            controller: controller,
            autofocus: true,
            maxLength: 20,
            decoration: InputDecoration(
              hintText: contact.sourceNickname,
              helperText: '留空则恢复显示对方原始昵称',
              border: const OutlineInputBorder(),
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('取消'),
            ),
            FilledButton(
              onPressed: () =>
                  Navigator.of(context).pop(controller.text.trim()),
              child: const Text('保存'),
            ),
          ],
        );
      },
    );
    controller.dispose();

    if (!mounted || nextName == null) {
      return;
    }

    try {
      await _userContactService.renameContact(
        contact.accountPubkeyHex,
        nextName,
      );
      if (!mounted) {
        return;
      }
      _reload();
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('通讯录昵称已更新')),
      );
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('更新失败：$e')),
      );
    }
  }

  Widget _buildEmptyState() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.perm_contact_calendar_outlined,
              size: 72,
              color: Colors.grey,
            ),
            const SizedBox(height: 18),
            const Text(
              '通讯录还是空的',
              style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 10),
            Text(
              '扫描其他用户的二维码后，会把对方的昵称和公钥加入通讯录。',
              textAlign: TextAlign.center,
              style: TextStyle(color: Colors.grey.shade700, height: 1.5),
            ),
            const SizedBox(height: 18),
            FilledButton.icon(
              onPressed: _scanContactQr,
              icon: const Icon(Icons.qr_code_scanner),
              label: const Text('扫码添加'),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('我的通讯录'),
        centerTitle: true,
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _scanContactQr,
        icon: const Icon(Icons.qr_code_scanner),
        label: const Text('扫码添加'),
      ),
      body: FutureBuilder<List<UserContact>>(
        future: _contactsFuture,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return const Center(child: CircularProgressIndicator());
          }
          final contacts = snapshot.data ?? const <UserContact>[];
          if (contacts.isEmpty) {
            return _buildEmptyState();
          }

          return ListView.separated(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 96),
            itemCount: contacts.length,
            separatorBuilder: (_, __) => const SizedBox(height: 10),
            itemBuilder: (context, index) {
              final contact = contacts[index];
              final hasLocalNickname =
                  (contact.localNickname?.trim().isNotEmpty ?? false);
              return Card(
                child: ListTile(
                  leading: CircleAvatar(
                    backgroundColor: const Color(0xFFE3EFE8),
                    child: Text(
                      contact.displayNickname.characters.first,
                      style: const TextStyle(
                        color: _ProfilePageState._inkGreen,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                  ),
                  title: Text(
                    contact.displayNickname,
                    style: const TextStyle(fontWeight: FontWeight.w700),
                  ),
                  subtitle: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      const SizedBox(height: 6),
                      Text(
                        contact.accountPubkeyHex,
                        style: const TextStyle(height: 1.4),
                      ),
                      if (hasLocalNickname)
                        Padding(
                          padding: const EdgeInsets.only(top: 6),
                          child: Text(
                            '原始昵称：${contact.sourceNickname}',
                            style: TextStyle(
                              color: Colors.grey.shade700,
                              height: 1.4,
                            ),
                          ),
                        ),
                    ],
                  ),
                  trailing: const Icon(Icons.edit_outlined),
                  onTap: () => _renameContact(contact),
                ),
              );
            },
          );
        },
      ),
    );
  }
}

class UserContactScannerPage extends StatefulWidget {
  const UserContactScannerPage({
    super.key,
    required this.selfAccountPubkeyHex,
  });

  final String selfAccountPubkeyHex;

  @override
  State<UserContactScannerPage> createState() => _UserContactScannerPageState();
}

class _UserContactScannerPageState extends State<UserContactScannerPage> {
  final MobileScannerController _controller = MobileScannerController();
  final UserContactService _userContactService = UserContactService();
  bool _handling = false;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _handleRawValue(String raw) async {
    if (_handling) {
      return;
    }
    _handling = true;
    await _controller.stop();

    var shouldRestart = true;
    try {
      final result = await _userContactService.addFromQrPayload(
        raw,
        selfAccountPubkeyHex: widget.selfAccountPubkeyHex,
      );
      shouldRestart = false;
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(result);
    } catch (e) {
      if (!mounted) {
        return;
      }
      await showDialog<void>(
        context: context,
        builder: (context) {
          return AlertDialog(
            title: const Text('无法识别通讯录二维码'),
            content: Text('$e'),
            actions: [
              TextButton(
                onPressed: () => Navigator.of(context).pop(),
                child: const Text('继续扫描'),
              ),
            ],
          );
        },
      );
    } finally {
      _handling = false;
      if (shouldRestart && mounted) {
        await _controller.start();
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫描用户二维码'),
        centerTitle: true,
      ),
      body: Stack(
        fit: StackFit.expand,
        children: [
          MobileScanner(
            controller: _controller,
            onDetect: (capture) async {
              final raw = capture.barcodes.first.rawValue;
              if (raw == null || raw.isEmpty) {
                return;
              }
              await _handleRawValue(raw);
            },
          ),
          Align(
            alignment: Alignment.topCenter,
            child: Container(
              margin: const EdgeInsets.all(16),
              padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
              decoration: BoxDecoration(
                color: Colors.black54,
                borderRadius: BorderRadius.circular(12),
              ),
              child: const Text(
                '扫描用户二维码后加入通讯录',
                style: TextStyle(color: Colors.white),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class UserQrPage extends StatelessWidget {
  const UserQrPage({
    super.key,
    required this.nickname,
    required this.avatarPath,
    required this.accountPubkeyHex,
    required this.qrPayload,
  });

  final String nickname;
  final String? avatarPath;
  final String accountPubkeyHex;
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
                _SquareAvatar(path: avatarPath, size: 54),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        nickname,
                        style: const TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                      const SizedBox(height: 6),
                      Text(
                        accountPubkeyHex,
                        style: TextStyle(
                          color: Colors.grey.shade700,
                          height: 1.4,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 24),
            Expanded(
              child: Center(
                child: Container(
                  padding: const EdgeInsets.all(20),
                  decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(24),
                    border: Border.all(color: const Color(0xFFE5ECE8)),
                    boxShadow: const [
                      BoxShadow(
                        color: Color(0x12000000),
                        blurRadius: 24,
                        offset: Offset(0, 10),
                      ),
                    ],
                  ),
                  child: QrImageView(
                    data: qrPayload,
                    version: QrVersions.auto,
                    size: 280,
                    backgroundColor: Colors.white,
                  ),
                ),
              ),
            ),
            Text(
              '其他用户扫码后可直接加入通讯录',
              style: TextStyle(color: Colors.grey.shade700),
            ),
            const SizedBox(height: 12),
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

class _HeaderBackground extends StatelessWidget {
  const _HeaderBackground({
    required this.path,
    required this.height,
  });

  final String? path;
  final double height;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final file = hasImage ? File(path!) : null;
    final validImage = file != null && file.existsSync();

    return Container(
      width: double.infinity,
      height: height,
      decoration: BoxDecoration(
        gradient: validImage
            ? null
            : const LinearGradient(
                colors: [
                  Color(0xFF0B3D2E),
                  Color(0xFF1E7A65),
                  Color(0xFFD8EFE6)
                ],
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
              ),
        image: validImage
            ? DecorationImage(
                image: FileImage(file),
                fit: BoxFit.cover,
              )
            : null,
      ),
      child: DecoratedBox(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            colors: [
              Colors.black.withValues(alpha: 0.08),
              Colors.black.withValues(alpha: 0.18),
            ],
            begin: Alignment.topCenter,
            end: Alignment.bottomCenter,
          ),
        ),
      ),
    );
  }
}

class _SquareAvatar extends StatelessWidget {
  const _SquareAvatar({
    required this.path,
    required this.size,
  });

  final String? path;
  final double size;

  @override
  Widget build(BuildContext context) {
    final hasImage = path != null && path!.trim().isNotEmpty;
    final file = hasImage ? File(path!) : null;
    final validImage = file != null && file.existsSync();

    return Container(
      width: size,
      height: size,
      decoration: BoxDecoration(
        color: const Color(0xFFE3EFE8),
        borderRadius: BorderRadius.circular(10),
      ),
      child: ClipRRect(
        borderRadius: BorderRadius.circular(10),
        child: validImage
            ? Image.file(file, fit: BoxFit.cover)
            : const Icon(
                Icons.person,
                size: 40,
                color: _ProfilePageState._inkGreen,
              ),
      ),
    );
  }
}
