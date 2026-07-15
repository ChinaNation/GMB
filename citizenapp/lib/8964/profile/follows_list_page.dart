import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/models/profile_presentation.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/profile/widgets/profile_avatar.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/ui/app_theme.dart';

enum FollowsType {
  following('关注', 'following'),
  followers('关注者', 'followers');

  const FollowsType(this.label, this.workerValue);

  final String label;
  final String workerValue;
}

/// 关注/粉丝列表页。缺少公开资料时仍使用与主页一致的稳定默认昵称和头像，
/// 钱包账户只显示在副标题，点击进入唯一用户主页。
class FollowsListPage extends StatefulWidget {
  const FollowsListPage({
    super.key,
    required this.ownerAccount,
    required this.type,
    required this.session,
    this.api,
  });

  final String ownerAccount;
  final FollowsType type;
  final SquareSession session;
  final CitizenProfileApi? api;

  @override
  State<FollowsListPage> createState() => _FollowsListPageState();
}

class _FollowsListPageState extends State<FollowsListPage> {
  late final CitizenProfileApi _api;
  final List<SquareFollowEntry> _entries = [];
  final Map<String, CitizenProfile> _profiles = <String, CitizenProfile>{};
  int? _cursor;
  bool _loading = false;
  bool _done = false;
  bool _failedFirst = false;

  @override
  void initState() {
    super.initState();
    _api = widget.api ?? CitizenProfileApi();
    _loadFirst();
  }

  Future<void> _loadFirst() async {
    setState(() {
      _loading = true;
      _failedFirst = false;
    });
    try {
      final page = await _api.fetchFollows(
        widget.ownerAccount,
        type: widget.type.workerValue,
        limit: 20,
        session: widget.session,
      );
      if (!mounted) return;
      setState(() {
        _entries
          ..clear()
          ..addAll(page.accounts);
        _cursor = page.nextCursor;
        _done = page.nextCursor == null;
        _loading = false;
      });
      unawaited(_loadProfiles(page.accounts));
    } on Exception {
      if (!mounted) return;
      setState(() {
        _loading = false;
        _failedFirst = _entries.isEmpty;
      });
    }
  }

  Future<void> _loadMore() async {
    if (_loading || _done || _cursor == null) return;
    setState(() => _loading = true);
    try {
      final page = await _api.fetchFollows(
        widget.ownerAccount,
        type: widget.type.workerValue,
        limit: 20,
        cursor: _cursor,
        session: widget.session,
      );
      if (!mounted) return;
      setState(() {
        _entries.addAll(page.accounts);
        _cursor = page.nextCursor;
        _done = page.nextCursor == null;
        _loading = false;
      });
      unawaited(_loadProfiles(page.accounts));
    } on Exception {
      if (!mounted) return;
      setState(() => _loading = false);
    }
  }

  bool _onScroll(ScrollNotification notification) {
    if (notification.metrics.pixels >=
        notification.metrics.maxScrollExtent - 300) {
      _loadMore();
    }
    return false;
  }

  /// 关注关系接口只返回账户；公开资料按当前分页并行补齐。单个资料失败时保留
  /// 稳定本地默认展示，不阻塞列表及其他用户。
  Future<void> _loadProfiles(List<SquareFollowEntry> entries) async {
    await Future.wait(entries.map((entry) async {
      if (_profiles.containsKey(entry.ownerAccount)) return;
      try {
        final profile = await _api.fetchProfile(
          entry.ownerAccount,
          session: widget.session,
        );
        _profiles[entry.ownerAccount] = profile;
      } on Exception {
        // 公开资料不可用时由 ProfilePresentation 稳定兜底。
      }
    }));
    if (mounted) setState(() {});
  }

  void _openProfile(String account) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => UserProfilePage(
          ownerAccount: account,
          isSelf: false,
          initialProfile: _profiles[account],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(widget.type.label), centerTitle: true),
      body: _body(),
    );
  }

  Widget _body() {
    if (_loading && _entries.isEmpty) {
      return const Center(child: CircularProgressIndicator());
    }
    if (_failedFirst) {
      return const Center(
        child:
            Text('加载失败，请返回重试', style: TextStyle(color: AppTheme.textTertiary)),
      );
    }
    if (_entries.isEmpty) {
      return Center(
        child: Text(
          widget.type == FollowsType.following ? '还没有关注任何人' : '还没有关注者',
          style: const TextStyle(color: AppTheme.textTertiary),
        ),
      );
    }
    return NotificationListener<ScrollNotification>(
      onNotification: _onScroll,
      child: ListView.separated(
        itemCount: _entries.length + (_loading ? 1 : 0),
        separatorBuilder: (_, __) => const Divider(height: 1),
        itemBuilder: (context, index) {
          if (index >= _entries.length) {
            return const Padding(
              padding: EdgeInsets.all(16),
              child: Center(
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ),
            );
          }
          final entry = _entries[index];
          final profile = _profiles[entry.ownerAccount];
          final presentation =
              ProfilePresentation.forAccount(entry.ownerAccount);
          final avatarKey = profile?.avatarObjectKey;
          return ListTile(
            leading: ProfileAvatar(
              seed: entry.ownerAccount,
              size: 42,
              imageUrl: avatarKey == null ? null : _api.mediaUrl(avatarKey),
              imageHeaders: <String, String>{
                'authorization': 'Bearer ${widget.session.sessionToken}',
              },
              identityLevel: profile?.identityLevel,
              membershipLevel: profile?.membershipLevel,
              membershipActive: profile?.membershipActive ?? false,
            ),
            title: Text(
              presentation.resolveDisplayName(
                publicName: profile?.displayName,
              ),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            subtitle: Text(
              _shorten(entry.ownerAccount),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            trailing: const Icon(Icons.chevron_right, size: 20),
            onTap: () => _openProfile(entry.ownerAccount),
          );
        },
      ),
    );
  }

  String _shorten(String account) {
    if (account.length <= 12) return account;
    return '${account.substring(0, 6)}...'
        '${account.substring(account.length - 6)}';
  }
}
