import 'package:wumin/qr/envelope.dart';

class LoginChallengeBody implements QrBody {
  const LoginChallengeBody({
    required this.system,
    required this.sysPubkey,
    required this.sysSig,
  });

  final String system;
  final String sysPubkey;
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
