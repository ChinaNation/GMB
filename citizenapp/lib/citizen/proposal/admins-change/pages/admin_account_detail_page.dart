import 'dart:async';

import 'package:flutter/material.dart';

import 'package:citizenapp/citizen/proposal/admins-change/models/admin_account.dart';
import 'package:citizenapp/citizen/proposal/admins-change/widgets/admin_account_card.dart';
import 'package:citizenapp/citizen/shared/admin_profile_card.dart';
import 'package:citizenapp/rpc/chain_rpc.dart';

class AdminAccountDetailPage extends StatefulWidget {
  const AdminAccountDetailPage({super.key, required this.account});

  final AdminAccountState account;

  @override
  State<AdminAccountDetailPage> createState() => _AdminAccountDetailPageState();
}

class _AdminAccountDetailPageState extends State<AdminAccountDetailPage> {
  Map<String, double> _balanceByAccount = const {};

  @override
  void initState() {
    super.initState();
    unawaited(_loadBalances());
  }

  static String _balanceKey(String account) {
    final trimmed = account.trim();
    return (trimmed.startsWith('0x') || trimmed.startsWith('0X')
            ? trimmed.substring(2)
            : trimmed)
        .toLowerCase();
  }

  Future<void> _loadBalances() async {
    final accounts = {
      for (final profile in widget.account.profiles)
        _balanceKey(profile.account),
    }.where((account) => account.isNotEmpty).toList(growable: false);
    if (accounts.isEmpty) return;
    try {
      final balances = await ChainRpc().fetchFinalizedBalances(accounts);
      if (mounted) setState(() => _balanceByAccount = balances);
    } catch (_) {
      // 中文注释:详情页余额失败只让余额值留空,不影响管理员资料展示。
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('管理员账户')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          AdminAccountCard(account: widget.account),
          const SizedBox(height: 12),
          for (var i = 0; i < widget.account.profiles.length; i++) ...[
            AdminProfileCard(
              profile: widget.account.profiles[i],
              index: i + 1,
              balanceYuan: _balanceByAccount[
                  _balanceKey(widget.account.profiles[i].account)],
            ),
            const SizedBox(height: 8),
          ],
        ],
      ),
    );
  }
}
