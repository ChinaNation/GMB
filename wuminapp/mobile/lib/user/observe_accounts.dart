import 'package:flutter/material.dart';
import 'package:isar/isar.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';
import 'package:wuminapp_mobile/wallet/capabilities/api_client.dart';
import 'package:wuminapp_mobile/wallet/capabilities/wallet_type_service.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_isar.dart';

class ObservedAccount {
  const ObservedAccount({
    required this.id,
    required this.orgName,
    required this.publicKey,
    required this.address,
    required this.balance,
    required this.source,
  });

  static const Object _noBalanceChange = Object();

  final String id;
  final String orgName;
  final String publicKey;
  final String address;
  final double? balance;
  final String source;

  ObservedAccount copyWith({
    String? id,
    String? orgName,
    String? publicKey,
    String? address,
    Object? balance = _noBalanceChange,
    String? source,
  }) {
    return ObservedAccount(
      id: id ?? this.id,
      orgName: orgName ?? this.orgName,
      publicKey: publicKey ?? this.publicKey,
      address: address ?? this.address,
      balance: identical(balance, _noBalanceChange)
          ? this.balance
          : balance as double?,
      source: source ?? this.source,
    );
  }
}

class ObservedAccountService {
  static const int _ss58Format = 2027;
  final Keyring _keyring = Keyring.sr25519;
  final ApiClient _apiClient = ApiClient();
  final WalletTypeService _walletTypeService = WalletTypeService();

  Future<List<ObservedAccount>> getObservedAccounts() async {
    final stored = await _load();
    if (stored.isEmpty) {
      return stored;
    }
    final refreshed = await _refreshBalances(stored);
    final changed = _hasBalanceChanged(stored, refreshed);
    if (changed) {
      await _save(refreshed);
    }
    return refreshed;
  }

  Future<void> addObservedAccount(String input) async {
    final value = _cleanInput(input);
    if (value.isEmpty) {
      throw Exception('请输入公钥或地址');
    }

    final pubkey = _normalizeInputToPubkey(value);
    if (pubkey == null) {
      throw Exception('输入格式无效，请输入 32 字节公钥或 SS58 地址');
    }

    final address = _encodeAddress(pubkey);
    final current = await getObservedAccounts();
    final exists = current.any((it) => it.address == address);
    if (exists) {
      throw Exception('该观察账户已存在');
    }

    final role = await _walletTypeService.resolveWalletType(pubkey);
    final orgName = role == WalletTypeService.defaultType
        ? '自定义观察账户'
        : _extractOrgName(role);
    final initialBalance = await _fetchBalanceOrNull(address, pubkey);

    final next = List<ObservedAccount>.from(current)
      ..add(
        ObservedAccount(
          id: 'manual:$pubkey',
          orgName: orgName,
          publicKey: pubkey,
          address: address,
          balance: initialBalance,
          source: 'manual',
        ),
      );
    await _save(next);
  }

  Future<void> removeObservedAccount(ObservedAccount item) async {
    final current = await getObservedAccounts();
    current.removeWhere((it) => it.id == item.id);
    await _save(current);
  }

  Future<void> renameObservedAccount(String id, String orgName) async {
    final name = orgName.trim();
    if (name.isEmpty) {
      throw Exception('观察账户名称不能为空');
    }
    final current = await getObservedAccounts();
    bool found = false;
    final updated = current.map((it) {
      if (it.id != id) {
        return it;
      }
      found = true;
      return it.copyWith(orgName: name);
    }).toList(growable: false);
    if (!found) {
      throw Exception('未找到观察账户');
    }
    await _save(updated);
  }

  String? _normalizeInputToPubkey(String input) {
    final direct = _normalizeHexPubkey(input);
    if (direct != null) {
      return direct;
    }
    try {
      final bytes = _keyring.decodeAddress(_cleanInput(input));
      return _toHex(bytes.toList(growable: false));
    } catch (_) {
      return null;
    }
  }

  String? _normalizeHexPubkey(String input) {
    var v = _cleanInput(input).toLowerCase();
    if (v.startsWith('0x')) {
      v = v.substring(2);
    }
    final ok = RegExp(r'^[0-9a-f]{64}$').hasMatch(v);
    if (!ok) {
      return null;
    }
    return v;
  }

