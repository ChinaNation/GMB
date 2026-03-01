import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:local_auth/local_auth.dart';
import 'package:wuminapp_mobile/pages/login_whitelist_page.dart';
import 'package:wuminapp_mobile/services/app_settings_service.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  final AppSettingsService _settingsService = AppSettingsService();
  final LocalAuthentication _localAuth = LocalAuthentication();
  bool _loading = true;
  bool _faceAuthEnabled = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final enabled = await _settingsService.isFaceAuthEnabled();
    if (!mounted) {
      return;
    }
    setState(() {
      _faceAuthEnabled = enabled;
      _loading = false;
    });
  }

  Future<void> _onFaceAuthChanged(bool value) async {
    if (value) {
      try {
        final supported = await _localAuth.isDeviceSupported();
        final enrolled = await _localAuth.getAvailableBiometrics();
        if (!supported || enrolled.isEmpty) {
          if (!mounted) {
            return;
          }
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('设备未开启人脸/指纹，请先在系统设置中录入')),
          );
          return;
        }

        final verified = await _localAuth.authenticate(
          localizedReason: '请进行生物识别以开启人脸识别签名',
          options: const AuthenticationOptions(
            biometricOnly: true,
            stickyAuth: false,
            useErrorDialogs: true,
          ),
        );
        if (!verified) {
          if (!mounted) {
            return;
          }
          ScaffoldMessenger.of(
            context,
          ).showSnackBar(const SnackBar(content: Text('未通过生物识别，未开启人脸识别')));
          return;
        }
      } on PlatformException catch (e) {
        if (!mounted) {
          return;
        }
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('生物识别不可用：${e.message ?? e.code}')),
        );
        return;
      }
    }

    await _settingsService.setFaceAuthEnabled(value);
    if (!mounted) {
      return;
    }
    setState(() {
      _faceAuthEnabled = value;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('设置'),
        centerTitle: true,
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                Card(
                  child: SwitchListTile(
                    secondary: const Icon(Icons.face_outlined),
                    title: const Text('人脸识别'),
                    value: _faceAuthEnabled,
                    onChanged: _onFaceAuthChanged,
                  ),
                ),
                Card(
                  child: ListTile(
                    leading: const Icon(Icons.verified_user_outlined),
                    title: const Text('登录白名单'),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () {
                      Navigator.of(context).push(
                        MaterialPageRoute(
                          builder: (_) => const LoginWhitelistPage(),
                        ),
                      );
                    },
                  ),
                ),
              ],
            ),
    );
  }
}
