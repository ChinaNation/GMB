import 'package:flutter/material.dart';

class AdminsChangeConfirmPage extends StatelessWidget {
  const AdminsChangeConfirmPage({super.key, required this.txHash});

  final String txHash;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('管理员更换')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.check_circle_outline,
                  size: 56, color: Colors.green),
              const SizedBox(height: 12),
              const Text('管理员更换提案已提交'),
              const SizedBox(height: 8),
              SelectableText(txHash, textAlign: TextAlign.center),
              const SizedBox(height: 16),
              FilledButton(
                onPressed: () => Navigator.of(context).pop(true),
                child: const Text('完成'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
