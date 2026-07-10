import 'package:flutter/material.dart';
import 'package:qr_flutter/qr_flutter.dart';

import 'package:citizenapp/signer/square_action_payload.dart';
import 'package:citizenapp/ui/app_theme.dart';

/// 广场账户动作签名响应页：展示 QR_V1 signResponse 二维码，供发起方（官网）扫回完成。
class QrSignResponsePage extends StatelessWidget {
  const QrSignResponsePage({
    super.key,
    required this.responseJson,
    required this.decoded,
  });

  /// signResponse envelope 的 JSON。
  final String responseJson;

  /// 已核对的动作内容（页面顶部再展示一次，闭环确认）。
  final SquareActionPayload decoded;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('签名结果')),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              Text(
                decoded.displayTitle,
                style: const TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 24),
              Center(
                child: QrImageView(
                  data: responseJson,
                  version: QrVersions.auto,
                  size: 240,
                  errorStateBuilder: (context, error) {
                    return Container(
                      width: 240,
                      height: 240,
                      padding: const EdgeInsets.all(10),
                      decoration: AppTheme.bannerDecoration(AppTheme.danger),
                      child: const Center(
                        child: Text(
                          '二维码渲染失败',
                          style: TextStyle(color: AppTheme.danger),
                        ),
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 24),
              const Text(
                '已完成签名。请在发起页面（官网）扫描此二维码以继续。',
                style: TextStyle(color: AppTheme.textSecondary),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 24),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('完成'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
