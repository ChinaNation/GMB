import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';
import 'package:path/path.dart' as path;

import 'package:citizenapp/8964/chain/square_chain_service.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_publish_service.dart';
import 'package:citizenapp/8964/storage/square_draft_store.dart';
import 'package:citizenapp/qr/pages/qr_sign_session_page.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/signer/qr_signer.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

class SquareComposePage extends StatefulWidget {
  const SquareComposePage({
    super.key,
    this.identityService = const SquareIdentityService(),
    this.publishService,
    this.draftStore,
  });

  final SquareIdentityService identityService;
  final SquarePublishService? publishService;
  final SquareDraftRepository? draftStore;

  @override
  State<SquareComposePage> createState() => _SquareComposePageState();
}

class _SquareComposePageState extends State<SquareComposePage> {
  final TextEditingController _textController = TextEditingController();
  late Future<SquareIdentityState> _identityFuture;
  late final SquarePublishService _publishService;
  late final SquareDraftRepository _draftStore;
  final ImagePicker _imagePicker = ImagePicker();
  final List<SquareLocalMediaDraft> _mediaDrafts = [];
  SquarePostCategory _category = SquarePostCategory.normal;
  SquarePublishStage _publishStage = SquarePublishStage.idle;
  bool _publishing = false;
  bool _draftRestored = false;

