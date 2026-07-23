import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/foundation.dart';

import '../8964/profile/user_qr_page.dart';
import '../my/user/contact_book_page.dart';
import '../qr/scan_dispatch_flow.dart';
import '../ui/app_theme.dart';
import '../wallet/core/wallet_manager.dart';
import 'chat_page.dart';
import 'chat_runtime.dart';
import 'chat_models.dart';
import 'chat_search_page.dart';
import 'group/ui/group_create_page.dart';
import 'group/ui/group_manage_page.dart';
import 'group/ui/open_group_chat.dart';
import 'storage/chat_store.dart';

typedef ChatSendTextFactory = ChatSendTextCallback? Function(
  String peerAccountId,
  String conversationId,
);
typedef ChatSyncFactory = ChatSyncCallback? Function(String peerAccountId);
typedef ChatSendMediaFactory = ChatSendMediaCallback? Function(
  String peerAccountId,
  String conversationId,
);
typedef ChatDownloadAttachmentFactory = ChatDownloadAttachmentCallback?
    Function(
  String peerAccountId,
);

/// 聊天页加号菜单 5 个动作的可注入入口。
///
/// 默认全为 null，各动作走真实实现；测试整体替换后即可断言路由，
/// 而不会真的拉起相机、建群页或通讯录（它们会触发 Isar / ChatRuntime / 相机）。
class ChatEntryOpeners {
  const ChatEntryOpeners({
    this.openScan,
    this.openReceivePay,
    this.openSendMessage,
    this.openCreateGroup,
    this.openAddFriend,
  });

  /// 扫一扫 = 交易·扫一扫统一分派（扫到用户二维码按收款人进入转账）。
  final ChatEntryOpener? openScan;

  /// 收付款 = 展示本人唯一用户二维码。
  final ChatEntryOpener? openReceivePay;

  /// 发私信 = 通讯录单选后直开私聊。
  final ChatEntryOpener? openSendMessage;

  /// 发群聊 = 通讯录多选（≥2 人）建群。
  final ChatEntryOpener? openCreateGroup;

  /// 加好友 = 扫对方二维码写入本人通讯录。
  final ChatEntryOpener? openAddFriend;
}

/// 加号菜单单个动作的入口签名；默认钱包等依赖一律由真实实现内部解析，
/// 注入替身时不触碰 WalletManager / Isar / 相机。
typedef ChatEntryOpener = Future<void> Function(BuildContext context);

/// 公民“聊天”Tab。
///
/// 顶部为搜索框（进入 [ChatSearchPage]），右上角加号弹出 5 个入口：
/// 扫一扫 / 收付款 / 发私信 / 发群聊 / 加好友。会话列表在其下方。
class ChatTab extends StatefulWidget {
  ChatTab({
    super.key,
    ChatStore? store,
    WalletManager? walletManager,
    this.accountId,
    this.sendTextFactory,
    this.sendMediaFactory,
    this.downloadAttachmentFactory,
    this.syncFactory,
    this.runtime,
    this.selectedTab,
    this.tabIndex = 2,
    this.openers,
  })  : store = store ?? ChatStore(),
        walletManager = walletManager ?? WalletManager();

  final ChatStore store;
  final WalletManager walletManager;
  final String? accountId;
  final ChatSendTextFactory? sendTextFactory;
  final ChatSendMediaFactory? sendMediaFactory;
  final ChatDownloadAttachmentFactory? downloadAttachmentFactory;
  final ChatSyncFactory? syncFactory;
  final ChatRuntime? runtime;
  final ValueListenable<int>? selectedTab;
  final int tabIndex;

  /// 加号菜单动作入口；仅测试注入，正式运行为 null 走真实实现。
  final ChatEntryOpeners? openers;

  @override
  State<ChatTab> createState() => _ChatTabState();
}

class _ChatTabState extends State<ChatTab> {
  // 聊天页只做前台轻量轮询；离开页面或 App 退后台即停止，不做后台常驻扫描。
  static const _normalPollInterval = Duration(seconds: 15);
  static const _backoffPollInterval = Duration(seconds: 30);

