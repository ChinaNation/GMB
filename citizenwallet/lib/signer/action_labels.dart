import 'package:citizenwallet/qr/generated/qr_action_registry.g.dart';

/// 交易 action 英文标识 → 中文显示名映射。
///
/// 钱包 UI 显示时查此表翻译，未命中必须红色拒绝，不允许原样显示英文。
/// 本表直接来自 qr-protocol 生成文件，禁止在钱包端另写第二套动作名。
const Map<String, String> actionLabels =
    GeneratedQrActionRegistry.actionLabelZhByKey;

/// QR 数字 action → decoder action_key。
///
/// 这里用于无法解码 payload 时仍能判断“动作是否已登记并有中文名”。
/// UI 只能显示中文动作名；查不到时必须红色拒绝。
const Map<int, String> actionKeyByCode =
    GeneratedQrActionRegistry.actionKeyByCode;

String? actionLabelForDecodedAction(String actionKey) =>
    actionLabels[actionKey];

String? actionLabelForQrAction(int actionCode) {
  final actionKey = actionKeyByCode[actionCode];
  if (actionKey == null) return null;
  return actionLabels[actionKey];
}
