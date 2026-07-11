import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_compose_page.dart';
import 'package:citizenapp/8964/pages/square_compose_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/8964/widgets/square_post_card.dart';
import 'package:citizenapp/ui/app_theme.dart';

class SquarePostDetailResult {
  const SquarePostDetailResult({
    this.deleted = false,
    this.replacement,
  });

  final bool deleted;
  final SquarePost? replacement;
}

enum _PostDetailAction { edit, delete }

class SquarePostDetailPage extends StatefulWidget {
  const SquarePostDetailPage({
    super.key,
    required this.post,
    this.api,
    this.sessionProvider,
  });

  final SquarePost post;
  final SquareApiClient? api;
  final SquareSessionProvider? sessionProvider;

  @override
  State<SquarePostDetailPage> createState() => _SquarePostDetailPageState();
}

class _SquarePostDetailPageState extends State<SquarePostDetailPage> {
  late final SquareApiClient _api;
  late final SquareSessionProvider _sessionProvider;
  bool _deleting = false;

  SquarePost get post => widget.post;

  @override
  void initState() {
    super.initState();
    _api = widget.api ?? SquareApiClient();
    _sessionProvider = widget.sessionProvider ?? SquareSessionProvider.instance;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('动态详情'),
        actions: [
          PopupMenuButton<_PostDetailAction>(
            enabled: !_deleting,
            onSelected: _handleAction,
            itemBuilder: (context) => const [
              PopupMenuItem(
                value: _PostDetailAction.edit,
                child: ListTile(
                  leading: Icon(Icons.edit_outlined),
                  title: Text('修改'),
                ),
              ),
              PopupMenuItem(
                value: _PostDetailAction.delete,
                child: ListTile(
                  leading: Icon(Icons.delete_outline),
                  title: Text('删除'),
                ),
              ),
            ],
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
        children: [
          SquarePostCard(post: post),
          const SizedBox(height: 12),
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: AppTheme.surfaceCard,
              borderRadius: BorderRadius.circular(AppTheme.radiusMd),
              border: Border.all(color: AppTheme.border),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  '链上索引',
                  style: TextStyle(
                    color: AppTheme.textPrimary,
                    fontSize: 15,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 10),
                _DetailRow(label: 'post_id', value: post.postId),
                _DetailRow(
                  label: 'content_hash',
                  value: post.contentHash ?? '',
                ),
                _DetailRow(
                  label: 'storage_receipt_id',
                  value: post.storageReceiptId ?? '',
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _handleAction(_PostDetailAction action) async {
    switch (action) {
      case _PostDetailAction.edit:
        await _editPost();
        break;
      case _PostDetailAction.delete:
        await _deletePost();
        break;
    }
  }

  Future<void> _editPost() async {
    final replacement = await Navigator.of(context).push<SquarePost>(
      MaterialPageRoute<SquarePost>(
        builder: (_) => post.contentFormat == SquarePostContentFormat.article
            ? SquareArticleComposePage(
                initialTitle: post.title,
                initialBody: post.text,
                initialCategory: post.postCategory,
                replacePostId: post.postId,
              )
            : SquareComposePage(
                initialText: post.text,
                initialCategory: post.postCategory,
                replacePostId: post.postId,
              ),
      ),
    );
    if (replacement == null || !mounted) return;
    Navigator.of(context).pop(SquarePostDetailResult(replacement: replacement));
  }

  Future<void> _deletePost() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('删除动态'),
        content: const Text('删除后将清理 Cloudflare 中的正文和媒体。链上发布记录保持不变。'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(false),
            child: const Text('取消'),
          ),
          FilledButton.icon(
            onPressed: () => Navigator.of(dialogContext).pop(true),
            icon: const Icon(Icons.delete_outline),
            label: const Text('删除'),
          ),
        ],
      ),
    );
    if (confirmed != true || !mounted) return;

    setState(() => _deleting = true);
    try {
      final session = await _sessionProvider.ensureSession();
      if (session == null) {
        throw const SquareApiException('请先选择默认热钱包');
      }
      if (session.ownerAccount != post.author.ownerAccount) {
        throw const SquareApiException('只能删除本人动态');
      }
      await _api.deletePost(session: session, postId: post.postId);
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
            content: Text('已删除'), backgroundColor: AppTheme.primaryDark),
      );
      Navigator.of(context).pop(const SquarePostDetailResult(deleted: true));
    } catch (error) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
            content: Text('删除失败：$error'), backgroundColor: AppTheme.danger),
      );
    } finally {
      if (mounted) {
        setState(() => _deleting = false);
      }
    }
  }
}

class _DetailRow extends StatelessWidget {
  const _DetailRow({
    required this.label,
    required this.value,
  });

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 120,
            child: Text(
              label,
              style: const TextStyle(
                color: AppTheme.textSecondary,
                fontSize: 12,
              ),
            ),
          ),
          Expanded(
            child: Text(
              value.isEmpty ? '-' : value,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
