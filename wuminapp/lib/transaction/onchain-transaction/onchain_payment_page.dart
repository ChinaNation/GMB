import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:smoldot/smoldot.dart' show LightClientStatusSnapshot;
import 'package:flutter_svg/flutter_svg.dart';
import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/ui/widgets/chain_progress_banner.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart' show Keyring;
import 'package:wuminapp_mobile/my/util/amount_format.dart';
import 'package:wuminapp_mobile/transaction/onchain-transaction/onchain_payment_models.dart';
import 'package:wuminapp_mobile/transaction/onchain-transaction/onchain_payment_service.dart';
import 'package:wuminapp_mobile/rpc/chain_rpc.dart';
import 'package:wuminapp_mobile/rpc/onchain.dart';
import 'package:wuminapp_mobile/transaction/shared/local_tx_store.dart';
import 'package:wuminapp_mobile/isar/wallet_isar.dart';
import 'package:wuminapp_mobile/qr/pages/qr_sign_session_page.dart';
import 'package:wuminapp_mobile/qr/bodies/sign_request_body.dart';
import 'package:wuminapp_mobile/signer/qr_signer.dart';
import 'package:wuminapp_mobile/my/user/user.dart' show ContactBookPage;
import 'package:wuminapp_mobile/my/user/user_service.dart' show UserContact;
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/pages/wallet_page.dart';
import 'package:wuminapp_mobile/wallet/pages/transaction_history_page.dart';

typedef OnchainPaymentExtraEntriesBuilder = List<Widget> Function(
  BuildContext context,
  WalletProfile? currentWallet,
);

typedef OnchainWalletPicker = Future<bool?> Function();
typedef OnchainCurrentWalletLoader = Future<WalletProfile?> Function();
typedef OnchainLocalRecordsLoader = Future<List<LocalTxEntity>> Function(
  String walletPubkeyHex, {
  int limit,
});

class OnchainPaymentPage extends StatelessWidget {
  const OnchainPaymentPage({super.key, this.initialToAddress});

  /// 预填收款地址（从通讯录等入口跳转时使用）。
  final String? initialToAddress;

  @override
  Widget build(BuildContext context) {
    return OnchainPaymentPanel(
      title: '链上支付',
      initialToAddress: initialToAddress,
    );
  }
}

class OnchainPaymentPanel extends StatefulWidget {
  const OnchainPaymentPanel({
    super.key,
    required this.title,
    this.initialToAddress,
    this.extraEntriesBuilder,
    this.walletPicker,
    this.currentWalletLoader,
    this.localRecordsLoader,
    this.enableDelayedLocalRecordRefresh = true,
  });

  final String title;

  /// 预填收款地址（从通讯录等入口跳转时使用）。
  final String? initialToAddress;

  /// 中文注释：交易 Tab 可在链状态提示下方、链上支付表单上方插入入口。
  /// onchain 模块不直接 import offchain / duoqian，跨功能编排留在 ui 层。
  final OnchainPaymentExtraEntriesBuilder? extraEntriesBuilder;

  /// 中文注释：默认打开我的钱包选择页；测试或宿主页面可替换选择流程。
  final OnchainWalletPicker? walletPicker;

  /// 中文注释：默认读取当前激活钱包；测试可替换为内存钱包。
  final OnchainCurrentWalletLoader? currentWalletLoader;

  /// 中文注释：默认读取本地流水；测试可替换为内存流水。
  final OnchainLocalRecordsLoader? localRecordsLoader;

  /// 中文注释：真机保留延迟刷新兜底；widget test 可关闭，避免残留 Timer。
  final bool enableDelayedLocalRecordRefresh;

  @override
  State<OnchainPaymentPanel> createState() => _OnchainPaymentPanelState();
}

