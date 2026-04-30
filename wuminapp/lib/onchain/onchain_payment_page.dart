import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/util/amount_format.dart';
import 'package:wuminapp_mobile/onchain/onchain_payment_models.dart';
import 'package:wuminapp_mobile/onchain/onchain_payment_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/trade/local_tx_store.dart';
import 'package:wuminapp_mobile/trade/pending_tx_reconciler.dart';
import 'package:wuminapp_mobile/Isar/wallet_isar.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/user/user.dart' show ContactBookPage;
import 'package:wuminapp_mobile/user/user_service.dart' show UserContact;
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/pages/wallet_page.dart';
import 'package:wuminapp_mobile/wallet/pages/transaction_history_page.dart';

class OnchainPaymentPage extends StatefulWidget {
  const OnchainPaymentPage({super.key, this.initialToAddress});

  /// 预填收款地址（从通讯录等入口跳转时使用）。
  final String? initialToAddress;

  @override
  State<OnchainPaymentPage> createState() => _OnchainPaymentPageState();
}

class _OnchainPaymentPageState extends State<OnchainPaymentPage> {
  /// 链的 SS58 地址前缀。
  static const int _ss58Prefix = 2027;

  /// 链上存在性保证金（Existential Deposit）= 111 分 = 1.11 元。
  /// 来源：primitives::core_const::ACCOUNT_EXISTENTIAL_DEPOSIT = 111
  static const double _edYuan = 1.11;
  final OnchainPaymentService _paymentService = OnchainPaymentService();
  final TextEditingController _toController = TextEditingController();
  final TextEditingController _amountController = TextEditingController();
  final String _selectedSymbol = 'GMB';

  WalletProfile? _currentWallet;
  bool _loadingWallet = true;
  bool _submitting = false;
  LightClientStatusSnapshot? _chainProgress;
  String? _chainProgressError;

  /// 本地链上转账记录（用于状态行显示）。
  List<LocalTxEntity> _localTxRecords = [];

  @override
  void initState() {
    super.initState();
    if (widget.initialToAddress != null) {
      _toController.text = widget.initialToAddress!;
    }
    _bootstrap();
  }

  @override
  void dispose() {
    _toController.dispose();
    _amountController.dispose();
    super.dispose();
  }

  Future<void> _bootstrap() async {
    await _reloadWallet();
    await _loadLocalRecords();
    // 触发全局对账；Reconciler 内部有并发保护，多次触发安全。
    unawaited(_runReconcileAndReload());
  }

  /// 触发全局对账并在完成后刷新本地列表。
  Future<void> _runReconcileAndReload() async {
    try {
      final updated = await PendingTxReconciler.instance.reconcileAll();
      if (updated > 0 && mounted) {
        await _loadLocalRecords();
      }
    } catch (e) {
      debugPrint('[交易记录] 对账失败: $e');
    }
  }

  /// 中文注释：从本地 Isar 加载链上转账记录。
  Future<void> _loadLocalRecords() async {
    if (_currentWallet == null) return;
    try {
      final records = await LocalTxStore.queryByWallet(
        _currentWallet!.address,
        limit: 100,
      );
      // 只取 transfer + out 的记录
      final filtered = records
          .where((r) => r.txType == 'transfer' && r.direction == 'out')
          .toList();
      if (mounted) {
        setState(() {
          _localTxRecords = filtered;
        });
      }
    } catch (e) {
      debugPrint('[链上交易] 加载本地记录失败: $e');
    }
  }

  int _countByStatus(String status) {
    return _localTxRecords.where((r) => r.status == status).length;
  }

  Color _statusColor(String status) {
    switch (status) {
      case 'pending':
        return AppTheme.warning;
      case 'confirmed':
        return AppTheme.success;
      case 'failed':
        return AppTheme.danger;
      default:
        return AppTheme.textSecondary;
    }
  }

