import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'package:wuminapp_mobile/ui/app_theme.dart';
import 'package:wuminapp_mobile/wallet/core/wallet_manager.dart';
import 'package:wuminapp_mobile/wallet/ui/cards/wallet_qr_dialog.dart';

/// 钱包身份卡(钱包详情页第 2 张卡)。
///
/// 中文注释:
/// - 样式参照 wumin 冷钱包 `wumin/lib/ui/wallet_detail_page.dart:277-342`,
///   翠绿 primaryGradient 背景 + 左图标 + 中钱包名/地址 + 右 QR 图标。
/// - 钱包名可点击进入编辑态;提交(回车 / onTapOutside)时通过 [onNameChanged]
///   回调让外层落盘。空字符串或与现值相同则回滚不报错,由编辑态自行处理。
/// - 地址点击复制并弹 SnackBar,展示规则为短地址 `前 8...后 6`。
/// - 右侧 QR 小图标弹出 WalletQrDialog,内容 `user_contact` 维持 WUMIN_QR_V1。
/// - 编辑态 TextField 字体/光标/下划线改黑色系,避开 Material TextField 默认
///   白底导致白字看不见的问题。展示态保持白色不变。
class WalletIdentityCard extends StatefulWidget {
  const WalletIdentityCard({
    super.key,
    required this.wallet,
    required this.onNameChanged,
  });

  final WalletProfile wallet;

  /// 钱包名提交回调。外层负责持久化,Widget 内部已做 trim 和空值回滚。
  final Future<void> Function(String) onNameChanged;

  @override
  State<WalletIdentityCard> createState() => _WalletIdentityCardState();
}

class _WalletIdentityCardState extends State<WalletIdentityCard> {
  /// 当前展示态的钱包名(与 widget.wallet.walletName 同步,编辑提交后更新)。
  late String _walletName;

  /// 是否处于编辑态。
  bool _isEditingName = false;

  /// 编辑态 TextField 的 controller。
  late final TextEditingController _nameController;

  @override
  void initState() {
    super.initState();
    _walletName = widget.wallet.walletName;
    _nameController = TextEditingController(text: _walletName);
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  /// 短地址:前 8 位 + ... + 后 6 位。地址过短时按原样返回。
  String get _shortAddress {
    final addr = widget.wallet.address;
    if (addr.length <= 14) return addr;
    return '${addr.substring(0, 8)}...${addr.substring(addr.length - 6)}';
  }

  /// 提交钱包名。trim 后空或与当前相同则回滚编辑态,不调用回调。
  Future<void> _submitName(String raw) async {
    final trimmed = raw.trim();
    if (trimmed.isEmpty || trimmed == _walletName) {
      setState(() {
        _isEditingName = false;
        _nameController.text = _walletName;
      });
      return;
    }
    try {
      await widget.onNameChanged(trimmed);
      if (!mounted) return;
      setState(() {
        _walletName = trimmed;
        _isEditingName = false;
      });
    } catch (_) {
      // 落盘失败由外层回调自行 SnackBar,这里仅负责回滚编辑态。
      if (!mounted) return;
      setState(() {
        _isEditingName = false;
        _nameController.text = _walletName;
      });
    }
  }

  /// 复制钱包地址并弹 SnackBar。
  void _copyAddress() {
    Clipboard.setData(ClipboardData(text: widget.wallet.address));
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('钱包地址已复制')),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(20),
      decoration: BoxDecoration(
        gradient: AppTheme.primaryGradient,
        borderRadius: BorderRadius.circular(AppTheme.radiusLg),
        boxShadow: [
          BoxShadow(
            color: AppTheme.primary.withAlpha(40),
            blurRadius: 16,
            offset: const Offset(0, 6),
          ),
        ],
      ),
      child: Row(
        children: [
          // 左:钱包图标 48x48 半透明白底。
          Container(
            width: 48,
            height: 48,
            decoration: BoxDecoration(
              color: Colors.white.withAlpha(30),
              borderRadius: BorderRadius.circular(12),
            ),
            child: const Icon(
              Icons.account_balance_wallet_rounded,
              color: Colors.white,
              size: 24,
            ),
          ),
          const SizedBox(width: 14),
          // 中:钱包名(点击可编辑) + 短地址(点击复制)。
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _isEditingName
                    ? TextField(
                        controller: _nameController,
                        autofocus: true,
                        style: const TextStyle(
                          fontSize: 18,
                          fontWeight: FontWeight.w700,
                          color: Colors.black87,
                        ),
                        cursorColor: Colors.black87,
                        decoration: const InputDecoration(
                          isDense: true,
                          contentPadding: EdgeInsets.symmetric(vertical: 4),
                          enabledBorder: UnderlineInputBorder(
                            borderSide: BorderSide(color: Colors.black54),
                          ),
                          focusedBorder: UnderlineInputBorder(
                            borderSide: BorderSide(color: Colors.black54),
                          ),
                        ),
                        textInputAction: TextInputAction.done,
                        onSubmitted: _submitName,
                        onTapOutside: (_) {
                          _submitName(_nameController.text);
                        },
                      )
                    : GestureDetector(
                        onTap: () {
                          setState(() {
                            _isEditingName = true;
                            _nameController.text = _walletName;
                          });
                        },
                        child: Text(
                          _walletName,
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: const TextStyle(
                            fontSize: 18,
                            fontWeight: FontWeight.w700,
                            color: Colors.white,
                          ),
                        ),
                      ),
                const SizedBox(height: 4),
                GestureDetector(
                  onTap: _copyAddress,
                  child: Text(
                    _shortAddress,
                    style: TextStyle(
                      fontSize: 13,
                      color: Colors.white.withAlpha(180),
                      fontFamily: 'monospace',
                    ),
                  ),
                ),
              ],
            ),
          ),
          // 右:QR 小图标 36x36,点击弹大二维码。
          GestureDetector(
            onTap: () => WalletQrDialog.show(
              context,
              wallet: widget.wallet,
              name: _walletName,
            ),
            child: Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: Colors.white.withAlpha(30),
                borderRadius: BorderRadius.circular(8),
              ),
              child: const Icon(
                Icons.qr_code_rounded,
                color: Colors.white,
                size: 20,
              ),
            ),
          ),
        ],
      ),
    );
  }
}