class _OnchainPaymentPanelState extends State<OnchainPaymentPanel> {
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
    await _reloadWalletAndLocalRecords();
    // 中文注释：交易流水确认由 ChainTxMonitor 写入，本页只做一次延迟本地刷新；
    // 不再发 nonce 轮询确认 RPC，避免增加节点负担。
    if (!widget.enableDelayedLocalRecordRefresh) {
      return;
    }
    unawaited(Future<void>.delayed(const Duration(seconds: 20), () async {
      if (!mounted || WalletIsar.instance.hasActiveOperation) {
        return;
      }
      await _loadLocalRecords();
    }));
  }

  /// 中文注释：从本地 Isar 加载链上转账记录。
  Future<void> _loadLocalRecords({WalletProfile? wallet}) async {
    final targetWallet = wallet ?? _currentWallet;
    if (targetWallet == null) {
      if (mounted && _localTxRecords.isNotEmpty) {
        setState(() {
          _localTxRecords = [];
        });
      }
      return;
    }
    final targetPubkey = LocalTxStore.normalizePubkey(targetWallet.pubkeyHex);
    try {
      final records = await _queryLocalRecords(
        targetPubkey,
        limit: 100,
      );
      // 中文注释：钱包流水不再保存 direction，支出由 amountDeltaFen 的负号判断。
      final filtered = records
          .where((r) =>
              r.type == 'transfer' && BigInt.parse(r.amountDeltaFen).isNegative)
          .toList();
      if (mounted) {
        final currentPubkey = _walletPubkey(_currentWallet);
        if (currentPubkey != targetPubkey) {
          return;
        }
        setState(() {
          _localTxRecords = filtered;
        });
      }
    } catch (e) {
      if (WalletIsar.instance.isBusyError(e)) {
        return;
      }
      debugPrint('[链上交易] 加载本地记录失败: $e');
    }
  }

  String? _walletPubkey(WalletProfile? wallet) {
    if (wallet == null) return null;
    return LocalTxStore.normalizePubkey(wallet.pubkeyHex);
  }

  Future<List<LocalTxEntity>> _queryLocalRecords(
    String walletPubkeyHex, {
    int limit = 100,
  }) {
    final loader = widget.localRecordsLoader;
    if (loader != null) {
      return loader(walletPubkeyHex, limit: limit);
    }
    return LocalTxStore.queryByWalletPubkey(walletPubkeyHex, limit: limit);
  }

  int _countByStatus(String status) {
    return _localTxRecords.where((r) => r.status == status).length;
  }

  Color _statusColor(String status) {
    switch (status) {
      case LocalTxStore.statusPending:
        return AppTheme.warning;
      case LocalTxStore.statusInBlock:
        return AppTheme.primaryDark;
      case LocalTxStore.statusFinalized:
        return AppTheme.success;
      case 'failed':
        return AppTheme.danger;
      default:
        return AppTheme.textSecondary;
    }
  }

  Future<void> _reloadWallet() async {
    WalletProfile? wallet;
    try {
      final loader = widget.currentWalletLoader;
      wallet = loader != null
          ? await loader()
          : await _paymentService.getCurrentWallet();
    } catch (e, st) {
      if (!WalletIsar.instance.isBusyError(e)) {
        debugPrint('[链上交易] 当前钱包加载失败: $e\n$st');
      }
    }
    if (!mounted) {
      return;
    }
    final nextPubkey = _walletPubkey(wallet);
    final currentPubkey = _walletPubkey(_currentWallet);
    setState(() {
      _currentWallet = wallet;
      _loadingWallet = false;
      if (nextPubkey != currentPubkey) {
        _localTxRecords = [];
      }
    });
  }

  Future<void> _reloadWalletAndLocalRecords() async {
    await _reloadWallet();
    await _loadLocalRecords();
  }

  Future<void> _openMyWalletPage() async {
    final picker = widget.walletPicker;
    final navigator = Navigator.of(context);
    final changed = picker != null
        ? await picker()
        : await navigator.push<bool>(
            MaterialPageRoute(
              builder: (_) => const MyWalletPage(selectForTrade: true),
            ),
          );
    if (!mounted) {
      return;
    }
    if (changed == true) {
      await _reloadWalletAndLocalRecords();
    }
  }

  Future<void> _openContactsPage() async {
    final contact = await Navigator.of(context).push<UserContact>(
      MaterialPageRoute(
        builder: (_) => ContactBookPage(
          selfAddress: _currentWallet?.address ?? '',
          selectForTrade: true,
        ),
      ),
    );
    if (!mounted || contact == null) return;
    setState(() {
      // 中文注释：通讯录、二维码和转账输入框统一使用 SS58 地址。
      _toController.text = contact.address;
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
          // 中文注释：冷钱包确认页按同一金额格式展示，确保 display 字段逐字可核对。
          final amountFormatted = AmountFormat.format(
                  AmountFormat.tryParse(_amountController.text) ?? 0,
                  symbol: '')
              .trim();
          final request = qrSigner.buildRequest(
            requestId: requestId,
            address: wallet.address,
            pubkey: '0x${wallet.pubkeyHex}',
            payloadHex: '0x${_toHex(payload)}',
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

      String? submittedTxHash;
      String? includedBlockHash;
      void handleWatchEvent(TxPoolWatchEvent event) {
        if (!event.isIncluded) return;
        includedBlockHash = event.blockHashHex ?? includedBlockHash;
        final txHash = submittedTxHash;
        final wallet = _currentWallet;
        if (txHash == null || wallet == null) return;
        unawaited(LocalTxStore.markLocalSubmitInBlock(
          walletPubkeyHex: wallet.pubkeyHex,
          txHash: txHash,
          blockHash: event.blockHashHex,
        ).then((_) => _loadLocalRecords()));
      }

      final result = await _paymentService.submitTransfer(
        OnchainPaymentDraft(
          toAddress: toAddress,
          amount: amount,
          symbol: _selectedSymbol,
        ),
        sign: signCallback,
        onWatchEvent: handleWatchEvent,
      );
      if (!mounted) {
        return;
      }

      // 交易已成功提交，后续写入本地记录失败不影响交易结果
      final txHash = result.txHash.toLowerCase();
      submittedTxHash = txHash;
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('签名成功，交易已发送，tx=$txHash')));
      _toController.clear();
      _amountController.clear();

      // 写入本地交易记录（失败不影响交易）
      try {
        final transferAmountFen = LocalTxStore.fenFromYuan(amount);
        final feeFen = LocalTxStore.fenFromYuan(estimatedFee);
        final amountDeltaFen =
            (-(BigInt.parse(transferAmountFen) + BigInt.parse(feeFen)))
                .toString();
        await LocalTxStore.upsertLocalSubmitTransfer(
          walletAddress: _currentWallet!.address,
          walletPubkeyHex: _currentWallet!.pubkeyHex,
          txHash: txHash,
          amountDeltaFen: amountDeltaFen,
          transferAmountFen: transferAmountFen,
          feeFen: feeFen,
          counterpartyAddress: toAddress,
          fromAddress: _currentWallet!.address,
          toAddress: toAddress,
          usedNonce: result.usedNonce,
          createdAtMillis: DateTime.now().millisecondsSinceEpoch,
          blockHash: includedBlockHash,
        );
        if (includedBlockHash != null) {
          await LocalTxStore.markLocalSubmitInBlock(
            walletPubkeyHex: _currentWallet!.pubkeyHex,
            txHash: txHash,
            blockHash: includedBlockHash,
          );
        }
        if (mounted) await _loadLocalRecords();

        // 中文注释：本机先展示 pending；交易池 inBlock 回调会升级为已出块，
        // finalized 区块事件再升级为已确认。这里仅兜底延迟刷新本地列表。
        unawaited(_reloadAfterChainEventWindow());
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

  Future<void> _reloadAfterChainEventWindow() async {
    await Future<void>.delayed(const Duration(seconds: 20));
    if (!mounted || WalletIsar.instance.hasActiveOperation) {
      return;
    }
    await _loadLocalRecords();
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
                      _buildStatusText(
                          '已提交',
                          _countByStatus(LocalTxStore.statusPending),
                          _statusColor(LocalTxStore.statusPending)),
                      _buildStatusText(
                          '已出块',
                          _countByStatus(LocalTxStore.statusInBlock),
                          _statusColor(LocalTxStore.statusInBlock)),
                      _buildStatusText(
                          '已确认',
                          _countByStatus(LocalTxStore.statusFinalized),
                          _statusColor(LocalTxStore.statusFinalized)),
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
                                walletPubkeyHex: _currentWallet!.pubkeyHex,
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
                  Expanded(
                    child: Center(
                      child: Text(
                        widget.title,
                        style: const TextStyle(
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
                  if (widget.extraEntriesBuilder != null)
                    ...widget.extraEntriesBuilder!(context, _currentWallet),
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
