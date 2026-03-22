import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:image_picker/image_picker.dart';
import 'package:local_auth/local_auth.dart';
import 'package:qr/qr.dart';
import 'package:qr_flutter/qr_flutter.dart';
import 'package:saver_gallery/saver_gallery.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:wuminapp_mobile/security/app_lock_service.dart';
import 'package:wuminapp_mobile/security/pin_input_page.dart';
import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart';
import 'package:wuminapp_mobile/trade/onchain/onchain_trade_page.dart';
import 'package:wuminapp_mobile/qr/transfer/transfer_qr_models.dart';
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

  UserProfileState _userProfile = const UserProfileState();

  String get _communicationAddress {
    return _userProfile.communicationAddress?.trim() ?? '';
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

  Future<void> _openMyQrPage() async {
    final address = _communicationAddress;
    if (address.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先在用户资料中设置通信账户')),
      );
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => _MyQrCodePage(
          nickname: _userProfile.nickname,
          address: address,
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
    return Padding(
      padding: const EdgeInsets.fromLTRB(14, 14, 14, 14),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _SquareAvatar(path: _userProfile.avatarPath, size: 84),
          const SizedBox(width: 12),
          Expanded(
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
          SizedBox(
            height: 84,
            child: Center(
              child: InkWell(
                onTap: _openProfileEdit,
                borderRadius: BorderRadius.circular(8),
                child: const Padding(
                  padding: EdgeInsets.all(4),
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
                    top: topPadding + 14,
                    right: 8,
                    child: InkWell(
                      onTap: _openMyQrPage,
                      borderRadius: BorderRadius.circular(12),
                      child: const Padding(
                        padding: EdgeInsets.all(8),
                        child: Icon(
                          Icons.qr_code_2,
                          size: 22,
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

  final GlobalKey _qrKey = GlobalKey();
  late UserProfileState _profile;
  SfidBindState _voteBindState =
      const SfidBindState(status: SfidBindStatus.unbound);
  bool _voteSubmitting = false;
  bool _isSavingQr = false;

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
    return TransferQrPayload(
      to: _profile.communicationAddress?.trim() ?? '',
      name: _profile.nickname,
    ).toRawJson();
  }

  // ---- 保存二维码 ----

  Future<void> _saveQrToGallery() async {
    if (_isSavingQr) return;
    setState(() => _isSavingQr = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) return;
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData =
          await image.toByteData(format: ui.ImageByteFormat.png);
      if (byteData == null || !mounted) return;
      final result = await SaverGallery.saveImage(
        byteData.buffer.asUint8List(),
        fileName: 'qr_${DateTime.now().millisecondsSinceEpoch}.png',
        androidRelativePath: 'Pictures/WuminApp',
        skipIfExists: false,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(result.isSuccess ? '已保存到相册' : '保存失败'),
        ),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _isSavingQr = false);
    }
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

  // ---- 昵称（= 通信钱包名称，双向同步） ----

  Future<void> _editNickname() async {
    if (_profile.communicationWalletIndex == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先设置通信账户')),
      );
      return;
    }
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
    // 双向同步：同时改钱包名称 + 用户资料中的通信钱包名称
    final walletManager = WalletManager();
    await walletManager.renameWallet(
        _profile.communicationWalletIndex!, nickname);
    final saved =
        await _profileService.updateCommunicationWalletName(nickname);
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
    final saved = await _profileService.setCommunicationWallet(
      walletIndex: wallet.walletIndex,
      address: wallet.address,
      walletName: wallet.walletName,
    );
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
                  ? Stack(
                      alignment: Alignment.center,
                      children: [
                        RepaintBoundary(
                          key: _qrKey,
                          child: Container(
                            color: Colors.white,
                            padding: const EdgeInsets.all(8),
                            child: CustomPaint(
                              size: const Size(180, 180),
                              painter: _HollowQrPainter(
                                data: _qrPayload,
                                hollowSize: 40,
                              ),
                            ),
                          ),
                        ),
                        Container(
                          width: 30,
                          height: 30,
                          decoration: BoxDecoration(
                            color: Colors.white,
                            borderRadius: BorderRadius.circular(4),
                            border: Border.all(
                              color: Colors.grey.shade300,
                              width: 1,
                            ),
                          ),
                          child: IconButton(
                            constraints: const BoxConstraints(),
                            padding: EdgeInsets.zero,
                            onPressed:
                                _isSavingQr ? null : _saveQrToGallery,
                            icon: _isSavingQr
                                ? const SizedBox(
                                    width: 14,
                                    height: 14,
                                    child: CircularProgressIndicator(
                                        strokeWidth: 2),
                                  )
                                : Icon(Icons.download,
                                    size: 18,
                                    color: Colors.grey.shade600),
                          ),
                        ),
                      ],
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
    this.selectForTrade = false,
  });

  final String selfAccountPubkeyHex;
  /// 为 true 时，点击联系人直接返回该联系人（而非弹窗修改昵称）。
  final bool selectForTrade;

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

  Future<void> _openContactDetail(UserContact contact) async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute(
        builder: (_) => _ContactDetailPage(contact: contact),
      ),
    );
    if (!mounted) return;
    _reload();
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

          // 按昵称字母排序
          final sorted = List<UserContact>.from(contacts)
            ..sort((a, b) => a.displayNickname
                .toLowerCase()
                .compareTo(b.displayNickname.toLowerCase()));

          return ListView.separated(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
            itemCount: sorted.length,
            separatorBuilder: (_, __) => const Divider(height: 1),
            itemBuilder: (context, index) {
              final contact = sorted[index];
              return ListTile(
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
                subtitle: Text(
                  contact.accountPubkeyHex,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.grey.shade600,
                  ),
                ),
                trailing: const Icon(Icons.chevron_right, size: 20),
                onTap: widget.selectForTrade
                    ? () => Navigator.of(context).pop(contact)
                    : () => _openContactDetail(contact),
              );
            },
          );
        },
      ),
    );
  }
}

class _MyQrCodePage extends StatefulWidget {
  const _MyQrCodePage({required this.nickname, required this.address});

  final String nickname;
  final String address;

  @override
  State<_MyQrCodePage> createState() => _MyQrCodePageState();
}

class _MyQrCodePageState extends State<_MyQrCodePage> {
  final GlobalKey _qrKey = GlobalKey();
  bool _saving = false;

  String get _qrData => TransferQrPayload(
        to: widget.address,
        name: widget.nickname,
      ).toRawJson();

  Future<void> _saveQr() async {
    if (_saving) return;
    setState(() => _saving = true);
    try {
      final boundary =
          _qrKey.currentContext?.findRenderObject() as RenderRepaintBoundary?;
      if (boundary == null) return;
      final image = await boundary.toImage(pixelRatio: 3.0);
      final byteData =
          await image.toByteData(format: ui.ImageByteFormat.png);
      if (byteData == null || !mounted) return;
      final result = await SaverGallery.saveImage(
        byteData.buffer.asUint8List(),
        fileName: 'my_qr_${DateTime.now().millisecondsSinceEpoch}.png',
        androidRelativePath: 'Pictures/WuminApp',
        skipIfExists: false,
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(result.isSuccess ? '已保存到相册' : '保存失败')),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('保存失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('我的二维码'),
        centerTitle: true,
      ),
      body: Column(
        children: [
          const Spacer(),
          Text(
            widget.nickname,
            style: const TextStyle(
              fontSize: 20,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 24),
          Stack(
            alignment: Alignment.center,
            children: [
              RepaintBoundary(
                key: _qrKey,
                child: Container(
                  color: Colors.white,
                  padding: const EdgeInsets.all(12),
                  child: CustomPaint(
                    size: const Size(240, 240),
                    painter: _HollowQrPainter(
                      data: _qrData,
                      hollowSize: 48,
                    ),
                  ),
                ),
              ),
              Container(
                width: 36,
                height: 36,
                decoration: BoxDecoration(
                  color: Colors.white,
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(
                    color: Colors.grey.shade300,
                    width: 1,
                  ),
                ),
                child: IconButton(
                  constraints: const BoxConstraints(),
                  padding: EdgeInsets.zero,
                  onPressed: _saving ? null : _saveQr,
                  icon: _saving
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child:
                              CircularProgressIndicator(strokeWidth: 2),
                        )
                      : Icon(Icons.download,
                          size: 20, color: Colors.grey.shade600),
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 32),
            child: Text(
              widget.address,
              textAlign: TextAlign.center,
              style: TextStyle(
                fontSize: 13,
                color: Colors.grey.shade600,
                height: 1.5,
              ),
            ),
          ),
          const Spacer(),
          Padding(
            padding: const EdgeInsets.only(bottom: 32),
            child: Text(
              '其他用户扫描此二维码可添加通讯录',
              style: TextStyle(color: Colors.grey.shade500, fontSize: 12),
            ),
          ),
        ],
      ),
    );
  }
}

class _ContactDetailPage extends StatelessWidget {
  const _ContactDetailPage({required this.contact});

  final UserContact contact;

  @override
  Widget build(BuildContext context) {
    final qrData = TransferQrPayload(
      to: contact.accountPubkeyHex,
      name: contact.displayNickname,
    ).toRawJson();

    return Scaffold(
      appBar: AppBar(
        title: const Text('联系人详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(20),
        children: [
          // 头像 + 昵称
          Center(
            child: Column(
              children: [
                CircleAvatar(
                  radius: 36,
                  backgroundColor: const Color(0xFFE3EFE8),
                  child: Text(
                    contact.displayNickname.characters.first,
                    style: const TextStyle(
                      fontSize: 28,
                      color: _inkGreen,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
                const SizedBox(height: 12),
                Text(
                  contact.displayNickname,
                  style: const TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 24),
          // 二维码
          Center(
            child: Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: Colors.white,
                borderRadius: BorderRadius.circular(16),
                border: Border.all(color: const Color(0xFFE5ECE8)),
              ),
              child: QrImageView(
                data: qrData,
                version: QrVersions.auto,
                size: 240,
                backgroundColor: Colors.white,
              ),
            ),
          ),
          const SizedBox(height: 16),
          // 地址 + 复制图标
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Flexible(
                  child: Text(
                    contact.accountPubkeyHex,
                    textAlign: TextAlign.center,
                    style: TextStyle(
                      fontSize: 13,
                      color: Colors.grey.shade600,
                      height: 1.5,
                    ),
                  ),
                ),
                const SizedBox(width: 4),
                IconButton(
                  constraints: const BoxConstraints(),
                  padding: EdgeInsets.zero,
                  iconSize: 18,
                  onPressed: () {
                    Clipboard.setData(
                        ClipboardData(text: contact.accountPubkeyHex));
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('地址已复制')),
                    );
                  },
                  icon: SvgPicture.asset(
                    'assets/icons/copy.svg',
                    width: 16,
                    height: 16,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 28),
          // 消息 + 转账按钮
          Row(
            children: [
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: () {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(content: Text('消息功能开发中')),
                    );
                  },
                  icon: const Icon(Icons.chat_bubble_outline, size: 18),
                  label: const Text('消息'),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: FilledButton.icon(
                  style: FilledButton.styleFrom(
                    backgroundColor: _inkGreen,
                  ),
                  onPressed: () {
                    Navigator.of(context).push(
                      MaterialPageRoute(
                        builder: (_) => OnchainTradePage(
                          initialToAddress: contact.accountPubkeyHex,
                        ),
                      ),
                    );
                  },
                  icon: const Icon(Icons.send, size: 18),
                  label: const Text('转账'),
                ),
              ),
            ],
          ),
        ],
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
  static const String _deviceLockKey = 'device_lock_enabled';
  static const FlutterSecureStorage _secure = FlutterSecureStorage();
  final LocalAuthentication _localAuth = LocalAuthentication();
  bool _deviceLockEnabled = false;
  bool _pinLockEnabled = false;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadSettings();
  }

  Future<void> _loadSettings() async {
    final deviceLockStr = await _secure.read(key: _deviceLockKey);
    final pinSet = await AppLockService.isPinSet();
    if (!mounted) return;
    setState(() {
      _deviceLockEnabled = deviceLockStr == 'true';
      _pinLockEnabled = pinSet;
      _loading = false;
    });
  }

  Future<void> _toggleDeviceLock(bool value) async {
    if (value) {
      final canCheck = await _localAuth.canCheckBiometrics;
      final isDeviceSupported = await _localAuth.isDeviceSupported();
      if (!canCheck && !isDeviceSupported) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('您的设备不支持生物识别或设备密码，无法开启设备锁')),
        );
        return;
      }

      try {
        final authenticated = await _localAuth.authenticate(
          localizedReason: '验证身份以开启设备锁',
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

    await _secure.write(
      key: _deviceLockKey,
      value: value.toString(),
    );
    if (!mounted) return;
    setState(() => _deviceLockEnabled = value);
  }

  Future<void> _togglePinLock(bool value) async {
    if (value) {
      // 开启：进入设置 PIN 页面
      final result = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => const PinInputPage(mode: PinInputMode.setup),
        ),
      );
      if (result == true && mounted) {
        setState(() => _pinLockEnabled = true);
      }
    } else {
      // 关闭：进入验证 PIN 页面（验证通过后删除）
      final result = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => const PinInputPage(mode: PinInputMode.remove),
        ),
      );
      if (result == true && mounted) {
        setState(() => _pinLockEnabled = false);
      }
    }
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
                  title: const Text('设备锁'),
                  subtitle: Text(
                    _pinLockEnabled
                        ? '请先关闭应用锁'
                        : '启动应用时需要生物识别或设备密码',
                  ),
                  value: _deviceLockEnabled,
                  onChanged: _pinLockEnabled ? null : _toggleDeviceLock,
                  activeThumbColor: Colors.white,
                  activeTrackColor: const Color(0xFF007A74),
                  secondary: const Icon(Icons.fingerprint),
                ),
                const Divider(height: 1),
                SwitchListTile(
                  title: const Text('应用锁'),
                  subtitle: Text(
                    _deviceLockEnabled
                        ? '请先关闭设备锁'
                        : '启动应用时需要输入 6 位数字密码',
                  ),
                  value: _pinLockEnabled,
                  onChanged: _deviceLockEnabled ? null : _togglePinLock,
                  activeThumbColor: Colors.white,
                  activeTrackColor: const Color(0xFF007A74),
                  secondary: const Icon(Icons.pin_outlined),
                ),
              ],
            ),
    );
  }
}

