import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/trade/pages/onchain_trade_page.dart';

class TradePage extends StatelessWidget {
  const TradePage({super.key});

  static const Color _inkGreen = Color(0xFF0B3D2E);

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      child: Column(
        children: [
          const SizedBox(height: 14),
          const Text(
            '金融',
            style: TextStyle(fontSize: 18, fontWeight: FontWeight.w700),
          ),
          Expanded(
            child: ListView(
              padding: const EdgeInsets.all(16),
              children: [
                Card(
                  child: ListTile(
                    leading: const Icon(Icons.link_outlined, color: _inkGreen),
                    title: const Text(
                      '链上交易',
                      style: TextStyle(fontWeight: FontWeight.w800),
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(
                            builder: (_) => const OnchainTradePage()),
                      );
                    },
                  ),
                ),
                const SizedBox(height: 10),
                const Card(
                  child: ListTile(
                    leading: Icon(
                      Icons.swap_horiz_outlined,
                      color: Colors.grey,
                    ),
                    title: Text(
                      '链下交易（开发中）',
                      style: TextStyle(
                        fontWeight: FontWeight.w800,
                        color: Colors.grey,
                      ),
                    ),
                    trailing: Icon(Icons.chevron_right, color: Colors.grey),
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
