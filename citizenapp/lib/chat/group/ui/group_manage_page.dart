import 'package:flutter/material.dart';

import 'package:citizenapp/chat/chat_runtime.dart';
import 'package:citizenapp/chat/group/group_model.dart';
import 'package:citizenapp/chat/storage/chat_store.dart';
import 'package:citizenapp/my/user/contact_service.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 成员管理页:名册 + 加/删(仅 admin)+ 改群名(仅 admin)+ 退群(任何人)。
class GroupManagePage extends StatefulWidget {
  const GroupManagePage({
    super.key,
    required this.groupId,
    this.runtime,
    this.store,
    this.accountId,
  });

  final String groupId;
  final ChatRuntime? runtime;
  final ChatStore? store;

  /// 本机账户;测试可注入以设定"我是谁"验 admin 门控。生产为 null → 取默认钱包。
  final String? accountId;

  @override
  State<GroupManagePage> createState() => _GroupManagePageState();
}

class _GroupManagePageState extends State<GroupManagePage> {
  late final ChatRuntime _runtime = widget.runtime ?? ChatRuntime();
  late final ChatStore _store = widget.store ?? ChatStore();

  ChatGroup? _group;
  String _myAccount = '';
  bool _loading = true;
  bool _busy = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final me = widget.accountId ??
          (await WalletManager().getDefaultWallet())?.accountId ??
          '';
      final group = await _store.readGroup(widget.groupId);
      if (!mounted) return;
      setState(() {
        _myAccount = me;
        _group = group;
        _loading = false;
      });
    } catch (error) {
      if (!mounted) return;
      setState(() {
        _error = '$error';
        _loading = false;
      });
    }
  }

  bool get _isAdmin => _group?.adminSet.contains(_myAccount) ?? false;

  Future<void> _run(Future<void> Function() action) async {
    if (_busy) return;
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      await action();
      await _load();
    } catch (error) {
      if (mounted) setState(() => _error = '$error');
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _rename() async {
    final controller = TextEditingController(text: _group?.name ?? '');
    final name = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('修改群名'),
        content: TextField(
          controller: controller,
          maxLength: 40,
          decoration: const InputDecoration(counterText: ''),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(controller.text.trim()),
            child: const Text('保存'),
          ),
        ],
      ),
    );
    if (name != null && name.isNotEmpty) {
      await _run(
          () => _runtime.renameGroup(groupId: widget.groupId, name: name));
    }
  }

  Future<void> _addMembers() async {
    final existing = _group?.memberAccountIds.toSet() ?? <String>{};
    final selected = await _pickContacts(existing);
    if (selected != null && selected.isNotEmpty) {
      await _run(() => _runtime.addGroupMembers(
            groupId: widget.groupId,
            inviteeAccounts: selected,
          ));
    }
  }

  Future<void> _leave() async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('退出群聊'),
        content: const Text('退群后将收不到后续消息。确定退出？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('退出'),
          ),
        ],
      ),
    );
    if (confirmed ?? false) {
      await _run(() => _runtime.leaveGroup(widget.groupId));
      if (mounted) Navigator.of(context).pop();
    }
  }

  Future<List<String>?> _pickContacts(Set<String> exclude) async {
    List<UserContact> contacts;
    try {
      contacts = await UserContactService().getContacts();
    } catch (_) {
      contacts = const <UserContact>[];
    }
    final selectable =
        contacts.where((c) => !exclude.contains(c.accountId)).toList();
    if (!mounted) return null;
    final chosen = <String>{};
    return showDialog<List<String>>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setLocal) => AlertDialog(
          title: const Text('添加成员'),
          content: SizedBox(
            width: double.maxFinite,
            child: selectable.isEmpty
                ? const Text('没有可添加的联系人')
                : ListView(
                    shrinkWrap: true,
                    children: [
                      for (final contact in selectable)
                        CheckboxListTile(
                          value: chosen.contains(contact.accountId),
                          onChanged: (value) => setLocal(() {
                            if (value ?? false) {
                              chosen.add(contact.accountId);
                            } else {
                              chosen.remove(contact.accountId);
                            }
                          }),
                          title: Text(
                            contact.contactName.isEmpty
                                ? _short(contact.accountId)
                                : contact.contactName,
                          ),
                        ),
                    ],
                  ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('取消'),
            ),
            TextButton(
              onPressed: () =>
                  Navigator.of(context).pop(chosen.toList(growable: false)),
              child: const Text('添加'),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final group = _group;
    return Scaffold(
      appBar: AppBar(
        title: Text(group?.name ?? '群聊'),
        actions: [
          if (_isAdmin)
            IconButton(
              tooltip: '改群名',
              icon: const Icon(Icons.edit_outlined),
              onPressed: _busy ? null : _rename,
            ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : group == null
              ? const Center(child: Text('群不存在或已退出'))
              : Column(
                  children: [
                    if (_error != null)
                      Padding(
                        padding: const EdgeInsets.all(16),
                        child: Text(
                          _error!,
                          style: TextStyle(
                              color: Theme.of(context).colorScheme.error),
                        ),
                      ),
                    Padding(
                      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
                      child: Row(
                        children: [
                          Text('成员 ${group.roster.length} / 1989',
                              style: Theme.of(context).textTheme.titleSmall),
                          const Spacer(),
                          if (_isAdmin)
                            TextButton.icon(
                              onPressed: _busy ? null : _addMembers,
                              icon: const Icon(Icons.person_add_alt_1),
                              label: const Text('添加'),
                            ),
                        ],
                      ),
                    ),
                    Expanded(
                      child: ListView(
                        children: [
                          for (final member in group.roster)
                            ListTile(
                              leading: CircleAvatar(
                                child: Text(
                                  member.accountId.isEmpty
                                      ? '?'
                                      : member.accountId.substring(0, 1),
                                ),
                              ),
                              title: Text(_short(member.accountId)),
                              subtitle:
                                  member.isAdmin ? const Text('管理员') : null,
                              trailing: (_isAdmin &&
                                      member.accountId != _myAccount &&
                                      member.accountId !=
                                          group.creatorAccountId)
                                  ? IconButton(
                                      tooltip: '移除',
                                      icon: const Icon(
                                          Icons.remove_circle_outline),
                                      onPressed: _busy
                                          ? null
                                          : () => _run(
                                              () => _runtime.removeGroupMembers(
                                                    groupId: widget.groupId,
                                                    targetAccounts: [
                                                      member.accountId
                                                    ],
                                                  )),
                                    )
                                  : null,
                            ),
                        ],
                      ),
                    ),
                    SafeArea(
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: OutlinedButton.icon(
                          onPressed: _busy ? null : _leave,
                          icon: const Icon(Icons.exit_to_app),
                          label: const Text('退出群聊'),
                          style: OutlinedButton.styleFrom(
                            foregroundColor:
                                Theme.of(context).colorScheme.error,
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
    );
  }
}

String _short(String address) {
  if (address.length <= 14) return address;
  return '${address.substring(0, 8)}…${address.substring(address.length - 6)}';
}
