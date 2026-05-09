import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:wuminapp_mobile/admins_change/codec/subject_id_codec.dart';

class AdminSetDiffCard extends StatelessWidget {
  const AdminSetDiffCard({
    super.key,
    required this.currentAdmins,
    required this.newAdmins,
  });

  final List<String> currentAdmins;
  final List<String> newAdmins;

  @override
  Widget build(BuildContext context) {
    final current = currentAdmins.map(AdminSubjectIdCodec.normalizeHex).toSet();
    final next = newAdmins.map(AdminSubjectIdCodec.normalizeHex).toSet();
    final added = next.where((item) => !current.contains(item)).toList();
    final removed = current.where((item) => !next.contains(item)).toList();
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
            Text('新增：${added.isEmpty ? '无' : added.map(_ss58).join('，')}'),
            const SizedBox(height: 4),
            Text('移除：${removed.isEmpty ? '无' : removed.map(_ss58).join('，')}'),
          ],
        ),
      ),
    );
  }

  static String _ss58(String hex) {
    return Keyring().encodeAddress(AdminSubjectIdCodec.hexDecode(hex), 2027);
  }
}
