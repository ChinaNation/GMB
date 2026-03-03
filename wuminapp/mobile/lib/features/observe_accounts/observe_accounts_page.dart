import 'package:flutter/material.dart';
import 'package:wuminapp_mobile/features/observe_accounts/observe_accounts_model.dart';
import 'package:wuminapp_mobile/features/observe_accounts/observe_accounts_service.dart';

class ObserveAccountsPage extends StatefulWidget {
  const ObserveAccountsPage({super.key});

  @override
  State<ObserveAccountsPage> createState() => _ObserveAccountsPageState();
}

class _ObserveAccountsPageState extends State<ObserveAccountsPage> {
  final ObservedAccountService _service = ObservedAccountService();
  late Future<List<ObservedAccount>> _accountsFuture;

  @override
  void initState() {
    super.initState();
    _accountsFuture = _service.getObservedAccounts();
  }

  void _reload() {
    _refreshNow();
  }

  Future<void> _refreshNow() async {
    final future = _service.getObservedAccounts();
    setState(() {
      _accountsFuture = future;
    });
    await future;
  }

  String _formatBalance(double? balance) {
    if (balance == null) {
      return '余额更新失败';
    }
    return '${balance.toStringAsFixed(2)} 元';
  }

  Future<void> _openObservedAccountDetail(ObservedAccount item) async {
    final changed = await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => ObserveAccountDetailPage(
          account: item,
          service: _service,
        ),
      ),
    );
    if (changed == true && mounted) {
      _reload();
    }
  }

  Future<void> _showAddDialog() async {
    final controller = TextEditingController();
    final added = await showDialog<bool>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('添加观察账户'),
          content: TextField(
            controller: controller,
            decoration: const InputDecoration(
              labelText: '账户公钥或地址',
              hintText: '请输入要观察的账户公钥或 SS58 地址',
              border: OutlineInputBorder(),
            ),
            minLines: 2,
            maxLines: 3,
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(false),
              child: const Text('取消'),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(true),
              child: const Text('添加'),
            ),
          ],
        );
      },
    );

    if (added != true) {
      return;
    }
    try {
      await _service.addObservedAccount(controller.text);
      if (!mounted) {
        return;
      }
      _reload();
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('观察账户已添加')));
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('$e')));
    }
  }

  Future<bool?> _confirmDelete(ObservedAccount item) {
    return showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('删除观察账户'),
        content: Text('确认删除“${item.orgName}”吗？'),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(true),
            child: const Text('删除'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('观察账户'),
        centerTitle: true,
        actions: [
          IconButton(
            tooltip: '添加观察账户',
            onPressed: _showAddDialog,
            icon: const Icon(Icons.add),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: FutureBuilder<List<ObservedAccount>>(
          future: _accountsFuture,
          builder: (context, snapshot) {
            if (snapshot.connectionState != ConnectionState.done) {
              return const Center(child: CircularProgressIndicator());
            }
            final accounts = snapshot.data ?? const <ObservedAccount>[];
            if (accounts.isEmpty) {
              return RefreshIndicator(
                onRefresh: _refreshNow,
                child: ListView(
                  physics: const AlwaysScrollableScrollPhysics(),
                  children: const [
                    SizedBox(height: 220),
                    Center(
                      child: Text('暂无观察账户，请点击右上角 + 添加'),
                    ),
                  ],
                ),
              );
            }
            return RefreshIndicator(
              onRefresh: _refreshNow,
              child: ListView.separated(
                physics: const AlwaysScrollableScrollPhysics(),
                itemCount: accounts.length,
                separatorBuilder: (context, index) =>
                    const SizedBox(height: 10),
                itemBuilder: (context, index) {
                  final item = accounts[index];
                  return Dismissible(
                    key: ValueKey(item.id),
                    direction: DismissDirection.endToStart,
                    background: Container(
                      alignment: Alignment.centerRight,
                      padding: const EdgeInsets.symmetric(horizontal: 20),
                      decoration: BoxDecoration(
                        color: Colors.red.shade400,
                        borderRadius: BorderRadius.circular(12),
                      ),
                      child: const Icon(
                        Icons.delete_outline,
                        color: Colors.white,
                      ),
                    ),
                    confirmDismiss: (_) => _confirmDelete(item),
                    onDismissed: (_) async {
                      final messenger = ScaffoldMessenger.of(context);
                      await _service.removeObservedAccount(item);
                      if (!mounted) {
                        return;
                      }
                      _reload();
                      messenger.showSnackBar(
                        const SnackBar(content: Text('已删除观察账户')),
                      );
                    },
                    child: Card(
                      child: InkWell(
                        borderRadius: BorderRadius.circular(12),
                        onTap: () => _openObservedAccountDetail(item),
                        child: Padding(
                          padding: const EdgeInsets.all(14),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Row(
                                children: [
                                  const Icon(Icons.remove_red_eye_outlined,
                                      size: 20),
                                  const SizedBox(width: 8),
                                  Expanded(
                                    child: Text(
                                      item.orgName,
                                      maxLines: 1,
                                      overflow: TextOverflow.ellipsis,
                                      style: const TextStyle(
                                        fontWeight: FontWeight.w700,
                                        fontSize: 16,
                                      ),
                                    ),
                                  ),
                                ],
                              ),
                              const SizedBox(height: 8),
                              Row(
                                children: [
                                  const Icon(
                                    Icons.monetization_on_outlined,
                                    size: 20,
                                    color: Color(0xFF0B3D2E),
                                  ),
                                  const SizedBox(width: 8),
                                  Text(
                                    _formatBalance(item.balance),
                                    style: const TextStyle(
                                      fontWeight: FontWeight.w600,
                                      fontSize: 16,
                                    ),
                                  ),
                                ],
                              ),
                              const SizedBox(height: 10),
                              const Text(
                                '观察地址：',
                                style: TextStyle(fontWeight: FontWeight.w700),
                              ),
                              const SizedBox(height: 4),
                              SelectableText(item.address),
                            ],
                          ),
                        ),
                      ),
                    ),
                  );
                },
              ),
            );
          },
        ),
      ),
    );
  }
}

