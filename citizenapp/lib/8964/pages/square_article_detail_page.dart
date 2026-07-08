import 'package:flutter/material.dart';

import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/pages/square_article_compose_page.dart';
import 'package:citizenapp/8964/pages/square_post_detail_page.dart';
import 'package:citizenapp/8964/profile/services/square_session_provider.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 文章详情：首图 + 标题 + 作者 + 正文全文 + 正文图（media_items[1..]）。
class SquareArticleDetailPage extends StatefulWidget {
  const SquareArticleDetailPage({
    super.key,
    required this.post,
    this.api,
    this.sessionProvider,
  });

  final SquarePost post;
  final SquareApiClient? api;
  final SquareSessionProvider? sessionProvider;

  @override
  State<SquareArticleDetailPage> createState() =>
      _SquareArticleDetailPageState();
}

enum _ArticleDetailAction { edit, delete }

class _SquareArticleDetailPageState extends State<SquareArticleDetailPage> {
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
    final media = post.mediaItems;
    final cover = media.isNotEmpty ? media.first : null;
    final bodyImages = media.length > 1 ? media.sublist(1) : const [];
    final title = post.title?.trim();

    return Scaffold(
      appBar: AppBar(
        title: const Text('文章'),
        centerTitle: true,
        actions: [
          PopupMenuButton<_ArticleDetailAction>(
            enabled: !_deleting,
            onSelected: _handleAction,
            itemBuilder: (context) => const [
              PopupMenuItem(
                value: _ArticleDetailAction.edit,
                child: ListTile(
                  leading: Icon(Icons.edit_outlined),
                  title: Text('修改'),
                ),
              ),
              PopupMenuItem(
                value: _ArticleDetailAction.delete,
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
        children: [
          if (cover != null && cover.url.isNotEmpty)
            Image.network(
              cover.url,
              fit: BoxFit.cover,
              errorBuilder: (_, __, ___) => const SizedBox.shrink(),
            ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (title != null && title.isNotEmpty)
                  Text(
                    title,
                    style: const TextStyle(
                      color: AppTheme.textPrimary,
                      fontSize: 22,
                      fontWeight: FontWeight.w700,
                      height: 1.35,
                    ),
                  ),
                const SizedBox(height: 8),
                Text(
                  post.author.title,
                  style: const TextStyle(
                    color: AppTheme.textTertiary,
                    fontSize: 13,
                  ),
                ),
                const SizedBox(height: 16),
                if (post.text.trim().isNotEmpty)
                  Text(
                    post.text.trim(),
                    style: const TextStyle(
                      color: AppTheme.textPrimary,
                      fontSize: 16,
                      height: 1.7,
                    ),
                  ),
                for (final image in bodyImages)
                  if (image.url.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: ClipRRect(
                        borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                        child: Image.network(
                          image.url,
                          fit: BoxFit.cover,
                          errorBuilder: (_, __, ___) => const SizedBox.shrink(),
                        ),
                      ),
                    ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _handleAction(_ArticleDetailAction action) async {
    switch (action) {
      case _ArticleDetailAction.edit:
        await _editArticle();
        break;
      case _ArticleDetailAction.delete:
        await _deleteArticle();
        break;
    }
  }

  Future<void> _editArticle() async {
    final replacement = await Navigator.of(context).push<SquarePost>(
      MaterialPageRoute<SquarePost>(
        builder: (_) => SquareArticleComposePage(
          initialTitle: post.title,
          initialBody: post.text,
          initialCategory: post.postCategory,
          replacePostId: post.postId,
        ),
      ),
    );
    if (replacement == null || !mounted) return;
    Navigator.of(context).pop(SquarePostDetailResult(replacement: replacement));
  }

  Future<void> _deleteArticle() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('删除文章'),
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
        throw const SquareApiException('只能删除本人文章');
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
