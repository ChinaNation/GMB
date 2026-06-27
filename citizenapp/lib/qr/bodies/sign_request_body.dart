import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenapp/qr/envelope.dart';

/// k = 1 签名请求。
///
/// CitizenApp、CID、节点桌面端都只生成这一种请求流向;具体业务场景由
/// `action`(`a`) 区分,扫码端展示内容必须由 `action + payload` 本地解码得出。
class SignRequestBody implements QrBody {
  const SignRequestBody({
    required this.action,
    required this.signerPubkey,
    required this.payload,
    this.sigAlg = 1,
  });

  /// 业务动作码 `a`:扫码流向以 `k` 表达,业务语义统一放这里。
  final int action;

  /// 签名算法 `g`:当前 1=sr25519。
  final int sigAlg;

  /// 期望签名者公钥 `u`:32 字节公钥的 base64url 无填充编码。
  final String signerPubkey;

  /// 待签载荷 `d`:原始 payload bytes 的 base64url 无填充编码。
  final String payload;

  Uint8List get payloadBytes => _b64ToBytes(payload, 'd');

  Uint8List get pubkeyBytes => _b64ToBytes(signerPubkey, 'u');

  String get payloadHex => '0x${_toHex(payloadBytes)}';

  String get pubkeyHex => '0x${_toHex(pubkeyBytes)}';

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'a': action,
        'g': sigAlg,
        'u': signerPubkey,
        'd': payload,
      };

  static SignRequestBody fromJson(Map<String, dynamic> data) {
    final action = data['a'];
    final sigAlg = data['g'];
    final signerPubkey = data['u'];
    final payload = data['d'];
    if (action is! int || action <= 0) {
      throw const FormatException('签名请求 a 必须为正整数');
    }
    if (sigAlg != 1) {
      throw const FormatException('签名请求 g 目前只允许 1(sr25519)');
    }
    if (signerPubkey is! String || signerPubkey.isEmpty) {
      throw const FormatException('签名请求 u 必填');
    }
    if (_b64ToBytes(signerPubkey, 'u').length != 32) {
      throw const FormatException('签名请求 u 必须解码为 32 字节');
    }
    if (payload is! String || payload.isEmpty) {
      throw const FormatException('签名请求 d 必填');
    }
    if (_b64ToBytes(payload, 'd').isEmpty) {
      throw const FormatException('签名请求 d 不能为空载荷');
    }
    return SignRequestBody(
      action: action,
      sigAlg: sigAlg as int,
      signerPubkey: signerPubkey,
      payload: payload,
    );
  }

  static SignRequestBody fromHex({
    required int action,
    required String pubkeyHex,
    required String payloadHex,
  }) {
    return SignRequestBody(
      action: action,
      signerPubkey: _b64NoPad(_hexToBytes(pubkeyHex)),
      payload: _b64NoPad(_hexToBytes(payloadHex)),
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
    throw FormatException('签名请求 $field 必须为 base64url');
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