  Future<void> _reloadWallet() async {
    final wallet = await _paymentService.getCurrentWallet();
    if (!mounted) {
      return;
    }
    setState(() {
      _currentWallet = wallet;
      _loadingWallet = false;
    });
  }

  Future<void> _openMyWalletPage() async {
    final changed = await Navigator.of(context).push<bool>(
      MaterialPageRoute(
        builder: (_) => const MyWalletPage(selectForTrade: true),
      ),
    );
    if (changed == true) {
      await _reloadWallet();
    }
  }

  Future<void> _openContactsPage() async {
    final contact = await Navigator.of(context).push<UserContact>(
      MaterialPageRoute(
        builder: (_) => ContactBookPage(
          selfAccountPubkeyHex: _currentWallet?.pubkeyHex ?? '',
          selectForTrade: true,
        ),
      ),
    );
    if (!mounted || contact == null) return;
    setState(() {
      _toController.text = contact.accountPubkeyHex;
    });
  }

  Future<void> _submit() async {
    final blockedReason = _submitBlockedReason;
    if (blockedReason != null) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(blockedReason)));
      }
      return;
    }
    if (_loadingWallet) {
      return;
    }
    if (_currentWallet == null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('请先创建或导入钱包')));
      await _openMyWalletPage();
      return;
    }

    final toAddress = _toController.text.trim();
    final amountRaw = _amountController.text.trim();
    if (toAddress.isEmpty || amountRaw.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请先填写完整的收款地址和金额')),
      );
      return;
    }
    final amountText = AmountFormat.stripCommas(amountRaw);

    // SS58 地址校验（prefix 2027）
    try {
      final decoded = Keyring().decodeAddress(toAddress);
      // 验证 prefix：重新编码后比对
      final reEncoded = Keyring().encodeAddress(decoded, _ss58Prefix);
      if (reEncoded != toAddress) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('收款地址不是本链地址（SS58 前缀不匹配）')),
        );
        return;
      }
    } catch (_) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('收款地址格式错误，请输入有效的 SS58 地址')),
      );
      return;
    }

    final amount = double.tryParse(amountText);
    if (amount == null || amount <= 0) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('金额格式不正确')));
      return;
    }

    // 预估手续费，展示确认对话框
    final estimatedFee = OnchainRpc.estimateTransferFeeYuan(amount);

    // 余额校验：转账金额 + 手续费 ≤ 可用余额（余额 - ED）
    final availableBalance = _currentWallet!.balance - _edYuan;
    if (amount + estimatedFee > availableBalance) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            '余额不足，可用余额：${AmountFormat.format(availableBalance, symbol: '')} 元'
            '（已扣除 ED ${AmountFormat.format(_edYuan, symbol: '')} 元）',
          ),
        ),
      );
      return;
    }
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('确认交易'),
        content: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
                '转账金额：${AmountFormat.format(amount, symbol: _selectedSymbol)}'),
            const SizedBox(height: 4),
            Text(
                '预估手续费：${AmountFormat.format(estimatedFee, symbol: _selectedSymbol)}'),
            const Divider(height: 16),
            Text(
              '合计：${AmountFormat.format(amount + estimatedFee, symbol: _selectedSymbol)}',
              style: const TextStyle(fontWeight: FontWeight.w700),
            ),
          ],
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(dialogContext, false),
            child: const Text('取消'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(dialogContext, true),
            child: const Text('确认'),
          ),
        ],
      ),
    );
    if (confirmed != true) return;

    setState(() {
      _submitting = true;
    });
    try {
      final wallet = _currentWallet!;
      final Future<Uint8List> Function(Uint8List payload) signCallback;

      if (wallet.isHotWallet) {
        // 热钱包：先验证设备密码/生物识别，再构造交易。
        // 密码验证必须在 RPC 调用之前，避免用户等 RPC 后才弹密码框。
        final walletManager = WalletManager();
        await walletManager.authenticateForSigning();
        // 验证通过后，签名回调跳过二次验证
        signCallback = (payload) =>
            walletManager.signWithWalletNoAuth(wallet.walletIndex, payload);
      } else {
        // 冷钱包：扫码签名。
        signCallback = (Uint8List payload) async {
          final qrSigner = QrSigner();
          final requestId = QrSigner.generateRequestId(prefix: 'tx-');
          final toAddr = _toController.text.trim();
          // 千分位格式化，与 PayloadDecoder._fenToYuan 对齐
          final amountFormatted = AmountFormat.format(
                  AmountFormat.tryParse(_amountController.text) ?? 0,
                  symbol: '')
              .trim();
          final rv = await ChainRpc().fetchRuntimeVersion();
          final request = qrSigner.buildRequest(
            requestId: requestId,
            address: wallet.address,
            pubkey: '0x${wallet.pubkeyHex}',
            payloadHex: '0x${_toHex(payload)}',
            specVersion: rv.specVersion,
            display: SignDisplay(
              action: 'transfer',
              summary: '转账 $amountFormatted $_selectedSymbol 给 $toAddr',
              fields: [
                // transfer 链端 fields 按 Registry = (to, amount_yuan)。
                // wumin decoder 输出 "X.XX GMB"(千分位),wuminapp 的
                // $amountFormatted 来自 AmountFormat.format 已自带千分位。
                SignDisplayField(key: 'to', label: '收款账户', value: toAddr),
                SignDisplayField(
                    key: 'amount_yuan',
                    label: '金额',
                    value: '$amountFormatted $_selectedSymbol'),
              ],
            ),
          );
          final requestJson = qrSigner.encodeRequest(request);

          if (!mounted) {
            throw Exception('页面已关闭，无法继续扫码签名');
          }
          final response = await Navigator.push<SignResponseEnvelope>(
            context,
            MaterialPageRoute(
              builder: (_) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}',
              ),
            ),
          );

          if (response == null) {
            throw Exception('签名已取消');
          }

          return Uint8List.fromList(_hexToBytes(response.body.signature));
        };
      }

      final result = await _paymentService.submitTransfer(
        OnchainPaymentDraft(
          toAddress: toAddress,
          amount: amount,
          symbol: _selectedSymbol,
        ),
        sign: signCallback,
      );
      if (!mounted) {
        return;
      }

      // 交易已成功提交，后续写入本地记录失败不影响交易结果
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('签名成功，交易已发送，tx=${result.txHash}')));
      _toController.clear();
      _amountController.clear();

      // 写入本地交易记录（失败不影响交易）
      try {
        final entity = LocalTxEntity()
          ..txId = result.txHash
          ..walletAddress = _currentWallet!.address
          ..txType = 'transfer'
          ..direction = 'out'
          ..fromAddress = _currentWallet!.address
          ..toAddress = toAddress
          ..amountYuan = amount
          ..feeYuan = estimatedFee
          ..status = 'pending'
          ..txHash = result.txHash
          ..usedNonce = result.usedNonce
          ..createdAtMillis = DateTime.now().millisecondsSinceEpoch;
        await LocalTxStore.insert(entity);
        if (mounted) await _loadLocalRecords();

        // 交由全局 Reconciler 兜底，不再依赖页面生命周期。
        unawaited(_quickConfirmAfterSubmit(result.txHash));
      } catch (e) {
        debugPrint('[交易记录] 写入本地失败: $e');
      }
    } on WalletAuthException catch (e) {
      if (!mounted) {
        return;
      }
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message)),
      );
    } on OnchainPaymentException catch (e) {
      if (!mounted) {
        return;
      }
      final message = e.code == OnchainPaymentErrorCode.broadcastFailed
          ? '交易发送失败：${e.message}'
          : '签名失败：${e.message}';
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(message)),
      );
    } catch (e) {
      if (!mounted) {
        return;
      }
      debugPrint('[链上交易] 未知异常: $e');
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('交易异常：$e')));
    } finally {
      if (mounted) {
        setState(() {
          _submitting = false;
        });
      }
    }
  }

  /// 提交成功后的快速确认：短轮询 3 次，快速把常见情况推到 confirmed。
  /// 长尾由全局 Reconciler 在启动/resume/周期调度时兜底。
  Future<void> _quickConfirmAfterSubmit(String txHash) async {
    for (var i = 0; i < 3; i++) {
      await Future.delayed(const Duration(seconds: 2));
      try {
        final outcome =
            await PendingTxReconciler.instance.reconcileSingle(txHash);
        if (outcome == ReconcileOutcome.confirmed ||
            outcome == ReconcileOutcome.lost) {
          if (mounted) await _loadLocalRecords();
          return;
        }
      } catch (e) {
        debugPrint('[交易记录] 快速确认失败: $e');
      }
    }
  }

  Widget _buildStatusText(String label, int count, Color color) {
    return Text(
      '$label $count',
      style: TextStyle(
        color: color,
        fontWeight: FontWeight.w700,
      ),
    );
  }

  Widget _buildSubmitCard() {
    return Container(
      decoration: AppTheme.cardDecoration(),
      child: Padding(
        padding: const EdgeInsets.fromLTRB(12, 12, 12, 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (_currentWallet != null)
              Padding(
                padding: const EdgeInsets.only(left: 0, bottom: 12),
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: [
                    Transform.rotate(
                      angle: 0.785398,
                      child: ClipRRect(
                        borderRadius: BorderRadius.circular(6),
                        child: Image.asset(
                          'assets/icons/icons8-96.png',
                          width: 22,
                          height: 22,
                        ),
                      ),
                    ),
                    const SizedBox(width: 6),
                    Text(
                      '钱包可用余额：${AmountFormat.format(_currentWallet!.balance, symbol: '')} 元',
                      style: const TextStyle(
                        fontSize: 13,
                        color: AppTheme.textSecondary,
                      ),
                    ),
                  ],
                ),
              ),
            TextField(
              controller: _toController,
              decoration: const InputDecoration(
                labelText: '收款地址',
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _amountController,
                    keyboardType:
                        const TextInputType.numberWithOptions(decimal: true),
                    inputFormatters: [ThousandSeparatorFormatter()],
                    style: const TextStyle(color: AppTheme.textPrimary),
                    decoration: const InputDecoration(
                      labelText: '金额',
                    ),
                  ),
                ),
                const SizedBox(width: 10),
                SizedBox(
                  width: 120,
                  child: InputDecorator(
                    decoration: const InputDecoration(
                      labelText: '币种',
                      contentPadding: EdgeInsets.symmetric(vertical: 16),
                    ),
                    child: Text(
                      _selectedSymbol,
                      textAlign: TextAlign.center,
                      style: const TextStyle(
                        color: AppTheme.textSecondary,
                        fontSize: 16,
                      ),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            SizedBox(
              width: double.infinity,
              child: FilledButton(
                onPressed: _canSubmit ? _submit : null,
                child: Text(_submitting ? '签名中' : '签名交易'),
              ),
            ),
            if (_submitBlockedReason != null &&
                !_loadingWallet &&
                _currentWallet != null)
              Padding(
                padding: const EdgeInsets.only(top: 8),
                child: Text(
                  _submitBlockedReason!,
                  style: const TextStyle(
                    fontSize: 12,
                    color: AppTheme.textSecondary,
                    height: 1.4,
                  ),
                ),
              ),
            const SizedBox(height: 12),
            // 链上交易状态行
            Row(
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                Expanded(
                  child: Wrap(
                    spacing: 16,
                    runSpacing: 8,
                    children: [
                      _buildStatusText('待确认', _countByStatus('pending'),
                          _statusColor('pending')),
                      _buildStatusText('已确认', _countByStatus('confirmed'),
                          _statusColor('confirmed')),
                      _buildStatusText('失败', _countByStatus('failed'),
                          _statusColor('failed')),
                    ],
                  ),
                ),
                InkWell(
                  onTap: _currentWallet != null
                      ? () {
                          Navigator.of(context).push(
                            MaterialPageRoute(
                              builder: (_) => TransactionHistoryPage(
                                walletAddress: _currentWallet!.address,
                              ),
                            ),
                          );
                        }
                      : null,
                  borderRadius: BorderRadius.circular(8),
                  child: const Padding(
                    padding: EdgeInsets.all(6),
                    child: Icon(Icons.chevron_right, size: 22),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(4, 10, 4, 0),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.center,
                children: [
                  IconButton(
                    tooltip: '我的通讯录',
                    onPressed: _openContactsPage,
                    icon: SvgPicture.asset(
                      'assets/icons/contact-round.svg',
                      width: 22,
                      height: 22,
                    ),
                  ),
                  const Expanded(
                    child: Center(
                      child: Text(
                        '链上支付',
                        style: TextStyle(
                            fontSize: 20, fontWeight: FontWeight.w700),
                      ),
                    ),
                  ),
                  IconButton(
                    tooltip: '选择交易钱包',
                    onPressed: _openMyWalletPage,
                    icon: SvgPicture.asset(
                      'assets/icons/wallet.svg',
                      width: 22,
                      height: 22,
                    ),
                  ),
                ],
              ),
            ),
            Expanded(
              child: ListView(
                physics: const AlwaysScrollableScrollPhysics(),
                padding: const EdgeInsets.all(16),
                children: [
                  ChainProgressBanner(
                    onProgressChanged: _handleChainProgressChanged,
                    onErrorChanged: _handleChainProgressErrorChanged,
                  ),
                  if (_currentWallet == null && !_loadingWallet)
                    Padding(
                      padding: const EdgeInsets.only(bottom: 12),
                      child: Container(
                        decoration: AppTheme.cardDecoration(),
                        child: Padding(
                          padding: const EdgeInsets.fromLTRB(12, 12, 12, 12),
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              const Text('未检测到钱包，无法执行链上签名与交易广播'),
                              const SizedBox(height: 8),
                              FilledButton(
                                onPressed: _openMyWalletPage,
                                child: const Text('去创建/导入钱包'),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  _buildSubmitCard(),
                  const SizedBox(height: 24),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _handleChainProgressChanged(LightClientStatusSnapshot? progress) {
    if (!mounted) return;
    setState(() {
      _chainProgress = progress;
    });
  }

  void _handleChainProgressErrorChanged(String? error) {
    if (!mounted) return;
    setState(() {
      _chainProgressError = error;
    });
  }

  bool get _canSubmit =>
      !_submitting &&
      !_loadingWallet &&
      _currentWallet != null &&
      _submitBlockedReason == null;

  String? get _submitBlockedReason {
    if (_submitting || _loadingWallet || _currentWallet == null) {
      return null;
    }

    final progress = _chainProgress;
    if (progress == null) {
      return _chainProgressError ?? '正在读取区块链状态，请稍后再试';
    }
    if (!progress.hasPeers) {
      return '轻节点尚未连接到区块链网络，请等待至少 1 个 peer';
    }
    if (progress.isSyncing) {
      return '轻节点仍在同步区块头，完成后才能签名交易';
    }
    if (!progress.isUsable) {
      return _chainProgressError ?? '区块链状态尚未就绪，请稍后再试';
    }
    return null;
  }
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

List<int> _hexToBytes(String input) {
  final text = input.startsWith('0x') ? input.substring(2) : input;
  if (text.isEmpty || text.length.isOdd) return const <int>[];
  final out = <int>[];
  for (var i = 0; i < text.length; i += 2) {
    out.add(int.parse(text.substring(i, i + 2), radix: 16));
  }
  return out;
}
