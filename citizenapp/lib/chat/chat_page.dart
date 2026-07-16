import 'dart:async';
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:emoji_picker_flutter/emoji_picker_flutter.dart';
import 'package:flutter_blurhash/flutter_blurhash.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';
import 'package:flutter_chat_ui/flutter_chat_ui.dart';
import 'package:file_picker/file_picker.dart';

import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/widgets/profile_avatar.dart';
import '../ui/app_theme.dart';
import 'chat_ui_adapter.dart';
import 'chat_flow.dart';
import 'chat_media_limits.dart';
import 'chat_payload.dart';
import 'compose/media_source_sheet.dart';
import 'compose/sticker_panel.dart';
import 'media/media_compressor.dart';
import 'media/media_mime.dart';
import 'media/media_picker.dart';
import 'media/media_probe.dart';
import 'stickers/sticker_pack.dart';
import 'storage/chat_store.dart';
import 'viewer/image_viewer_page.dart';
import 'viewer/video_player_page.dart';

typedef ChatSendTextCallback = Future<void> Function(String text);
typedef ChatSendMediaCallback = Future<void> Function(
  ChatMediaDraft media,
);
typedef ChatSendStickerCallback = Future<void> Function(
  String packId,
  String stickerId,
);
typedef ChatSyncCallback = Future<int> Function();
typedef ChatStartRealtimeCallback = Future<Future<void> Function()?> Function({
  required Future<void> Function() onNotice,
  Future<void> Function()? onDisconnected,
});
typedef ChatDownloadAttachmentCallback = Future<ChatDownloadedAttachment>
    Function(
  String conversationId,
  String controlPlaintext,
);
typedef ChatPickMediaCallback = Future<ChatMediaDraft?> Function();
typedef ChatResolveMediaPathCallback = Future<String?> Function(
  String conversationId,
  String attachmentId,
  String fileName,
  String contentType,
  int clearByteSize,
);
typedef ChatDeleteConversationCallback = Future<void> Function();

/// 公民 Chat 聊天详情页。
///
/// 页面只使用现成聊天 UI 展示和输入，消息真源仍是本地
/// [ChatStore]，发送和同步由上层注入的 P2P/MLS 状态机完成。
class ChatPage extends StatefulWidget {
  ChatPage({
    super.key,
    required this.conversationId,
    required this.ownerAccount,
    required this.peerUserId,
    required this.title,
    this.isGroup = false,
    ChatStore? store,
    this.onSendText,
    this.onSendMedia,
    this.onSendSticker,
    this.onDownloadAttachment,
    this.onResolveMediaPath,
    this.pickMedia,
    this.onSync,
    this.onStartRealtime,
    this.onDeleteConversation,
  }) : store = store ?? ChatStore();

  final String conversationId;
  final String ownerAccount;
  final String peerUserId;
  final String title;

  /// 群聊模式:入站消息按各自 `senderAccount` 归属并在气泡上方显示发送者名。
  final bool isGroup;
  final ChatStore store;
  final ChatSendTextCallback? onSendText;
  final ChatSendMediaCallback? onSendMedia;
  final ChatSendStickerCallback? onSendSticker;
  final ChatDownloadAttachmentCallback? onDownloadAttachment;
  final ChatResolveMediaPathCallback? onResolveMediaPath;
  final ChatPickMediaCallback? pickMedia;
  final ChatSyncCallback? onSync;
  final ChatStartRealtimeCallback? onStartRealtime;
  final ChatDeleteConversationCallback? onDeleteConversation;

  @override
  State<ChatPage> createState() => _ChatPageState();
}

class _ChatPageState extends State<ChatPage> {
  // 实时连接不可用时只重试发送设备本机队列；失败后退避，避免弱网持续请求。
  static const _normalPollInterval = Duration(seconds: 8);
  static const _backoffPollInterval = Duration(seconds: 30);
  // 实时已连时仍保留的低频心跳兜底：即使 WS 推送静默丢失，也能在此间隔内收到。
  static const _heartbeatPollInterval = Duration(seconds: 20);