  List<ChatConversationPreview> _conversations = const [];
  String _accountId = '';
  bool _loading = true;
  bool _polling = false;
  bool _realtimeConnecting = false;
  String? _error;
  Timer? _pollTimer;
  String? _realtimeWallet;
  Future<void> Function()? _stopRealtime;
  late final _ChatTabLifecycleObserver _lifecycleObserver;
  Future<void>? _coordinatorInFlight;
  bool _appResumed = false;

  bool get _isActive =>
      (widget.selectedTab == null ||
          widget.selectedTab!.value == widget.tabIndex) &&
      _appResumed;

  @override
  void initState() {
    super.initState();
    final lifecycleState = WidgetsBinding.instance.lifecycleState;
    _appResumed =
        lifecycleState == null || lifecycleState == AppLifecycleState.resumed;
    _lifecycleObserver = _ChatTabLifecycleObserver(
      onResume: () {
        _appResumed = true;
        _requestCoordinate();
      },
      onPause: () {
        _appResumed = false;
        _pauseSync();
      },
    );
    WidgetsBinding.instance.addObserver(_lifecycleObserver);
    widget.selectedTab?.addListener(_onSelectedTabChanged);
    // 本页常驻 IndexedStack；切换默认用户钱包（= 切换聊天身份）后经
    // walletsRevision 广播重载，会话列表 accountId 立即切到新默认用户，
    // 不再等 App 退后台回前台。
    WalletManager.walletsRevision.addListener(_onWalletsChanged);
    WidgetsBinding.instance.addPostFrameCallback((_) => _requestCoordinate());
  }