class ObserveAccountDetailPage extends StatefulWidget {
  const ObserveAccountDetailPage({
    super.key,
    required this.account,
    required this.service,
  });

  final ObservedAccount account;
  final ObservedAccountService service;

  @override
  State<ObserveAccountDetailPage> createState() =>
      _ObserveAccountDetailPageState();
}

class _ObserveAccountDetailPageState extends State<ObserveAccountDetailPage> {
  late final TextEditingController _nameController;
  bool _saving = false;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: widget.account.orgName);
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('观察账户名称不能为空')));
      return;
    }
    if (name == widget.account.orgName) {
      Navigator.of(context).pop(false);
      return;
    }
    setState(() {
      _saving = true;
    });
    try {
      await widget.service.renameObservedAccount(widget.account.id, name);
      if (!mounted) {
        return;
      }
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('$e')));
    } finally {
      if (mounted) {
        setState(() {
          _saving = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('观察账户详情'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const Text(
            '观察地址：',
            style: TextStyle(fontSize: 14, fontWeight: FontWeight.w700),
          ),
          const SizedBox(height: 4),
          SelectableText(widget.account.address),
          const SizedBox(height: 14),
          TextField(
            controller: _nameController,
            decoration: const InputDecoration(
              labelText: '观察账户名称',
              hintText: '请输入观察账户名称',
              border: OutlineInputBorder(),
            ),
            textInputAction: TextInputAction.done,
          ),
          const SizedBox(height: 20),
          Align(
            alignment: Alignment.center,
            child: SizedBox(
              width: 190,
              child: FilledButton(
                onPressed: _saving ? null : _save,
                child: Text(
                  _saving ? '保存中...' : '保存观察账户信息',
                  style: const TextStyle(
                    fontWeight: FontWeight.w800,
                    fontSize: 16,
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
