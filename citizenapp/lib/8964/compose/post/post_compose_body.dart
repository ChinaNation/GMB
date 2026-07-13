import 'dart:io';

import 'package:flutter/material.dart';
import 'package:image_picker/image_picker.dart';

import 'package:citizenapp/8964/compose/compose_payload.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_media_draft.dart';
import 'package:citizenapp/ui/app_theme.dart';

const int dynamicTextMax = 300;
const int dynamicMaxImages = 9;

/// 选中媒体后把临时文件持久化到草稿目录（壳注入）；null 时用原路径。
typedef ComposeMediaPersistor = Future<SquareLocalMediaDraft> Function(
    SquareLocalMediaDraft media);

/// 动态编辑区：正文 + 媒体。图片/视频不手选——第一次选中的类型锁定子类。
/// 计数右侧带框加号开选择器；下方只显示已选（九宫格图 / 单视频，发布页预览恒横屏）。
class SquarePostComposeBody extends StatefulWidget {
  const SquarePostComposeBody({
    super.key,
    this.initialText,
    this.initialMedia = const <SquareLocalMediaDraft>[],
    this.onChanged,
    this.persistMedia,
  });

  final String? initialText;
  final List<SquareLocalMediaDraft> initialMedia;

  /// 内容变化时回调（壳据此防抖自动保存草稿）。
  final VoidCallback? onChanged;
  final ComposeMediaPersistor? persistMedia;

  @override
  State<SquarePostComposeBody> createState() => SquarePostComposeBodyState();
}

class SquarePostComposeBodyState extends State<SquarePostComposeBody>
    implements ComposeBodyCollector {
  final TextEditingController _text = TextEditingController();
  final ImagePicker _picker = ImagePicker();
  final List<SquareLocalMediaDraft> _media = [];

  @override
  void initState() {
    super.initState();
    _text.text = widget.initialText ?? '';
    _media.addAll(widget.initialMedia);
    _text.addListener(() => widget.onChanged?.call());
  }

  /// 从草稿恢复：正文 + 媒体（媒体已是持久路径）。
  void restore(SquareComposeDraft draft) {
    setState(() {
      _text.text = draft.text;
      _media
        ..clear()
        ..addAll(draft.media);
    });
  }

  @override
  void dispose() {
    _text.dispose();
    super.dispose();
  }

  SquareMediaKind? get _lockedKind =>
      _media.isEmpty ? null : _media.first.mediaKind;

  int get _imageCount =>
      _media.where((m) => m.mediaKind == SquareMediaKind.image).length;

  bool get _canAdd {
    final kind = _lockedKind;
    if (kind == SquareMediaKind.video) return false; // 视频仅 1 个。
    return _imageCount < dynamicMaxImages; // 未锁定或锁图，未满 9 张可加。
  }

  @override
  ComposePayload collect() {
    if (_media.isEmpty) {
      return const ComposePayload.invalid('请至少选择一张图片或一个视频');
    }
    if (_text.text.trim().length > dynamicTextMax) {
      return const ComposePayload.invalid('动态文字不能超过 $dynamicTextMax 字');
    }
    return ComposePayload.ok(
      text: _text.text.trim(),
      mediaDrafts: List<SquareLocalMediaDraft>.unmodifiable(_media),
    );
  }

  @override
  ComposeSnapshot snapshot() => ComposeSnapshot(
        text: _text.text,
        media: List<SquareLocalMediaDraft>.of(_media),
      );

  Future<void> _onAdd() async {
    final kind = _lockedKind;
    if (kind == SquareMediaKind.image) {
      await _pickImages();
      return;
    }
    // 未锁定：先让用户选图片还是视频，第一次选定即锁定子类。
    final choice = await showModalBottomSheet<SquareMediaKind>(
      context: context,
      builder: (_) => SafeArea(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ListTile(
              leading: const Icon(Icons.image_outlined),
              title: const Text('图片'),
              onTap: () => Navigator.pop(context, SquareMediaKind.image),
            ),
            ListTile(
              leading: const Icon(Icons.videocam_outlined),
              title: const Text('视频'),
              onTap: () => Navigator.pop(context, SquareMediaKind.video),
            ),
          ],
        ),
      ),
    );
    if (choice == SquareMediaKind.image) await _pickImages();
    if (choice == SquareMediaKind.video) await _pickVideo();
  }

  Future<void> _pickImages() async {
    final images = await _picker.pickMultiImage();
    if (images.isEmpty || !mounted) return;
    final next = <SquareLocalMediaDraft>[];
    for (final image in images) {
      var draft = await buildSquareMediaDraft(image, SquareMediaKind.image);
      final persist = widget.persistMedia;
      if (persist != null) draft = await persist(draft);
      next.add(draft);
    }
    if (!mounted) return;
    setState(() {
      _media.addAll(next.take(dynamicMaxImages - _imageCount));
    });
    widget.onChanged?.call();
  }

  Future<void> _pickVideo() async {
    final video = await _picker.pickVideo(source: ImageSource.gallery);
    if (video == null || !mounted) return;
    var draft = await buildSquareMediaDraft(video, SquareMediaKind.video);
    final persist = widget.persistMedia;
    if (persist != null) draft = await persist(draft);
    if (!mounted) return;
    setState(() => _media.add(draft));
    widget.onChanged?.call();
  }

  void _remove(int index) {
    setState(() => _media.removeAt(index));
    widget.onChanged?.call();
  }

  @override
  Widget build(BuildContext context) {
    final kind = _lockedKind;
    final label = kind == SquareMediaKind.video
        ? '视频'
        : kind == SquareMediaKind.image
            ? '图片'
            : '图片 / 视频';
    final count = kind == SquareMediaKind.video
        ? '${_media.length}/1'
        : '$_imageCount/$dynamicMaxImages';

    return ListView(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
      children: [
        TextField(
          controller: _text,
          minLines: 5,
          maxLines: 10,
          maxLength: dynamicTextMax,
          decoration: const InputDecoration(hintText: '写下你的动态…'),
        ),
        const SizedBox(height: 8),
        Row(
          children: [
            Text('$label  ',
                style: const TextStyle(
                    color: AppTheme.textSecondary, fontSize: 13)),
            Text(count,
                style: const TextStyle(
                    color: AppTheme.textTertiary, fontSize: 13)),
            const SizedBox(width: 10),
            if (_canAdd)
              InkWell(
                onTap: _onAdd,
                borderRadius: BorderRadius.circular(6),
                child: Container(
                  width: 26,
                  height: 26,
                  decoration: BoxDecoration(
                    border: Border.all(color: AppTheme.textTertiary),
                    borderRadius: BorderRadius.circular(6),
                  ),
                  child: const Icon(Icons.add, size: 18),
                ),
              ),
          ],
        ),
        if (_media.isNotEmpty) ...[
          const SizedBox(height: 10),
          if (kind == SquareMediaKind.video)
            _VideoPreview(draft: _media.first, onRemove: () => _remove(0))
          else
            _ImageGrid(media: _media, onRemove: _remove),
        ],
      ],
    );
  }
}

