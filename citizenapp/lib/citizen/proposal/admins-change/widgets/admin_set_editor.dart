import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/citizen/shared/admin_profile_card.dart';

class AdminSetEditor extends StatefulWidget {
  const AdminSetEditor({
    super.key,
    required this.admins,
    this.profiles = const [],
    this.balances = const {},
    required this.onChanged,
  });

  final List<String> admins;
  final List<AdminProfile> profiles;
  final Map<String, double> balances;
  final ValueChanged<List<String>> onChanged;

  @override
  State<AdminSetEditor> createState() => _AdminSetEditorState();
}

class _AdminSetEditorState extends State<AdminSetEditor> {
  final _controller = TextEditingController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final profilesByAccount = {
      for (final profile in widget.profiles)
        AdminAccountIdCodec.normalizeHex(profile.account): profile,
    };
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          children: [
            for (var i = 0; i < widget.admins.length; i++) ...[
              AdminProfileCard(
                profile: profilesByAccount[
                        AdminAccountIdCodec.normalizeHex(widget.admins[i])] ??
                    AdminProfile(
                      account: AdminAccountIdCodec.normalizeHex(
                        widget.admins[i],
                      ),
                    ),
                index: i + 1,
                compact: true,
                balanceYuan: widget.balances[
                    AdminAccountIdCodec.normalizeHex(widget.admins[i])],
                trailing: IconButton(
                  constraints: const BoxConstraints.tightFor(
                    width: AdminProfileCard.actionHeight,
                    height: AdminProfileCard.actionHeight,
                  ),
                  padding: EdgeInsets.zero,
                  tooltip: '移除',
                  icon: const Icon(Icons.remove_circle_outline),
                  onPressed: () => widget.onChanged([
                    for (final admin in widget.admins)
                      if (admin != widget.admins[i]) admin,
                  ]),
                ),
              ),
              const SizedBox(height: 8),
            ],
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _controller,
                    decoration: const InputDecoration(labelText: '管理员公钥 hex'),
                  ),
                ),
                const SizedBox(width: 8),
                FilledButton(onPressed: _add, child: const Text('添加')),
              ],
            ),
          ],
        ),
      ),
    );
  }

  void _add() {
    final clean = AdminAccountIdCodec.normalizeHex(_controller.text);
    if (clean.length != 64 || widget.admins.contains(clean)) return;
    widget.onChanged([...widget.admins, clean]);
    _controller.clear();
  }
}
