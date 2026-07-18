// 扫码确认页 reviewFields 字段名中文翻译单源。
//
// decoder(payload_decoder.dart)的 `reviewFields` 保留英文机器 key 用于
// 跨端验真,到 UI 层统一经本文件翻译。payload_decoder 新增 reviewFields key
// 时必须先登记 `citizenchain/crates/qr-protocol/registry/fields.yaml`,
// 再同步本表并补测试。未登记字段必须红色拒绝,不得 fallback 展示英文 key。
import 'package:citizenwallet/qr/generated/qr_action_registry.g.dart';

/// fields value 转换(如 approve: true → 赞成)。
String fieldValueText(String key, String value) {
  if (key == 'approve') return value == 'true' ? '赞成' : '反对';
  return value;
}

bool hasFieldLabel(String key) => fieldLabelTextOrNull(key) != null;

/// reviewFields key → 中文字段名；未登记返回 null,调用方必须红色拒绝。
String? fieldLabelTextOrNull(String key) {
  if (key.startsWith('amount_')) {
    final accountName = key.substring('amount_'.length);
    return accountName.isEmpty ? '账户金额' : '$accountName金额';
  }
  return GeneratedQrActionRegistry.fieldLabelForKey(key);
}

/// reviewFields key → 中文字段名。
///
/// 仅保留给既有测试和非签名确认辅助场景；真正签名放行必须使用
/// [fieldLabelTextOrNull] / [hasFieldLabel]。未登记字段直接抛错,不能生成展示兜底。
String fieldLabelText(String key) {
  final label = fieldLabelTextOrNull(key);
  if (label == null) {
    throw StateError('签名字段缺少中文名称');
  }
  return label;
}
