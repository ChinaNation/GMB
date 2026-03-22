import 'package:flutter/material.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:local_auth/local_auth.dart';

import '../security/app_lock_service.dart';
import '../security/pin_input_page.dart';

/// 冷钱包设置页。
class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  static const String _deviceLockKey = 'device_lock_enabled';
  static const FlutterSecureStorage _secure = FlutterSecureStorage();
  final LocalAuthentication _localAuth = LocalAuthentication();
  bool _deviceLockEnabled = false;
  bool _pinLockEnabled = false;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _loadSettings();
  }

  Future<void> _loadSettings() async {
    final deviceLockStr = await _secure.read(key: _deviceLockKey);
    final pinSet = await AppLockService.isPinSet();
    if (!mounted) return;
    setState(() {
      _deviceLockEnabled = deviceLockStr == 'true';
      _pinLockEnabled = pinSet;
      _loading = false;
    });
  }

  Future<void> _toggleDeviceLock(bool value) async {
    if (value) {
      final canCheck = await _localAuth.canCheckBiometrics;
      final isDeviceSupported = await _localAuth.isDeviceSupported();
      if (!canCheck && !isDeviceSupported) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('您的设备不支持生物识别或设备密码，无法开启设备锁')),
        );
        return;
      }

      try {
        final authenticated = await _localAuth.authenticate(
          localizedReason: '验证身份以开启设备锁',
          options: const AuthenticationOptions(
            stickyAuth: true,
            biometricOnly: false,
          ),
        );
        if (!authenticated) return;
      } catch (e) {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('身份验证失败：$e')),
        );
        return;
      }
    }

    await _secure.write(key: _deviceLockKey, value: value.toString());
    if (!mounted) return;
    setState(() => _deviceLockEnabled = value);
  }

  Future<void> _togglePinLock(bool value) async {
    if (value) {
      final result = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => const PinInputPage(mode: PinInputMode.setup),
        ),
      );
      if (result == true && mounted) {
        setState(() => _pinLockEnabled = true);
      }
    } else {
      final result = await Navigator.of(context).push<bool>(
        MaterialPageRoute(
          builder: (_) => const PinInputPage(mode: PinInputMode.remove),
        ),
      );
      if (result == true && mounted) {
        setState(() => _pinLockEnabled = false);
      }
    }
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
              children: [
                SwitchListTile(
                  title: const Text('设备锁'),
                  subtitle: Text(
                    _pinLockEnabled
                        ? '请先关闭应用锁'
                        : '启动应用时需要生物识别或设备密码',
                  ),
                  value: _deviceLockEnabled,
                  onChanged: _pinLockEnabled ? null : _toggleDeviceLock,
                  activeThumbColor: Colors.white,
                  activeTrackColor: Theme.of(context).colorScheme.primary,
                  secondary: const Icon(Icons.fingerprint),
                ),
                const Divider(height: 1),
                SwitchListTile(
                  title: const Text('应用锁'),
                  subtitle: Text(
                    _deviceLockEnabled
                        ? '请先关闭设备锁'
                        : '启动应用时需要输入 6 位数字密码',
                  ),
                  value: _pinLockEnabled,
                  onChanged: _deviceLockEnabled ? null : _togglePinLock,
                  activeThumbColor: Colors.white,
                  activeTrackColor: Theme.of(context).colorScheme.primary,
                  secondary: const Icon(Icons.pin_outlined),
                ),
              ],
            ),
    );
  }
}
