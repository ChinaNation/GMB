import 'dart:convert';
import 'dart:typed_data';

import 'package:citizenwallet/qr/envelope.dart';
import 'package:citizenwallet/qr/qr_protocols.dart';

class SignRequestBody implements QrBody {
  const SignRequestBody({
    required this.action,
    required this.signerPublicKey,
    required this.payload,
    this.alg = 1,
  });

  /// 业务动作码 `a`:扫码流向以 `k` 表达,业务语义统一放这里。
  final int action;

  /// 签名算法 `g`:当前 1=sr25519。
  final int alg;

  /// 期望签名者公钥 `u`:32 字节公钥的 base64url 无填充编码。
  final String signerPublicKey;

  /// 审阅载荷 `d`:原始 review_payload bytes 的 base64url 无填充编码。
  ///
  /// 普通链交易必须是可完整解码和中文展示的 review_payload；实际签名字节由
  /// 签名端按 action 重新计算，不能把 32 字节 signing bytes 冒充成这里的载荷。
  final String payload;

  Uint8List get payloadBytes => _b64ToBytes(payload, 'd');

  Uint8List get signerPublicKeyBytes => _b64ToBytes(signerPublicKey, 'u');

  String get payloadHex => '0x${_toHex(payloadBytes)}';

  String get signerPublicKeyHex => '0x${_toHex(signerPublicKeyBytes)}';

  @override
  Map<String, dynamic> toJson() => <String, dynamic>{
        'a': action,
        'g': alg,
        'u': signerPublicKey,
        'd': payload,
      };

  static SignRequestBody fromJson(Map<String, dynamic> data) {
    requireExactKeys(data, const {'a', 'g', 'u', 'd'}, 'sign_request.b');
    final action = data['a'];
    final alg = data['g'];
    final signerPublicKey = data['u'];
    final payload = data['d'];
    if (action is! int || action <= 0) {
      throw const FormatException('签名请求 a 必须为正整数');
    }
    if (alg != 1) {
      throw const FormatException('签名请求 g 目前只允许 1(sr25519)');
    }
    if (signerPublicKey is! String) {
      throw const FormatException('签名请求 u 必须是字符串');
    }
    // 注册局占号/换绑域签名:b.u 留空,钱包扫码自填本账户;其余动作 u 必填 32B。
    if (QrActions.isSelfAccountDomainAction(action)) {
      if (signerPublicKey.isNotEmpty) {
        throw const FormatException('占号/换绑签名请求 u 必须留空(钱包自填账户)');
      }
    } else {
      if (signerPublicKey.isEmpty) {
        throw const FormatException('签名请求 u 必填');
      }
      if (_b64ToBytes(signerPublicKey, 'u').length != 32) {
        throw const FormatException('签名请求 u 必须解码为 32 字节');
      }
    }
    if (payload is! String || payload.isEmpty) {
      throw const FormatException('签名请求 d 必填');
    }
    if (_b64ToBytes(payload, 'd').isEmpty) {
      throw const FormatException('签名请求 d 不能为空载荷');
    }
    return SignRequestBody(
      action: action,
      alg: alg as int,
      signerPublicKey: signerPublicKey,
      payload: payload,
    );
  }

  static SignRequestBody fromHex({
    required int action,
    required String signerPublicKeyHex,
    required String payloadHex,
  }) {
    // 注册局占号/换绑请求按协议把 u 留空，由离线钱包选择账户后原位填授权模板零槽。
    // 测试/本地构造入口必须与 fromJson 遵守同一规则，不能强迫伪造 32 字节账户。
    final signerPublicKeyBytes = signerPublicKeyHex.isEmpty &&
            QrActions.isSelfAccountDomainAction(action)
        ? Uint8List(0)
        : _strictHexBytes(
            signerPublicKeyHex,
            field: 'signer_public_key',
            expectedBytes: 32,
          );
    return SignRequestBody(
      action: action,
      signerPublicKey: _b64NoPad(signerPublicKeyBytes),
      payload: _b64NoPad(_strictHexBytes(payloadHex, field: 'payload')),
    );
  }
}

String _b64NoPad(List<int> bytes) =>
    base64Url.encode(bytes).replaceAll('=', '');

Uint8List _b64ToBytes(String input, String field) {
  if (input.isEmpty ||
      input.contains('=') ||
      !RegExp(r'^[A-Za-z0-9_-]+$').hasMatch(input)) {
    throw FormatException('签名请求 $field 必须为无填充 base64url');
  }
  final normalized =
      input.padRight(input.length + ((4 - input.length % 4) % 4), '=');
  try {
    final bytes = Uint8List.fromList(base64Url.decode(normalized));
    if (_b64NoPad(bytes) != input) {
      throw const FormatException();
    }
    return bytes;
  } catch (_) {
    throw FormatException('签名请求 $field 必须为无填充 base64url');
  }
}

List<int> _strictHexBytes(
  String input, {
  required String field,
  int? expectedBytes,
}) {
  if (!input.startsWith('0x')) {
    throw FormatException('$field 必须以小写 0x 开头');
  }
  final text = input.substring(2);
  if (text.isEmpty ||
      text.length.isOdd ||
      !RegExp(r'^[0-9a-f]+$').hasMatch(text)) {
    throw FormatException('$field 必须是小写偶数字节十六进制');
  }
  final bytes = List<int>.generate(
    text.length ~/ 2,
    (i) => int.parse(text.substring(i * 2, i * 2 + 2), radix: 16),
    growable: false,
  );
  if (expectedBytes != null && bytes.length != expectedBytes) {
    throw FormatException('$field 必须是 $expectedBytes 字节');
  }
  return bytes;
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
