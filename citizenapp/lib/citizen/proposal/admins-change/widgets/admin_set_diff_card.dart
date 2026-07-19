import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/shared/account_derivation.dart';
import 'package:citizenapp/my/util/amount_format.dart';

class AdminSetDiffCard extends StatelessWidget {
  const AdminSetDiffCard({
    super.key,
    required this.currentAdmins,
    required this.admins,
    this.balances = const {},
  });

  final List<AdminPerson> currentAdmins;
  final List<AdminPerson> admins;
  final Map<String, double> balances;

  @override
  Widget build(BuildContext context) {
    final current = {
      for (final admin in currentAdmins)
        AdminAccountIdCodec.normalizeHex(admin.admin_account): admin,
    };
    final next = {
      for (final admin in admins)
        AdminAccountIdCodec.normalizeHex(admin.admin_account): admin,
    };
    final added = next.entries
        .where((entry) => !current.containsKey(entry.key))
        .map((entry) => entry.value)
        .toList();
    final removed = current.entries
        .where((entry) => !next.containsKey(entry.key))
        .map((entry) => entry.value)
        .toList();
    final renamed = next.entries
        .where((entry) {
          final old = current[entry.key];
          return old != null &&
              (old.family_name != entry.value.family_name ||
                  old.given_name != entry.value.given_name);
        })
        .map((entry) => entry.value)
        .toList();
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text('变更差异',
                style: TextStyle(fontSize: 15, fontWeight: FontWeight.w700)),
            const SizedBox(height: 8),
            _buildAccountList('新增', added),
            const SizedBox(height: 8),
            _buildAccountList('移除', removed),
            const SizedBox(height: 8),
            _buildAccountList('姓名调整', renamed),
          ],
        ),
      ),
    );
  }

  Widget _buildAccountList(String title, List<AdminPerson> admins) {
    if (admins.isEmpty) return Text('$title：无');
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('$title：'),
        const SizedBox(height: 6),
        for (final admin in admins) ...[
          ListTile(
            dense: true,
            title: Text('${admin.family_name}${admin.given_name}'),
            subtitle: Text(
              '${ss58FromHex(admin.admin_account)}\n'
              '余额：${AmountFormat.formatThousands(balances[AdminAccountIdCodec.normalizeHex(admin.admin_account)])} 元',
            ),
          ),
          const SizedBox(height: 6),
        ],
      ],
    );
  }
}
