import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/qr/envelope.dart';

/// k = 2 签名响应。
///
/// 生成方按同一 `i` 找回本地 session,因此响应只需要签名者公钥和签名本身。
class SignResponseBody implements QrBody {
  const SignResponseBody({
    required this.signerPublicKey,
    required this.signature,
  });

  /// 签名者公钥 `u`:32 字节 base64url 无填充。
  final String signerPublicKey;

  /// 签名 `s`:64 字节 sr25519 signature base64url 无填充。
  final String signature;

  Uint8List get signerPublicKeyBytes => _b64ToBytes(signerPublicKey, 'u');

  Uint8List get signatureBytes => _b64ToBytes(signature, 's');

  String get signerPublicKeyHex => '0x${_toHex(signerPublicKeyBytes)}';

  String get signatureHex => '0x${_toHex(signatureBytes)}';

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'u': signerPublicKey,
        's': signature,
      };

  static SignResponseBody fromJson(Map<String, dynamic> data) {
    final signerPublicKey = data['u'];
    final signature = data['s'];
    if (signerPublicKey is! String ||
        _b64ToBytes(signerPublicKey, 'u').length != 32) {
      throw const FormatException('签名响应 u 必须为 32 字节 base64url');
    }
    if (signature is! String || _b64ToBytes(signature, 's').length != 64) {
      throw const FormatException('签名响应 s 必须为 64 字节 base64url');
    }
    return SignResponseBody(
      signerPublicKey: signerPublicKey,
      signature: signature,
    );
  }

  static SignResponseBody fromHex({
    required String signerPublicKeyHex,
    required String signatureHex,
  }) {
    return SignResponseBody(
      signerPublicKey: _b64NoPad(_hexToBytes(signerPublicKeyHex)),
      signature: _b64NoPad(_hexToBytes(signatureHex)),
    );
  }
}

String _b64NoPad(List<int> bytes) =>
    base64Url.encode(bytes).replaceAll('=', '');

Uint8List _b64ToBytes(String input, String field) {
  final normalized =
      input.padRight(input.length + ((4 - input.length % 4) % 4), '=');
  try {
    return Uint8List.fromList(base64Url.decode(normalized));
  } catch (_) {
    throw FormatException('签名响应 $field 必须为 base64url');
  }
}

List<int> _hexToBytes(String input) {
  final text = input.startsWith('0x') || input.startsWith('0X')
      ? input.substring(2)
      : input;
  if (text.isEmpty || text.length.isOdd) {
    throw const FormatException('hex 必须为偶数字节');
  }
  return List<int>.generate(
    text.length ~/ 2,
    (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
    growable: false,
  );
}

String _toHex(List<int> bytes) {
  const chars = '0123456789abcdef';
  final buffer = StringBuffer();
  for (final byte in bytes) {
    buffer
      ..write(chars[(byte >> 4) & 0x0f])
      ..write(chars[byte & 0x0f]);
  }
  return buffer.toString();
}
