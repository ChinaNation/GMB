import 'package:flutter/material.dart';

import 'package:wuminapp_mobile/rpc/clearing_bank_directory.dart';
import 'package:wuminapp_mobile/trade/offchain/bind_clearing_bank_page.dart';
import 'package:wuminapp_mobile/trade/offchain/clearing_bank_prefs.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 「设置清算行」真实入口。
///
/// 中文注释:
/// - SFID 负责搜索机构资料;链上 `ClearingBankNodes` 负责确认该机构是否已经声明
///   可连接的清算行节点。
/// - 页面只缓存绑定快照,不把本地缓存当作权威状态;绑定、切换和支付仍以链上校验为准。
class ClearingBankSettingsPage extends StatefulWidget {
  const ClearingBankSettingsPage({
    super.key,
    required this.wallet,
    this.directory,
  });

  final WalletProfile wallet;
  final ClearingBankDirectory? directory;

  @override
  State<ClearingBankSettingsPage> createState() =>
      _ClearingBankSettingsPageState();
}

class _ClearingBankSettingsPageState extends State<ClearingBankSettingsPage> {
  static const String _sfidBaseUrl = String.fromEnvironment(
    'SFID_BASE_URL',
    defaultValue: 'http://127.0.0.1:8080',
  );

  final TextEditingController _searchCtrl = TextEditingController();

  late final ClearingBankDirectory _directory =
      widget.directory ?? ClearingBankDirectory(sfidBaseUrl: _sfidBaseUrl);

  ClearingBankBindingSnapshot? _current;
  List<ClearingBankCandidate> _items = const [];
  bool _loadingCurrent = true;
  bool _searching = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadCurrent();
  }

  @override
  void dispose() {
    _searchCtrl.dispose();
    super.dispose();
  }

  Future<void> _loadCurrent() async {
    final snapshot = await ClearingBankPrefs.loadSnapshot(
      widget.wallet.walletIndex,
    );
    if (!mounted) return;
    setState(() {
      _current = snapshot;
      _loadingCurrent = false;
    });
  }

  Future<void> _search() async {
    setState(() {
      _searching = true;
      _error = null;
    });
    try {
      final items = await _directory.search(_searchCtrl.text.trim());
      if (!mounted) return;
      setState(() => _items = items);
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = '搜索失败:$e');
    } finally {
      if (mounted) setState(() => _searching = false);
    }
  }

  Future<void> _openBind(ClearingBankCandidate item) async {
    if (!item.canBind) return;
    final current = _current;
    final isSwitch = current != null && current.sfidId != item.info.sfidId;
    final changed = await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => BindClearingBankPage(
          wallet: widget.wallet,
          bank: item.info,
          endpoint: item.endpoint,
          switchMode: isSwitch,
        ),
      ),
    );
    if (changed == true) {
      await _loadCurrent();
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('设置清算行'),
        centerTitle: true,
      ),
      body: RefreshIndicator(
        onRefresh: _loadCurrent,
        child: ListView(
          padding: const EdgeInsets.all(12),
          children: [
            _currentCard(),
            const SizedBox(height: 12),
            _searchBox(),
            if (_error != null) ...[
              const SizedBox(height: 12),
              Text(_error!, style: const TextStyle(color: Colors.red)),
            ],
            const SizedBox(height: 12),
            if (_searching)
              const Center(child: CircularProgressIndicator())
            else if (_items.isEmpty)
              const Padding(
                padding: EdgeInsets.only(top: 48),
                child: Center(
                  child: Text(
                    '暂无结果',
                    style: TextStyle(color: AppTheme.textTertiary),
                  ),
                ),
              )
            else
              ..._items.map(_candidateTile),
          ],
        ),
      ),
    );
  }

  Widget _currentCard() {
    if (_loadingCurrent) {
      return const ListTile(
        leading: CircularProgressIndicator(),
        title: Text('正在读取当前绑定'),
      );
    }
    final current = _current;
    if (current == null) {
      return const ListTile(
        leading: Icon(Icons.account_balance_outlined),
        title: Text('尚未绑定清算行'),
        subtitle: Text('搜索已加入清算网络的机构后绑定'),
      );
    }
    return ListTile(
      leading: const Icon(Icons.account_balance),
      title: Text(current.institutionName.isEmpty
          ? current.sfidId
          : current.institutionName),
      subtitle: Text('${current.sfidId}\n${current.wssUrl}'),
      isThreeLine: true,
      trailing: TextButton(
        onPressed: () async {
          await ClearingBankPrefs.clear(widget.wallet.walletIndex);
          await _loadCurrent();
        },
        child: const Text('清除缓存'),
      ),
    );
  }

  Widget _searchBox() {
    return Row(
      children: [
        Expanded(
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
            textInputAction: TextInputAction.search,
            onSubmitted: (_) => _search(),
          ),
        ),
        const SizedBox(width: 8),
        IconButton.filled(
          onPressed: _searching ? null : _search,
          icon: const Icon(Icons.search),
          tooltip: '搜索',
        ),
      ],
    );
  }

  Widget _candidateTile(ClearingBankCandidate item) {
    final info = item.info;
    final endpoint = item.endpoint;
    final name =
        info.institutionName.isEmpty ? '(未命名机构)' : info.institutionName;
    final current = _current;
    final isCurrent = current?.sfidId == info.sfidId;
    final buttonText = isCurrent ? '已绑定' : (current == null ? '绑定' : '切换');

    return ListTile(
      leading: Icon(
        item.canBind ? Icons.verified_outlined : Icons.block,
        color: item.canBind ? Colors.green : AppTheme.textTertiary,
      ),
      title: Text(name),
      subtitle: Text(
        '${info.sfidId}\n'
        '${endpoint == null ? '未查询到链上节点声明' : endpoint.wssUrl}',
      ),
      isThreeLine: true,
      trailing: FilledButton(
        onPressed: item.canBind && !isCurrent ? () => _openBind(item) : null,
        child: Text(buttonText),
      ),
    );
  }
}