  @override
  void didUpdateWidget(covariant ChatTab oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.selectedTab != widget.selectedTab) {
      oldWidget.selectedTab?.removeListener(_onSelectedTabChanged);
      widget.selectedTab?.addListener(_onSelectedTabChanged);
      _onSelectedTabChanged();
    }
  }

  @override
  void dispose() {
    WalletManager.walletsRevision.removeListener(_onWalletsChanged);
    widget.selectedTab?.removeListener(_onSelectedTabChanged);
    _pauseSync();
    WidgetsBinding.instance.removeObserver(_lifecycleObserver);
    super.dispose();
  }

  void _onSelectedTabChanged() {
    if (_isActive) {
      _requestCoordinate();
    } else {
      _pauseSync();
    }
  }

  /// init、进入 Tab、App resume 全部汇入同一个 coordinator；同一时刻只允许
  /// 一个 reload/sync 链，避免系统 UI 导致 lifecycle 抖动时重复初始化。
  void _requestCoordinate() {
    if (!mounted || !_isActive || _coordinatorInFlight != null) {
      return;
    }
    late final Future<void> created;
    created = _reload(syncFirst: true).whenComplete(() {
      if (identical(_coordinatorInFlight, created)) {
        _coordinatorInFlight = null;
      }
    });
    _coordinatorInFlight = created;
  }

  Future<void> _onWalletsChanged() async {
    // 先廉价比对(纯 Isar 读):默认聊天身份没变的钱包操作(重命名/导入
    // 未置顶钱包)不触发发送队列重试,避免整页转圈与无谓网络请求。
    final address = await _readAccountId();
    if (!mounted || address == _accountId) return;
    if (_accountId.isNotEmpty) {
      widget.runtime?.invalidateAccount(_accountId);
    }
    _pauseSync();
    _requestCoordinate();
  }

  /// _reload 世代号:含网络同步(秒级),切默认钱包后并发 reload 乱序完成时
  /// 旧身份不得覆写新身份,也不得以旧身份重建轮询/realtime。
  int _reloadGeneration = 0;

  Future<void> _reload({bool syncFirst = false}) async {
    if (!_isActive) {
      return;
    }
    final generation = ++_reloadGeneration;
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final activeWallet = widget.accountId ?? await _readAccountId();
      if (!_isActive) {
        return;
      }
      if (syncFirst && activeWallet.isNotEmpty) {
        await _retryOutgoingSilently();
      }
      final conversations = await widget.store.readConversationPreviews(
        accountId: activeWallet.isEmpty ? null : activeWallet,
      );
      if (!mounted || !_isActive || generation != _reloadGeneration) {
        return;
      }
      setState(() {
        _conversations = conversations;
        _accountId = activeWallet;
      });
      _configurePolling(activeWallet);
    } catch (error) {
      if (mounted && generation == _reloadGeneration) {
        setState(() {
          _error = error.toString();
        });
      }
    } finally {
      if (mounted && generation == _reloadGeneration) {
        setState(() {
          _loading = false;
        });
      }
    }
  }

  Future<bool> _retryOutgoingSilently() async {
    if (!_isActive) {
      return false;
    }
    final runtime = widget.runtime;
    if (runtime == null) {
      return true;
    }
    try {
      await runtime.retryOutgoing();
      return true;
    } catch (_) {
      return false;
    }
  }

  void _configurePolling(String activeWallet) {
    if (!_isActive || activeWallet.isEmpty || widget.runtime == null) {
      _pauseSync();
      return;
    }
    if (_realtimeWallet != null && _realtimeWallet != activeWallet) {
      _pauseSync();
    }
    if (_stopRealtime != null) {
      return;
    }
    _schedulePoll(_normalPollInterval);
    unawaited(_startRealtime(activeWallet));
  }

  Future<bool> _startRealtime(String activeWallet) async {
    final runtime = widget.runtime;
    if (!_isActive || runtime == null || activeWallet.isEmpty) {
      return false;
    }
    if (_stopRealtime != null || _realtimeConnecting) {
      return _stopRealtime != null;
    }
    _realtimeConnecting = true;
    try {
      final stop = await runtime.startRealtimeSync(
        onNotice: () => _syncAndRefresh(activeWallet),
        onDisconnected: () async {
          _stopRealtime = null;
          _realtimeWallet = null;
          if (_isActive &&
              mounted &&
              widget.runtime != null &&
              _accountId.isNotEmpty) {
            _schedulePoll(_backoffPollInterval);
          }
        },
      );
      if (!mounted || !_isActive || _accountId != activeWallet) {
        await stop?.call();
        return false;
      }
      _stopRealtime = stop;
      _realtimeWallet = activeWallet;
      if (stop != null) {
        _stopPolling();
      }
      return stop != null;
    } catch (_) {
      return false;
    } finally {
      _realtimeConnecting = false;
    }
  }

  Future<void> _syncAndRefresh(String accountId) async {
    if (!_isActive) {
      return;
    }
    await _retryOutgoingSilently();
    final conversations = await widget.store.readConversationPreviews(
      accountId: accountId,
    );
    if (mounted && _accountId == accountId) {
      setState(() {
        _conversations = conversations;
      });
    }
  }

  void _schedulePoll(Duration delay) {
    if (!_isActive) {
      return;
    }
    _pollTimer?.cancel();
    _pollTimer = Timer(delay, _runPoll);
  }

  void _stopPolling() {
    _pollTimer?.cancel();
    _pollTimer = null;
  }

  void _pauseSync() {
    _stopPolling();
    final stop = _stopRealtime;
    _stopRealtime = null;
    _realtimeWallet = null;
    if (stop != null) {
      unawaited(stop());
    }
  }

  Future<void> _runPoll() async {
    if (!mounted ||
        !_isActive ||
        widget.runtime == null ||
        _accountId.isEmpty) {
      return;
    }
    if (_stopRealtime != null) {
      return;
    }
    if (_polling) {
      _schedulePoll(_backoffPollInterval);
      return;
    }
    _polling = true;
    var ok = true;
    try {
      ok = await _retryOutgoingSilently();
      final conversations = await widget.store.readConversationPreviews(
        accountId: _accountId,
      );
      if (mounted) {
        setState(() {
          _conversations = conversations;
        });
      }
    } catch (_) {
      ok = false;
    }
    _polling = false;
    if (_isActive &&
        mounted &&
        widget.runtime != null &&
        _accountId.isNotEmpty) {
      if (ok && await _startRealtime(_accountId)) {
        return;
      }
      _schedulePoll(ok ? _normalPollInterval : _backoffPollInterval);
    }
  }

  Future<String> _readAccountId() async {
    final runtimeAddress = await widget.runtime?.readAccountId();
    if (runtimeAddress != null && runtimeAddress.isNotEmpty) {
      return runtimeAddress;
    }
    // 身份统一取默认用户钱包（列表中最靠前的热钱包）。
    final wallet = await widget.walletManager.getDefaultWallet();
    return wallet?.accountId ?? '';
  }

  Future<void> _deleteLocalConversation(String conversationId) {
    final runtime = widget.runtime;
    if (runtime != null) {
      return runtime.deleteLocalConversation(conversationId);
    }
    return widget.store.deleteConversation(conversationId);
  }

  Future<void> _confirmAndDeleteConversation(
    ChatConversationPreview preview,
  ) async {
    final confirmed = await _confirmDeleteConversation(context);
    if (!confirmed || !mounted) {
      return;
    }
    try {
      await _deleteLocalConversation(preview.conversationId);
      if (!mounted) {
        return;
      }
      setState(() {
        _conversations = _conversations
            .where(
              (item) => item.conversationId != preview.conversationId,
            )
            .toList(growable: false);
        _error = null;
      });
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    }
  }

  void _openConversation(ChatConversationPreview preview) {
    if (_accountId.isEmpty) {
      setState(() {
        _error = '请先在「我的 → 我的钱包」创建热钱包';
      });
      return;
    }
    if (preview.isGroup) {
      openGroupChat(
        context,
        groupId: preview.conversationId,
        title: preview.title,
      ).then((_) => _reload());
      return;
    }
    _openDirectConversation(preview);
  }

  void _openCreateGroup() {
    if (!_requireAccount()) return;
    final opener = widget.openers?.openCreateGroup;
    if (opener != null) {
      unawaited(opener(context));
      return;
    }
    Navigator.of(context)
        .push(
          MaterialPageRoute<void>(builder: (_) => const GroupCreatePage()),
        )
        .then((_) => _reload());
  }

  /// 没有热钱包时统一提示并拦截；聊天与收付款都依赖默认热钱包账户。
  bool _requireAccount() {
    if (_accountId.isEmpty) {
      setState(() => _error = '请先在「我的 → 我的钱包」创建热钱包');
      return false;
    }
    return true;
  }

  /// 加号菜单动作分派。
  Future<void> _onEntryAction(_ChatEntryAction action) async {
    switch (action) {
      case _ChatEntryAction.scan:
        await _openScan();
      case _ChatEntryAction.receivePay:
        await _openReceivePay();
      case _ChatEntryAction.sendMessage:
        await _openSendMessage();
      case _ChatEntryAction.createGroup:
        _openCreateGroup();
      case _ChatEntryAction.addFriend:
        await _openAddFriend();
    }
  }

  /// 扫一扫 = 交易·扫一扫统一分派；扫到用户二维码按收款人进入转账。
  Future<void> _openScan() async {
    final opener = widget.openers?.openScan;
    if (opener != null) {
      await opener(context);
      return;
    }
    final wallet = await widget.walletManager.getDefaultWallet();
    if (!mounted) return;
    await openScanDispatchFlow(context: context, paymentWallet: wallet);
  }

  /// 收付款 = 展示本人唯一用户二维码，他人扫码后按收款人向我转账。
  Future<void> _openReceivePay() async {
    if (!_requireAccount()) return;
    final opener = widget.openers?.openReceivePay;
    if (opener != null) {
      await opener(context);
      return;
    }
    final wallet = await widget.walletManager.getDefaultWallet();
    if (!mounted) return;
    if (wallet == null) {
      setState(() => _error = '请先在「我的 → 我的钱包」创建热钱包');
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) => UserQrPage(
          accountId: wallet.accountId,
          contactName: wallet.walletName,
        ),
      ),
    );
  }

  /// 发私信 = 通讯录单选，点联系人直接开私聊。
  Future<void> _openSendMessage() async {
    if (!_requireAccount()) return;
    final opener = widget.openers?.openSendMessage;
    if (opener != null) {
      await opener(context);
      return;
    }
    await Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) =>
            const ContactBookPage(mode: ContactPickMode.pickForMessage),
      ),
    );
    if (!mounted) return;
    await _reload();
  }

  /// 加好友 = 扫对方二维码写入本人密文通讯录。
  Future<void> _openAddFriend() async {
    if (!_requireAccount()) return;
    final opener = widget.openers?.openAddFriend;
    if (opener != null) {
      await opener(context);
      return;
    }
    await scanAndAddContact(context, selfAccountId: _accountId);
  }

  /// 搜索 = 进入独立搜索页；透传 store 与账户，避免搜索页重复解析依赖。
  Future<void> _openSearch() async {
    await Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) => ChatSearchPage(
          store: widget.store,
          accountId: _accountId,
        ),
      ),
    );
  }

  void _openGroupManage(ChatConversationPreview preview) {
    Navigator.of(context)
        .push(
          MaterialPageRoute<void>(
            builder: (_) => GroupManagePage(groupId: preview.conversationId),
          ),
        )
        .then((_) => _reload());
  }

  void _openDirectConversation(ChatConversationPreview preview) {
    Navigator.of(context)
        .push(
          MaterialPageRoute<void>(
            builder: (context) => ChatPage(
              conversationId: preview.conversationId,
              accountId: _accountId,
              peerUserId: preview.peerAccountId,
              title: preview.title,
              store: widget.store,
              onSendText: widget.sendTextFactory?.call(
                    preview.peerAccountId,
                    preview.conversationId,
                  ) ??
                  (widget.runtime == null
                      ? null
                      : (text) => widget.runtime!.sendText(
                            peerAccountId: preview.peerAccountId,
                            conversationId: preview.conversationId,
                            text: text,
                          )),
              onSendMedia: widget.sendMediaFactory?.call(
                    preview.peerAccountId,
                    preview.conversationId,
                  ) ??
                  (widget.runtime == null
                      ? null
                      : (media) => widget.runtime!.sendMedia(
                            peerAccountId: preview.peerAccountId,
                            conversationId: preview.conversationId,
                            media: media,
                          )),
              onSendSticker: widget.runtime == null
                  ? null
                  : (packId, stickerId) => widget.runtime!.sendSticker(
                        peerAccountId: preview.peerAccountId,
                        conversationId: preview.conversationId,
                        packId: packId,
                        stickerId: stickerId,
                      ),
              onResolveMediaPath: widget.runtime == null
                  ? null
                  : (
                      String conversationId,
                      String attachmentId,
                      String fileName,
                      String contentType,
                      int clearByteSize,
                    ) =>
                      widget.runtime!.resolveCachedMediaPath(
                        conversationId: conversationId,
                        attachmentId: attachmentId,
                        fileName: fileName,
                        contentType: contentType,
                        clearByteSize: clearByteSize,
                      ),
              onDownloadAttachment: widget.downloadAttachmentFactory?.call(
                    preview.peerAccountId,
                  ) ??
                  (widget.runtime == null
                      ? null
                      : (
                          String conversationId,
                          String controlPlaintext,
                        ) =>
                          widget.runtime!.downloadAttachment(
                            conversationId: conversationId,
                            controlPlaintext: controlPlaintext,
                          )),
              onSync: widget.syncFactory?.call(preview.peerAccountId) ??
                  (widget.runtime == null
                      ? null
                      : () => widget.runtime!.retryOutgoing()),
              onStartRealtime: widget.runtime?.startRealtimeSync,
              onDeleteConversation: () => _deleteLocalConversation(
                preview.conversationId,
              ),
            ),
          ),
        )
        .then((_) => _reload());
  }

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: ColoredBox(
        color: AppTheme.scaffoldBg,
        child: RefreshIndicator(
          onRefresh: () => _reload(syncFirst: true),
          child: CustomScrollView(
            slivers: [
              SliverToBoxAdapter(child: _ChatHeader(onAction: _onEntryAction)),
              if (_error != null)
                SliverToBoxAdapter(child: _ErrorBanner(message: _error!)),
              if (!_loading && _accountId.isNotEmpty)
                SliverToBoxAdapter(
                  child: _SearchEntry(onTap: () => unawaited(_openSearch())),
                ),
              if (_loading)
                const SliverFillRemaining(
                  hasScrollBody: false,
                  child: Center(child: CircularProgressIndicator()),
                )
              else if (_accountId.isEmpty)
                const SliverFillRemaining(
                  hasScrollBody: false,
                  child: _NoAccount(),
                )
              else if (_conversations.isNotEmpty)
                SliverList.builder(
                  itemCount: _conversations.length,
                  itemBuilder: (context, index) {
                    final preview = _conversations[index];
                    return _ConversationTile(
                      preview: preview,
                      onTap: () => _openConversation(preview),
                      onDelete: () => _confirmAndDeleteConversation(preview),
                      onManage: preview.isGroup
                          ? () => _openGroupManage(preview)
                          : null,
                    );
                  },
                )
              else
                const SliverFillRemaining(
                  hasScrollBody: false,
                  child: _EmptyConversationList(),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ChatTabLifecycleObserver extends WidgetsBindingObserver {
  _ChatTabLifecycleObserver({
    required this.onResume,
    required this.onPause,
  });

  final VoidCallback onResume;
  final VoidCallback onPause;

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.resumed) {
      onResume();
    } else {
      onPause();
    }
  }
}

/// 加号菜单的 5 个动作。
enum _ChatEntryAction { scan, receivePay, sendMessage, createGroup, addFriend }

class _ChatHeader extends StatelessWidget {
  const _ChatHeader({required this.onAction});

  final ValueChanged<_ChatEntryAction> onAction;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(20, 18, 20, 12),
      child: Row(
        children: [
          const Expanded(
            child: Text(
              '聊天',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.w700,
                color: AppTheme.textPrimary,
              ),
            ),
          ),
          // 用 PopupMenuButton 自带的锚定弹出，避免手算右上角坐标。
          PopupMenuButton<_ChatEntryAction>(
            tooltip: '新建',
            icon: const Icon(Icons.add_rounded, color: AppTheme.textSecondary),
            onSelected: onAction,
            itemBuilder: (_) => const [
              PopupMenuItem(
                value: _ChatEntryAction.scan,
                child: Text('扫一扫'),
              ),
              PopupMenuItem(
                value: _ChatEntryAction.receivePay,
                child: Text('收付款'),
              ),
              PopupMenuItem(
                value: _ChatEntryAction.sendMessage,
                child: Text('发私信'),
              ),
              PopupMenuItem(
                value: _ChatEntryAction.createGroup,
                child: Text('发群聊'),
              ),
              PopupMenuItem(
                value: _ChatEntryAction.addFriend,
                child: Text('加好友'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _ConversationTile extends StatelessWidget {
  const _ConversationTile({
    required this.preview,
    required this.onTap,
    required this.onDelete,
    this.onManage,
  });

  final ChatConversationPreview preview;
  final VoidCallback onTap;
  final Future<void> Function() onDelete;
  final VoidCallback? onManage;

  @override
  Widget build(BuildContext context) {
    final subtitle = preview.lastMessage.trim().isEmpty
        ? preview.peerAccountId
        : preview.lastMessage.trim();
    return Dismissible(
      key: ValueKey('chat-conversation-${preview.conversationId}'),
      direction: DismissDirection.endToStart,
      background: const _DeleteDismissBackground(),
      confirmDismiss: (_) async {
        await onDelete();
        return false;
      },
      child: _ListTileShell(
        title: preview.isGroup ? '👥 ${preview.title}' : preview.title,
        subtitle: subtitle,
        trailing: _statusText(preview.deliveryState),
        unreadCount: preview.unreadCount,
        onTap: onTap,
        onLongPress: onManage,
        isGroup: preview.isGroup,
      ),
    );
  }
}

/// 顶部搜索入口：点击进入 [ChatSearchPage]（会话 / 联系人 / 聊天记录）。
class _SearchEntry extends StatelessWidget {
  const _SearchEntry({required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 4, 16, 8),
      child: Material(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(10),
        child: InkWell(
          borderRadius: BorderRadius.circular(10),
          onTap: onTap,
          child: const Padding(
            padding: EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            child: Row(
              children: [
                Icon(
                  Icons.search_rounded,
                  size: 20,
                  color: AppTheme.textTertiary,
                ),
                SizedBox(width: 8),
                Text(
                  '搜索',
                  style: TextStyle(color: AppTheme.textTertiary, fontSize: 15),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _DeleteDismissBackground extends StatelessWidget {
  const _DeleteDismissBackground();

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: Colors.red.shade600,
          borderRadius: BorderRadius.circular(8),
        ),
        child: const Align(
          alignment: Alignment.centerRight,
          child: Padding(
            padding: EdgeInsets.only(right: 20),
            child: Icon(Icons.delete_outline_rounded, color: Colors.white),
          ),
        ),
      ),
    );
  }
}

class _ListTileShell extends StatelessWidget {
  const _ListTileShell({
    required this.title,
    required this.subtitle,
    required this.trailing,
    required this.unreadCount,
    required this.onTap,
    this.onLongPress,
    this.isGroup = false,
  });

  final String title;
  final String subtitle;
  final String trailing;
  final int unreadCount;
  final VoidCallback onTap;
  final VoidCallback? onLongPress;
  final bool isGroup;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: Material(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(8),
        child: InkWell(
          borderRadius: BorderRadius.circular(8),
          onTap: onTap,
          onLongPress: onLongPress,
          child: Padding(
            padding: const EdgeInsets.all(14),
            child: Row(
              children: [
                CircleAvatar(
                  backgroundColor: AppTheme.primary.withAlpha(24),
                  child: Icon(
                    isGroup ? Icons.groups_outlined : Icons.person_outline,
                    color: AppTheme.primary,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontWeight: FontWeight.w700,
                          color: AppTheme.textPrimary,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        subtitle,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: const TextStyle(
                          fontSize: 13,
                          color: AppTheme.textSecondary,
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 10),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.end,
                  children: [
                    Text(
                      trailing,
                      style: const TextStyle(
                        fontSize: 12,
                        color: AppTheme.textSecondary,
                      ),
                    ),
                    if (unreadCount > 0) ...[
                      const SizedBox(height: 6),
                      CircleAvatar(
                        radius: 10,
                        backgroundColor: AppTheme.primary,
                        child: Text(
                          '$unreadCount',
                          style: const TextStyle(
                            color: Colors.white,
                            fontSize: 11,
                          ),
                        ),
                      ),
                    ],
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

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
      child: Text(
        message,
        style: const TextStyle(color: Colors.red, fontSize: 12),
      ),
    );
  }
}

class _NoAccount extends StatelessWidget {
  const _NoAccount();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Padding(
        padding: EdgeInsets.fromLTRB(32, 32, 32, 80),
        child: Text(
          '请先在「我的 → 我的钱包」创建热钱包',
          textAlign: TextAlign.center,
          style: TextStyle(
            color: AppTheme.textSecondary,
            fontSize: 15,
            fontWeight: FontWeight.w500,
          ),
        ),
      ),
    );
  }
}

class _EmptyConversationList extends StatelessWidget {
  const _EmptyConversationList();

  @override
  Widget build(BuildContext context) {
    return const Center(
      child: Padding(
        padding: EdgeInsets.fromLTRB(32, 32, 32, 80),
        child: Text(
          '暂无会话',
          style: TextStyle(
            color: AppTheme.textSecondary,
            fontSize: 15,
            fontWeight: FontWeight.w500,
          ),
        ),
      ),
    );
  }
}

String _statusText(ChatMessageDeliveryState state) {
  return switch (state) {
    ChatMessageDeliveryState.queued => '排队',
    ChatMessageDeliveryState.sending => '发送中',
    ChatMessageDeliveryState.sent => '已发送',
    ChatMessageDeliveryState.receivedByDevice => '已接收',
    ChatMessageDeliveryState.failed => '失败',
  };
}

Future<bool> _confirmDeleteConversation(BuildContext context) async {
  final confirmed = await showDialog<bool>(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('删除聊天记录'),
      content: const Text('确定删除这台设备上的聊天记录？'),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(false),
          child: const Text('取消'),
        ),
        TextButton(
          onPressed: () => Navigator.of(context).pop(true),
          child: const Text('删除'),
        ),
      ],
    ),
  );
  return confirmed ?? false;
}
