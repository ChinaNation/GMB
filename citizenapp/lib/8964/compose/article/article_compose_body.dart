import 'dart:io';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';

import 'package:citizenapp/8964/compose/article/article_blocks.dart';
import 'package:citizenapp/8964/compose/compose_payload.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/compose/post/post_compose_body.dart'
    show ComposeMediaPersistor;
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_media_draft.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 编辑侧图文块（内部）：文本块（控制器+焦点）或图片块（本地草稿）。
sealed class _EditBlock {
  const _EditBlock();
}

final class _TextEditBlock extends _EditBlock {
  _TextEditBlock([String text = ''])
      : controller = TextEditingController(text: text),
        focus = FocusNode();
  final TextEditingController controller;
  final FocusNode focus;
}

final class _ImageEditBlock extends _EditBlock {
  const _ImageEditBlock(this.draft);
  final SquareLocalMediaDraft draft;
}

/// 文章编辑区：标题 + 正文计数固定顶部；下方图文块可滚动，正文内可插入横屏图片。
class SquareArticleComposeBody extends StatefulWidget {
  const SquareArticleComposeBody({
    super.key,
    this.initialTitle,
    this.initialText,
    this.onChanged,
    this.persistMedia,
  });

  final String? initialTitle;
  final String? initialText;

  /// 内容变化时回调（壳据此防抖自动保存草稿）。
  final VoidCallback? onChanged;
  final ComposeMediaPersistor? persistMedia;

  @override
  State<SquareArticleComposeBody> createState() =>
      SquareArticleComposeBodyState();
}

