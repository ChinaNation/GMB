import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/pages/my_wallet_page.dart';
import 'package:wuminapp_mobile/pages/settings_page.dart';

class ProfilePage extends StatelessWidget {
  const ProfilePage({super.key});

  static const Color _inkGreen = Color(0xFF0B3D2E);

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Card(
          child: ListTile(
            leading: SvgPicture.asset(
              'assets/icons/wallet.svg',
              width: 22,
              height: 22,
              colorFilter: const ColorFilter.mode(_inkGreen, BlendMode.srcIn),
            ),
            title: const Text(
              '钱包',
              style: TextStyle(fontWeight: FontWeight.w700),
            ),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const MyWalletPage()),
              );
            },
          ),
        ),
        Card(
          child: ListTile(
            leading: const Icon(
              Icons.settings_outlined,
              color: _inkGreen,
              size: 22,
            ),
            title: const Text(
              '设置',
              style: TextStyle(fontWeight: FontWeight.w700),
            ),
            trailing: const Icon(Icons.chevron_right),
            onTap: () {
              Navigator.of(context).push(
                MaterialPageRoute(builder: (_) => const SettingsPage()),
              );
            },
          ),
        ),
      ],
    );
  }
}