  late final InMemoryChatController _chatController;
  late final _ChatLifecycleObserver _lifecycleObserver;
  // 自绘 composer 的文本控制器:贴纸面板与(步骤3b)表情插入共享,发送后由
  // Composer 自动清空。
  final TextEditingController _composerController = TextEditingController();
  bool _loading = true;
  bool _syncing = false;
  bool _attachmentBusy = false;
  bool _deleting = false;
  bool _polling = false;
  bool _realtimeConnecting = false;
  bool _appResumed = false;
  _ComposerPanel _openPanel = _ComposerPanel.none;
  String? _error;
  Timer? _pollTimer;
  Future<void> Function()? _stopRealtime;
  Future<void>? _openCoordinatorInFlight;

  final MediaPicker _mediaPicker = MediaPicker();
  final MediaCompressor _mediaCompressor = MediaCompressor();
  final MediaProbe _mediaProbe = MediaProbe();

  @override
  void initState() {
    super.initState();
    _chatController = InMemoryChatController();
    final lifecycleState = WidgetsBinding.instance.lifecycleState;
    _appResumed =
        lifecycleState == null || lifecycleState == AppLifecycleState.resumed;
    _lifecycleObserver = _ChatLifecycleObserver(
      onResume: () {
        _appResumed = true;
        _requestOpenCoordinate();
      },
      onPause: () {
        _appResumed = false;
        _pauseSync();
      },
    );
    WidgetsBinding.instance.addObserver(_lifecycleObserver);
    WidgetsBinding.instance
        .addPostFrameCallback((_) => _requestOpenCoordinate());
  }

  /// 首次打开和 resume 共享同一个同步 future，系统生命周期抖动不得重复建立
  /// WebSocket 或重复重试本机发送队列。
  void _requestOpenCoordinate() {
    if (!mounted || !_appResumed || _openCoordinatorInFlight != null) {
      return;
    }
    late final Future<void> created;
    created = _syncOnOpen().whenComplete(() {
      if (identical(_openCoordinatorInFlight, created)) {
        _openCoordinatorInFlight = null;
      }
    });
    _openCoordinatorInFlight = created;
  }

  @override
  void dispose() {
    _pauseSync();
    WidgetsBinding.instance.removeObserver(_lifecycleObserver);
    _chatController.dispose();
    _composerController.dispose();
    super.dispose();
  }

  Future<void> _reloadMessages() async {
    // 只清错误提示,不重置 _loading:首屏骨架靠初始 _loading=true 驱动,后续重载
    // (发消息/贴纸/同步后)保持 Chat 常驻——否则整块 Chat 连同贴纸面板/分类 Tab
    // 会 unmount 重建,连发贴纸时面板闪走、Tab 跳回第一个。
    if (_error != null) {
      setState(() {
        _error = null;
      });
    }
    try {
      final messages = await widget.store.readMessages(widget.conversationId);
      final mediaPaths = await _resolveMediaPaths(messages);
      await _chatController.setMessages(
        storedMessagesToChatMessages(
          messages,
          ownerAccount: widget.ownerAccount,
          resolveLocalMediaPath: (content) => mediaPaths[content.attachmentId],
        ),
        animated: false,
      );
    } catch (error) {
      _error = error.toString();
    } finally {
      // 首次加载结束翻下骨架;之后 _loading 恒为 false,此处成幂等空转。
      if (mounted && (_loading || _error != null)) {
        setState(() {
          _loading = false;
        });
      }
    }
  }

  /// 预解析媒体消息在本机缓存中的绝对路径,按 attachment_id 建表。字节未到达
  /// (对方离线/仍在传输)的媒体不入表,由渲染层显示占位。
  Future<Map<String, String>> _resolveMediaPaths(
    List<ChatStoredMessage> messages,
  ) async {
    final resolver = widget.onResolveMediaPath;
    if (resolver == null) {
      return const {};
    }
    final paths = <String, String>{};
    for (final message in messages) {
      final content = ChatPayloadCodec.decode(message.plaintext ?? '');
      final attachmentId = content.attachmentId ?? '';
      if (!content.isMedia || attachmentId.isEmpty) {
        continue;
      }
      // 门④对应:声明超限的媒体已在字节层拒收、UI 显"已拒收",不解析路径。
      if (ChatMediaLimits.exceedsForKind(content.kind, content.byteSize ?? 0)) {
        continue;
      }
      final path = await resolver(
        widget.conversationId,
        attachmentId,
        content.fileName ?? '',
        content.mime ?? 'application/octet-stream',
        content.byteSize ?? 0,
      );
      if (path != null && path.isNotEmpty) {
        paths[attachmentId] = path;
      }
    }
    return paths;
  }

