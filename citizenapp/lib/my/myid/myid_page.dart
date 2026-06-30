import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:citizenapp/my/myid/myid_service.dart';
import 'package:citizenapp/my/myid/myid_sign_page.dart';
import 'package:citizenapp/qr/bodies/user_contact_body.dart';
import 'package:citizenapp/qr/envelope.dart';
import 'package:citizenapp/qr/qr_protocols.dart';
import 'package:citizenapp/ui/app_theme.dart';
import 'package:citizenapp/wallet/core/wallet_manager.dart';
import 'package:citizenapp/wallet/pages/wallet_page.dart';
import 'package:qr_flutter/qr_flutter.dart';

class MyIdPage extends StatefulWidget {
  const MyIdPage({super.key, this.myIdService});

  final MyIdService? myIdService;

  @override
  State<MyIdPage> createState() => _MyIdPageState();
}

class _MyIdPageState extends State<MyIdPage> {
  late final MyIdService _myIdService;

  MyIdState _state = const MyIdState(archiveStatus: MyIdArchiveStatus.unset);
  bool _submitting = false;

  @override
  void initState() {
    super.initState();
    _myIdService = widget.myIdService ?? MyIdService();
    _loadState();
  }

  Future<void> _loadState() async {
    final localState = await _myIdService.getState();
    if (!mounted) return;
    setState(() {
      _state = localState;
    });
    final synced = await _myIdService.syncFromBackend();
    if (!mounted) return;
    setState(() {
      _state = synced;
    });
  }

