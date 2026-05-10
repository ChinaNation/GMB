import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/governance/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

class AdminSubjectCard extends StatelessWidget {
  const AdminSubjectCard({super.key, required this.subject});

  final AdminSubjectState subject;

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
                  Text(subject.kindLabel,
                      style: const TextStyle(
                          fontSize: 16, fontWeight: FontWeight.w700)),
                  const SizedBox(height: 4),
                  Text('管理员 ${subject.admins.length} 人，阈值 ${subject.threshold}',
                      style: const TextStyle(color: AppTheme.textSecondary)),
                ],
              ),
            ),
            Chip(label: Text(subject.statusLabel)),
          ],
        ),
      ),
    );
  }
}
