import 'dart:convert';
import 'dart:typed_data';

import 'package:sr25519/sr25519.dart' as sr25519;

import '../qr/login/login_models.dart';

/// 统一的 sr25519 验签工具。
class Sr25519MessageVerifier {
  bool verify({
    required String pubkeyHex,
    required Uint8List message,
    required String signatureHex,
  }) {
    try {
      final publicKey = sr25519.PublicKey.newPublicKey(_hexToBytes(pubkeyHex));
      final signature = sr25519.Signature.fromBytes(
        Uint8List.fromList(_hexToBytes(signatureHex)),
      );
      final (verified, exception) =
          sr25519.Sr25519.verify(publicKey, signature, message);
      if (exception != null) {
        return false;
      }
      return verified;
    } catch (_) {
      return false;
    }
  }

  List<int> _hexToBytes(String input) {
    final text = _normalizeHex(input);
    if (text.isEmpty || text.length.isOdd) {
      throw ArgumentError('hex 长度无效');
    }
    return List<int>.generate(
      text.length ~/ 2,
      (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
      growable: false,
    );
  }
}

/// 登录挑战系统签名验证器。
///
/// 当前协议只校验“这张二维码是否由持有 `sys_pubkey` 对应私钥的一方签发”。
class LoginSystemSignatureVerifier {
  LoginSystemSignatureVerifier({
    Sr25519MessageVerifier? messageVerifier,
  }) : _messageVerifier = messageVerifier ?? Sr25519MessageVerifier();

  final Sr25519MessageVerifier _messageVerifier;

  Future<void> verify(LoginChallenge challenge) async {
    final message = Uint8List.fromList(
      utf8.encode(_buildChallengeMessage(challenge)),
    );
    final verified = _messageVerifier.verify(
      pubkeyHex: challenge.sysPubkey,
      message: message,
      signatureHex: challenge.sysSig,
    );
    if (!verified) {
      throw const LoginException(
        LoginErrorCode.invalidSystemSignature,
        '系统挑战签名验证失败',
      );
    }
  }

  // 签名消息中的 sys_pubkey 必须与后端签名时的格式完全一致（含 0x 前缀）。
  String _buildChallengeMessage(LoginChallenge challenge) {
    return [
      challenge.proto,
      challenge.system,
      challenge.challenge,
      challenge.issuedAt.toString(),
      challenge.expiresAt.toString(),
      challenge.sysPubkey,
    ].join('|');
  }
}

String _normalizeHex(String input) {
  final trimmed = input.trim();
  if (trimmed.startsWith('0x') || trimmed.startsWith('0X')) {
    return trimmed.substring(2).toLowerCase();
  }
  return trimmed.toLowerCase();
}