  Future<void> _selectWallet() async {
    if (_submitting) return;
    final wallet = await Navigator.of(context).push<WalletProfile>(
      MaterialPageRoute(
        builder: (_) => const MyWalletPage(
          selectForBind: true,
          bindPurposeLabel: '电子护照',
        ),
      ),
    );
    if (!mounted || wallet == null) return;
    setState(() {
      _submitting = true;
    });
    try {
      final nextState = await _myIdService.selectWallet(
        walletAddress: wallet.address,
        walletPubkeyHex: wallet.pubkeyHex,
        walletIndex: wallet.walletIndex,
        isColdWallet: wallet.isColdWallet,
      );
      if (!mounted) return;
      setState(() {
        _state = nextState;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('钱包已选择')),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('选择钱包失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() {
          _submitting = false;
        });
      }
    }
  }

  String _statusLabel() {
    return switch (_state.archiveStatus) {
      MyIdArchiveStatus.unset => '未设置',
      MyIdArchiveStatus.pending => '待登记',
      MyIdArchiveStatus.registered => '已登记',
    };
  }

  Color _statusColor() {
    return switch (_state.archiveStatus) {
      MyIdArchiveStatus.unset => AppTheme.textTertiary,
      MyIdArchiveStatus.pending => AppTheme.warning,
      MyIdArchiveStatus.registered => AppTheme.success,
    };
  }

  Widget _buildStatusBadge() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: _statusColor().withAlpha(25),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Text(
        _statusLabel(),
        style: TextStyle(
          fontSize: 12,
          color: _statusColor(),
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }

  String _identityIdText() {
    final cidNumber = _state.cidNumber?.trim();
    return cidNumber == null || cidNumber.isEmpty ? '未生成' : cidNumber;
  }

  String _passportNoText() {
    final passportNo = _state.passportNo?.trim();
    return passportNo == null || passportNo.isEmpty ? '未生成' : passportNo;
  }

  String _identityStatusText() {
    // 中文注释：identityStatus 是身份 CID 状态,不是本机档案状态；
    // 只有 CID 明确返回 NORMAL 才显示正常，其他状态统一按异常展示。
    return _state.identityStatus?.trim().toUpperCase() == 'NORMAL'
        ? '状态：正常'
        : '状态：异常';
  }

  String _validityText() {
    final validFrom = _formatDate(_state.passportValidFrom);
    final validUntil = _formatDate(_state.passportValidUntil);
    if (validFrom == null || validUntil == null) {
      return '有效期：未生成';
    }
    return '有效期：$validFrom-$validUntil';
  }

  String? _walletQrPayload() {
    final address = _state.walletAddress?.trim();
    if (address == null || address.isEmpty) return null;
    return QrEnvelope<UserContactBody>(
      kind: QrKind.userContact,
      id: null,
      issuedAt: null,
      expiresAt: null,
      body: UserContactBody(address: address, contactName: '电子护照钱包'),
    ).toRawJson();
  }

  String? _formatDate(String? raw) {
    final value = raw?.trim();
    if (value == null || value.isEmpty) return null;
    final parts = value.split('-');
    if (parts.length != 3) return null;
    final year = int.tryParse(parts[0]);
    final month = int.tryParse(parts[1]);
    final day = int.tryParse(parts[2]);
    if (year == null || month == null || day == null) return null;
    // 中文注释：后端返回 YYYY-MM-DD 日期，不按本地时区转换，避免护照日期跨天。
    return '${year.toString().padLeft(4, '0')}年'
        '${month.toString().padLeft(2, '0')}月'
        '${day.toString().padLeft(2, '0')}日';
  }

  Future<void> _openSignPage() async {
    final walletIndex = _state.walletIndex;
    final address = _state.walletAddress?.trim();
    final pubkey = _state.walletPubkeyHex?.trim().replaceFirst('0x', '');
    if (walletIndex == null ||
        address == null ||
        address.isEmpty ||
        pubkey == null ||
        pubkey.isEmpty ||
        _state.isColdWallet) {
      return;
    }
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (_) => MyIdSignPage(
          wallet: WalletProfile(
            walletIndex: walletIndex,
            walletName: '电子护照钱包',
            walletIcon: 'account_balance_wallet',
            balance: 0,
            address: address,
            pubkeyHex: pubkey,
            alg: 'sr25519',
            ss58: 2027,
            createdAtMillis: _state.updatedAtMillis ?? 0,
            source: 'myid',
            signMode: 'local',
          ),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final walletQrPayload = _walletQrPayload();
    final actionLabel = _state.walletAddress == null
        ? '选择钱包'
        : _state.archiveStatus == MyIdArchiveStatus.registered
            ? '更新钱包'
            : '更换钱包';
    final canSign = _state.walletIndex != null &&
        !_state.isColdWallet &&
        (_state.walletAddress?.trim().isNotEmpty ?? false) &&
        (_state.walletPubkeyHex?.trim().isNotEmpty ?? false);
    final hasSelectedWallet = _state.walletAddress?.trim().isNotEmpty ?? false;
    final showPendingActionRow = hasSelectedWallet &&
        _state.archiveStatus != MyIdArchiveStatus.registered;
    return Scaffold(
      appBar: AppBar(
        title: const Text('电子护照'),
        centerTitle: true,
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Container(
            padding: const EdgeInsets.all(16),
            decoration: AppTheme.cardDecoration(),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    Container(
                      width: 44,
                      height: 44,
                      decoration: BoxDecoration(
                        color: AppTheme.primary.withAlpha(18),
                        borderRadius: BorderRadius.circular(AppTheme.radiusSm),
                      ),
                      child: const Icon(
                        Icons.badge_outlined,
                        color: AppTheme.primary,
                        size: 24,
                      ),
                    ),
                    const SizedBox(width: 12),
                    const Expanded(
                      child: Text(
                        '电子护照',
                        style: TextStyle(
                          fontSize: 20,
                          fontWeight: FontWeight.w700,
                          color: AppTheme.textPrimary,
                        ),
                      ),
                    ),
                    _buildStatusBadge(),
                  ],
                ),
                const SizedBox(height: 18),
                const Text(
                  '护照号',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _passportNoText(),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 15,
                    color: AppTheme.textPrimary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 14),
                const Text(
                  '身份 CID',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _identityIdText(),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 15,
                    color: AppTheme.textPrimary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 14),
                const Text(
                  '投票账户地址',
                  style: TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _state.walletAddress ?? '未设置',
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 15,
                    color: AppTheme.textPrimary,
                    height: 1.4,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _identityStatusText(),
                  style: TextStyle(
                    fontSize: 13,
                    color:
                        _state.identityStatus?.trim().toUpperCase() == 'NORMAL'
                            ? AppTheme.success
                            : AppTheme.danger,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  _validityText(),
                  style: const TextStyle(
                    fontSize: 13,
                    color: AppTheme.textSecondary,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ],
            ),
          ),
          if (walletQrPayload != null) ...[
            const SizedBox(height: 16),
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16),
              decoration: AppTheme.cardDecoration(),
              child: Column(
                children: [
                  QrImageView(
                    data: walletQrPayload,
                    version: QrVersions.auto,
                    size: 220,
                    eyeStyle: const QrEyeStyle(
                      eyeShape: QrEyeShape.square,
                      color: AppTheme.primary,
                    ),
                    dataModuleStyle: const QrDataModuleStyle(
                      dataModuleShape: QrDataModuleShape.square,
                      color: AppTheme.primary,
                    ),
                  ),
                  const SizedBox(height: 10),
                  const Text(
                    '钱包地址二维码',
                    style: TextStyle(
                      fontSize: 13,
                      color: AppTheme.textSecondary,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ],
              ),
            ),
          ],
          const SizedBox(height: 16),
          if (showPendingActionRow)
            Row(
              children: [
                Expanded(
                  child: FilledButton.icon(
                    onPressed: _submitting ? null : _selectWallet,
                    icon: _submitting
                        ? const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          )
                        : const Icon(Icons.account_balance_wallet_outlined),
                    label: Text(_submitting ? '处理中...' : '更换钱包'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: canSign ? _openSignPage : null,
                    icon: SvgPicture.asset(
                      'assets/icons/scan-line.svg',
                      width: 18,
                      height: 18,
                      colorFilter: const ColorFilter.mode(
                        AppTheme.primary,
                        BlendMode.srcIn,
                      ),
                    ),
                    label: const Text('扫码签名'),
                  ),
                ),
              ],
            )
          else
            SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: _submitting ? null : _selectWallet,
                icon: _submitting
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.account_balance_wallet_outlined),
                label: Text(_submitting ? '处理中...' : actionLabel),
              ),
            ),
        ],
      ),
    );
  }
}
