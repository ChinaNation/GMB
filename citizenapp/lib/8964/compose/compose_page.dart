import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/compose/article/article_compose_body.dart';
import 'package:citizenapp/8964/compose/compose_payload.dart';
import 'package:citizenapp/8964/compose/compose_type.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft_media.dart';
import 'package:citizenapp/8964/compose/drafts/compose_draft_store.dart';
import 'package:citizenapp/8964/compose/drafts/drafts_page.dart';
import 'package:citizenapp/8964/compose/post/post_compose_body.dart';
import 'package:citizenapp/8964/models/square_models.dart';
import 'package:citizenapp/8964/services/square_compose_signers.dart';
import 'package:citizenapp/8964/services/square_identity_state.dart';
import 'package:citizenapp/8964/services/square_publish_service.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 广场统一发布页：一页发 图片/视频/文章 及竞选变体。
///
/// 顶栏 取消/草稿/发布；头像 + 类型下拉（普通 2 项、认证公民 4 项）；
/// 动态/文章各自子编辑区（[SquarePostComposeBody] / [SquareArticleComposeBody]）由本壳协调发布。
class SquareComposePage extends StatefulWidget {
  const SquareComposePage({
    super.key,
    this.identityService = const SquareIdentityService(),
    this.publishService,
    this.draftStore,
    this.initialType = SquareComposeType.post,
    this.initialText,
    this.initialTitle,
    this.replacePostId,
  });

  final SquareIdentityService identityService;
  final SquarePublishService? publishService;
  final SquareComposeDraftRepository? draftStore;
  final SquareComposeType initialType;

  /// 编辑既有帖时预填正文/标题；媒体需重选（远端资源无法转回本地草稿）。
  final String? initialText;
  final String? initialTitle;
  final String? replacePostId;

  @override
  State<SquareComposePage> createState() => _SquareComposePageState();
}

class _SquareComposePageState extends State<SquareComposePage> {
  final _postKey = GlobalKey<SquarePostComposeBodyState>();
  final _articleKey = GlobalKey<SquareArticleComposeBodyState>();

  late final SquarePublishService _publishService;
  late final SquareComposeDraftRepository _draftStore;
  late Future<SquareIdentityState> _identityFuture;

  /// 本次编辑对应的草稿 id（开页即建；从草稿箱恢复时切到该草稿 id）。
  late String _draftId;
  SquareIdentityState? _identity;
  Timer? _autosaveTimer;
  bool _draftSaved = false;

  SquareComposeType _type = SquareComposeType.post;
  SquarePublishStage _stage = SquarePublishStage.idle;
  bool _publishing = false;

  @override
  void initState() {
    super.initState();
    _publishService = widget.publishService ?? SquarePublishService();
    _draftStore = widget.draftStore ?? SquareComposeDraftStore.instance;
    _draftId = 'd${DateTime.now().microsecondsSinceEpoch}';
    _type = widget.initialType;
    _identityFuture = widget.identityService.loadCurrent()
      ..then((identity) {
        if (mounted) _identity = identity;
      });
  }

  @override
  void dispose() {
    _autosaveTimer?.cancel();
    super.dispose();
  }

  ComposeBodyCollector? get _activeBody => _type.isArticle
      ? _articleKey.currentState as ComposeBodyCollector?
      : _postKey.currentState as ComposeBodyCollector?;

  Future<SquareLocalMediaDraft> _persistMedia(SquareLocalMediaDraft media) =>
      ComposeDraftMedia.persist(_draftId, media);

  /// 内容变化触发：防抖 800ms 后自动保存草稿。
  void _scheduleAutosave() {
    _autosaveTimer?.cancel();
    _autosaveTimer =
        Timer(const Duration(milliseconds: 800), () => _saveDraft());
  }

  /// 快照当前内容存草稿；空内容不存（已存过则删除）。
  Future<void> _saveDraft() async {
    final owner = _identity?.ownerAccount;
    if (owner == null || owner.isEmpty) return;
    final snapshot = _activeBody?.snapshot();
    if (snapshot == null) return;
    if (snapshot.isEmpty) {
      if (_draftSaved) {
        await _draftStore.delete(owner, _draftId);
        _draftSaved = false;
      }
      return;
    }
    await _draftStore.save(SquareComposeDraft(
      draftId: _draftId,
      ownerAccount: owner,
      contentFormat: _type.contentFormat,
      postCategory: _type.category,
      title: snapshot.title,
      text: snapshot.text,
      media: snapshot.media,
      contentBlocks: snapshot.contentBlocks,
      updatedAtMillis: DateTime.now().millisecondsSinceEpoch,
    ));
    _draftSaved = true;
  }