/// 视频预览：发布页恒横屏 16:9（不管源横竖）。
class _VideoPreview extends StatelessWidget {
  const _VideoPreview({required this.draft, required this.onRemove});

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
            const ColoredBox(color: AppTheme.surfaceElevated),
            const Center(
              child: Icon(Icons.play_circle_fill_rounded,
                  size: 42, color: AppTheme.textTertiary),
            ),
            Positioned(
              top: 6,
              right: 6,
              child: _RemoveButton(onTap: onRemove),
            ),
          ],
        ),
      ),
    );
  }
}

class _ImageGrid extends StatelessWidget {
  const _ImageGrid({required this.media, required this.onRemove});

  final List<SquareLocalMediaDraft> media;
  final ValueChanged<int> onRemove;

  @override
  Widget build(BuildContext context) {
    return GridView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: media.length,
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        crossAxisSpacing: 6,
        mainAxisSpacing: 6,
      ),
      itemBuilder: (context, index) => ClipRRect(
        borderRadius: BorderRadius.circular(AppTheme.radiusSm),
        child: Stack(
          fit: StackFit.expand,
          children: [
            Image.file(File(media[index].path), fit: BoxFit.cover),
            Positioned(
              top: 3,
              right: 3,
              child: _RemoveButton(onTap: () => onRemove(index)),
            ),
          ],
        ),
      ),
    );
  }
}

class _RemoveButton extends StatelessWidget {
  const _RemoveButton({required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          color: Colors.black.withAlpha(0x80),
          shape: BoxShape.circle,
        ),
        padding: const EdgeInsets.all(2),
        child: const Icon(Icons.close, size: 15, color: Colors.white),
      ),
    );
  }
}
