import 'package:flutter/material.dart';

import 'package:citizenapp/governance/admins-change/models/admin_account.dart';
import 'package:citizenapp/ui/app_theme.dart';

class AdminAccountCard extends StatelessWidget {
  const AdminAccountCard({super.key, required this.account});

  final AdminAccountState account;

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 0,
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(account.kindLabel,
                      style: const TextStyle(
                          fontSize: 16, fontWeight: FontWeight.w700)),
                  const SizedBox(height: 4),
                  Text('管理员 ${account.admins.length} 人，阈值 ${account.threshold}',
                      style: const TextStyle(color: AppTheme.textSecondary)),
                ],
              ),
            ),
            Chip(label: Text(account.statusLabel)),
          ],
        ),
      ),
    );
  }
}
