import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/codec/account_id_codec.dart';
import 'package:citizenapp/citizen/shared/admin_profile.dart';
import 'package:citizenapp/citizen/shared/admin_profile_card.dart';

class AdminSetDiffCard extends StatelessWidget {
  const AdminSetDiffCard({
    super.key,
    required this.currentAdmins,
    required this.admins,
    this.currentProfiles = const [],
    this.balances = const {},
  });

  final List<String> currentAdmins;
  final List<String> admins;
  final List<AdminProfile> currentProfiles;
  final Map<String, double> balances;

  @override
  Widget build(BuildContext context) {
    final current = currentAdmins.map(AdminAccountIdCodec.normalizeHex).toSet();
    final next = admins.map(AdminAccountIdCodec.normalizeHex).toSet();
    final added = next.where((item) => !current.contains(item)).toList();
    final removed = current.where((item) => !next.contains(item)).toList();
    final profilesByAccount = {
      for (final profile in currentProfiles)
        AdminAccountIdCodec.normalizeHex(profile.account): profile,
    };
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
            _buildProfileList('新增', added, profilesByAccount),
            const SizedBox(height: 8),
            _buildProfileList('移除', removed, profilesByAccount),
          ],
        ),
      ),
    );
  }

  Widget _buildProfileList(
    String title,
    List<String> accounts,
    Map<String, AdminProfile> profilesByAccount,
  ) {
    if (accounts.isEmpty) return Text('$title：无');
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('$title：'),
        const SizedBox(height: 6),
        for (final account in accounts) ...[
          AdminProfileCard(
            profile:
                profilesByAccount[account] ?? AdminProfile(account: account),
            compact: true,
            balanceYuan: balances[AdminAccountIdCodec.normalizeHex(account)],
          ),
          const SizedBox(height: 6),
        ],
      ],
    );
  }
}
