import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/rpc/sfid_public.dart';
import 'package:wuminapp_mobile/trade/offchain/bind_clearing_bank_page.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';

/// 扫码支付清算体系 Step 1 新增:清算行列表 + 搜索页。
///
/// 中文注释:
/// - 数据源:SFID 系统 `GET /api/v1/app/clearing-banks/search`
/// - 过滤:省/市/keyword + 分页(本步默认每页 20)
/// - 点击单行 → 跳转 `BindClearingBankPage` 完成绑定
/// - 与旧 `lib/wallet/ui/bind_clearing_page.dart`(43 省储行硬编码)并存,
///   旧页面 Step 2 删除。
class ClearingBankListPage extends StatefulWidget {
  const ClearingBankListPage({
    super.key,
    required this.wallet,
    required this.sfidBaseUrl,
  });

  /// 当前钱包(传给绑定页用于签名)。
  final WalletProfile wallet;

  /// SFID 后端 baseUrl(由调用方注入,例如全局环境变量解析后传入)。
  final String sfidBaseUrl;

  @override
  State<ClearingBankListPage> createState() => _ClearingBankListPageState();
}

class _ClearingBankListPageState extends State<ClearingBankListPage> {
  final TextEditingController _kwCtrl = TextEditingController();
  final TextEditingController _provCtrl = TextEditingController();
  final TextEditingController _cityCtrl = TextEditingController();

  late final SfidPublicApi _api;
  bool _loading = false;
  String? _error;
  ClearingBankSearchResult? _result;

  @override
  void initState() {
    super.initState();
    _api = SfidPublicApi(baseUrl: widget.sfidBaseUrl);
    _runSearch(); // 首次进入加载全国前 20 条
  }

  @override
  void dispose() {
    _kwCtrl.dispose();
    _provCtrl.dispose();
    _cityCtrl.dispose();
    _api.close();
    super.dispose();
  }

  Future<void> _runSearch() async {
    setState(() {
      _loading = true;
      _error = null;
    });
    try {
      final r = await _api.searchClearingBanks(
        province: _provCtrl.text,
        city: _cityCtrl.text,
        keyword: _kwCtrl.text,
        page: 1,
        size: 20,
      );
      if (!mounted) return;
      setState(() {
        _result = r;
        _loading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _loading = false;
      });
    }
  }

  Future<void> _openBindPage(ClearingBankInfo bank) async {
    if (bank.mainAccount == null || bank.mainAccount!.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('该清算行主账户尚未上链,暂不可绑定')),
      );
      return;
    }
    await Navigator.push<bool>(
      context,
      MaterialPageRoute(
        builder: (_) => BindClearingBankPage(
          wallet: widget.wallet,
          bank: bank,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('选择清算行(扫码支付)')),
      body: Column(
        children: [
          _buildFilterBar(),
          if (_loading) const LinearProgressIndicator(),
          if (_error != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Text(
                '加载失败:$_error',
                style: const TextStyle(color: Colors.red),
              ),
            ),
          Expanded(child: _buildList()),
        ],
      ),
    );
  }

  Widget _buildFilterBar() {
    return Padding(
      padding: const EdgeInsets.fromLTRB(12, 8, 12, 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _provCtrl,
                  decoration: const InputDecoration(
                    hintText: '省份(留空=全国)',
                    isDense: true,
                  ),
                ),
              ),
              const SizedBox(width: 8),
              Expanded(
                child: TextField(
                  controller: _cityCtrl,
                  decoration: const InputDecoration(
                    hintText: '城市(可选)',
                    isDense: true,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 6),
          Row(
            children: [
              Expanded(
                child: TextField(
                  controller: _kwCtrl,
                  decoration: const InputDecoration(
                    hintText: '关键字(机构名/SFID)',
                    prefixIcon: Icon(Icons.search, size: 20),
                    isDense: true,
                  ),
                ),
              ),
              const SizedBox(width: 8),
              FilledButton(
                onPressed: _loading ? null : _runSearch,
                child: const Text('搜索'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildList() {
    final items = _result?.items ?? const <ClearingBankInfo>[];
    if (!_loading && items.isEmpty) {
      return const Center(
        child: Text('暂无符合条件的清算行', style: TextStyle(color: Colors.grey)),
      );
    }
    return ListView.separated(
      itemCount: items.length,
      separatorBuilder: (_, __) =>
          const Divider(height: 1, indent: 16, endIndent: 16),
      itemBuilder: (context, idx) {
        final bank = items[idx];
        final name = bank.institutionName.isEmpty
            ? '(未命名机构)'
            : bank.institutionName;
        return ListTile(
          title: Text(name, style: const TextStyle(fontSize: 15)),
          subtitle: Text(
            '${bank.province} ${bank.city} · ${bank.a3} · ${bank.sfidId}',
            style: const TextStyle(fontSize: 12, color: Colors.grey),
          ),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _openBindPage(bank),
        );
      },
    );
  }
}