class SquareArticleComposeBodyState extends State<SquareArticleComposeBody>
    implements ComposeBodyCollector {
  final TextEditingController _title = TextEditingController();
  final ImagePicker _picker = ImagePicker();
  SquareLocalMediaDraft? _cover;
  final List<_EditBlock> _blocks = [];

  @override
  void initState() {
    super.initState();
    _title.text = widget.initialTitle ?? '';
    _title.addListener(_onChanged);
    _blocks.add(_newTextBlock(widget.initialText ?? ''));
  }

  @override
  void dispose() {
    _title.dispose();
    for (final block in _blocks) {
      if (block is _TextEditBlock) {
        block.controller.dispose();
        block.focus.dispose();
      }
    }
    super.dispose();
  }

  _TextEditBlock _newTextBlock([String text = '']) {
    final block = _TextEditBlock(text);
    block.controller.addListener(_onChanged);
    return block;
  }

  void _onChanged() {
    setState(() {});
    widget.onChanged?.call();
  }

  int get _bodyLength {
    var total = 0;
    for (final block in _blocks) {
      if (block is _TextEditBlock) total += block.controller.text.length;
    }
    return total;
  }

  @override
  ComposePayload collect() {
    final cover = _cover;
    final draftBlocks = <ArticleDraftBlock>[];
    final textParts = <String>[];
    for (final block in _blocks) {
      switch (block) {
        case _TextEditBlock():
          draftBlocks.add(ArticleDraftText(block.controller.text));
          if (block.controller.text.trim().isNotEmpty) {
            textParts.add(block.controller.text.trim());
          }
        case _ImageEditBlock():
          draftBlocks.add(ArticleDraftImage(block.draft));
      }
    }
    final error = articleValidationError(
      title: _title.text,
      hasCover: cover != null,
      body: textParts.join('\n\n'),
    );
    if (error != null) return ComposePayload.invalid(error);
    final parts = buildArticleManifest(cover: cover!, body: draftBlocks);
    return ComposePayload.ok(
      text: parts.text,
      title: _title.text.trim(),
      mediaDrafts: parts.mediaDrafts,
      contentBlocks: parts.contentBlocks,
    );
  }

  @override
  ComposeSnapshot snapshot() {
    final media = <SquareLocalMediaDraft>[];
    if (_cover != null) media.add(_cover!);
    final blocks = <Map<String, Object?>>[];
    final textParts = <String>[];
    for (final block in _blocks) {
      switch (block) {
        case _TextEditBlock():
          blocks.add({'t': 'text', 'text': block.controller.text});
          if (block.controller.text.trim().isNotEmpty) {
            textParts.add(block.controller.text.trim());
          }
        case _ImageEditBlock():
          media.add(block.draft);
          blocks.add({'t': 'image', 'media_index': media.length - 1});
      }
    }
    return ComposeSnapshot(
      text: textParts.join('\n\n'),
      title: _title.text,
      media: media,
      contentBlocks: blocks,
    );
  }

  /// 从草稿恢复：还原首图 + 图文块。首图 = 未被任一图片块引用的 media[0]。
  void restore(SquareComposeDraft draft) {
    for (final block in _blocks) {
      if (block is _TextEditBlock) {
        block.controller.dispose();
        block.focus.dispose();
      }
    }
    final media = draft.media;
    final rawBlocks = draft.contentBlocks;
    final rebuilt = <_EditBlock>[];
    SquareLocalMediaDraft? cover;
    if (rawBlocks != null && rawBlocks.isNotEmpty) {
      final referenced = <int>{};
      for (final block in rawBlocks) {
        if (block['t'] == 'image' && block['media_index'] is int) {
          referenced.add(block['media_index'] as int);
        }
      }
      cover = media.isNotEmpty && !referenced.contains(0) ? media[0] : null;
      for (final block in rawBlocks) {
        if (block['t'] == 'text') {
          rebuilt.add(_newTextBlock(block['text']?.toString() ?? ''));
        } else if (block['t'] == 'image' && block['media_index'] is int) {
          final i = block['media_index'] as int;
          if (i >= 0 && i < media.length) rebuilt.add(_ImageEditBlock(media[i]));
        }
      }
    } else {
      // 无块（旧文章草稿）：首图=media[0]、正文=单文本块、其余媒体作图片块。
      cover = media.isNotEmpty ? media.first : null;
      rebuilt.add(_newTextBlock(draft.text));
      for (var i = 1; i < media.length; i++) {
        rebuilt.add(_ImageEditBlock(media[i]));
      }
    }
    if (rebuilt.isEmpty) rebuilt.add(_newTextBlock());
    setState(() {
      _cover = cover;
      _blocks
        ..clear()
        ..addAll(rebuilt);
      _title.text = draft.title ?? '';
    });
  }

  Future<void> _pickCover() async {
    final image = await _picker.pickImage(source: ImageSource.gallery);
    if (image == null || !mounted) return;
    var draft = await buildSquareMediaDraft(image, SquareMediaKind.image);
    final persist = widget.persistMedia;
    if (persist != null) draft = await persist(draft);
    if (!mounted) return;
    setState(() => _cover = draft);
    widget.onChanged?.call();
  }

  /// 在当前焦点文本块后插入图片块 + 一个新文本块，光标落到新文本块，实现图文混排。
  Future<void> _insertImage() async {
    final image = await _picker.pickImage(source: ImageSource.gallery);
    if (image == null || !mounted) return;
    var draft = await buildSquareMediaDraft(image, SquareMediaKind.image);
    final persist = widget.persistMedia;
    if (persist != null) draft = await persist(draft);
    if (!mounted) return;
    var index = _blocks.indexWhere(
        (b) => b is _TextEditBlock && b.focus.hasFocus);
    if (index < 0) index = _blocks.length - 1;
    final newText = _newTextBlock();
    setState(() {
      _blocks
        ..insert(index + 1, _ImageEditBlock(draft))
        ..insert(index + 2, newText);
    });
    widget.onChanged?.call();
    WidgetsBinding.instance
        .addPostFrameCallback((_) => newText.focus.requestFocus());
  }

  void _removeImageBlock(int index) {
    final block = _blocks[index];
    setState(() => _blocks.removeAt(index));
    if (block is _TextEditBlock) {
      block.controller.dispose();
      block.focus.dispose();
    }
    widget.onChanged?.call();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        // 固定区：标题 + 首图缩略 + 正文计数 + 插入。
        Container(
          decoration: const BoxDecoration(
            color: AppTheme.surfaceCard,
            border: Border(bottom: BorderSide(color: AppTheme.border)),
          ),
          padding: const EdgeInsets.fromLTRB(16, 10, 16, 10),
          child: Column(
            children: [
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _title,
                      maxLength: articleTitleMax,
                      buildCounter: (_,
                              {required currentLength,
                              required isFocused,
                              maxLength}) =>
                          null,
                      decoration: const InputDecoration(
                        isDense: true,
                        border: InputBorder.none,
                        hintText: '标题',
                      ),
                      style: const TextStyle(
                        color: AppTheme.textPrimary,
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Text('${_title.text.length}/$articleTitleMax',
                      style: const TextStyle(
                          color: AppTheme.textTertiary, fontSize: 11)),
                  const SizedBox(width: 8),
                  _CoverThumb(cover: _cover, onPick: _pickCover, onRemove: () {
                    setState(() => _cover = null);
                  }),
                ],
              ),
              const SizedBox(height: 8),
              Row(
                children: [
                  const Text('正文',
                      style: TextStyle(
                          color: AppTheme.textSecondary, fontSize: 12)),
                  const SizedBox(width: 8),
                  Text('$_bodyLength/$articleBodyMax',
                      style: const TextStyle(
                          color: AppTheme.textTertiary, fontSize: 11)),
                  const Spacer(),
                  OutlinedButton.icon(
                    onPressed: _insertImage,
                    icon: const Icon(Icons.add_photo_alternate_outlined,
                        size: 16),
                    label: const Text('插入'),
                    style: OutlinedButton.styleFrom(
                      visualDensity: VisualDensity.compact,
                      padding:
                          const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
                    ),
                  ),
                ],
              ),
            ],
          ),
        ),
        // 可滚区：图文块。
        Expanded(
          child: ListView.builder(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
            itemCount: _blocks.length,
            itemBuilder: (context, index) {
              final block = _blocks[index];
              if (block is _ImageEditBlock) {
                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  child: _InlineImage(
                    draft: block.draft,
                    onRemove: () => _removeImageBlock(index),
                  ),
                );
              }
              block as _TextEditBlock;
              return TextField(
                controller: block.controller,
                focusNode: block.focus,
                maxLines: null,
                decoration: const InputDecoration(
                  isDense: true,
                  border: InputBorder.none,
                  hintText: '正文…',
                ),
                style: const TextStyle(
                  color: AppTheme.textPrimary,
                  fontSize: 16,
                  height: 1.7,
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}

/// 首图缩略：未选=带框加号；已选=小缩略 + 叉（删除恢复加号）。发布页不显示大封面。
class _CoverThumb extends StatelessWidget {
  const _CoverThumb({
    required this.cover,
    required this.onPick,
    required this.onRemove,
  });

  final SquareLocalMediaDraft? cover;
  final VoidCallback onPick;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    final draft = cover;
    if (draft == null) {
      return InkWell(
        onTap: onPick,
        borderRadius: BorderRadius.circular(6),
        child: Container(
          width: 28,
          height: 28,
          decoration: BoxDecoration(
            border: Border.all(color: AppTheme.textTertiary),
            borderRadius: BorderRadius.circular(6),
          ),
          child: const Icon(Icons.add_photo_alternate_outlined, size: 16),
        ),
      );
    }
    return SizedBox(
      width: 32,
      height: 32,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          ClipRRect(
            borderRadius: BorderRadius.circular(6),
            child: Image.file(File(draft.path),
                width: 28, height: 28, fit: BoxFit.cover),
          ),
          Positioned(
            top: -6,
            right: -6,
            child: GestureDetector(
              onTap: onRemove,
              child: const CircleAvatar(
                radius: 8,
                backgroundColor: AppTheme.surfaceCard,
                child: Icon(Icons.close, size: 12, color: AppTheme.textSecondary),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// 正文内联图：恒横屏 16:9 + 删除叉。
class _InlineImage extends StatelessWidget {
  const _InlineImage({required this.draft, required this.onRemove});

  final SquareLocalMediaDraft draft;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    return ClipRRect(
      borderRadius: BorderRadius.circular(AppTheme.radiusMd),
      child: AspectRatio(
        aspectRatio: 16 / 9,
        child: Stack(
          fit: StackFit.expand,
          children: [
            Image.file(File(draft.path), fit: BoxFit.cover),
            Positioned(
              top: 6,
              right: 6,
              child: GestureDetector(
                onTap: onRemove,
                child: Container(
                  decoration: BoxDecoration(
                    color: Colors.black.withAlpha(0x80),
                    shape: BoxShape.circle,
                  ),
                  padding: const EdgeInsets.all(3),
                  child: const Icon(Icons.close, size: 16, color: Colors.white),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
