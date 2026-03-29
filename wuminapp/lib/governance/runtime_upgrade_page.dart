import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import '../ui/app_theme.dart';
import 'package:flutter/services.dart';
import 'package:file_picker/file_picker.dart';

import 'runtime_upgrade_service.dart';
import '../qr/pages/qr_sign_session_page.dart';
import '../rpc/chain_rpc.dart';
import '../signer/qr_signer.dart';
import '../wallet/core/wallet_manager.dart';
import '../wallet/capabilities/api_client.dart';

/// Runtime 升级提案创建页面。
class RuntimeUpgradePage extends StatefulWidget {
  const RuntimeUpgradePage({super.key, required this.adminWallets});

  final List<WalletProfile> adminWallets;

  @override
  State<RuntimeUpgradePage> createState() => _RuntimeUpgradePageState();
}

class _RuntimeUpgradePageState extends State<RuntimeUpgradePage> {

  final _reasonController = TextEditingController();

  bool _submitting = false;
  bool _fetchingSnapshot = false;
  late WalletProfile _selectedWallet;

  String? _wasmFileName;
  int? _wasmFileSize;
  Uint8List? _wasmCode;
  String? _reasonError;

  @override
  void initState() {
    super.initState();
    _selectedWallet = widget.adminWallets.first;
  }

  @override
  void dispose() {
    _reasonController.dispose();
    super.dispose();
  }

  bool _validateReason() {
    final reason = _reasonController.text.trim();
    if (reason.isEmpty) {
      setState(() => _reasonError = '请输入升级理由');
      return false;
    }
    final bytes = utf8.encode(reason);
    if (bytes.length > 1024) {
      setState(() => _reasonError = '升级理由超过 1024 字节限制（当前 ${bytes.length} 字节）');
      return false;
    }
    setState(() => _reasonError = null);
    return true;
  }

