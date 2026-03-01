import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/trade/pages/onchain_trade_page.dart';

class TradePage extends StatelessWidget {
  const TradePage({super.key});

  static const Color _inkGreen = Color(0xFF0B3D2E);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Card(
          child: ListTile(
            leading: const Icon(Icons.link_outlined, color: _inkGreen),
            title: const Text(
              '链上交易',
              style: TextStyle(fontWeight: FontWeight.w800),
            ),
            subtitle: const Text('支持转账构建、签名、广播、状态追踪与交易记录。'),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const OnchainTradePage()),
              );
            },
          ),
        ),
        const SizedBox(height: 10),
        const Card(
          child: ListTile(
            leading: Icon(Icons.swap_horiz_outlined, color: _inkGreen),
            title: Text(
              '链下交易（开发中）',
              style: TextStyle(fontWeight: FontWeight.w800),
            ),
            subtitle: Text('将逐步实现订单、撮合、清结算和自动对账。'),
          ),
        ),
      ],
    );
  }
}
