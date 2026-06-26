import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/widgets/admin_account_card.dart';

class AdminAccountDetailPage extends StatelessWidget {
  const AdminAccountDetailPage({super.key, required this.account});

  final AdminAccountState account;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('管理员账户')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          AdminAccountCard(account: account),
          const SizedBox(height: 12),
          for (final admin in account.admins)
            ListTile(title: Text(admin), dense: true),
        ],
      ),
    );
  }
}
