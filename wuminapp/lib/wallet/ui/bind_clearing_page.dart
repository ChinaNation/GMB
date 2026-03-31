import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_banks.dart';

/// 绑定清算省储行选择页面。
///
/// 用户从 43 个省储行中选择一个作为清算行。
/// 选择后返回 [ClearingBank] 给上层页面。
class BindClearingPage extends StatefulWidget {
  const BindClearingPage({
    super.key,
    this.currentShenfenId,
  });

  /// 当前已绑定的省储行 shenfen_id（高亮显示）。
  final String? currentShenfenId;

  @override
  State<BindClearingPage> createState() => _BindClearingPageState();
}

class _BindClearingPageState extends State<BindClearingPage> {
  String _searchText = '';

  List<ClearingBank> get _filteredBanks {
    if (_searchText.isEmpty) return clearingBanks;
    return clearingBanks
        .where((b) => b.shenfenName.contains(_searchText))
        .toList();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('选择清算省储行'),
        centerTitle: true,
      ),
      body: Column(
        children: [
          // 搜索框
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
            child: TextField(
              onChanged: (v) => setState(() => _searchText = v.trim()),
              decoration: const InputDecoration(
                hintText: '搜索省储行名称',
                prefixIcon: Icon(Icons.search, size: 20),
                isDense: true,
              ),
            ),
          ),
          // 省储行列表
          Expanded(
            child: ListView.separated(
              itemCount: _filteredBanks.length,
              separatorBuilder: (_, __) =>
                  const Divider(height: 1, indent: 16, endIndent: 16),
              itemBuilder: (context, index) {
                final bank = _filteredBanks[index];
                final isCurrent = bank.shenfenId == widget.currentShenfenId;
                return ListTile(
                  title: Text(
                    bank.shenfenName,
                    style: TextStyle(
                      fontSize: 15,
                      fontWeight:
                          isCurrent ? FontWeight.w700 : FontWeight.normal,
                      color: isCurrent
                          ? AppTheme.primary
                          : AppTheme.textPrimary,
                    ),
                  ),
                  trailing: isCurrent
                      ? const Icon(Icons.check, color: AppTheme.primary, size: 20)
                      : null,
                  onTap: () {
                    if (isCurrent) {
                      // 已绑定，不重复操作
                      Navigator.pop(context);
                      return;
                    }
                    _confirmBind(bank);
                  },
                );
              },
            ),
          ),
        ],
      ),
    );
  }

  Future<void> _confirmBind(ClearingBank bank) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('确认绑定'),
        content: Text(
          '确认将清算省储行绑定为「${bank.shenfenName}」？\n\n'
          '绑定后 1 年内不可更换。',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('确认绑定'),
          ),
        ],
      ),
    );
    if (confirmed == true && mounted) {
      // TODO: 调用链上 bind_clearing_institution extrinsic
      // 当前先返回选择结果，第2步对接后补充链上调用
      Navigator.pop(context, bank);
    }
  }
}