  @override
  void initState() {
    super.initState();
    _publishService = widget.publishService ?? SquarePublishService();
    _draftStore = widget.draftStore ?? SquareDraftStore.instance;
    _identityFuture = widget.identityService.loadCurrent();
    _identityFuture
        .then(_restoreDraftForIdentity)
        .catchError((Object e) => debugPrint('[SquareComposePage] 身份加载失败: $e'));
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  void _selectCategory(
    Set<SquarePostCategory> values,
    SquareIdentityState identity,
  ) {
    final next = values.first;
    if (next == SquarePostCategory.campaign && !identity.isCertified) {
      return;
    }
    setState(() => _category = next);
  }

  Future<void> _pickImages() async {
    final images = await _imagePicker.pickMultiImage();
    if (images.isEmpty || !mounted) return;
    final next = <SquareLocalMediaDraft>[];
    for (final image in images) {
      next.add(await _draftFromXFile(image, SquareMediaKind.image));
    }
    setState(() {
      final capacity = 9 - _mediaDrafts.length;
      _mediaDrafts.addAll(next.take(capacity));
    });
  }

  Future<void> _pickVideo() async {
    final video = await _imagePicker.pickVideo(source: ImageSource.gallery);
    if (video == null || !mounted) return;
    final draft = await _draftFromXFile(video, SquareMediaKind.video);
    setState(() {
      if (_mediaDrafts.length < 9) _mediaDrafts.add(draft);
    });
  }

  Future<SquareLocalMediaDraft> _draftFromXFile(
    XFile file,
    SquareMediaKind mediaKind,
  ) async {
    final name = file.name.isNotEmpty ? file.name : path.basename(file.path);
    final contentType = file.mimeType ?? _inferContentType(name, mediaKind);
    return SquareLocalMediaDraft(
      mediaKind: mediaKind,
      path: file.path,
      fileName: name,
      contentType: contentType,
      byteSize: await file.length(),
    );
  }

  String _inferContentType(String fileName, SquareMediaKind mediaKind) {
    final ext = path.extension(fileName).toLowerCase();
    if (mediaKind == SquareMediaKind.video) {
      return ext == '.webm' ? 'video/webm' : 'video/mp4';
    }
    return switch (ext) {
      '.png' => 'image/png',
      '.webp' => 'image/webp',
      _ => 'image/jpeg',
    };
  }

  void _removeMedia(int index) {
    setState(() => _mediaDrafts.removeAt(index));
  }

  Future<void> _restoreDraftForIdentity(SquareIdentityState identity) async {
    if (_draftRestored || !identity.hasWallet) return;
    _draftRestored = true;
    try {
      final draft = await _draftStore.read(identity.ownerAccount);
      if (!mounted || draft == null) return;
      setState(() {
        if (_textController.text.trim().isEmpty) {
          _textController.text = draft.text;
        }
        _mediaDrafts
          ..clear()
          ..addAll(draft.mediaDrafts);
        _category = draft.postCategory == SquarePostCategory.campaign &&
                !identity.isCertified
            ? SquarePostCategory.normal
            : draft.postCategory;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('已恢复上次未完成草稿')),
      );
    } catch (e) {
      debugPrint('[SquareComposePage] 恢复广场草稿失败: $e');
    }
  }

  Future<void> _submit(SquareIdentityState identity) async {
    if (_publishing) return;
    if (!identity.hasWallet ||
        identity.walletIndex == null ||
        identity.pubkeyHex == null) {
      _showError('请先创建或选择钱包');
      return;
    }
    if (_category == SquarePostCategory.campaign && !identity.isCertified) {
      _showError('当前钱包未认证，不能发布竞选内容');
      return;
    }
    if (_mediaDrafts.isEmpty) {
      _showError('请至少选择一张图片或一个视频');
      return;
    }

    setState(() {
      _publishing = true;
      _publishStage = SquarePublishStage.signingIn;
    });

    WalletManager? hotWalletManager;
    try {
      if (identity.isHotWallet) {
        hotWalletManager = WalletManager();
        await hotWalletManager.authenticateForSigning();
      }

      final result = await _publishService.publish(
        identity: identity,
        postCategory: _category,
        text: _textController.text,
        mediaDrafts: List<SquareLocalMediaDraft>.unmodifiable(_mediaDrafts),
        signLoginPayload: (payload) =>
            _signLoginPayload(identity, hotWalletManager, payload),
        signChainPayload: (payload) =>
            _signChainPayload(identity, hotWalletManager, payload),
        onStage: (stage) {
          if (!mounted) return;
          setState(() => _publishStage = stage);
        },
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('动态已入块：${_truncate(result.txHash)}'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );
      Navigator.of(context).pop(result.post);
    } catch (e) {
      if (!mounted) return;
      _showError('发布失败：$e');
    } finally {
      if (mounted) {
        setState(() {
          _publishing = false;
          _publishStage = SquarePublishStage.idle;
        });
      }
    }
  }

  Future<String> _signLoginPayload(
    SquareIdentityState identity,
    WalletManager? hotWalletManager,
    String signingPayload,
  ) async {
    final signature = await _signPayload(
      identity: identity,
      hotWalletManager: hotWalletManager,
      payload: Uint8List.fromList(utf8.encode(signingPayload)),
      action: QrActions.login,
      requestPrefix: 'square-login-',
    );
    return '0x${_hexEncode(signature)}';
  }

  Future<Uint8List> _signChainPayload(
    SquareIdentityState identity,
    WalletManager? hotWalletManager,
    Uint8List payload,
  ) {
    return _signPayload(
      identity: identity,
      hotWalletManager: hotWalletManager,
      payload: payload,
      action: QrActions.chain(
        SquareChainService.palletIndex,
        SquareChainService.publishSquarePostCallIndex,
      ),
      requestPrefix: 'square-post-',
    );
  }

  Future<Uint8List> _signPayload({
    required SquareIdentityState identity,
    required WalletManager? hotWalletManager,
    required Uint8List payload,
    required int action,
    required String requestPrefix,
  }) async {
    final walletIndex = identity.walletIndex;
    final pubkeyHex = identity.pubkeyHex;
    if (walletIndex == null || pubkeyHex == null) {
      throw const SquarePublishException('当前钱包信息不完整');
    }
    if (hotWalletManager != null) {
      return hotWalletManager.signWithWalletNoAuth(walletIndex, payload);
    }

    final qrSigner = QrSigner();
    final request = qrSigner.buildRequest(
      requestId: QrSigner.generateRequestId(prefix: requestPrefix),
      pubkey: '0x$pubkeyHex',
      payloadHex: '0x${_hexEncode(payload)}',
      action: action,
    );
    final requestJson = qrSigner.encodeRequest(request);
    if (!mounted) throw const SquarePublishException('页面已关闭');
    final response = await Navigator.push<SignResponseEnvelope>(
      context,
      MaterialPageRoute(
        builder: (_) => QrSignSessionPage(
          request: request,
          requestJson: requestJson,
          expectedPubkey: '0x$pubkeyHex',
        ),
      ),
    );
    if (response == null) throw const SquarePublishException('签名已取消');
    return Uint8List.fromList(_hexDecode(response.body.signatureHex));
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), backgroundColor: AppTheme.danger),
    );
  }

  String _truncate(String value) {
    if (value.length <= 18) return value;
    return '${value.substring(0, 10)}...${value.substring(value.length - 6)}';
  }

  String _hexEncode(List<int> bytes) =>
      bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();

  List<int> _hexDecode(String input) {
    final text = input.startsWith('0x') ? input.substring(2) : input;
    final out = <int>[];
    for (var i = 0; i < text.length; i += 2) {
      out.add(int.parse(text.substring(i, i + 2), radix: 16));
    }
    return out;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('发布动态')),
      body: FutureBuilder<SquareIdentityState>(
        future: _identityFuture,
        builder: (context, snapshot) {
          final identity =
              snapshot.data ?? const SquareIdentityState(ownerAccount: '');
          if (_category == SquarePostCategory.campaign &&
              !identity.isCertified) {
            _category = SquarePostCategory.normal;
          }

          return SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _IdentityBanner(identity: identity),
                const SizedBox(height: 14),
                SegmentedButton<SquarePostCategory>(
                  showSelectedIcon: false,
                  segments: [
                    const ButtonSegment<SquarePostCategory>(
                      value: SquarePostCategory.normal,
                      label: Text('普通'),
                      icon: Icon(Icons.article_outlined),
                    ),
                    ButtonSegment<SquarePostCategory>(
                      value: SquarePostCategory.campaign,
                      enabled: identity.isCertified,
                      label: const Text('竞选'),
                      icon: const Icon(Icons.campaign_outlined),
                    ),
                  ],
                  selected: {_category},
                  onSelectionChanged: (values) =>
                      _selectCategory(values, identity),
                ),
                if (!identity.isCertified) ...[
                  const SizedBox(height: 10),
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(12),
                    decoration: AppTheme.bannerDecoration(AppTheme.warning),
                    child: const Text(
                      '当前钱包未认证，不能发布竞选内容。',
                      style: TextStyle(
                        color: AppTheme.textPrimary,
                        fontSize: 13,
                        height: 1.35,
                      ),
                    ),
                  ),
                ],
                const SizedBox(height: 16),
                TextField(
                  controller: _textController,
                  minLines: 6,
                  maxLines: 10,
                  maxLength: 500,
                  decoration: const InputDecoration(
                    hintText: '写下你的动态',
                  ),
                ),
                const SizedBox(height: 8),
                Row(
                  children: [
                    _MediaActionButton(
                      icon: Icons.image_outlined,
                      label: '图片',
                      onPressed: _publishing ? null : _pickImages,
                    ),
                    const SizedBox(width: 10),
                    _MediaActionButton(
                      icon: Icons.videocam_outlined,
                      label: '视频',
                      onPressed: _publishing ? null : _pickVideo,
                    ),
                  ],
                ),
                if (_mediaDrafts.isNotEmpty) ...[
                  const SizedBox(height: 12),
                  _SelectedMediaList(
                    mediaDrafts: _mediaDrafts,
                    onRemove: _publishing ? null : _removeMedia,
                  ),
                ],
                const SizedBox(height: 22),
                FilledButton.icon(
                  onPressed: identity.hasWallet && !_publishing
                      ? () => _submit(identity)
                      : null,
                  icon: _publishing
                      ? const SizedBox(
                          width: 18,
                          height: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.publish_rounded),
                  label: Text(_publishing ? _publishStage.label : '发布'),
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _IdentityBanner extends StatelessWidget {
  const _IdentityBanner({required this.identity});

  final SquareIdentityState identity;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: AppTheme.surfaceWhite,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        border: Border.all(color: AppTheme.border),
      ),
      child: Row(
        children: [
          Icon(
            identity.isCertified
                ? Icons.verified_user_rounded
                : Icons.account_circle_outlined,
            color:
                identity.isCertified ? AppTheme.primary : AppTheme.textTertiary,
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  identity.walletName ?? '当前钱包',
                  style: const TextStyle(
                    color: AppTheme.textPrimary,
                    fontSize: 14,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  identity.isCertified
                      ? 'CID ${identity.cidNumber}'
                      : identity.accountLabel,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: AppTheme.textSecondary,
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _MediaActionButton extends StatelessWidget {
  const _MediaActionButton({
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  final IconData icon;
  final String label;
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: OutlinedButton.icon(
        onPressed: onPressed,
        icon: Icon(icon),
        label: Text(label),
      ),
    );
  }
}

class _SelectedMediaList extends StatelessWidget {
  const _SelectedMediaList({
    required this.mediaDrafts,
    required this.onRemove,
  });

  final List<SquareLocalMediaDraft> mediaDrafts;
  final ValueChanged<int>? onRemove;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        for (var i = 0; i < mediaDrafts.length; i++)
          Padding(
            padding:
                EdgeInsets.only(bottom: i == mediaDrafts.length - 1 ? 0 : 8),
            child: _SelectedMediaTile(
              draft: mediaDrafts[i],
              onRemove: onRemove == null ? null : () => onRemove!(i),
            ),
          ),
      ],
    );
  }
}

class _SelectedMediaTile extends StatelessWidget {
  const _SelectedMediaTile({
    required this.draft,
    required this.onRemove,
  });

  final SquareLocalMediaDraft draft;
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    final isVideo = draft.mediaKind == SquareMediaKind.video;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: AppTheme.surfaceWhite,
        borderRadius: BorderRadius.circular(AppTheme.radiusMd),
        border: Border.all(color: AppTheme.border),
      ),
      child: Row(
        children: [
          Icon(
            isVideo ? Icons.videocam_outlined : Icons.image_outlined,
            color: AppTheme.primary,
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  draft.fileName,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: AppTheme.textPrimary,
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 2),
                Text(
                  '${draft.mediaKind.label} · ${_formatBytes(draft.byteSize)}',
                  style: const TextStyle(
                    color: AppTheme.textSecondary,
                    fontSize: 12,
                  ),
                ),
              ],
            ),
          ),
          IconButton(
            tooltip: '移除',
            onPressed: onRemove,
            icon: const Icon(Icons.close_rounded),
          ),
        ],
      ),
    );
  }

  String _formatBytes(int bytes) {
    if (bytes >= 1024 * 1024) {
      return '${(bytes / 1024 / 1024).toStringAsFixed(1)} MB';
    }
    return '${(bytes / 1024).ceil()} KB';
  }
}
