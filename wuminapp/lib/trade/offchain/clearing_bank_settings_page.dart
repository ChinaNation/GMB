import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 「设置清算行」占位页。
///
/// 中文注释:
/// - 从钱包详情右上角三点菜单「清算行」入口进入。
/// - 本轮**仅占位**:顶部搜索框 + 空列表 + 「暂无结果」文案,**不接 API,不放
///   假数据,不读写任何缓存**。等后续清算行需求细化后再补实际数据源和交互。
/// - 构造函数保留 [wallet] 参数,后续接真实列表时(绑定 / 搜索)会用到当前
///   钱包的 SS58 / pubkey 作为入参。
class ClearingBankSettingsPage extends StatefulWidget {
  const ClearingBankSettingsPage({super.key, required this.wallet});

  final WalletProfile wallet;

  @override
  State<ClearingBankSettingsPage> createState() =>
      _ClearingBankSettingsPageState();
}

class _ClearingBankSettingsPageState extends State<ClearingBankSettingsPage> {
  final TextEditingController _searchCtrl = TextEditingController();

  /// 搜索关键词,当前仅用于 UI 占位 setState,不参与任何查询。
  // ignore: unused_field
  String _query = '';

  @override
  void dispose() {
    _searchCtrl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('设置清算行'),
        centerTitle: true,
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: TextField(
              controller: _searchCtrl,
              decoration: const InputDecoration(
                hintText: '搜索清算行',
                prefixIcon: Icon(Icons.search),
                isDense: true,
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.all(Radius.circular(8)),
                ),
              ),
              onChanged: (value) {
                setState(() {
                  _query = value;
                });
              },
            ),
          ),
          const Expanded(
            child: Center(
              child: Text(
                '暂无结果',
                style: TextStyle(color: AppTheme.textTertiary),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
