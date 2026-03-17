import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:image_picker/image_picker.dart';
import 'package:local_auth/local_auth.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/user/user_service.dart';
import 'package:wuminapp_mobile/wallet/capabilities/sfid_binding_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/wallet_page.dart';

class ProfilePage extends StatefulWidget {
  const ProfilePage({super.key});

  @override
  State<ProfilePage> createState() => _ProfilePageState();
}

const Color _inkGreen = Color(0xFF0B3D2E);

class _ProfilePageState extends State<ProfilePage> {
  final ImagePicker _imagePicker = ImagePicker();
  final UserProfileService _userProfileService = UserProfileService();

  UserProfileState _userProfile = const UserProfileState(
    nickname: UserProfileService.defaultNickname,
    nicknameCustomized: false,
  );

  bool get _isQrEnabled {
    return _communicationAddress.isNotEmpty;
  }

  String get _communicationAddress {
    return _userProfile.communicationAddress?.trim() ?? '';
  }

  String get _userQrPayload {
    return UserQrPayload(
      nickname: _userProfile.nickname,
      address: _communicationAddress,
    ).toRawJson();
  }

  @override
  void initState() {
    super.initState();
    _loadState();
  }

  Future<void> _loadState() async {
    final profile = await _userProfileService.getState();
    if (!mounted) {
      return;
    }
    setState(() {
      _userProfile = profile;
    });
  }

