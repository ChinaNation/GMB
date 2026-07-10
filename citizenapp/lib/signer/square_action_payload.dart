import 'dart:convert';
import 'dart:typed_data';

/// 广场账户动作签名 payload（两色内容核对用）。
///
/// 布局须与 Worker `account/action_challenge.buildActionScalePayload` 逐字节一致：
///   scaleString(action) ‖ scaleString(owner) ‖ scaleString(challengeId)
///   [‖ scaleString(context/level)] ‖ u64_le(expiresAt)
/// 其中 context 仅在 action == 'subscribe_membership' 时存在（= 会员等级）。
class SquareActionPayload {
  const SquareActionPayload({
    required this.action,
    required this.ownerAccount,
    required this.challengeId,
    required this.expiresAt,
    this.context,
  });

  final String action;
  final String ownerAccount;
  final String challengeId;
  final int expiresAt;

  /// 动作专属绑定字段（subscribe_membership = 会员等级）。
  final String? context;

  /// 人类可读的动作标题，展示给用户核对（禁盲签）。
  String get displayTitle {
    switch (action) {
      case 'subscribe_membership':
        return context == null ? '订阅会员' : '订阅会员（$context）';
      case 'cancel_membership':
        return '取消订阅';
      case 'delete_account':
        return '注销用户';
      default:
        return action;
    }
  }
}

const Set<String> _knownActions = {
  'subscribe_membership',
  'cancel_membership',
  'delete_account',
};

/// 解码 payloadHex。无法识别 / 布局不符 / 未知动作一律返回 null（调用方须禁止签名）。
SquareActionPayload? decodeSquareActionPayload(String payloadHex) {
  try {
    final bytes = _hexToBytes(payloadHex);
    var offset = 0;

    final (action, o1) = _readString(bytes, offset);
    offset = o1;
    if (!_knownActions.contains(action)) return null;

    final (owner, o2) = _readString(bytes, offset);
    offset = o2;
    final (challengeId, o3) = _readString(bytes, offset);
    offset = o3;

    String? context;
    if (action == 'subscribe_membership') {
      final (level, o4) = _readString(bytes, offset);
      context = level;
      offset = o4;
    }

    // 结尾必须正好剩 u64（8 字节）过期时间，多一分少一分都拒。
    if (bytes.length - offset != 8) return null;
    final expiresAt = _u64Le(bytes, offset);

    return SquareActionPayload(
      action: action,
      ownerAccount: owner,
      challengeId: challengeId,
      expiresAt: expiresAt,
      context: context,
    );
  } on Object {
    return null;
  }
}

(String, int) _readString(Uint8List bytes, int offset) {
  final (len, next) = _readCompact(bytes, offset);
  final end = next + len;
  final str = utf8.decode(bytes.sublist(next, end));
  return (str, end);
}

/// SCALE compact 解码（支持 1/2/4 字节模式，覆盖 payload 各字段长度）。
(int, int) _readCompact(Uint8List bytes, int offset) {
  final first = bytes[offset];
  final mode = first & 0x03;
  if (mode == 0) {
    return (first >> 2, offset + 1);
  }
  if (mode == 1) {
    final value = (first | (bytes[offset + 1] << 8)) >> 2;
    return (value, offset + 2);
  }
  if (mode == 2) {
    final raw = first |
        (bytes[offset + 1] << 8) |
        (bytes[offset + 2] << 16) |
        (bytes[offset + 3] << 24);
    return (raw >>> 2, offset + 4);
  }
  throw const FormatException('SCALE compact big-integer 不支持');
}

int _u64Le(Uint8List bytes, int offset) {
  var value = 0;
  for (var i = 7; i >= 0; i--) {
    value = (value << 8) | bytes[offset + i];
  }
  return value;
}

Uint8List _hexToBytes(String input) {
  final text = input.startsWith('0x') || input.startsWith('0X')
      ? input.substring(2)
      : input;
  if (text.length.isOdd) {
    throw const FormatException('hex 长度必须为偶数');
  }
  final out = Uint8List(text.length ~/ 2);
  for (var i = 0; i < out.length; i++) {
    out[i] = int.parse(text.substring(i * 2, i * 2 + 2), radix: 16);
  }
  return out;
}
