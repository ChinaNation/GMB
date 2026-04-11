import 'package:wuminapp_mobile/qr/envelope.dart';

/// kind = login_challenge
///
/// 由 SFID/CPMS 后端生成,冷钱包 wumin 扫码后验证系统签名。
class LoginChallengeBody implements QrBody {
  const LoginChallengeBody({
    required this.system,
    required this.sysPubkey,
    required this.sysSig,
  });

  /// `"sfid"` 或 `"cpms"`
  final String system;

  /// 系统公钥 `0x` + hex
  final String sysPubkey;

  /// 系统签名 `0x` + hex(对统一签名原文的 sr25519 签名)
  final String sysSig;

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'system': system,
        'sys_pubkey': sysPubkey,
        'sys_sig': sysSig,
      };

  static LoginChallengeBody fromJson(Map<String, dynamic> data) {
    final system = data['system'];
    final sysPubkey = data['sys_pubkey'];
    final sysSig = data['sys_sig'];
    if (system is! String || system.isEmpty) {
      throw const FormatException('login_challenge.system 必填');
    }
    if (system != 'sfid' && system != 'cpms') {
      throw FormatException('login_challenge.system 非法: $system');
    }
    if (sysPubkey is! String || !sysPubkey.startsWith('0x')) {
      throw const FormatException('login_challenge.sys_pubkey 必填 0x hex');
    }
    if (sysSig is! String || !sysSig.startsWith('0x')) {
      throw const FormatException('login_challenge.sys_sig 必填 0x hex');
    }
    return LoginChallengeBody(
      system: system,
      sysPubkey: sysPubkey,
      sysSig: sysSig,
    );
  }
}
