# ADR-003: QR 签名 display.fields 改为自描述 List 格式

## 状态

已采纳 — 2026-03-23

## 背景

冷钱包（wumin）属于离线设备，不经常更新。但在线端（wuminapp）的签名类型会持续增加（更换管理员、更换投票 key、决议发行投票等）。

原有协议中 `display.fields` 是 `Map<String, String>`，冷钱包需要硬编码 `_fieldLabels` 将 key 翻译成中文标签。每新增一个签名类型，必须同时修改三个位置：

1. wuminapp 在线端构造 display.fields
2. wumin `offline_sign_page.dart` 的 `_fieldLabels` 映射表
3. wumin `offline_sign_page.dart` 的 `_actionLabels` 映射表

这种模式不可持续，容易遗漏且强制冷钱包同步更新。

## 决策

将 `display.fields` 从 `Map<String, String>` 改为 `List<Map<String, dynamic>>`，每个条目自带 label：

```json
{
  "action": "propose_transfer",
  "summary": "国储会 提案转账 100.00 GMB 给 5Grw...",
  "fields": [
    {"key": "org", "label": "付款机构", "value": "国储会"},
    {"key": "beneficiary", "label": "收款账户", "value": "5Grw..."},
    {"key": "amount_yuan", "label": "金额", "value": "100.00 GMB", "format": "currency"},
    {"key": "remark", "label": "备注", "value": "日常拨款"}
  ]
}
```

字段说明：
- `key`：字段标识，PayloadDecoder 用此 key 做交叉验证
- `label`：中文标签，冷钱包直接用于显示
- `value`：原始值，用于显示和验证比对
- `format`（可选）：`"currency"` 表示金额，渲染时添加千分位逗号

`action` 标签也由在线端自带，通过新增 `action_label` 字段传递。

## 影响

### wuminapp（在线端）
- 所有构造 `display` 的位置改为 List 格式
- 新增 `action_label` 字段传递交易类型中文名

### wumin（冷钱包）
- 删除 `_fieldLabels` 和 `_actionLabels` 静态映射表
- 渲染逻辑改为遍历 List，直接使用每个条目的 `label`
- `format: "currency"` 时应用千分位格式化（仅影响显示，不影响验证）
- `OfflineSignService.verifyPayload` 中字段比对逻辑适配 List 格式

### PayloadDecoder
- 不变。`DecodedPayload.fields` 仍为 `Map<String, String>`
- 验证时从 display List 中按 key 查找对应 value 进行比对

### 协议兼容性
- 此变更不向后兼容，两端必须同步更新
- 协议版本号保持 `WUMIN_QR_V1` 不变（冷钱包与在线端始终配套部署）

## 好处

- **新增签名类型只改在线端**，冷钱包零改动
- 消除三处硬编码对齐的维护负担
- 金额千分位格式化由 `format` 字段驱动，可扩展其他格式
