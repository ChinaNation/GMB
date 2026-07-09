import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_compose_signers.dart';
import 'package:citizenapp/8964/services/square_media_draft.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_publish_service.dart';
import 'package:citizenapp/ui/app_theme.dart';

const int articleTitleMin = 10;
const int articleTitleMax = 50;
const int articleBodyMax = 30000;
const int articleBodyImagesMax = 100;

/// 文章发布校验（纯函数，便于单测）。返回错误文案，null 表示通过。
String? articleValidationError({
  required String title,
  required bool hasCover,
  required String body,
}) {
  final trimmedTitle = title.trim();
  if (trimmedTitle.length < articleTitleMin ||
      trimmedTitle.length > articleTitleMax) {
    return '标题需 $articleTitleMin–$articleTitleMax 字';
  }
  if (!hasCover) {
    return '请选择 1 张首图';
  }
  final trimmedBody = body.trim();
  if (trimmedBody.isEmpty) {
    return '正文不能为空';
  }
  if (trimmedBody.length > articleBodyMax) {
    return '正文不能超过 $articleBodyMax 字';
  }
  return null;
}

/// 文章长文发布页：标题（10-50）+ 首图（必填 1 张）+ 正文（≤30000）+ 正文图（≤100）。
///
/// 链上仍只发 normal/campaign；manifest 标 content_format=article + title。
/// media_items[0]=首图，[1..]=正文图。发布服务会按会员等级再次校验真实额度。
class SquareArticleComposePage extends StatefulWidget {
  const SquareArticleComposePage({
    super.key,
    this.identityService = const SquareIdentityService(),
    this.publishService,
    this.initialTitle,
    this.initialBody,
    this.initialCategory,
    this.replacePostId,
  });

  final SquareIdentityService identityService;
  final SquarePublishService? publishService;
  final String? initialTitle;
  final String? initialBody;
  final SquarePostCategory? initialCategory;
  final String? replacePostId;

  @override
  State<SquareArticleComposePage> createState() =>
      _SquareArticleComposePageState();
}

class _SquareArticleComposePageState extends State<SquareArticleComposePage> {
  final TextEditingController _titleController = TextEditingController();
  final TextEditingController _bodyController = TextEditingController();
  final ImagePicker _imagePicker = ImagePicker();
  late final SquarePublishService _publishService;
  late Future<SquareIdentityState> _identityFuture;

  SquareLocalMediaDraft? _cover;
  final List<SquareLocalMediaDraft> _bodyImages = [];
  SquarePostCategory _category = SquarePostCategory.normal;
  SquarePublishStage _publishStage = SquarePublishStage.idle;
  bool _publishing = false;

  @override
  void initState() {
    super.initState();
    _publishService = widget.publishService ?? SquarePublishService();
    _titleController.text = widget.initialTitle ?? '';
    _bodyController.text = widget.initialBody ?? '';
    _category = widget.initialCategory ?? SquarePostCategory.normal;
    _identityFuture = widget.identityService.loadCurrent();
  }

  @override
  void dispose() {
    _titleController.dispose();
    _bodyController.dispose();
    super.dispose();
  }

  Future<void> _pickCover() async {
    final image = await _imagePicker.pickImage(source: ImageSource.gallery);
    if (image == null || !mounted) return;
    final draft = await buildSquareMediaDraft(image, SquareMediaKind.image);
    if (!mounted) return;
    setState(() => _cover = draft);
  }

  Future<void> _pickBodyImages() async {
    final images = await _imagePicker.pickMultiImage();
    if (images.isEmpty || !mounted) return;
    final next = <SquareLocalMediaDraft>[];
    for (final image in images) {
      next.add(await buildSquareMediaDraft(image, SquareMediaKind.image));
    }
    if (!mounted) return;
    setState(() {
      final capacity = articleBodyImagesMax - _bodyImages.length;
      _bodyImages.addAll(next.take(capacity));
    });
  }

  String? _validate() => articleValidationError(
        title: _titleController.text,
        hasCover: _cover != null,
        body: _bodyController.text,
      );

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