  String _encodeAddress(String pubkeyHex) {
    final bytes = <int>[];
    for (var i = 0; i < pubkeyHex.length; i += 2) {
      bytes.add(int.parse(pubkeyHex.substring(i, i + 2), radix: 16));
    }
    return _keyring.encodeAddress(bytes, _ss58Format);
  }

  String _extractOrgName(String roleName) {
    if (roleName.endsWith('管理员')) {
      return roleName.substring(0, roleName.length - 3);
    }
    return roleName;
  }

  String _toHex(List<int> bytes) {
    const chars = '0123456789abcdef';
    final buf = StringBuffer();
    for (final b in bytes) {
      buf
        ..write(chars[(b >> 4) & 0x0f])
        ..write(chars[b & 0x0f]);
    }
    return buf.toString();
  }

  String _cleanInput(String input) {
    var v = input.trim();
    v = v.replaceAll(' ', '');
    v = v.replaceAll('\n', '');
    v = v.replaceAll('\r', '');
    v = v.replaceAll('\t', '');
    v = v.replaceAll('"', '');
    v = v.replaceAll("'", '');
    v = v.replaceAll('`', '');
    v = v.replaceAll(',', '');
    v = v.replaceAll('，', '');
    v = v.replaceAll('。', '');
    v = v.replaceAll('；', '');
    v = v.replaceAll(';', '');
    return v;
  }

  Future<void> _save(List<ObservedAccount> items) async {
    final isar = await WalletIsar.instance.db();
    final data = items
        .map(
          (it) => ObservedAccountEntity()
            ..accountId = it.id
            ..orgName = it.orgName
            ..publicKey = it.publicKey
            ..address = it.address
            ..balance = it.balance
            ..source = it.source,
        )
        .toList(growable: false);

    await isar.writeTxn(() async {
      await isar.observedAccountEntitys.clear();
      if (data.isNotEmpty) {
        await isar.observedAccountEntitys.putAll(data);
      }
    });
  }

  Future<List<ObservedAccount>> _refreshBalances(
      List<ObservedAccount> items) async {
    final out = <ObservedAccount>[];
    for (final item in items) {
      final balance = await _fetchBalanceOrNull(item.address, item.publicKey);
      out.add(item.copyWith(balance: balance));
    }
    return out;
  }

  Future<double?> _fetchBalanceOrNull(String address, String pubkey) async {
    try {
      final data = await _apiClient.fetchWalletBalance(address);
      return data.balance;
    } catch (e) {
      try {
        final fallback = await _apiClient.fetchWalletBalance('0x$pubkey');
        return fallback.balance;
      } catch (fallbackError) {
        debugPrint(
          'observe account balance refresh failed: $address / $pubkey '
          'err=$e fallbackErr=$fallbackError',
        );
        return null;
      }
    }
  }

  bool _hasBalanceChanged(
      List<ObservedAccount> previous, List<ObservedAccount> next) {
    if (previous.length != next.length) {
      return true;
    }
    for (var i = 0; i < previous.length; i++) {
      if (previous[i].balance != next[i].balance) {
        return true;
      }
    }
    return false;
  }

  Future<List<ObservedAccount>> _load() async {
    final isar = await WalletIsar.instance.db();
    final rows = await isar.observedAccountEntitys.where().anyId().findAll();
    if (rows.isEmpty) {
      return <ObservedAccount>[];
    }

    final out = <ObservedAccount>[];
    for (final row in rows) {
      final pubkey = _normalizeHexPubkey(row.publicKey);
      if (pubkey == null) {
        continue;
      }

      final normalizedAddress = row.address.trim().isNotEmpty
          ? row.address.trim()
          : _encodeAddress(pubkey);
      final role = await _walletTypeService.resolveWalletType(pubkey);
      final normalizedOrg = row.orgName.trim().isNotEmpty
          ? row.orgName.trim()
          : (role == WalletTypeService.defaultType
              ? '自定义观察账户'
              : _extractOrgName(role));

      out.add(
        ObservedAccount(
          id: row.accountId,
          orgName: normalizedOrg,
          publicKey: pubkey,
          address: normalizedAddress,
          balance: row.balance,
          source: row.source,
        ),
      );
    }

    out.sort((a, b) => a.orgName.compareTo(b.orgName));
    return out;
  }
}

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
