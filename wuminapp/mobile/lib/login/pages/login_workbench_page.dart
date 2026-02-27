import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/login/pages/login_debug_page.dart';
import 'package:wuminapp_mobile/login/pages/qr_scan_page.dart';

class LoginWorkbenchPage extends StatelessWidget {
  const LoginWorkbenchPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('扫码登录工作台'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Card(
            child: ListTile(
              leading: const Icon(Icons.qr_code_scanner),
              title: const Text('扫码登录'),
              subtitle: const Text('扫描系统挑战码并生成离线登录回执二维码'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(builder: (_) => const QrScanPage()),
                );
              },
            ),
          ),
          Card(
            child: ListTile(
              leading: const Icon(Icons.developer_mode),
              title: const Text('开发调试'),
              subtitle: const Text('可视化解析挑战、预览签名原文、生成回执 JSON'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                Navigator.of(context).push(
                  MaterialPageRoute(builder: (_) => const LoginDebugPage()),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}
