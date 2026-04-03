import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:polkadart_keyring/polkadart_keyring.dart';

import '../qr/pages/qr_sign_session_page.dart';
import '../signer/qr_signer.dart';
import '../ui/app_theme.dart';
import '../wallet/core/wallet_manager.dart';
import 'activation_service.dart';
import 'institution_data.dart';

/// 管理员列表页面。
///
/// 展示机构所有管理员的完整 SS58 地址，支持 QR 扫码激活。
class AdminListPage extends StatefulWidget {
  const AdminListPage({
    super.key,
    required this.institution,
    required this.admins,
    required this.importedColdPubkeys,
    required this.activatedPubkeys,
    required this.badgeColor,
    this.onActivated,
  });

  final InstitutionInfo institution;
  final List<String> admins;

  /// 用户已导入的冷钱包公钥集合（小写 hex，不含 0x）。
  final Set<String> importedColdPubkeys;

  /// 已激活的管理员公钥集合（小写 hex）。
  final Set<String> activatedPubkeys;
  final Color badgeColor;

  /// 激活成功后的回调（通知父页面刷新）。
  final VoidCallback? onActivated;

  @override
  State<AdminListPage> createState() => _AdminListPageState();
}

class _AdminListPageState extends State<AdminListPage> {
  late Set<String> _activatedPubkeys;

  @override
  void initState() {
    super.initState();
    _activatedPubkeys = Set.of(widget.activatedPubkeys);
  }

  void _onAdminActivated(String pubkeyHex) {
    setState(() {
      _activatedPubkeys.add(pubkeyHex);
    });
    widget.onActivated?.call();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '管理员列表',
          style: TextStyle(fontSize: 17, fontWeight: FontWeight.w700),
        ),
        centerTitle: true,
        backgroundColor: Colors.white,
        foregroundColor: AppTheme.primaryDark,
        elevation: 0,
        scrolledUnderElevation: 0.5,
      ),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: [
          // 机构信息
          _buildInstitutionHeader(),
          const SizedBox(height: 16),
          // 管理员总数
          Text(
            '共 ${widget.admins.length} 位管理员　通过阈值 ${widget.institution.internalThreshold}',
            style: TextStyle(fontSize: 13, color: AppTheme.textTertiary),
          ),
          const SizedBox(height: 12),
          // 管理员列表
          if (widget.admins.isEmpty)
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 24),
              child: Center(
                child: Text(
                  '暂无管理员信息',
                  style: TextStyle(fontSize: 14, color: AppTheme.textTertiary),
                ),
              ),
            )
          else
            ...List.generate(widget.admins.length, (index) {
              final pubkey = widget.admins[index];
              final isImported = widget.importedColdPubkeys.contains(pubkey);
              final isActivated = _activatedPubkeys.contains(pubkey);
              return _AdminTile(
                index: index + 1,
                pubkeyHex: pubkey,
                isImported: isImported,
                isActivated: isActivated,
                institution: widget.institution,
                onActivated: () => _onAdminActivated(pubkey),
              );
            }),
        ],
      ),
    );
  }

  Widget _buildInstitutionHeader() {
    return Row(
      children: [
        Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: widget.badgeColor.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(10),
          ),
          child:
              Icon(Icons.people_outline, size: 18, color: widget.badgeColor),
        ),
        const SizedBox(width: 10),
        Expanded(
          child: Text(
            widget.institution.name,
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: AppTheme.primaryDark,
            ),
          ),
        ),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
          decoration: BoxDecoration(
            color: widget.badgeColor.withValues(alpha: 0.10),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Text(
            OrgType.label(widget.institution.orgType),
            style: TextStyle(
              fontSize: 11,
              color: widget.badgeColor,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
      ],
    );
  }
}

class _AdminTile extends StatelessWidget {
  const _AdminTile({
    required this.index,
    required this.pubkeyHex,
    required this.isImported,
    required this.isActivated,
    required this.institution,
    required this.onActivated,
  });

  final int index;
  final String pubkeyHex;

  /// 用户是否已导入此公钥的冷钱包。
  final bool isImported;

  /// 此管理员是否已激活。
  final bool isActivated;
  final InstitutionInfo institution;
  final VoidCallback onActivated;

  String _toSs58() {
    try {
      final bytes = _hexToBytes(pubkeyHex);
      return Keyring().encodeAddress(bytes, 2027);
    } catch (_) {
      return '0x$pubkeyHex';
    }
  }

  static Uint8List _hexToBytes(String hex) {
    final clean = hex.startsWith('0x') ? hex.substring(2) : hex;
    final result = Uint8List(clean.length ~/ 2);
    for (var i = 0; i < result.length; i++) {
      result[i] = int.parse(clean.substring(i * 2, i * 2 + 2), radix: 16);
    }
    return result;
  }

  Future<void> _startActivation(BuildContext context) async {
    // 检查是否为冷钱包（热钱包不允许激活）
    final wallets = await WalletManager().getWallets();
    final wallet = wallets.where((w) {
      final pk = w.pubkeyHex.toLowerCase().replaceFirst('0x', '');
      return pk == pubkeyHex;
    }).firstOrNull;

    if (wallet == null) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('未找到对应钱包')),
      );
      return;
    }

    if (wallet.isHotWallet) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('管理员仅支持冷钱包激活')),
      );
      return;
    }

    // 构建激活签名请求
    final activationService = ActivationService();
    final (:request, :json) = activationService.buildActivationRequest(
      pubkeyHex: pubkeyHex,
      shenfenId: institution.shenfenId,
    );

    if (!context.mounted) return;

    // 跳转 QR 签名会话页
    final response = await Navigator.of(context).push<QrSignResponse>(
      MaterialPageRoute(
        builder: (_) => QrSignSessionPage(
          request: request,
          requestJson: json,
          expectedPubkey: pubkeyHex,
        ),
      ),
    );

    if (response == null || !context.mounted) return;

    // 验证并存储激活记录
    try {
      await activationService.activateViaQr(
        pubkeyHex: pubkeyHex,
        shenfenId: institution.shenfenId,
        response: response,
      );
      onActivated();
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('管理员激活成功')),
      );
    } catch (e) {
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('激活失败：$e')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final address = _toSs58();

    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      decoration: BoxDecoration(
        color: isActivated
            ? AppTheme.success.withValues(alpha: 0.06)
            : AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(
          color: isActivated
              ? AppTheme.success.withValues(alpha: 0.3)
              : AppTheme.border,
        ),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 24,
            child: Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Text(
                '$index',
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                  color: AppTheme.textTertiary,
                ),
              ),
            ),
          ),
          Expanded(
            child: Text(
              address,
              style: TextStyle(
                fontSize: 12,
                fontFamily: 'monospace',
                color: isActivated
                    ? AppTheme.primaryDark
                    : AppTheme.textSecondary,
                height: 1.4,
              ),
            ),
          ),
          const SizedBox(width: 6),
          // 激活按钮：仅对已导入冷钱包的管理员显示
          if (isImported)
            isActivated
                ? Container(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                    decoration: BoxDecoration(
                      color: AppTheme.success.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: const Text(
                      '已激活',
                      style: TextStyle(
                        fontSize: 11,
                        color: AppTheme.success,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  )
                : GestureDetector(
                    onTap: () => _startActivation(context),
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                          horizontal: 8, vertical: 4),
                      decoration: BoxDecoration(
                        color: AppTheme.primary.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(8),
                      ),
                      child: const Text(
                        '激活',
                        style: TextStyle(
                          fontSize: 11,
                          color: AppTheme.primary,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                  ),
        ],
      ),
    );
  }
}
