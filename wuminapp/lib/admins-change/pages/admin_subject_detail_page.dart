import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/admins-change/models/admin_subject.dart';
import 'package:wuminapp_mobile/admins-change/widgets/admin_subject_card.dart';

class AdminSubjectDetailPage extends StatelessWidget {
  const AdminSubjectDetailPage({super.key, required this.subject});

  final AdminSubjectState subject;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('管理员主体')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          AdminSubjectCard(subject: subject),
          const SizedBox(height: 12),
          for (final admin in subject.admins)
            ListTile(title: Text(admin), dense: true),
        ],
      ),
    );
  }
}
