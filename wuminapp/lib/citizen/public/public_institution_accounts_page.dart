import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/citizen/public/data/public_institution_accounts.dart';
import 'package:wuminapp_mobile/citizen/public/data/public_institution_chain_data.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';

/// 机构全部账户页(ADR-018 §九 卡C):主 + 费 + 自定义,余额批量。
class PublicInstitutionAccountsPage extends StatefulWidget {
  const PublicInstitutionAccountsPage({
    super.key,
    required this.institution,
    required this.chainData,
  });

  final PublicInstitutionEntity institution;
  final PublicInstitutionChainData chainData;

  @override
  State<PublicInstitutionAccountsPage> createState() =>
      _PublicInstitutionAccountsPageState();
}

class _PublicInstitutionAccountsPageState
    extends State<PublicInstitutionAccountsPage> {
  late List<PublicAccountRow> _rows = deriveAccountRows(widget.institution);
  bool _balanceLoading = true;

  @override
  void initState() {
    super.initState();
    _loadBalances();
  }

  Future<void> _loadBalances() async {
    try {
      final balances = await widget.chainData
          .balances(_rows.map((r) => r.addressHex).toList());
      if (!mounted) return;
      setState(() {
        _rows = _rows
            .map((r) => r.withBalance(balances[r.addressHex]))
            .toList(growable: false);
        _balanceLoading = false;
      });
    } on Exception {
      if (!mounted) return;
      setState(() => _balanceLoading = false);
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
      body: ListView.separated(
        padding: const EdgeInsets.all(16),
        itemCount: _rows.length,
        separatorBuilder: (_, __) => const SizedBox(height: 10),
        itemBuilder: (context, i) => _AccountCard(
          row: _rows[i],
          balanceLoading: _balanceLoading,
        ),
      ),
    );
  }
}

class _AccountCard extends StatelessWidget {
  const _AccountCard({required this.row, required this.balanceLoading});

  final PublicAccountRow row;
  final bool balanceLoading;

  @override
  Widget build(BuildContext context) {
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
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(row.label,
                  style: const TextStyle(
                      fontSize: 14,
                      fontWeight: FontWeight.w600,
                      color: AppTheme.textPrimary)),
              Text(
                balanceLoading
                    ? '—'
                    : (row.balanceYuan == null
                        ? '未激活'
                        : '${row.balanceYuan!.toStringAsFixed(2)} 元'),
                style: const TextStyle(
                    fontSize: 13.5,
                    fontWeight: FontWeight.w600,
                    color: AppTheme.primary),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Text(row.addressSs58,
              style: const TextStyle(
                  fontSize: 11.5, color: AppTheme.textTertiary)),
        ],
      ),
    );
  }
}
