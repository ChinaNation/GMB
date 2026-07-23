import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_cache.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/widgets/profile_avatar.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/chat/open_direct_chat.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/qr/pages/qr_scan_page.dart';
import 'package:citizenapp/transaction/onchain-transaction/onchain_payment_page.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 通讯录页使用模式。
enum ContactPickMode {
  /// 常规浏览:点联系人进主页,卡片带「转账/私信/改名/删除」操作菜单。
  browse,

  /// 选收款人(交易页发起):点联系人即返回该联系人,由调用方预填收款地址。
  pickForTransfer,

  /// 选私信对象(聊天页「发私信」):点联系人直接打开一对一聊天,不带操作菜单与扫码入口。
  pickForMessage,
}

/// “我的通讯录”唯一页面。联系人关系本地优先、后台密文同步；公开头像、昵称、
/// 签名与身份徽章复用统一用户资料，点击后进入现有 [UserProfilePage]。
class ContactBookPage extends StatefulWidget {
  const ContactBookPage({
    super.key,
    this.mode = ContactPickMode.browse,
    this.service,
    this.profileApi,
    this.profileCache,
    this.sessionProvider,
    this.initialProfiles = const <String, CitizenProfile>{},
    this.directChatOpener,
    this.transferOpener,
  });

  /// 页面模式:浏览 / 选收款人 / 选私信对象;不改变通讯录所属默认用户。
  final ContactPickMode mode;
  final UserContactService? service;
  final CitizenProfileApi? profileApi;
  final CitizenProfileCache? profileCache;
  final SquareSessionProvider? sessionProvider;
  final Map<String, CitizenProfile> initialProfiles;
  final DirectChatOpener? directChatOpener;

  /// 测试可替换页面打开器；正式运行始终进入现有链上支付页面。
  final Future<void> Function(
    BuildContext context, {
    required String toSs58Address,
  })? transferOpener;

  @override
  State<ContactBookPage> createState() => _ContactBookPageState();
}

class _ContactBookPageState extends State<ContactBookPage> {
  late final UserContactService _service =
      widget.service ?? UserContactService();
  late final CitizenProfileApi _profileApi =
      widget.profileApi ?? CitizenProfileApi();
  late final CitizenProfileCache _profileCache =
      widget.profileCache ?? const CitizenProfileCache();
  late final SquareSessionProvider _sessionProvider =
      widget.sessionProvider ?? SquareSessionProvider.instance;
  final TextEditingController _searchController = TextEditingController();

  List<UserContact> _contacts = const <UserContact>[];
  final Map<String, CitizenProfile> _profiles = <String, CitizenProfile>{};
  SquareSession? _session;
  ContactSyncState _syncState =
      const ContactSyncState(phase: ContactSyncPhase.idle);
  bool _loading = true;
  String _query = '';
  String _accountId = '';

  @override
  void initState() {
    super.initState();
    _profiles.addAll(widget.initialProfiles);
    _service.syncState.addListener(_onSyncStateChanged);
    unawaited(_load());
  }

  @override
  void dispose() {
    _service.syncState.removeListener(_onSyncStateChanged);
    _searchController.dispose();
    super.dispose();
  }

  void _onSyncStateChanged() {
    if (mounted) setState(() => _syncState = _service.syncState.value);
  }