  Future<void> _pickBackgroundImage() async {
    try {
      final picked = await _imagePicker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 1600,
        maxHeight: 1600,
      );
      if (picked == null) return;
      final saved =
          await _userProfileService.updateBackgroundPath(picked.path);
      if (!mounted) return;
      setState(() {
        _userProfile = saved;
      });
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('设置背景图失败：$e')),
      );
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
          accountPubkeyHex: _communicationAddress,
          qrPayload: _userQrPayload,
        ),
      ),
    );
  }

  Future<void> _openContacts() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => ContactBookPage(
          selfAccountPubkeyHex: _communicationAddress,
        ),
      ),
    );
    await _loadState();
  }

  Future<void> _openProfileEdit() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => ProfileEditPage(initialState: _userProfile),
      ),
    );
    if (!mounted) return;
    await _loadState();
  }

  Widget _buildProfileCard() {
    final qrColor = _isQrEnabled ? Colors.white : Colors.white38;
    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
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
                Align(
                  alignment: Alignment.centerRight,
                  child: InkWell(
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
                  colorFilter: const ColorFilter.mode(
                      Color(0xFF008080), BlendMode.srcIn),
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
                  colorFilter: const ColorFilter.mode(
                      Color(0xFFDE3163), BlendMode.srcIn),
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
                  color: Color(0xFF2F4F4F),
                  size: 22,
                ),
                title: '设置',
                onTap: () {
                  Navigator.of(context).push(
                    MaterialPageRoute(builder: (_) => const _SettingsPage()),
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
  final UserProfileService _profileService = UserProfileService();
  final SfidBindingService _sfidBindingService = SfidBindingService();

  late UserProfileState _profile;
  SfidBindState _voteBindState =
      const SfidBindState(status: SfidBindStatus.unbound);
  bool _voteSubmitting = false;

  @override
  void initState() {
    super.initState();
    _profile = widget.initialState;
    _loadVoteState();
  }

  Future<void> _loadVoteState() async {
    final state = await _sfidBindingService.getState();
    if (!mounted) return;
    setState(() {
      _voteBindState = state;
    });
  }

  // ---- 二维码数据 ----

  bool get _isQrReady {
    return (_profile.communicationAddress?.trim().isNotEmpty ?? false);
  }

  String get _qrPayload {
    return UserQrPayload(
      nickname: _profile.nickname,
      address: _profile.communicationAddress?.trim() ?? '',
    ).toRawJson();
  }

  // ---- 头像 ----

  Future<void> _pickAvatar() async {
    try {
      final picked = await _imagePicker.pickImage(
        source: ImageSource.gallery,
        maxWidth: 1024,
        maxHeight: 1024,
      );
      if (picked == null || !mounted) return;
      final saved = await _profileService.updateAvatarPath(picked.path);
      if (!mounted) return;
      setState(() {
        _profile = saved;
      });
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('访问相册失败：$e')),
      );
    }
  }

  // ---- 昵称 ----

  Future<void> _editNickname() async {
    final controller = TextEditingController(text: _profile.nickname);
    final nickname = await showDialog<String>(
      context: context,
      builder: (dialogContext) {
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
              onPressed: () => Navigator.of(dialogContext).pop(),
              child: const Text('取消'),
            ),
            FilledButton(
              onPressed: () =>
                  Navigator.of(dialogContext).pop(controller.text.trim()),
              child: const Text('保存'),
            ),
          ],
        );
      },
    );
    await WidgetsBinding.instance.endOfFrame;
    controller.dispose();
    if (!mounted || nickname == null || nickname.trim().isEmpty) return;
    final saved = await _profileService.updateNickname(nickname);
    if (!mounted) return;
    setState(() {
      _profile = saved;
    });
  }

  // ---- 通信账户 ----

  Future<void> _selectCommunicationWallet() async {
    final wallet = await Navigator.of(context).push<WalletProfile>(
      MaterialPageRoute(
        builder: (_) => const MyWalletPage(selectForBind: true),
      ),
    );
    if (!mounted || wallet == null) return;
    final saved =
        await _profileService.updateCommunicationAddress(wallet.address);
    if (!mounted) return;
    setState(() {
      _profile = saved;
    });
  }

  // ---- 投票账户 ----

  Future<void> _selectVoteWallet() async {
    if (_voteSubmitting) return;
    final wallet = await Navigator.of(context).push<WalletProfile>(
      MaterialPageRoute(
        builder: (_) => const MyWalletPage(selectForBind: true),
      ),
    );
    if (!mounted || wallet == null) return;
    setState(() {
      _voteSubmitting = true;
    });
    try {
      final state = await _sfidBindingService.submitBinding(
        wallet.address,
        wallet.pubkeyHex,
      );
      if (!mounted) return;
      setState(() {
        _voteBindState = state;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('已提交到 SFID 系统，等待绑定结果')),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('投票账户绑定失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() {
          _voteSubmitting = false;
        });
      }
    }
  }

  String _voteStatusLabel() {
    return switch (_voteBindState.status) {
      SfidBindStatus.unbound => '未设置',
      SfidBindStatus.pending => '绑定中',
      SfidBindStatus.bound => '已绑定',
    };
  }

  Color _voteStatusColor() {
    return switch (_voteBindState.status) {
      SfidBindStatus.unbound => Colors.grey,
      SfidBindStatus.pending => Colors.orange,
      SfidBindStatus.bound => _inkGreen,
    };
  }

  // ---- 通用行构建 ----

  Widget _buildSettingRow({
    required String label,
    String? value,
    Widget? trailing,
    VoidCallback? onTap,
  }) {
    return InkWell(
      onTap: onTap,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 18),
        child: Row(
          children: [
            Text(
              label,
              style: const TextStyle(fontSize: 16),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                value ?? '',
                textAlign: TextAlign.right,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  fontSize: 14,
                  color: Colors.grey.shade600,
                ),
              ),
            ),
            if (trailing != null) ...[
              const SizedBox(width: 4),
              trailing,
            ],
            const SizedBox(width: 4),
            Icon(Icons.chevron_right, size: 20, color: Colors.grey.shade400),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('用户资料'),
        centerTitle: true,
      ),
      body: ListView(
        children: [
          // ---- 用户二维码 ----
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 20),
            child: Center(
              child: _isQrReady
                  ? QrImageView(
                      data: _qrPayload,
                      version: QrVersions.auto,
                      size: 180,
                    )
                  : Container(
                      width: 180,
                      height: 180,
                      decoration: BoxDecoration(
                        color: Colors.grey.shade200,
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: const Center(
                        child: Text(
                          '请设置通信账户后\n生成二维码',
                          textAlign: TextAlign.center,
                          style: TextStyle(color: Colors.grey),
                        ),
                      ),
                    ),
            ),
          ),
          const Divider(height: 1),
          // ---- 用户头像 ----
          InkWell(
            onTap: _pickAvatar,
            child: Padding(
              padding:
                  const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
              child: Row(
                children: [
                  _SquareAvatar(path: _profile.avatarPath, size: 44),
                  const Spacer(),
                  Icon(Icons.chevron_right,
                      size: 20, color: Colors.grey.shade400),
                ],
              ),
            ),
          ),
          const Divider(height: 1),
          // ---- 用户昵称 ----
          _buildSettingRow(
            label: _profile.nickname,
            onTap: _editNickname,
          ),
          const Divider(height: 1),
          // ---- 通信账户 ----
          _buildSettingRow(
            label: '通信账户',
            value: _profile.communicationAddress ?? '未设置',
            onTap: _selectCommunicationWallet,
          ),
          const Divider(height: 1),
          // ---- 投票账户 ----
          _buildSettingRow(
            label: '投票账户',
            value: _voteBindState.walletAddress ?? '未设置',
            trailing: _voteBindState.status != SfidBindStatus.unbound
                ? Container(
                    padding: const EdgeInsets.symmetric(
                        horizontal: 6, vertical: 2),
                    decoration: BoxDecoration(
                      color: _voteStatusColor().withAlpha(25),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      _voteStatusLabel(),
                      style: TextStyle(
                        fontSize: 11,
                        color: _voteStatusColor(),
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  )
                : null,
            onTap: _voteSubmitting ? null : _selectVoteWallet,
          ),
          const Divider(height: 1),
        ],
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
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => QrScanPage(
          mode: QrScanMode.contact,
          selfAccountPubkeyHex: widget.selfAccountPubkeyHex,
        ),
      ),
    );
    if (!mounted) return;
    _reload();
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
        actions: [
          IconButton(
            onPressed: _scanContactQr,
            icon: SvgPicture.asset(
              'assets/icons/scan-line.svg',
              width: 20,
              height: 20,
            ),
          ),
        ],
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
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 24),
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
                        color: _inkGreen,
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
                color: _inkGreen,
              ),
      ),
    );
  }
}

