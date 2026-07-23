import 'package:flutter/material.dart';

import 'package:citizenapp/chat/chat_runtime.dart';
import 'package:citizenapp/chat/group/ui/open_group_chat.dart';
import 'package:citizenapp/my/user/contact_service.dart';

/// 建群页:输群名 + 从通讯录多选成员 → `createGroup`(创建者自动为 admin)。
class GroupCreatePage extends StatefulWidget {
  const GroupCreatePage({super.key, this.runtime, this.contactService});

  final ChatRuntime? runtime;
  final UserContactService? contactService;

  @override
  State<GroupCreatePage> createState() => _GroupCreatePageState();
}

class _GroupCreatePageState extends State<GroupCreatePage> {
  final TextEditingController _nameController = TextEditingController();
  late final UserContactService _contactService =
      widget.contactService ?? UserContactService();
  late final ChatRuntime _runtime = widget.runtime ?? ChatRuntime();

  List<UserContact> _contacts = const <UserContact>[];
  final Set<String> _selected = <String>{};
  bool _loading = true;
  bool _creating = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _nameController.addListener(() => setState(() {}));
    _load();
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _load() async {
    try {
      final contacts = await _contactService.getContacts();
      if (!mounted) return;
      setState(() {
        _contacts = contacts;
        _loading = false;
      });
    } catch (error) {
      if (!mounted) return;
      setState(() {
        _error = '加载通讯录失败: $error';
        _loading = false;
      });
    }
  }

  /// 发群聊最少 2 人(1 人应走「发私信」),群名非空方可创建。
  bool get _canCreate =>
      _nameController.text.trim().isNotEmpty && _selected.length >= 2;

  Future<void> _create() async {
    if (!_canCreate || _creating) return;
    setState(() {
      _creating = true;
      _error = null;
    });
    try {
      final group = await _runtime.createGroup(
        name: _nameController.text.trim(),
        inviteeAccounts: _selected.toList(growable: false),
      );
      if (!mounted) return;
      Navigator.of(context).pop();
      await openGroupChat(context, groupId: group.groupId, title: group.name);
    } catch (error) {
      if (!mounted) return;
      setState(() {
        _error = '$error';
        _creating = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('新建群聊'),
        actions: [
          TextButton(
            onPressed: _canCreate && !_creating ? _create : null,
            child: _creating
                ? const SizedBox(
                    width: 18,
                    height: 18,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Text('创建'),
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : Column(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
                  child: TextField(
                    controller: _nameController,
                    maxLength: 40,
                    decoration: const InputDecoration(
                      labelText: '群名称',
                      counterText: '',
                    ),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Row(
                    children: [
                      Text(
                        '选择成员',
                        style: Theme.of(context).textTheme.titleSmall,
                      ),
                      const Spacer(),
                      // 未满 2 人时提示门槛,满足后只显示已选人数。
                      Text(
                        _selected.length < 2
                            ? '已选 ${_selected.length}·至少 2 人'
                            : '已选 ${_selected.length}',
                      ),
                    ],
                  ),
                ),
                if (_error != null)
                  Padding(
                    padding: const EdgeInsets.all(16),
                    child: Text(
                      _error!,
                      style:
                          TextStyle(color: Theme.of(context).colorScheme.error),
                    ),
                  ),
                Expanded(
                  child: _contacts.isEmpty
                      ? const Center(child: Text('通讯录为空,先在「我的 → 通讯录」添加联系人'))
                      : ListView.builder(
                          itemCount: _contacts.length,
                          itemBuilder: (context, index) {
                            final contact = _contacts[index];
                            final checked =
                                _selected.contains(contact.accountId);
                            return CheckboxListTile(
                              value: checked,
                              onChanged: (value) => setState(() {
                                if (value ?? false) {
                                  _selected.add(contact.accountId);
                                } else {
                                  _selected.remove(contact.accountId);
                                }
                              }),
                              title: Text(
                                contact.contactName.isEmpty
                                    ? _short(contact.accountId)
                                    : contact.contactName,
                              ),
                              subtitle: Text(
                                _short(contact.accountId),
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                              ),
                            );
                          },
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