  /// 退出/取消前把待保存的草稿立即落盘。
  Future<void> _flushAndPop() async {
    _autosaveTimer?.cancel();
    await _saveDraft();
    if (mounted) Navigator.of(context).maybePop();
  }

  Future<void> _deleteCurrentDraft() async {
    final owner = _identity?.ownerAccount;
    if (owner == null || !_draftSaved) return;
    await _draftStore.delete(owner, _draftId);
    _draftSaved = false;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: FutureBuilder<SquareIdentityState>(
          future: _identityFuture,
          builder: (context, snapshot) {
            final identity =
                snapshot.data ?? const SquareIdentityState(ownerAccount: '');
            final canCampaign = identity.isCandidate;
            // 无竞选身份时把竞选类降级，避免非竞选身份用户停留在竞选档。
            final effectiveType = _type.degradedIfNotCampaignEligible(canCampaign);
            if (effectiveType != _type) {
              WidgetsBinding.instance.addPostFrameCallback((_) {
                if (mounted) setState(() => _type = effectiveType);
              });
            }
            return Column(
              children: [
                _TopBar(
                  publishing: _publishing,
                  stageLabel: _stage.label,
                  onCancel: _flushAndPop,
                  onDrafts: _openDrafts,
                  onPublish: identity.hasWallet && !_publishing
                      ? () => _publish(identity)
                      : null,
                ),
                _TypeBar(
                  identity: identity,
                  type: effectiveType,
                  canCampaign: canCampaign,
                  onChanged: (next) => setState(() => _type = next),
                ),
                Expanded(
                  child: IndexedStack(
                    index: effectiveType.isArticle ? 1 : 0,
                    children: [
                      SquarePostComposeBody(
                        key: _postKey,
                        initialText: widget.initialType.isArticle
                            ? null
                            : widget.initialText,
                        onChanged: _scheduleAutosave,
                        persistMedia: _persistMedia,
                      ),
                      SquareArticleComposeBody(
                        key: _articleKey,
                        initialTitle: widget.initialTitle,
                        initialText: widget.initialType.isArticle
                            ? widget.initialText
                            : null,
                        onChanged: _scheduleAutosave,
                        persistMedia: _persistMedia,
                      ),
                    ],
                  ),
                ),
                const _QuotaFooter(),
              ],
            );
          },
        ),
      ),
    );
  }

  Future<void> _openDrafts() async {
    final owner = _identity?.ownerAccount;
    if (owner == null || owner.isEmpty) return;
    // 先把当前内容落盘，避免进草稿箱丢失。
    _autosaveTimer?.cancel();
    await _saveDraft();
    if (!mounted) return;
    final selected = await Navigator.of(context).push<SquareComposeDraft>(
      MaterialPageRoute<SquareComposeDraft>(
        builder: (_) => DraftsPage(ownerAccount: owner, store: _draftStore),
      ),
    );
    if (selected == null || !mounted) return;
    // 切到该草稿并恢复到对应子编辑器。
    final restoredType = SquareComposeType.fromPost(
      isArticle: selected.isArticle,
      isCampaign: selected.isCampaign,
    );
    setState(() {
      _type = restoredType;
      _draftId = selected.draftId;
      _draftSaved = true;
    });
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (restoredType.isArticle) {
        _articleKey.currentState?.restore(selected);
      } else {
        _postKey.currentState?.restore(selected);
      }
    });
  }

  Future<void> _publish(SquareIdentityState identity) async {
    if (_publishing) return;
    final collector = _activeBody;
    if (collector == null) return;
    final payload = collector.collect();
    if (!payload.isValid) {
      _showError(payload.error!);
      return;
    }
    if (_type.isCampaign && !identity.isCandidate) {
      _showError('只有竞选身份的公民才能发布竞选内容');
      return;
    }
    setState(() {
      _publishing = true;
      _stage = SquarePublishStage.signingIn;
    });
    final signers = SquareComposeSigners(context: context, identity: identity);
    try {
      final result = await _publishService.publish(
        identity: identity,
        postCategory: _type.category,
        contentFormat: _type.contentFormat,
        text: payload.text,
        title: payload.title,
        contentBlocks: payload.contentBlocks,
        mediaDrafts: payload.mediaDrafts,
        signLoginPayload: signers.signLogin,
        signChainPayload: signers.signChain,
        replacePostId: widget.replacePostId,
        onStage: (stage) {
          if (mounted) setState(() => _stage = stage);
        },
      );
      // 发布成功：删除该草稿（含媒体目录）。
      _autosaveTimer?.cancel();
      await _deleteCurrentDraft();
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(result.cleanupWarning ?? '已发布'),
          backgroundColor: AppTheme.primaryDark,
        ),
      );
      Navigator.of(context).pop(result.post);
    } catch (e) {
      // 失败保留草稿（已由自动保存落盘）；用户可再次点发布重试。
      if (mounted) _showError('发布失败：$e');
    } finally {
      if (mounted) {
        setState(() {
          _publishing = false;
          _stage = SquarePublishStage.idle;
        });
      }
    }
  }

  void _showError(String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message), backgroundColor: AppTheme.danger),
    );
  }
}