class _SettingsPage extends StatefulWidget {
  const _SettingsPage();

  @override
  State<_SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<_SettingsPage> {
  static const String _appLockKey = 'app_lock_enabled';
  final LocalAuthentication _localAuth = LocalAuthentication();
  bool _appLockEnabled = false;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadSettings();
  }

  Future<void> _loadSettings() async {
    final prefs = await SharedPreferences.getInstance();
    if (!mounted) return;
    setState(() {
      _appLockEnabled = prefs.getBool(_appLockKey) ?? false;
      _loading = false;
    });
  }

  Future<void> _toggleAppLock(bool value) async {
    if (value) {
      // 开启前先检查设备是否支持生物识别或设备密码
      final canCheck = await _localAuth.canCheckBiometrics;
      final isDeviceSupported = await _localAuth.isDeviceSupported();
      if (!canCheck && !isDeviceSupported) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('您的设备不支持生物识别或设备密码，无法开启应用锁')),
        );
        return;
      }

      // 验证一次身份，确认用户可以通过认证
      try {
        final authenticated = await _localAuth.authenticate(
          localizedReason: '验证身份以开启应用锁',
          options: const AuthenticationOptions(
            stickyAuth: true,
            biometricOnly: false,
          ),
        );
        if (!authenticated) return;
      } catch (e) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('身份验证失败：$e')),
        );
        return;
      }
    }

    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_appLockKey, value);
    if (!mounted) return;
    setState(() {
      _appLockEnabled = value;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('设置'),
        centerTitle: true,
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              children: [
                SwitchListTile(
                  title: const Text('应用锁'),
                  subtitle: const Text('启动应用时需要生物识别或设备密码'),
                  value: _appLockEnabled,
                  onChanged: _toggleAppLock,
                  activeThumbColor: Colors.white,
                  activeTrackColor: const Color(0xFF007A74),
                  secondary: const Icon(Icons.lock_outline),
                ),
              ],
            ),
    );
  }
}