  Future<void> _syncOnOpen() async {
    final sync = widget.onSync;
    if (sync == null) {
      await _reloadMessages();
      return;
    }
    await _syncAndReload(silent: true);
    if (mounted && widget.onSync != null) {
      final realtimeReady = await _startRealtime();
      if (!realtimeReady && mounted && widget.onSync != null) {
        _schedulePoll(_normalPollInterval);
      }
    }
  }

  Future<bool> _startRealtime() async {
    final starter = widget.onStartRealtime;
    if (!_appResumed || starter == null) {
      return false;
    }
    if (_stopRealtime != null || _realtimeConnecting) {
      return _stopRealtime != null;
    }
    _realtimeConnecting = true;
    try {
      final stop = await starter(
        onNotice: () => _syncAndReload(silent: true),
        onDisconnected: () async {
          _stopRealtime = null;
          if (_appResumed && mounted && widget.onSync != null) {
            _schedulePoll(_backoffPollInterval);
          }
        },
      );
      if (!mounted || !_appResumed) {
        await stop?.call();
        return false;
      }
      _stopRealtime = stop;
      if (stop != null) {
        // 实时已连也保留低频心跳兜底，防止推送静默丢失导致收不到新消息。
        _schedulePoll(_heartbeatPollInterval);
      }
      return stop != null;
    } catch (_) {
      return false;
    } finally {
      _realtimeConnecting = false;
    }
  }

  Future<bool> _syncAndReload({required bool silent}) async {
    final sync = widget.onSync;
    if (sync == null) {
      if (!silent && mounted) {
        setState(() {
          _error = '当前会话尚未绑定同步链路';
        });
      }
      return false;
    }
    try {
      await sync();
      await _reloadMessages();
      return true;
    } catch (error) {
      if (!silent && mounted) {
        setState(() {
          _error = error.toString();
        });
      }
      return false;
    }
  }

