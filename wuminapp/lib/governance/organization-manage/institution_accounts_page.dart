import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;

import 'package:wuminapp_mobile/governance/shared/institution_info.dart';
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/smoldot_client.dart';
import 'package:wuminapp_mobile/transaction/shared/account_balance_snapshot_store.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 治理机构全部账户页:主账户 + 费用 / 安全基金 / 两和基金 / 永久质押,余额批量。
///
/// 中文注释:对标公权 `PublicInstitutionAccountsPage`,使两端「机构账户」入口体验
/// 统一(独立一行 + 箭头进本页)。余额先读本地快照,未命中地址一次 `fetchFinalizedBalances`
/// 批量查链,避免逐条 RPC(N+1)。
class GovernanceInstitutionAccountsPage extends StatefulWidget {
  const GovernanceInstitutionAccountsPage({
    super.key,
    required this.institution,
    required this.badgeColor,
  });

  final InstitutionInfo institution;
  final Color badgeColor;

  @override
  State<GovernanceInstitutionAccountsPage> createState() =>
      _GovernanceInstitutionAccountsPageState();
}

class _GovernanceInstitutionAccountsPageState
    extends State<GovernanceInstitutionAccountsPage> {
  final ChainRpc _chainRpc = ChainRpc();

  late final List<({String name, String hex, IconData icon})> _sources =
      _accountSources();

  Map<String, double> _balances = const {};
  bool _loading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadBalances();
  }

  List<({String name, String hex, IconData icon})> _accountSources() {
    final accounts = widget.institution.accounts;
    final items = <({String name, String hex, IconData icon})>[
      (
        name: '主账户',
        hex: widget.institution.mainAddress,
        icon: Icons.account_balance_wallet_outlined,
      ),
    ];
    final fee = accounts?.feeAddress;
    if (fee != null) {
      items.add((name: '费用账户', hex: fee, icon: Icons.receipt_long_outlined));
    }
    final safety = accounts?.safetyFundAddress;
    if (safety != null) {
      items.add((
        name: '安全基金账户',
        hex: safety,
        icon: Icons.health_and_safety_outlined,
      ));
    }
    final he = accounts?.heFundAddress;
    if (he != null) {
      items.add((name: '两和基金账户', hex: he, icon: Icons.handshake_outlined));
    }
    final stake = accounts?.stakeAddress;
    if (stake != null) {
      items.add((name: '永久质押', hex: stake, icon: Icons.lock_outline));
    }
    return items;
  }

  Future<void> _loadBalances() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final store = AccountBalanceSnapshotStore.instance;
      final cached = <String, double>{};
      final toFetch = <String>[];
      for (final s in _sources) {
        final local = await store.readFresh(s.hex);
        if (local != null) {
          cached[s.hex] = local.balanceYuan;
        } else {
          toFetch.add(s.hex);
        }
      }
      final fetched = toFetch.isEmpty
          ? const <String, double>{}
          : await _chainRpc.fetchFinalizedBalances(toFetch);
      for (final entry in fetched.entries) {
        try {
          await store.put(accountHex: entry.key, balanceYuan: entry.value);
        } catch (_) {
          // 快照写入失败不影响展示。
        }
      }
      if (!mounted) return;
      setState(() {
        _balances = {...cached, ...fetched};
        _loading = false;
      });
    } on Exception catch (e) {
      if (!mounted) return;
      setState(() {
        _error = SmoldotClientManager.instance.buildUserFacingError(e);
        _loading = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.scaffoldBg,
      appBar: AppBar(
        title: const Text('全部账户'),
        backgroundColor: AppTheme.surfaceWhite,
        foregroundColor: AppTheme.textPrimary,
        elevation: 0,
      ),
      body: RefreshIndicator(
        onRefresh: _loadBalances,
        child: ListView.separated(
          padding: const EdgeInsets.all(16),
          itemCount: _sources.length,
          separatorBuilder: (_, __) => const SizedBox(height: 10),
          itemBuilder: (context, i) => _accountCard(_sources[i]),
        ),
      ),
    );
  }

  Widget _accountCard(({String name, String hex, IconData icon}) source) {
    final balance = _balances[source.hex];
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppTheme.surfaceCard,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: AppTheme.border),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Container(
                width: 28,
                height: 28,
                decoration: BoxDecoration(
                  color: widget.badgeColor.withValues(alpha: 0.08),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Icon(source.icon, size: 15, color: widget.badgeColor),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(source.name,
                    style: const TextStyle(
                        fontSize: 14,
                        fontWeight: FontWeight.w600,
                        color: AppTheme.textPrimary)),
              ),
              Text(
                _loading && balance == null
                    ? '—'
                    : (balance == null
                        ? '未读取'
                        : AmountFormat.format(balance, symbol: 'GMB')),
                style: const TextStyle(
                    fontSize: 13.5,
                    fontWeight: FontWeight.w700,
                    color: AppTheme.textPrimary),
              ),
            ],
          ),
          const SizedBox(height: 6),
          // 完整 SS58 地址,允许换行,不截断。
          Text(_accountHexToSs58(source.hex),
              style: const TextStyle(
                  fontSize: 11.5, color: AppTheme.textTertiary)),
          if (_error != null) ...[
            const SizedBox(height: 6),
            Text(_error!,
                style: const TextStyle(fontSize: 11.5, color: AppTheme.danger)),
          ],
        ],
      ),
    );
  }

  String _accountHexToSs58(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    final bytes = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < bytes.length; i++) {
      bytes[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return Keyring().encodeAddress(bytes, 2027);
  }
}