/// 自绘二维码，中央 [hollowSize] 像素区域不绘制任何模块（真正留白）。
class _HollowQrPainter extends CustomPainter {
  _HollowQrPainter({required this.data, required this.hollowSize});

  final String data;
  final double hollowSize;

  @override
  void paint(Canvas canvas, Size size) {
    final qrCode = QrCode.fromData(
      data: data,
      errorCorrectLevel: QrErrorCorrectLevel.H,
    );
    final qrImage = QrImage(qrCode);
    final moduleCount = qrImage.moduleCount;
    final moduleSize = size.width / moduleCount;
    final paint = Paint()..color = const Color(0xFF000000);

    final hollowModules = (hollowSize / moduleSize).ceil();
    final hollowStart = (moduleCount - hollowModules) ~/ 2;
    final hollowEnd = hollowStart + hollowModules;

    for (var row = 0; row < moduleCount; row++) {
      for (var col = 0; col < moduleCount; col++) {
        if (qrImage.isDark(row, col)) {
          if (row >= hollowStart &&
              row < hollowEnd &&
              col >= hollowStart &&
              col < hollowEnd) {
            continue;
          }
          canvas.drawRect(
            Rect.fromLTWH(
              col * moduleSize,
              row * moduleSize,
              moduleSize,
              moduleSize,
            ),
            paint,
          );
        }
      }
    }
  }

  @override
  bool shouldRepaint(_HollowQrPainter oldDelegate) {
    return oldDelegate.data != data || oldDelegate.hollowSize != hollowSize;
  }
}