  void _schedulePoll(Duration delay) {
    if (!_appResumed) {
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
    if (stop != null) {
      unawaited(stop());
    }
  }

  Future<void> _runPoll() async {
    if (!mounted || !_appResumed || widget.onSync == null) {
      return;
    }
    if (_polling) {
      _schedulePoll(_backoffPollInterval);
      return;
    }
    _polling = true;
    final ok = await _syncAndReload(silent: true);
    _polling = false;
    if (!mounted || !_appResumed || widget.onSync == null) {
      return;
    }
    // 实时在线：保留低频心跳兜底，按心跳间隔继续复查。
    if (_stopRealtime != null) {
      _schedulePoll(_heartbeatPollInterval);
      return;
    }
    // 实时离线：尝试重连；重连成功由 _startRealtime 起心跳，否则常规/退避轮询。
    if (ok && await _startRealtime()) {
      return;
    }
    _schedulePoll(ok ? _normalPollInterval : _backoffPollInterval);
  }

  Future<void> _handleSend(String text) async {
    final normalized = text.trim();
    if (normalized.isEmpty) {
      return;
    }
    final sender = widget.onSendText;
    if (sender == null) {
      setState(() {
        _error = '当前会话尚未绑定发送链路';
      });
      return;
    }
    try {
      await sender(normalized);
      await _reloadMessages();
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    }
  }

  Future<void> _handleMediaTap() async {
    final sender = widget.onSendMedia;
    if (sender == null) {
      setState(() {
        _error = '当前会话尚未绑定媒体发送链路';
      });
      return;
    }
    setState(() {
      _attachmentBusy = true;
      _error = null;
    });
    try {
      final draft = await (widget.pickMedia?.call() ?? _pickViaSheet());
      if (draft == null) {
        return;
      }
      await sender(draft);
      await _reloadMessages();
    } on ChatMediaTooLargeException {
      if (mounted) {
        setState(() {
          _error = '文件超出大小上限：图片最大 100MB，视频/文件最大 5GB';
        });
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _attachmentBusy = false;
        });
      }
    }
  }

  /// 发送贴纸:只把 `(packId, stickerId)` 交给发送链路(走 MLS 信封瞬时中转,
  /// 零字节、零 WebRTC)。面板保持打开以便连发,不自动关闭。
  Future<void> _handleSendSticker(String packId, String stickerId) async {
    final sender = widget.onSendSticker;
    if (sender == null) {
      setState(() {
        _error = '当前会话尚未绑定发送链路';
      });
      return;
    }
    try {
      await sender(packId, stickerId);
      await _reloadMessages();
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    }
  }

  /// 表情/贴纸面板互斥切换:同一个开则关,不同则切换;打开任一都收起键盘腾空间。
  void _togglePanel(_ComposerPanel panel) {
    setState(() {
      _openPanel = _openPanel == panel ? _ComposerPanel.none : panel;
    });
    if (_openPanel != _ComposerPanel.none) {
      FocusScope.of(context).unfocus();
    }
  }

  /// 加号 → 弹来源选择 → 采集 → 压缩门控 → 探测 → 组装路径型 [ChatMediaDraft]。
  Future<ChatMediaDraft?> _pickViaSheet() async {
    final source = await showChatMediaSourceSheet(context);
    if (source == null || !mounted) {
      return null;
    }
    final picked = await _pickFromSource(source);
    if (picked == null) {
      return null;
    }
    // 压缩门控:图超限压一次仍超则抛;视频/文件超限抛(采集侧,门①上游)。
    final finalPath = await _mediaCompressor.ensureWithinLimit(
      path: picked.path,
      kind: picked.kind,
    );
    final probe = await _mediaProbe.probe(path: finalPath, kind: picked.kind);
    final byteSize = await File(finalPath).length();
    return ChatMediaDraft(
      kind: picked.kind,
      fileName: picked.fileName,
      contentType: picked.mime,
      sourcePath: finalPath,
      byteSize: byteSize,
      width: probe.width,
      height: probe.height,
      durationMs: probe.durationMs,
      blurhash: probe.blurhash,
    );
  }

  Future<PickedMediaFile?> _pickFromSource(ChatMediaSource source) {
    return switch (source) {
      ChatMediaSource.galleryImage => _mediaPicker.galleryImage(),
      ChatMediaSource.cameraPhoto => _mediaPicker.cameraPhoto(),
      ChatMediaSource.galleryVideo => _mediaPicker.galleryVideo(),
      ChatMediaSource.cameraVideo => _mediaPicker.cameraVideo(),
      ChatMediaSource.file => _pickFileViaFilePicker(),
    };
  }

  /// 通用文件(非图非视频)走 file_picker,取路径不载入字节。
  Future<PickedMediaFile?> _pickFileViaFilePicker() async {
    final result = await FilePicker.platform.pickFiles(
      allowMultiple: false,
      withData: false,
    );
    if (result == null || result.files.isEmpty) {
      return null;
    }
    final file = result.files.single;
    final path = file.path;
    if (path == null) {
      throw StateError('无法读取所选文件');
    }
    final mime = mimeFromFileName(file.name);
    return PickedMediaFile(
      path: path,
      fileName: file.name,
      mime: mime,
      kind: mediaKindFromMime(mime),
    );
  }

  /// 把已收到的媒体从本机缓存另存并提示。控制载荷来自消息 metadata。
  Future<void> _downloadMedia(String controlPlaintext) async {
    if (controlPlaintext.isEmpty) {
      setState(() {
        _error = '媒体控制消息为空，无法保存';
      });
      return;
    }
    final downloader = widget.onDownloadAttachment;
    if (downloader == null) {
      setState(() {
        _error = '当前会话尚未绑定媒体下载链路';
      });
      return;
    }
    setState(() {
      _attachmentBusy = true;
      _error = null;
    });
    try {
      final downloaded = await downloader(
        widget.conversationId,
        controlPlaintext,
      );
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('已保存：${downloaded.fileName}')),
        );
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _attachmentBusy = false;
        });
      }
    }
  }

  Future<void> _handleSync() async {
    final sync = widget.onSync;
    if (sync == null) {
      setState(() {
        _error = '当前会话尚未绑定同步链路';
      });
      return;
    }
    setState(() {
      _syncing = true;
      _error = null;
    });
    try {
      await _syncAndReload(silent: false);
    } finally {
      if (mounted) {
        setState(() {
          _syncing = false;
        });
      }
    }
  }

  Future<void> _handleDeleteConversation() async {
    final confirmed = await _confirmDeleteConversation(context);
    if (!confirmed || !mounted) {
      return;
    }
    setState(() {
      _deleting = true;
      _error = null;
    });
    try {
      _pauseSync();
      final deleter = widget.onDeleteConversation ??
          () => widget.store.deleteConversation(widget.conversationId);
      await deleter();
      if (!mounted) {
        return;
      }
      if (Navigator.of(context).canPop()) {
        Navigator.of(context).pop(true);
      } else {
        await _chatController.setMessages(const [], animated: false);
        setState(() {
          _deleting = false;
        });
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _deleting = false;
          _error = error.toString();
        });
      }
    }
  }

  void _openPeerProfile() {
    Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) => UserProfilePage(
          ownerAccount: widget.peerUserId,
          isSelf: false,
        ),
      ),
    );
  }

  // 群聊文本:入站消息在气泡上方显示发送者名(连续同发送者只在首条显示)。
  // 复用 flyer 默认气泡 SimpleTextMessage 的 topWidget 挂 Username(经 resolveUser 解析)。
  Widget _buildGroupTextMessage(
    BuildContext context,
    TextMessage message,
    int index, {
    required bool isSentByMe,
    MessageGroupStatus? groupStatus,
  }) {
    final showSender = !isSentByMe && (groupStatus?.isFirst ?? true);
    return SimpleTextMessage(
      message: message,
      index: index,
      topWidget: showSender ? Username(userId: message.authorId) : null,
    );
  }

  // 图片消息:blurhash 占位(字节未到)→ 本机图(点开全屏可缩放/存相册)。
  // 内联图按显示宽度 cacheWidth 降采样解码,100MB 图也不整解码。
  Widget _buildImageMessage(
    BuildContext context,
    ImageMessage message,
    int index, {
    required bool isSentByMe,
    MessageGroupStatus? groupStatus,
  }) {
    final maxWidth = MediaQuery.of(context).size.width * 0.62;
    final hasFile = message.source.isNotEmpty;
    final ratio = _mediaAspectRatio(message.width, message.height);
    final cacheWidth =
        (maxWidth * MediaQuery.of(context).devicePixelRatio).round();
    final Widget content = hasFile
        ? GestureDetector(
            onTap: () => _openImageViewer(message),
            child: Image.file(
              File(message.source),
              fit: BoxFit.cover,
              cacheWidth: cacheWidth,
              errorBuilder: (_, __, ___) => _mediaPlaceholder(
                icon: Icons.broken_image_rounded,
                label: '图片无法显示',
              ),
            ),
          )
        : _blurhashOrPlaceholder(message.blurhash, '接收中…');
    return _mediaAligned(
      isSentByMe,
      ClipRRect(
        borderRadius: BorderRadius.circular(14),
        child: SizedBox(
          width: maxWidth,
          child: AspectRatio(aspectRatio: ratio, child: content),
        ),
      ),
      senderId: message.authorId,
      groupStatus: groupStatus,
    );
  }

  // 视频消息:blurhash 封面 + 播放图标;字节就绪点开播放页(可存相册)。
  Widget _buildVideoMessage(
    BuildContext context,
    VideoMessage message,
    int index, {
    required bool isSentByMe,
    MessageGroupStatus? groupStatus,
  }) {
    final maxWidth = MediaQuery.of(context).size.width * 0.62;
    final hasFile = message.source.isNotEmpty;
    final hash = message.metadata?['blurhash']?.toString();
    final ratio = _mediaAspectRatio(message.width, message.height);
    return _mediaAligned(
      isSentByMe,
      GestureDetector(
        onTap: hasFile ? () => _openVideoPlayer(message) : null,
        child: ClipRRect(
          borderRadius: BorderRadius.circular(14),
          child: SizedBox(
            width: maxWidth,
            child: AspectRatio(
              aspectRatio: ratio,
              child: Stack(
                fit: StackFit.expand,
                children: [
                  (hash != null && hash.isNotEmpty)
                      ? BlurHash(hash: hash, imageFit: BoxFit.cover)
                      : Container(color: AppTheme.surfaceCard),
                  const Center(
                    child: Icon(
                      Icons.play_circle_fill_rounded,
                      size: 44,
                      color: Colors.white70,
                    ),
                  ),
                  if (!hasFile)
                    const Positioned(
                      left: 0,
                      right: 0,
                      bottom: 8,
                      child: Text(
                        '接收中…',
                        textAlign: TextAlign.center,
                        style: TextStyle(fontSize: 12, color: Colors.white),
                      ),
                    ),
                ],
              ),
            ),
          ),
        ),
      ),
      senderId: message.authorId,
      groupStatus: groupStatus,
    );
  }

  double _mediaAspectRatio(double? width, double? height) {
    if (width != null && height != null && width > 0 && height > 0) {
      return (width / height).clamp(0.6, 1.9);
    }
    return 1.0;
  }

  Widget _blurhashOrPlaceholder(String? hash, String label) {
    if (hash != null && hash.isNotEmpty) {
      return Stack(
        fit: StackFit.expand,
        children: [
          BlurHash(hash: hash, imageFit: BoxFit.cover),
          Positioned(
            left: 0,
            right: 0,
            bottom: 8,
            child: Text(
              label,
              textAlign: TextAlign.center,
              style: const TextStyle(fontSize: 12, color: Colors.white),
            ),
          ),
        ],
      );
    }
    return _mediaPlaceholder(icon: Icons.image_rounded, label: label);
  }

  void _openImageViewer(ImageMessage message) {
    final fileName = message.metadata?['file_name']?.toString() ?? '图片';
    Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) => ImageViewerPage(
          filePath: message.source,
          fileName: fileName,
        ),
      ),
    );
  }

  void _openVideoPlayer(VideoMessage message) {
    final fileName =
        message.metadata?['file_name']?.toString() ?? message.name ?? '视频';
    Navigator.of(context).push<void>(
      MaterialPageRoute<void>(
        builder: (_) => VideoPlayerPage(
          filePath: message.source,
          fileName: fileName,
        ),
      ),
    );
  }

  // 文件消息:文件条 + 点按另存。
  Widget _buildFileMessage(
    BuildContext context,
    FileMessage message,
    int index, {
    required bool isSentByMe,
    MessageGroupStatus? groupStatus,
  }) {
    final control =
        message.metadata?['attachment_control_plaintext']?.toString() ?? '';
    return _mediaAligned(
      isSentByMe,
      GestureDetector(
        onTap: () => unawaited(_downloadMedia(control)),
        child: Container(
          constraints: BoxConstraints(
            maxWidth: MediaQuery.of(context).size.width * 0.7,
          ),
          padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
          decoration: BoxDecoration(
            color: AppTheme.surfaceCard,
            borderRadius: BorderRadius.circular(14),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(
                Icons.insert_drive_file_rounded,
                size: 28,
                color: AppTheme.textSecondary,
              ),
              const SizedBox(width: 10),
              Flexible(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      message.name,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    if (message.size != null)
                      Text(
                        _formatByteSize(message.size!),
                        style: const TextStyle(
                          fontSize: 11,
                          color: AppTheme.textSecondary,
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
      senderId: message.authorId,
      groupStatus: groupStatus,
    );
  }

  Widget _mediaAligned(
    bool isSentByMe,
    Widget child, {
    String? senderId,
    MessageGroupStatus? groupStatus,
  }) {
    final aligned = Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      child: Align(
        alignment: isSentByMe ? Alignment.centerRight : Alignment.centerLeft,
        child: child,
      ),
    );
    // 群聊入站媒体/贴纸在气泡上方显示发送者名(连续同发送者只首条),与文本一致。
    final showSender = widget.isGroup &&
        !isSentByMe &&
        senderId != null &&
        (groupStatus?.isFirst ?? true);
    if (!showSender) return aligned;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.only(left: 16, top: 4),
          child: Username(
            userId: senderId,
            style: const TextStyle(
              fontSize: 11,
              color: AppTheme.textSecondary,
            ),
          ),
        ),
        aligned,
      ],
    );
  }

  Widget _mediaPlaceholder({required IconData icon, required String label}) {
    return Container(
      color: AppTheme.surfaceCard,
      alignment: Alignment.center,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, color: AppTheme.textSecondary, size: 28),
          const SizedBox(height: 6),
          Text(
            label,
            style: const TextStyle(
              fontSize: 12,
              color: AppTheme.textSecondary,
            ),
          ),
        ],
      ),
    );
  }

  static const double _stickerRenderSize = 128;

  /// 贴纸消息:按 `(packId, stickerId)` 渲染内置 Fluent 3D PNG,无气泡大图。
  /// id 未内置(对端资产旧/缺)或解码失败时降级为占位,绝不崩。
  Widget _buildStickerMessage(
    BuildContext context,
    CustomMessage message,
    int index, {
    required bool isSentByMe,
    MessageGroupStatus? groupStatus,
  }) {
    final packId = message.metadata?['pack_id']?.toString() ?? '';
    final stickerId = message.metadata?['sticker_id']?.toString() ?? '';
    final known = StickerPack.isKnown(packId: packId, stickerId: stickerId);
    final Widget content = known
        ? Image.asset(
            StickerPack.assetPath(stickerId),
            fit: BoxFit.contain,
            errorBuilder: (_, __, ___) => _stickerFallback(),
          )
        : _stickerFallback();
    return _mediaAligned(
      isSentByMe,
      SizedBox(
        width: _stickerRenderSize,
        height: _stickerRenderSize,
        child: content,
      ),
      senderId: message.authorId,
      groupStatus: groupStatus,
    );
  }

  Widget _stickerFallback() => _mediaPlaceholder(
        icon: Icons.emoji_emotions_outlined,
        label: '[贴纸]',
      );

  /// 自绘 composer:复用现成 [Composer](发送/附件仍走 Chat 注入的回调),topWidget
  /// 挂表情/贴纸开关工具条与两个互斥面板。
  Widget _buildComposer(BuildContext context) {
    return Composer(
      textEditingController: _composerController,
      topWidget: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (_openPanel == _ComposerPanel.emoji) _buildEmojiPanel(context),
          if (_openPanel == _ComposerPanel.sticker)
            StickerPanel(
              onPick: (packId, stickerId) =>
                  unawaited(_handleSendSticker(packId, stickerId)),
            ),
          _composerToolbar(),
        ],
      ),
    );
  }

  /// 表情面板:Unicode emoji 由 [EmojiPicker] 直接插入 `_composerController` 光标处,
  /// 随文本走 `sendText`——纯客户端、零协议变更。
  Widget _buildEmojiPanel(BuildContext context) {
    final height = math.min(264.0, MediaQuery.sizeOf(context).height * 0.4);
    return SizedBox(
      height: height,
      child: EmojiPicker(
        textEditingController: _composerController,
        config: Config(
          height: height,
          checkPlatformCompatibility: true,
          emojiViewConfig: const EmojiViewConfig(
            columns: 8,
            backgroundColor: AppTheme.surfaceCard,
          ),
          categoryViewConfig: const CategoryViewConfig(
            backgroundColor: AppTheme.surfaceCard,
            indicatorColor: AppTheme.accent,
            iconColorSelected: AppTheme.accent,
            backspaceColor: AppTheme.accent,
          ),
          bottomActionBarConfig: const BottomActionBarConfig(
            backgroundColor: AppTheme.surfaceCard,
            buttonColor: AppTheme.surfaceElevated,
            buttonIconColor: AppTheme.textSecondary,
            showSearchViewButton: false,
          ),
        ),
      ),
    );
  }

  Widget _composerToolbar() {
    return Align(
      alignment: Alignment.centerLeft,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          IconButton(
            key: const ValueKey('chat-emoji-toggle'),
            tooltip: '表情',
            onPressed: () => _togglePanel(_ComposerPanel.emoji),
            color: _openPanel == _ComposerPanel.emoji
                ? AppTheme.accent
                : AppTheme.textSecondary,
            icon: const Icon(Icons.emoji_emotions_outlined),
          ),
          IconButton(
            key: const ValueKey('chat-sticker-toggle'),
            tooltip: '贴纸',
            onPressed: () => _togglePanel(_ComposerPanel.sticker),
            color: _openPanel == _ComposerPanel.sticker
                ? AppTheme.accent
                : AppTheme.textSecondary,
            icon: const Icon(Icons.sticky_note_2_outlined),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final peerName = ProfilePresentation.forAccount(widget.peerUserId)
        .resolveDisplayName(publicName: widget.title);
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        backgroundColor: AppTheme.surfaceCard,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
        titleSpacing: 0,
        title: InkWell(
          key: const ValueKey('chat-peer-profile-entry'),
          borderRadius: BorderRadius.circular(10),
          onTap: _openPeerProfile,
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              ProfileAvatar(seed: widget.peerUserId, size: 36),
              const SizedBox(width: 9),
              Flexible(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      peerName,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: const TextStyle(
                        fontSize: 17,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    Text(
                      _shortAccount(widget.peerUserId),
                      style: const TextStyle(
                        fontSize: 12,
                        color: AppTheme.textSecondary,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        actions: [
          IconButton(
            tooltip: '同步',
            onPressed: _syncing || _deleting ? null : _handleSync,
            icon: _syncing
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.sync_rounded),
          ),
          PopupMenuButton<_ChatMenuAction>(
            tooltip: '更多',
            icon: const Icon(Icons.more_vert_rounded),
            enabled: !_deleting,
            onSelected: (action) {
              switch (action) {
                case _ChatMenuAction.deleteConversation:
                  unawaited(_handleDeleteConversation());
              }
            },
            itemBuilder: (context) => const [
              PopupMenuItem(
                value: _ChatMenuAction.deleteConversation,
                child: Row(
                  children: [
                    Icon(Icons.delete_outline_rounded, size: 18),
                    SizedBox(width: 10),
                    Text('删除聊天记录'),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
      body: Column(
        children: [
          if (_attachmentBusy || _deleting)
            const LinearProgressIndicator(minHeight: 2),
          if (_error != null)
            Container(
              width: double.infinity,
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
              color: Colors.red.withAlpha(20),
              child: Text(
                _error!,
                style: const TextStyle(color: Colors.red, fontSize: 12),
              ),
            ),
          Expanded(
            child: _loading
                ? const Center(child: CircularProgressIndicator())
                : Chat(
                    currentUserId: widget.ownerAccount,
                    chatController: _chatController,
                    onMessageSend: _handleSend,
                    onAttachmentTap: _handleMediaTap,
                    backgroundColor: AppTheme.scaffoldBg,
                    builders: Builders(
                      textMessageBuilder:
                          widget.isGroup ? _buildGroupTextMessage : null,
                      imageMessageBuilder: _buildImageMessage,
                      videoMessageBuilder: _buildVideoMessage,
                      fileMessageBuilder: _buildFileMessage,
                      customMessageBuilder: _buildStickerMessage,
                      composerBuilder: _buildComposer,
                    ),
                    resolveUser: (id) async {
                      final isMe = id == widget.ownerAccount;
                      return User(
                        id: id,
                        name: isMe
                            ? '我'
                            : widget.isGroup
                                ? ProfilePresentation.forAccount(id).fallbackName
                                : peerName,
                      );
                    },
                  ),
          ),
        ],
      ),
    );
  }
}

String _formatByteSize(int bytes) {
  if (bytes < 1024) return '$bytes B';
  if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
  return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
}

enum _ChatMenuAction { deleteConversation }

/// composer 上方可切换的面板;表情与贴纸互斥,none 表示都收起。
enum _ComposerPanel { none, emoji, sticker }

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

class _ChatLifecycleObserver extends WidgetsBindingObserver {
  _ChatLifecycleObserver({
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

String _shortAccount(String value) {
  if (value.length <= 16) {
    return value;
  }
  return '${value.substring(0, 8)}...${value.substring(value.length - 6)}';
}
