import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/my/util/amount_format.dart';

class AdminSetEditor extends StatefulWidget {
  const AdminSetEditor({
    super.key,
    required this.admins,
    this.balances = const {},
    required this.onChanged,
  });

  final List<AdminPerson> admins;
  final Map<String, double> balances;
  final ValueChanged<List<AdminPerson>> onChanged;

  @override
  State<AdminSetEditor> createState() => _AdminSetEditorState();
}

class _AdminSetEditorState extends State<AdminSetEditor> {
  final _accountController = TextEditingController();
  final _familyNameController = TextEditingController(text: '管理');
  final _givenNameController = TextEditingController(text: '员');

  @override
  void dispose() {
    _accountController.dispose();
    _familyNameController.dispose();
    _givenNameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          children: [
            for (var i = 0; i < widget.admins.length; i++) ...[
              _buildAdminEditor(i),
              const SizedBox(height: 8),
            ],
            TextField(
              controller: _accountController,
              decoration: const InputDecoration(labelText: '管理员公钥 hex'),
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _familyNameController,
                    decoration: const InputDecoration(labelText: '姓'),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: _givenNameController,
                    decoration: const InputDecoration(labelText: '名'),
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

  Widget _buildAdminEditor(int index) {
    final admin = widget.admins[index];
    return Container(
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        border: Border.all(color: Theme.of(context).dividerColor),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Text('${index + 1}'),
              const SizedBox(width: 8),
              Expanded(child: Text(ss58FromHex(admin.admin_account))),
              IconButton(
                padding: EdgeInsets.zero,
                tooltip: '移除',
                icon: const Icon(Icons.remove_circle_outline),
                onPressed: () => widget.onChanged([
                  for (var i = 0; i < widget.admins.length; i++)
                    if (i != index) widget.admins[i],
                ]),
              ),
            ],
          ),
          Text(
            '余额：${AmountFormat.formatThousands(widget.balances[AdminAccountIdCodec.normalizeHex(admin.admin_account)])} 元',
          ),
          const SizedBox(height: 8),
          Row(
            children: [
              Expanded(
                child: TextFormField(
                  key: ValueKey('${admin.admin_account}-$index-family'),
                  initialValue: admin.family_name,
                  decoration: const InputDecoration(labelText: '姓'),
                  onChanged: (value) => _updateName(
                    index,
                    admin.copyWith(family_name: value),
                  ),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextFormField(
                  key: ValueKey('${admin.admin_account}-$index-given'),
                  initialValue: admin.given_name,
                  decoration: const InputDecoration(labelText: '名'),
                  onChanged: (value) => _updateName(
                    index,
                    admin.copyWith(given_name: value),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _updateName(int index, AdminPerson next) {
    widget.onChanged([
      for (var i = 0; i < widget.admins.length; i++)
        if (i == index) next else widget.admins[i],
    ]);
  }

  void _add() {
    final clean = AdminAccountIdCodec.normalizeHex(_accountController.text);
    if (clean.length != 64 ||
        widget.admins.any((admin) => admin.admin_account == clean)) {
      return;
    }
    widget.onChanged([
      ...widget.admins,
      AdminPerson(
        admin_account: clean,
        family_name: _familyNameController.text.trim().isEmpty
            ? '管理'
            : _familyNameController.text.trim(),
        given_name: _givenNameController.text.trim().isEmpty
            ? '员'
            : _givenNameController.text.trim(),
      ),
    ]);
    _accountController.clear();
    _familyNameController.text = '管理';
    _givenNameController.text = '员';
  }
}
