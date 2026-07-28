import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/ui/widgets/bip39_input.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';

/// 「添加账户」两模式：下一个序号 / 指定序号。
enum AddAccountMode { next, specify }

/// [parseAccountIndices] 的结果：成功携带解析出的序号列表，失败携带可直接展示的
/// 错误文案。范围 / 重复 / 已存在等业务校验交给 [WalletManager.addAccounts] 兜底，
/// 此处只负责「能否变成整数列表」这一层（空 / 非数字）。
class AccountIndexParse {
  const AccountIndexParse.success(List<int> this.indices) : error = null;
  const AccountIndexParse.failure(String this.error) : indices = null;

  final List<int>? indices;
  final String? error;

  bool get isSuccess => indices != null;
}

/// 把「空格分隔的多个序号」解析成 `List<int>`。
///
/// 连续（`1 2 3`）与断续（`1 5 9`）都按空白切分；忽略多余空白与首尾空格。
/// 只做最基础的两类校验：整段为空、单个 token 非数字；越界 / 重复 / 已存在留给
/// [WalletManager.addAccounts] 单源抛出，避免把业务规则抄两份。
AccountIndexParse parseAccountIndices(String raw) {
  final tokens = raw
      .trim()
      .split(RegExp(r'\s+'))
      .where((token) => token.isNotEmpty)
      .toList(growable: false);
  if (tokens.isEmpty) {
    return const AccountIndexParse.failure('请输入至少一个账户序号');
  }
  final indices = <int>[];
  for (final token in tokens) {
    final value = int.tryParse(token);
    if (value == null) {
      return AccountIndexParse.failure('序号必须是数字：$token');
    }
    indices.add(value);
  }
  return AccountIndexParse.success(indices);
}

/// 弹出「添加账户」底部面板；成功追加返回 `true`，否则返回 `null`。
Future<bool?> showAddAccountSheet(
  BuildContext context, {
  required String masterId,
}) {
  return showModalBottomSheet<bool>(
    context: context,
    isScrollControlled: true,
    builder: (_) => AddAccountSheet(masterId: masterId),
  );
}

/// 无根多账户追加面板：录入本钱包助记词 → 选「下一个」或「指定序号」→ 落库。
///
/// 无根设备不保存助记词，追加账户须重新录入；[WalletManager.addAccounts] 会先做
/// 归属校验（助记词派生的账户0 必须等于 [masterId]），不符抛 [WalletAuthException]。
class AddAccountSheet extends StatefulWidget {
  const AddAccountSheet({super.key, required this.masterId});

  final String masterId;

  @override
  State<AddAccountSheet> createState() => _AddAccountSheetState();
}

class _AddAccountSheetState extends State<AddAccountSheet> {
  final WalletManager _walletManager = WalletManager();
  final TextEditingController _mnemonicController = TextEditingController();
  final TextEditingController _indexController = TextEditingController();

  AddAccountMode _mode = AddAccountMode.next;
  int? _nextIndex;
  bool _submitting = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadNextIndex();
  }

  @override
  void dispose() {
    _mnemonicController.dispose();
    _indexController.dispose();
    super.dispose();
  }

  Future<void> _loadNextIndex() async {
    try {
      final accounts = await _walletManager.getAccounts(widget.masterId);
      final maxIndex = accounts
          .map((account) => account.accountIndex)
          .fold<int>(0, (max, index) => index > max ? index : max);
      if (!mounted) return;
      setState(() => _nextIndex = maxIndex + 1);
    } catch (_) {
      // 拿不到下一个序号只影响副标题展示，不阻塞添加（addNextAccount 内部会自算）。
    }
  }

  String _describeError(Object error) {
    if (error is WalletAuthException) {
      return error.message;
    }
    final text = error.toString();
    const prefix = 'Exception: ';
    return text.startsWith(prefix) ? text.substring(prefix.length) : text;
  }

  Future<void> _submit() async {
    if (_submitting) return;
    setState(() {
      _submitting = true;
      _error = null;
    });
    try {
      final mnemonic = _mnemonicController.text;
      if (_mode == AddAccountMode.next) {
        await _walletManager.addNextAccount(widget.masterId, mnemonic);
      } else {
        final parsed = parseAccountIndices(_indexController.text);
        if (!parsed.isSuccess) {
          setState(() => _error = parsed.error);
          return;
        }
        await _walletManager.addAccounts(
          widget.masterId,
          mnemonic,
          parsed.indices!,
        );
      }
      if (!mounted) return;
      Navigator.of(context).pop(true);
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = _describeError(e));
    } finally {
      if (mounted) {
        setState(() => _submitting = false);
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final bottomInset = MediaQuery.viewInsetsOf(context).bottom;
    return SafeArea(
      child: Padding(
        padding: EdgeInsets.fromLTRB(16, 8, 16, 16 + bottomInset),
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Center(
                child: Container(
                  width: 36,
                  height: 4,
                  margin: const EdgeInsets.only(bottom: 16),
                  decoration: BoxDecoration(
                    color: AppTheme.textTertiary,
                    borderRadius: BorderRadius.circular(2),
                  ),
                ),
              ),
              const Text(
                '添加账户',
                style: TextStyle(
                  fontSize: 17,
                  fontWeight: FontWeight.w700,
                  color: AppTheme.textPrimary,
                ),
              ),
              const SizedBox(height: 6),
              const Text(
                '无根设备不保存助记词，追加账户需重新录入本钱包助记词校验归属。',
                style: TextStyle(fontSize: 12, color: AppTheme.textSecondary),
              ),
              const SizedBox(height: 12),
              Bip39InputField(controller: _mnemonicController, wordCount: 0),
              const SizedBox(height: 16),
              SegmentedButton<AddAccountMode>(
                segments: const [
                  ButtonSegment(
                    value: AddAccountMode.next,
                    label: Text('下一个账户'),
                    icon: Icon(Icons.add_circle_outline, size: 18),
                  ),
                  ButtonSegment(
                    value: AddAccountMode.specify,
                    label: Text('指定序号'),
                    icon: Icon(Icons.tag_rounded, size: 18),
                  ),
                ],
                selected: {_mode},
                onSelectionChanged: (value) => setState(() {
                  _mode = value.first;
                  _error = null;
                }),
              ),
              const SizedBox(height: 12),
              if (_mode == AddAccountMode.next)
                Text(
                  _nextIndex == null ? '将派生下一个账户' : '将派生 //$_nextIndex',
                  style: const TextStyle(
                    fontSize: 12,
                    color: AppTheme.textSecondary,
                  ),
                )
              else
                TextField(
                  controller: _indexController,
                  keyboardType: TextInputType.number,
                  inputFormatters: [
                    FilteringTextInputFormatter.allow(RegExp(r'[0-9 ]')),
                  ],
                  decoration: const InputDecoration(
                    labelText: '账户序号',
                    hintText: '空格分隔，如 1 5 9',
                    helperText:
                        '连续或断续均可，范围 1–${WalletManager.maxAccountIndex}',
                    border: OutlineInputBorder(),
                  ),
                ),
              if (_error != null) ...[
                const SizedBox(height: 12),
                Text(
                  _error!,
                  style: const TextStyle(color: AppTheme.danger, fontSize: 13),
                ),
              ],
              const SizedBox(height: 16),
              SizedBox(
                width: double.infinity,
                height: 46,
                child: FilledButton(
                  onPressed: _submitting ? null : _submit,
                  child: Text(_submitting ? '添加中…' : '确认添加'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