  Future<void> _pickWasmFile() async {
    try {
      final result = await FilePicker.platform.pickFiles(
        type: FileType.any,
        withData: true,
        // iOS 上 withReadStream 作为备选
      );
      if (result == null || result.files.isEmpty) return;
      final file = result.files.first;

      Uint8List? fileBytes = file.bytes;

      // iOS 上 withData 可能返回 null，需通过文件路径读取
      if (fileBytes == null && file.path != null) {
        final f = File(file.path!);
        if (await f.exists()) {
          fileBytes = await f.readAsBytes();
        }
      }

      if (fileBytes == null) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('无法读取文件内容')),
        );
        return;
      }
      // 校验文件格式：
      // 1. 标准 WASM 魔数：\0asm（0x00 0x61 0x73 0x6D）
      // 2. Substrate 压缩格式（.compact.compressed.wasm）：前缀不固定，
      //    但 Substrate set_code 会自动解压，两种格式均合法。
      // 只拒绝明显不是 WASM 的文件（太小）。
      if (fileBytes.length < 8) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('文件太小，不是有效的 WASM 二进制')),
        );
        return;
      }
      const maxSize = 5 * 1024 * 1024;
      if (fileBytes.length > maxSize) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('文件大小超过 5MB 限制')),
        );
        return;
      }
      if (!mounted) return;
      setState(() {
        _wasmFileName = file.name;
        _wasmFileSize = fileBytes!.length;
        _wasmCode = Uint8List.fromList(fileBytes);
      });
    } catch (e) {
      debugPrint('[RuntimeUpgrade] 文件选择失败: $e');
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('文件选择失败：$e')),
      );
    }
  }

  String _formatFileSize(int bytes) {
    if (bytes < 1024) return '$bytes B';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(1)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(2)} MB';
  }

  Future<void> _submit() async {
    if (!_validateReason()) return;

    if (_wasmCode == null) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('请选择 WASM 文件')),
      );
      return;
    }

    final wallet = _selectedWallet;

    setState(() => _submitting = true);

    try {
      // 获取人口快照
      setState(() => _fetchingSnapshot = true);
      final apiClient = ApiClient();
      final snapshot =
          await apiClient.fetchPopulationSnapshot(wallet.pubkeyHex);
      if (!mounted) return;
      setState(() => _fetchingSnapshot = false);

      Future<Uint8List> signCallback(Uint8List payload) async {
        // 管理员操作统一通过 QR 码签名（wumin 冷钱包）
        final qrSigner = QrSigner();
        final rv = await ChainRpc().fetchRuntimeVersion();
        final request = qrSigner.buildRequest(
          requestId: QrSigner.generateRequestId(prefix: 'upgrade-'),
          account: wallet.address,
          pubkey: '0x${wallet.pubkeyHex}',
          payloadHex: '0x${_toHex(payload)}',
          specVersion: rv.specVersion,
          display: {
            'action': 'propose_upgrade',
            'action_label': '升级提案',
            'summary': '提交运行时升级提案',
            'fields': [],
          },
        );
        final requestJson = qrSigner.encodeRequest(request);
        final response = await Navigator.push<QrSignResponse>(
          context,
          MaterialPageRoute(
            builder: (_) => QrSignSessionPage(
                request: request,
                requestJson: requestJson,
                expectedPubkey: '0x${wallet.pubkeyHex}'),
          ),
        );
        if (response == null) throw Exception('签名已取消');
        return Uint8List.fromList(_hexToBytes(response.signature));
      }

      final signerPubkey = Uint8List.fromList(_hexToBytes(wallet.pubkeyHex));

      // 将 SFID 返回的 nonce（UTF-8 字符串）和 signature（hex）转为原始字节
      final nonceBytes =
          Uint8List.fromList(utf8.encode(snapshot.snapshotNonce));
      final sigHex = snapshot.signature;
      final sigClean = sigHex.startsWith('0x') ? sigHex.substring(2) : sigHex;
      final sigBytes = Uint8List(sigClean.length ~/ 2);
      for (var i = 0; i < sigBytes.length; i++) {
        sigBytes[i] =
            int.parse(sigClean.substring(i * 2, i * 2 + 2), radix: 16);
      }

      final service = RuntimeUpgradeService();
      await service.submitProposeRuntimeUpgrade(
        reason: _reasonController.text.trim(),
        wasmCode: _wasmCode!,
        eligibleTotal: snapshot.eligibleTotal,
        snapshotNonce: nonceBytes,
        signature: sigBytes,
        fromAddress: wallet.address,
        signerPubkey: signerPubkey,
        sign: signCallback,
      );

      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('提交成功')),
      );
      Navigator.of(context).pop(true);
    } on WalletAuthException catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(e.message)),
      );
    } catch (e) {
      if (!mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('提交失败：$e')),
      );
    } finally {
      if (mounted) {
        setState(() {
          _submitting = false;
          _fetchingSnapshot = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.white,
      appBar: AppBar(
        title: const Text(
          '发起升级提案',
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
          // ──── 标题 ────
          _buildTitleHeader(),
          const SizedBox(height: 16),

          // ──── 发起管理员 ────
          _buildLabel('发起管理员'),
          const SizedBox(height: 6),
          _buildAdminSelector(),
          const SizedBox(height: 16),

          // ──── 升级理由 ────
          _buildLabel('升级理由'),
          const SizedBox(height: 6),
          TextField(
            controller: _reasonController,
            maxLines: 5,
            decoration: InputDecoration(
              hintText: '输入升级理由（最多 341 个汉字）',
              hintStyle: TextStyle(color: AppTheme.textTertiary, fontSize: 14),
              filled: true,
              fillColor: AppTheme.surfaceMuted,
              enabledBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.border),
              ),
              focusedBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.primaryDark),
              ),
              errorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
              ),
              focusedErrorBorder: OutlineInputBorder(
                borderRadius: BorderRadius.circular(8),
                borderSide: const BorderSide(color: AppTheme.danger),
              ),
              errorText: _reasonError,
            ),
            style: const TextStyle(fontSize: 14),
          ),
          const SizedBox(height: 16),

          // ──── WASM 文件选择 ────
          _buildLabel('WASM 文件'),
          const SizedBox(height: 6),
          _buildWasmFilePicker(),
          const SizedBox(height: 24),

          // ──── 提交按钮 ────
          SizedBox(
            width: double.infinity,
            height: 48,
            child: FilledButton(
              style: FilledButton.styleFrom(
                backgroundColor: AppTheme.primaryDark,
                shape: RoundedRectangleBorder(
                  borderRadius: BorderRadius.circular(10),
                ),
              ),
              onPressed: _submitting ? null : _submit,
              child: _submitting
                  ? Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const SizedBox(
                          width: 20,
                          height: 20,
                          child: CircularProgressIndicator(
                            strokeWidth: 2,
                            color: Colors.white,
                          ),
                        ),
                        if (_fetchingSnapshot) ...[
                          const SizedBox(width: 8),
                          const Text(
                            '获取人口快照...',
                            style: TextStyle(
                              fontSize: 14,
                              color: Colors.white,
                            ),
                          ),
                        ],
                      ],
                    )
                  : const Text(
                      '提交升级提案',
                      style: TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: Colors.white,
                      ),
                    ),
            ),
          ),
        ],
      ),
    );
  }

  // ──── 子组件 ────

  Widget _buildTitleHeader() {
    return Row(
      children: [
        Container(
          width: 36,
          height: 36,
          decoration: BoxDecoration(
            color: AppTheme.info.withValues(alpha: 0.12),
            borderRadius: BorderRadius.circular(10),
          ),
          child: const Icon(Icons.arrow_upward,
              size: 18, color: AppTheme.info),
        ),
        const SizedBox(width: 10),
        const Expanded(
          child: Text(
            'Runtime 升级提案',
            style: TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: AppTheme.primaryDark,
            ),
          ),
        ),
      ],
    );
  }

  String _truncateAddress(String address) {
    if (address.length <= 16) return address;
    return '${address.substring(0, 8)}...${address.substring(address.length - 8)}';
  }

  Widget _buildAdminSelector() {
    final wallets = widget.adminWallets;
    if (wallets.length == 1) {
      // 只有一个管理员钱包，直接展示
      return Container(
        width: double.infinity,
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
        decoration: BoxDecoration(
          color: AppTheme.success.withValues(alpha: 0.06),
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: AppTheme.success.withValues(alpha: 0.2)),
        ),
        child: Row(
          children: [
            const Icon(Icons.verified_user, size: 16, color: AppTheme.success),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                _truncateAddress(wallets.first.address),
                style: const TextStyle(
                  fontSize: 13,
                  fontFamily: 'monospace',
                  color: AppTheme.success,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
          ],
        ),
      );
    }
    // 多个管理员钱包，下拉选择
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12),
      decoration: BoxDecoration(
        color: AppTheme.surfaceMuted,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: AppTheme.success.withValues(alpha: 0.3)),
      ),
      child: DropdownButtonHideUnderline(
        child: DropdownButton<int>(
          value: _selectedWallet.walletIndex,
          isExpanded: true,
          icon: const Icon(Icons.arrow_drop_down, color: AppTheme.primaryDark),
          items: wallets.map((w) {
            return DropdownMenuItem<int>(
              value: w.walletIndex,
              child: Row(
                children: [
                  const Icon(Icons.verified_user,
                      size: 14, color: AppTheme.success),
                  const SizedBox(width: 6),
                  Expanded(
                    child: Text(
                      _truncateAddress(w.address),
                      style: const TextStyle(
                        fontSize: 13,
                        fontFamily: 'monospace',
                      ),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            );
          }).toList(),
          onChanged: (index) {
            if (index == null) return;
            setState(() {
              _selectedWallet =
                  wallets.firstWhere((w) => w.walletIndex == index);
            });
          },
        ),
      ),
    );
  }

  Widget _buildLabel(String text) {
    return Text(
      text,
      style: const TextStyle(
        fontSize: 13,
        fontWeight: FontWeight.w600,
        color: AppTheme.primaryDark,
      ),
    );
  }

  Widget _buildWasmFilePicker() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: double.infinity,
          child: OutlinedButton.icon(
            onPressed: _pickWasmFile,
            icon: const Icon(Icons.upload_file, size: 18),
            label: const Text('选择 WASM 文件'),
            style: OutlinedButton.styleFrom(
              foregroundColor: AppTheme.primaryDark,
              side: const BorderSide(color: AppTheme.border),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(8),
              ),
              padding: const EdgeInsets.symmetric(vertical: 14),
            ),
          ),
        ),
        const SizedBox(height: 8),
        if (_wasmFileName != null && _wasmFileSize != null)
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: AppTheme.success.withValues(alpha: 0.06),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: AppTheme.success.withValues(alpha: 0.2)),
            ),
            child: Row(
              children: [
                const Icon(Icons.check_circle, size: 16, color: AppTheme.success),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    '$_wasmFileName (${_formatFileSize(_wasmFileSize!)})',
                    style: const TextStyle(
                      fontSize: 13,
                      color: AppTheme.success,
                      fontWeight: FontWeight.w500,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                GestureDetector(
                  onTap: () {
                    setState(() {
                      _wasmFileName = null;
                      _wasmFileSize = null;
                      _wasmCode = null;
                    });
                  },
                  child: Icon(Icons.close, size: 16, color: AppTheme.textTertiary),
                ),
              ],
            ),
          )
        else
          Text(
            '支持 .wasm 文件，最大 5MB',
            style: TextStyle(fontSize: 12, color: AppTheme.textTertiary),
          ),
      ],
    );
  }
}

// ──── 工具函数 ────

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