  Future<void> _load() async {
    try {
      final accountId = await _service.getAccountId();
      final contacts = await _service.getContacts();
      final syncState = await _service.readSyncState();
      if (!mounted) return;
      setState(() {
        _accountId = accountId;
        _contacts = contacts;
        _syncState = syncState;
        _loading = false;
      });
      await _loadProfiles(contacts);
      await _sync();
    } on Exception catch (error) {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _syncState = ContactSyncState(
          phase: ContactSyncPhase.failed,
          message: error.toString(),
        );
      });
    }
  }

  Future<void> _sync() async {
    final contacts = await _service.sync();
    if (!mounted) return;
    setState(() => _contacts = contacts);
    await _loadProfiles(contacts);
  }

  /// 先读公开资料缓存，再以四个一组有界刷新，避免大通讯录产生瞬时请求尖峰。
  Future<void> _loadProfiles(List<UserContact> contacts) async {
    for (final contact in contacts) {
      if (_profiles.containsKey(contact.accountId)) continue;
      final cached = await _profileCache.read(contact.accountId);
      if (cached != null) _profiles[contact.accountId] = cached;
    }
    if (mounted) setState(() {});
    try {
      _session ??= await _sessionProvider.ensureSession();
    } on Exception {
      return;
    }
    for (var offset = 0; offset < contacts.length; offset += 4) {
      final end = offset + 4 < contacts.length ? offset + 4 : contacts.length;
      final batch = contacts.sublist(offset, end);
      await Future.wait(batch.map((contact) async {
        try {
          final profile = await _profileApi.fetchProfile(
            contact.accountId,
            session: _session,
          );
          _profiles[contact.accountId] = profile;
          await _profileCache.write(profile);
        } on Exception {
          // 保留缓存或稳定默认头像，单个用户资料失败不阻塞通讯录。
        }
      }));
      if (mounted) setState(() {});
    }
  }

  Future<void> _scanContactQr() async {
    // 复用统一「扫码加好友」收口(写库与提示在扫码页内完成),扫毕本地刷新。
    await scanAndAddContact(context, selfAccountId: _accountId);
    if (!mounted) return;
    final contacts = await _service.getContacts();
    if (!mounted) return;
    setState(() => _contacts = contacts);
    unawaited(_sync());
  }

  Future<void> _rename(UserContact contact) async {
    final formKey = GlobalKey<FormState>();
    var draftName = contact.contactName;
    final name = await showDialog<String>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('修改名称'),
        content: Form(
          key: formKey,
          child: TextFormField(
            initialValue: contact.contactName,
            autofocus: true,
            maxLength: 40,
            decoration: const InputDecoration(hintText: '名称'),
            onChanged: (value) => draftName = value,
            validator: (value) =>
                value == null || value.trim().isEmpty ? '名称不能为空' : null,
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () {
              if (formKey.currentState?.validate() != true) return;
              Navigator.of(dialogContext).pop(draftName.trim());
            },
            child: const Text('保存'),
          ),
        ],
      ),
    );
    if (name == null) return;
    final contacts = await _service.renameContact(contact.accountId, name);
    if (mounted) setState(() => _contacts = contacts);
  }

  Future<void> _transfer(UserContact contact) async {
    if (widget.mode == ContactPickMode.pickForTransfer) {
      Navigator.of(context).pop(contact);
      return;
    }
    final opener = widget.transferOpener;
    if (opener != null) {
      await opener(context, toSs58Address: contact.ss58Address);
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) => OnchainPaymentPage(
          initialToAddress: contact.ss58Address,
        ),
      ),
    );
  }

  Future<void> _message(UserContact contact) async {
    final profile = _profiles[contact.accountId];
    final title = ProfilePresentation.forAccount(contact.accountId)
        .resolveDisplayName(publicName: profile?.displayName);
    final opener = widget.directChatOpener ?? openDirectChat;
    await opener(
      context,
      peerAccountId: contact.accountId,
      title: title,
    );
  }

  Future<void> _delete(UserContact contact) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('删除联系人'),
        content: Text('确定从通讯录删除“${contact.contactName}”？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            style: TextButton.styleFrom(foregroundColor: AppTheme.danger),
            onPressed: () => Navigator.of(dialogContext).pop(true),
            child: const Text('删除'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;
    final contacts = await _service.deleteContact(contact.accountId);
    if (mounted) setState(() => _contacts = contacts);
  }

  void _open(UserContact contact) {
    switch (widget.mode) {
      case ContactPickMode.pickForTransfer:
        // 交易发起:把选中的联系人返回给收款栏。
        Navigator.of(context).pop(contact);
      case ContactPickMode.pickForMessage:
        // 发私信:点联系人直接打开与其的一对一聊天(复用统一私信收口)。
        unawaited(_message(contact));
      case ContactPickMode.browse:
        Navigator.of(context).push<void>(
          MaterialPageRoute<void>(
            builder: (_) => UserProfilePage(
              accountId: contact.accountId,
              isSelf: false,
              initialProfile: _profiles[contact.accountId],
            ),
          ),
        );
    }
  }

  List<UserContact> get _visibleContacts {
    final query = _query.trim().toLowerCase();
    final visible = _contacts.where((contact) {
      if (query.isEmpty) return true;
      final profile = _profiles[contact.accountId];
      final publicName = ProfilePresentation.forAccount(contact.accountId)
          .resolveDisplayName(publicName: profile?.displayName)
          .toLowerCase();
      return contact.contactName.toLowerCase().contains(query) ||
          contact.accountId.toLowerCase().contains(query) ||
          publicName.contains(query);
    }).toList(growable: false)
      ..sort((a, b) =>
          a.contactName.toLowerCase().compareTo(b.contactName.toLowerCase()));
    return visible;
  }

  @override
  Widget build(BuildContext context) {
    final visible = _visibleContacts;
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: Text(
          widget.mode == ContactPickMode.pickForMessage ? '选择联系人' : '我的通讯录',
        ),
        centerTitle: true,
        actions: [
          // 纯选人(发私信)模式只保留选择,不提供扫码加联系人入口。
          if (widget.mode != ContactPickMode.pickForMessage)
            IconButton(
              tooltip: '扫码添加联系人',
              onPressed: _scanContactQr,
              icon: SvgPicture.asset(
                'assets/icons/scan-line.svg',
                width: 20,
                height: 20,
              ),
            ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : RefreshIndicator(
              onRefresh: _sync,
              child: ListView(
                physics: const AlwaysScrollableScrollPhysics(),
                padding: const EdgeInsets.fromLTRB(16, 10, 16, 28),
                children: [
                  _SyncBanner(state: _syncState, onRetry: _sync),
                  const SizedBox(height: 10),
                  TextField(
                    key: const ValueKey('contact-search'),
                    controller: _searchController,
                    onChanged: (value) => setState(() => _query = value),
                    decoration: InputDecoration(
                      hintText: '搜索姓名、昵称或钱包账户',
                      prefixIcon: const Icon(Icons.search_rounded),
                      suffixIcon: _query.isEmpty
                          ? null
                          : IconButton(
                              onPressed: () {
                                _searchController.clear();
                                setState(() => _query = '');
                              },
                              icon: const Icon(Icons.close_rounded),
                            ),
                      filled: true,
                      fillColor: AppTheme.surfaceCard,
                      border: OutlineInputBorder(
                        borderRadius: BorderRadius.circular(14),
                        borderSide: BorderSide.none,
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  if (_contacts.isEmpty)
                    const _EmptyContacts()
                  else if (visible.isEmpty)
                    const Padding(
                      padding: EdgeInsets.symmetric(vertical: 56),
                      child: Center(child: Text('没有匹配的联系人')),
                    )
                  else
                    for (final contact in visible) ...[
                      _ContactCard(
                        contact: contact,
                        profile: _profiles[contact.accountId],
                        avatarUrl: _avatarUrl(_profiles[contact.accountId]),
                        avatarHeaders: _session == null
                            ? null
                            : <String, String>{
                                'authorization':
                                    'Bearer ${_session!.sessionToken}',
                              },
                        // 纯选私信模式只允许点选,不显示逐项操作菜单。
                        showActions:
                            widget.mode != ContactPickMode.pickForMessage,
                        onTap: () => _open(contact),
                        onTransfer: () => _transfer(contact),
                        onMessage: () => _message(contact),
                        onRename: () => _rename(contact),
                        onDelete: () => _delete(contact),
                      ),
                      const SizedBox(height: 10),
                    ],
                ],
              ),
            ),
    );
  }

  String? _avatarUrl(CitizenProfile? profile) {
    final key = profile?.avatarObjectKey;
    return key == null ? null : _profileApi.mediaUrl(key);
  }
}

class _ContactCard extends StatelessWidget {
  const _ContactCard({
    required this.contact,
    required this.profile,
    required this.avatarUrl,
    required this.avatarHeaders,
    required this.showActions,
    required this.onTap,
    required this.onTransfer,
    required this.onMessage,
    required this.onRename,
    required this.onDelete,
  });

  final UserContact contact;
  final CitizenProfile? profile;
  final String? avatarUrl;
  final Map<String, String>? avatarHeaders;

  /// 是否显示逐项操作菜单(转账/私信/改名/删除);纯选人模式为 false。
  final bool showActions;
  final VoidCallback onTap;
  final VoidCallback onTransfer;
  final VoidCallback onMessage;
  final VoidCallback onRename;
  final VoidCallback onDelete;

  @override
  Widget build(BuildContext context) {
    final publicName = ProfilePresentation.forAccount(contact.accountId)
        .resolveDisplayName(publicName: profile?.displayName);
    final bio = profile?.bio.trim() ?? '';
    final secondary = '$publicName · ${_shortAddress(contact.ss58Address)}';
    return Material(
      key: ValueKey('contact-card-${contact.accountId}'),
      color: AppTheme.surfaceCard,
      borderRadius: BorderRadius.circular(16),
      child: InkWell(
        borderRadius: BorderRadius.circular(16),
        onTap: onTap,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minHeight: 88),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                ProfileAvatar(
                  seed: contact.accountId,
                  size: 52,
                  imageUrl: avatarUrl,
                  imageHeaders: avatarHeaders,
                  identityLevel: profile?.identityLevel,
                  membershipLevel: profile?.membershipLevel,
                  membershipActive: profile?.membershipActive ?? false,
                  borderRadius: 14,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        contact.contactName,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          color: AppTheme.textPrimary,
                          fontSize: 16,
                          fontWeight: FontWeight.w700,
                        ),
                      ),
                      const SizedBox(height: 3),
                      Text(
                        secondary,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          color: AppTheme.textTertiary,
                          fontSize: 12,
                        ),
                      ),
                      if (bio.isNotEmpty) ...[
                        const SizedBox(height: 3),
                        Text(
                          bio,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            color: AppTheme.textSecondary,
                            fontSize: 12,
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
                if (showActions)
                  PopupMenuButton<_ContactMenuAction>(
                    tooltip: '联系人操作',
                    onSelected: (action) => switch (action) {
                      _ContactMenuAction.transfer => onTransfer(),
                      _ContactMenuAction.message => onMessage(),
                      _ContactMenuAction.rename => onRename(),
                      _ContactMenuAction.delete => onDelete(),
                    },
                    itemBuilder: (_) => const [
                      PopupMenuItem(
                        value: _ContactMenuAction.transfer,
                        child: Text('转账'),
                      ),
                      PopupMenuItem(
                        value: _ContactMenuAction.message,
                        child: Text('私信'),
                      ),
                      PopupMenuItem(
                        value: _ContactMenuAction.rename,
                        child: Text('修改名称'),
                      ),
                      PopupMenuItem(
                        value: _ContactMenuAction.delete,
                        child: Text(
                          '删除联系人',
                          style: TextStyle(color: AppTheme.danger),
                        ),
                      ),
                    ],
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

enum _ContactMenuAction { transfer, message, rename, delete }

class _SyncBanner extends StatelessWidget {
  const _SyncBanner({required this.state, required this.onRetry});

  final ContactSyncState state;
  final Future<void> Function() onRetry;

  @override
  Widget build(BuildContext context) {
    final retryable = state.phase == ContactSyncPhase.failed ||
        state.phase == ContactSyncPhase.offline;
    return InkWell(
      onTap: retryable ? () => unawaited(onRetry()) : null,
      borderRadius: BorderRadius.circular(12),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 5),
        child: Row(
          children: [
            Icon(
              state.phase == ContactSyncPhase.synced
                  ? Icons.cloud_done_outlined
                  : state.phase == ContactSyncPhase.syncing
                      ? Icons.sync_rounded
                      : Icons.cloud_outlined,
              size: 16,
              color: retryable ? AppTheme.warning : AppTheme.textTertiary,
            ),
            const SizedBox(width: 6),
            Text(
              state.label,
              style: TextStyle(
                color: retryable ? AppTheme.warning : AppTheme.textTertiary,
                fontSize: 12,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _EmptyContacts extends StatelessWidget {
  const _EmptyContacts();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.symmetric(vertical: 64, horizontal: 28),
      child: Column(
        children: [
          Icon(
            Icons.perm_contact_calendar_outlined,
            size: 52,
            color: AppTheme.primary,
          ),
          SizedBox(height: 16),
          Text(
            '通讯录还是空的',
            style: TextStyle(fontSize: 19, fontWeight: FontWeight.w700),
          ),
          SizedBox(height: 8),
          Text(
            '扫描其他用户的二维码添加联系人，密文同步后换设备也能恢复。',
            textAlign: TextAlign.center,
            style: TextStyle(color: AppTheme.textSecondary, height: 1.5),
          ),
        ],
      ),
    );
  }
}

String _shortAddress(String address) {
  if (address.length <= 14) return address;
  return '${address.substring(0, 6)}...${address.substring(address.length - 5)}';
}

/// 通讯录 / 聊天页共用的「扫码加好友」收口。
///
/// 打开 contact 模式扫码页,扫到用户名片码即写入本人密文通讯录
/// (写库与「已加入/已更新通讯录」提示均在扫码页内完成)。[selfAccountId]
/// 传本人账户,供扫码页拒绝把自己加为联系人。调用方在返回后自行刷新列表。
Future<void> scanAndAddContact(
  BuildContext context, {
  required String selfAccountId,
}) async {
  await Navigator.of(context).push<void>(
    MaterialPageRoute<void>(
      builder: (_) => QrScanPage(
        mode: QrScanMode.contact,
        selfAddress: selfAccountId,
      ),
    ),
  );
}