class _TopBar extends StatelessWidget {
  const _TopBar({
    required this.publishing,
    required this.stageLabel,
    required this.onCancel,
    required this.onDrafts,
    required this.onPublish,
  });

  final bool publishing;
  final String stageLabel;
  final VoidCallback onCancel;
  final VoidCallback onDrafts;
  final VoidCallback? onPublish;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(8, 6, 12, 6),
      child: Row(
        children: [
          TextButton(onPressed: publishing ? null : onCancel, child: const Text('取消')),
          const Spacer(),
          TextButton(onPressed: publishing ? null : onDrafts, child: const Text('草稿')),
          const SizedBox(width: 4),
          FilledButton(
            onPressed: onPublish,
            child: publishing
                ? Text(stageLabel)
                : const Text('发布'),
          ),
        ],
      ),
    );
  }
}

class _TypeBar extends StatelessWidget {
  const _TypeBar({
    required this.identity,
    required this.type,
    required this.canCampaign,
    required this.onChanged,
  });

  final SquareIdentityState identity;
  final SquareComposeType type;
  final bool canCampaign;
  final ValueChanged<SquareComposeType> onChanged;

  @override
  Widget build(BuildContext context) {
    final options = SquareComposeType.optionsFor(canCampaign: canCampaign);
    final name = identity.walletName ?? '我';
    final initial = name.isEmpty ? '我' : name.substring(0, 1);
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 2, 16, 8),
      child: Row(
        children: [
          CircleAvatar(
            radius: 17,
            backgroundColor: AppTheme.primary.withAlpha(0x22),
            child: Text(initial,
                style: const TextStyle(
                    color: AppTheme.primary, fontWeight: FontWeight.w600)),
          ),
          const SizedBox(width: 10),
          Container(
            decoration: BoxDecoration(
              border: Border.all(color: AppTheme.border),
              borderRadius: BorderRadius.circular(20),
            ),
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: DropdownButtonHideUnderline(
              child: DropdownButton<SquareComposeType>(
                value: type,
                isDense: true,
                borderRadius: BorderRadius.circular(12),
                items: [
                  for (final option in options)
                    DropdownMenuItem(value: option, child: Text(option.label)),
                ],
                onChanged: (next) {
                  if (next != null) onChanged(next);
                },
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _QuotaFooter extends StatelessWidget {
  const _QuotaFooter();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      decoration: const BoxDecoration(
        border: Border(top: BorderSide(color: AppTheme.border)),
      ),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
      child: const Row(
        children: [
          Icon(Icons.workspace_premium_outlined,
              size: 16, color: AppTheme.textTertiary),
          SizedBox(width: 6),
          Text('发布额度按会员套餐计',
              style: TextStyle(color: AppTheme.textTertiary, fontSize: 12)),
        ],
      ),
    );
  }
}
