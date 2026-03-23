import 'package:flutter/material.dart';

import '../util/sensitive_page_mixin.dart';
import '../wallet/wallet_manager.dart';

/// 创建新钱包页面。
///
/// 创建成功后展示助记词，要求用户确认已备份。
class CreateWalletPage extends StatefulWidget {
  const CreateWalletPage({super.key});

  @override
  State<CreateWalletPage> createState() => _CreateWalletPageState();
}

class _CreateWalletPageState extends State<CreateWalletPage>
    with SensitivePageMixin {
  final WalletManager _walletManager = WalletManager();
  bool _creating = false;
  int _wordCount = 12;
  WalletCreationResult? _result;

  Future<void> _create() async {
    setState(() => _creating = true);
    try {
      final result = await _walletManager.createWallet(wordCount: _wordCount);
      if (!mounted) return;
      setState(() => _result = result);
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('创建失败：$e')),
      );
    } finally {
      if (mounted) setState(() => _creating = false);
    }
  }

  void _confirmBackup() {
    Navigator.of(context).pop(true);
  }

  @override
  Widget build(BuildContext context) {
    // 截屏/录屏时隐藏助记词展示
    if (sensitiveContentHidden && _result != null) {
      return buildHiddenPlaceholder(message: '助记词已隐藏');
    }
    return Scaffold(
      appBar: AppBar(
        title: const Text('创建钱包'),
        centerTitle: true,
      ),
      body: _result != null ? _buildMnemonicView() : _buildCreateView(),
    );
  }

  Widget _buildCreateView() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.add_circle_outline,
              size: 64,
              color: Theme.of(context).colorScheme.primary,
            ),
            const SizedBox(height: 16),
            const Text(
              '创建新钱包',
              style: TextStyle(fontSize: 20, fontWeight: FontWeight.w700),
            ),
            const SizedBox(height: 8),
            const Text(
              '将生成一组助记词，请务必安全保存',
              style: TextStyle(color: Colors.black54),
            ),
            const SizedBox(height: 24),
            SegmentedButton<int>(
              segments: const [
                ButtonSegment(value: 12, label: Text('12 个单词')),
                ButtonSegment(value: 24, label: Text('24 个单词')),
              ],
              selected: {_wordCount},
              onSelectionChanged: (v) => setState(() => _wordCount = v.first),
            ),
            const SizedBox(height: 8),
            Text(
              _wordCount == 24 ? '256 位熵，安全性更高' : '128 位熵，标准安全强度',
              style: TextStyle(color: Colors.grey.shade600, fontSize: 12),
            ),
            const SizedBox(height: 24),
            SizedBox(
              width: double.infinity,
              child: FilledButton(
                onPressed: _creating ? null : _create,
                child: Text(_creating ? '创建中...' : '创建钱包'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMnemonicView() {
    final result = _result!;
    final words = result.mnemonic.split(' ');
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          decoration: BoxDecoration(
            color: Colors.orange.shade50,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: Colors.orange.shade200),
          ),
          child: Row(
            children: [
              Icon(Icons.warning_amber, color: Colors.orange.shade700, size: 20),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  '请安全保存助记词，这是恢复钱包的唯一凭证',
                  style: TextStyle(
                    color: Colors.orange.shade700,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  '助记词（请手抄备份，不支持复制）',
                  style: TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 12),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: List.generate(words.length, (i) {
                    return Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 10,
                        vertical: 6,
                      ),
                      decoration: BoxDecoration(
                        color: Colors.grey.shade100,
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: Text(
                        '${i + 1}. ${words[i]}',
                        style: const TextStyle(
                          fontFamily: 'monospace',
                          fontWeight: FontWeight.w500,
                        ),
                      ),
                    );
                  }),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        Card(
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  result.profile.walletName,
                  style: const TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  result.profile.address,
                  style: TextStyle(
                    fontSize: 13,
                    color: Colors.grey.shade600,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 24),
        SizedBox(
          width: double.infinity,
          child: FilledButton(
            onPressed: _confirmBackup,
            child: const Text('已备份，完成'),
          ),
        ),
      ],
    );
  }
}
