import 'package:flutter/material.dart';

import 'package:citizenapp/8964/profile/models/citizen_profile.dart';
import 'package:citizenapp/8964/profile/services/citizen_profile_api.dart';
import 'package:citizenapp/8964/profile/user_profile_page.dart';
import 'package:citizenapp/8964/services/square_api_client.dart';
import 'package:citizenapp/ui/app_theme.dart';

enum FollowsType {
  following('关注', 'following'),
  followers('关注者', 'followers');

  const FollowsType(this.label, this.workerValue);

  final String label;
  final String workerValue;
}

/// 关注/粉丝列表页。行显示短地址，点击进入对应用户主页。
/// 展示名/头像懒加载增强留待后续。
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

  void _openProfile(String account) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => UserProfilePage(ownerAccount: account, isSelf: false),
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
          return ListTile(
            leading: CircleAvatar(
              backgroundColor: AppTheme.primary.withAlpha(20),
              child: const Icon(Icons.person, color: AppTheme.primary),
            ),
            title: Text(
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