  Future<void> _submit(SquareIdentityState identity) async {
    if (_publishing) return;
    if (!identity.hasWallet ||
        identity.walletIndex == null ||
        identity.pubkeyHex == null) {
      _showError('请先创建或选择钱包');
      return;
    }
    if (_category == SquarePostCategory.campaign && !identity.isCertified) {
      _showError('当前钱包未认证，不能发布竞选文章');
      return;
    }
    final error = _validate();
    if (error != null) {
      _showError(error);
      return;
    }

    setState(() {
      _publishing = true;
      _publishStage = SquarePublishStage.signingIn;
    });

    final signers = SquareComposeSigners(context: context, identity: identity);
    try {
      final mediaDrafts = <SquareLocalMediaDraft>[_cover!, ..._bodyImages];
      final result = await _publishService.publish(
        identity: identity,
        postCategory: _category,
        contentFormat: SquarePostContentFormat.article,
        title: _titleController.text.trim(),
        text: _bodyController.text.trim(),
        mediaDrafts: List<SquareLocalMediaDraft>.unmodifiable(mediaDrafts),
        signLoginPayload: signers.signLogin,
        signChainPayload: signers.signChain,
        replacePostId: widget.replacePostId,
        onStage: (stage) {
          if (!mounted) return;
          setState(() => _publishStage = stage);
        },
      );
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(result.cleanupWarning ?? '文章已发布'),
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

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), backgroundColor: AppTheme.danger),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar:
          AppBar(title: Text(widget.replacePostId == null ? '发布文章' : '修改文章')),
      body: FutureBuilder<SquareIdentityState>(
        future: _identityFuture,
        builder: (context, snapshot) {
          final identity = snapshot.data;
          final isCertified = identity?.isCertified ?? false;
          if (_category == SquarePostCategory.campaign && !isCertified) {
            _category = SquarePostCategory.normal;
          }
          return Column(
            children: [
              Expanded(
                child: ListView(
                  padding: const EdgeInsets.all(16),
                  children: [
                    if (identity != null) ...[
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
                          decoration: AppTheme.bannerDecoration(
                            AppTheme.warning,
                          ),
                          child: const Text(
                            '当前钱包未认证，不能发布竞选文章。',
                            style: TextStyle(
                              color: AppTheme.textPrimary,
                              fontSize: 13,
                              height: 1.35,
                            ),
                          ),
                        ),
                      ],
                      const SizedBox(height: 12),
                    ],
                    TextField(
                      controller: _titleController,
                      maxLength: articleTitleMax,
                      decoration: const InputDecoration(
                        labelText: '标题',
                        hintText: '10–50 字',
                      ),
                    ),
                    const SizedBox(height: 12),
                    _CoverPicker(cover: _cover, onTap: _pickCover),
                    const SizedBox(height: 16),
                    TextField(
                      controller: _bodyController,
                      maxLength: articleBodyMax,
                      maxLines: 12,
                      decoration: const InputDecoration(
                        labelText: '正文',
                        alignLabelWithHint: true,
                      ),
                    ),
                    const SizedBox(height: 12),
                    _BodyImages(
                      images: _bodyImages,
                      max: articleBodyImagesMax,
                      onAdd: _pickBodyImages,
                      onRemove: (i) => setState(() => _bodyImages.removeAt(i)),
                    ),
                  ],
                ),
              ),
              SafeArea(
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: FilledButton(
                    onPressed: (identity == null || _publishing)
                        ? null
                        : () => _submit(identity),
                    child: Text(
                      _publishing ? _publishStage.label : '签名发布',
                    ),
                  ),
                ),
              ),
            ],
          );
        },
      ),
    );
  }
}

class _CoverPicker extends StatelessWidget {
  const _CoverPicker({required this.cover, required this.onTap});

  final SquareLocalMediaDraft? cover;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(AppTheme.radiusMd),
      child: Container(
        height: 140,
        decoration: BoxDecoration(
          color: AppTheme.surfaceElevated,
          borderRadius: BorderRadius.circular(AppTheme.radiusMd),
          border: Border.all(color: AppTheme.border),
        ),
        child: Center(
          child: cover == null
              ? const Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.add_photo_alternate_outlined,
                        size: 28, color: AppTheme.textTertiary),
                    SizedBox(height: 6),
                    Text('选择首图（必填）',
                        style: TextStyle(color: AppTheme.textTertiary)),
                  ],
                )
              : Text('首图已选择：${cover!.fileName}',
                  style: const TextStyle(color: AppTheme.textSecondary)),
        ),
      ),
    );
  }
}

class _BodyImages extends StatelessWidget {
  const _BodyImages({
    required this.images,
    required this.max,
    required this.onAdd,
    required this.onRemove,
  });

  final List<SquareLocalMediaDraft> images;
  final int max;
  final VoidCallback onAdd;
  final void Function(int index) onRemove;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Text('正文配图（${images.length}/$max）',
                style: const TextStyle(color: AppTheme.textSecondary)),
            const Spacer(),
            TextButton.icon(
              onPressed: images.length >= max ? null : onAdd,
              icon: const Icon(Icons.add, size: 18),
              label: const Text('添加'),
            ),
          ],
        ),
        if (images.isNotEmpty)
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (var i = 0; i < images.length; i++)
                Chip(
                  label: Text(
                    images[i].fileName,
                    overflow: TextOverflow.ellipsis,
                  ),
                  onDeleted: () => onRemove(i),
                ),
            ],
          ),
      ],
    );
  }
}
