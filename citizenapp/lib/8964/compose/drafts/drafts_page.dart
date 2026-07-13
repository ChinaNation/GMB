import 'dart:io';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft_store.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 草稿箱：全类型缩略卡，新→旧；右滑删除；点击返回该草稿供发布页恢复。
class DraftsPage extends StatefulWidget {
  const DraftsPage({
    super.key,
    required this.ownerAccount,
    this.store,
  });

  final String ownerAccount;
  final SquareComposeDraftRepository? store;

  @override
  State<DraftsPage> createState() => _DraftsPageState();
}

class _DraftsPageState extends State<DraftsPage> {
  late final SquareComposeDraftRepository _store;
  late Future<List<SquareComposeDraft>> _future;

  @override
  void initState() {
    super.initState();
    _store = widget.store ?? SquareComposeDraftStore.instance;
    _future = _store.list(widget.ownerAccount);
  }

  Future<void> _delete(SquareComposeDraft draft) async {
    await _store.delete(widget.ownerAccount, draft.draftId);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('草稿箱'), centerTitle: true),
      body: FutureBuilder<List<SquareComposeDraft>>(
        future: _future,
        builder: (context, snapshot) {
          if (snapshot.connectionState != ConnectionState.done) {
            return const Center(child: CircularProgressIndicator());
          }
          final drafts = snapshot.data ?? const <SquareComposeDraft>[];
          if (drafts.isEmpty) {
            return const Center(
              child: Text('还没有草稿',
                  style: TextStyle(color: AppTheme.textTertiary)),
            );
          }
          return ListView.separated(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
            itemCount: drafts.length,
            separatorBuilder: (_, __) => const SizedBox(height: 10),
            itemBuilder: (context, index) {
              final draft = drafts[index];
              return Dismissible(
                key: ValueKey(draft.draftId),
                direction: DismissDirection.endToStart,
                background: Container(
                  alignment: Alignment.centerRight,
                  padding: const EdgeInsets.only(right: 20),
                  decoration: BoxDecoration(
                    color: AppTheme.danger,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: const Icon(Icons.delete_outline, color: Colors.white),
                ),
                onDismissed: (_) => _delete(draft),
                child: _DraftCard(
                  draft: draft,
                  onTap: () => Navigator.of(context).pop(draft),
                ),
              );
            },
          );
        },
      ),
    );
  }
}

class _DraftCard extends StatelessWidget {
  const _DraftCard({required this.draft, required this.onTap});

  final SquareComposeDraft draft;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(12),
      child: Container(
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
          color: AppTheme.surfaceCard,
          border: Border.all(color: AppTheme.border),
          borderRadius: BorderRadius.circular(12),
        ),
        child: Row(
          children: [
            _Thumb(draft: draft),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      _TypeChip(label: draft.typeLabel, campaign: draft.isCampaign),
                      const Spacer(),
                      Text(_relativeTime(draft.updatedAtMillis),
                          style: const TextStyle(
                              color: AppTheme.textTertiary, fontSize: 11)),
                    ],
                  ),
                  const SizedBox(height: 4),
                  Text(
                    draft.summary.isEmpty ? '（无正文）' : draft.summary,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                        color: AppTheme.textPrimary, fontSize: 13),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  static String _relativeTime(int millis) {
    final diff = DateTime.now()
        .difference(DateTime.fromMillisecondsSinceEpoch(millis));
    if (diff.inMinutes < 1) return '刚刚';
    if (diff.inHours < 1) return '${diff.inMinutes} 分钟前';
    if (diff.inDays < 1) return '${diff.inHours} 小时前';
    return '${diff.inDays} 天前';
  }
}

class _Thumb extends StatelessWidget {
  const _Thumb({required this.draft});

  final SquareComposeDraft draft;

  @override
  Widget build(BuildContext context) {
    final media = draft.media.isNotEmpty ? draft.media.first : null;
    final isVideo = media?.mediaKind == SquareMediaKind.video;
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: SizedBox(
        width: 48,
        height: 48,
        child: (media != null && !isVideo && File(media.path).existsSync())
            ? Image.file(File(media.path), fit: BoxFit.cover)
            : ColoredBox(
                color: AppTheme.surfaceElevated,
                child: Icon(
                  isVideo
                      ? Icons.play_circle_fill_rounded
                      : draft.isArticle
                          ? Icons.article_outlined
                          : Icons.image_outlined,
                  color: AppTheme.textTertiary,
                ),
              ),
      ),
    );
  }
}

class _TypeChip extends StatelessWidget {
  const _TypeChip({required this.label, required this.campaign});

  final String label;
  final bool campaign;

  @override
  Widget build(BuildContext context) {
    final color = campaign ? AppTheme.danger : AppTheme.primary;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 1),
      decoration: BoxDecoration(
        color: color.withAlpha(0x1F),
        borderRadius: BorderRadius.circular(20),
      ),
      child: Text(label,
          style: TextStyle(
              color: color, fontSize: 10, fontWeight: FontWeight.w600)),
    );
  }
}
