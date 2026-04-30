# 任务卡：将 wuminapp 链下扫码支付 offchain 功能统一迁移到 lib/offchain，移除 lib/trade/offchain 业务目录，钱包和交易页仅保留入口 UI

- 任务编号：20260430-095923
- 状态：done
- 所属模块：wuminapp
- 当前负责人：Codex
- 创建时间：2026-04-30 09:59:23

## 任务需求

将 wuminapp 链下扫码支付 offchain 功能统一迁移到 lib/offchain，移除 lib/trade/offchain 业务目录，钱包和交易页仅保留入口 UI

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- wuminapp/WUMINAPP_TECHNICAL.md

## 模块模板

- 模板来源：memory/08-tasks/templates/wuminapp.md

### 默认改动范围

- `wuminapp`

### 先沟通条件

- 修改 Isar 数据结构
- 修改认证流程
- 修改关键交互路径


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/wuminapp.md

# WuMinApp 模块执行清单

- App 只是交互入口，不承担信任根职责
- Isar 结构、认证流程、关键交互变化前必须先沟通
- 关键 Flutter 交互与本地存储逻辑必须补中文注释
- 文档与残留必须一起收口


## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/wuminapp.md

# WuMinApp 完成标准

- App 仍然只是交互入口
- 关键 Flutter 交互和 Isar 逻辑已补中文注释
- 文档已同步更新
- 关键交互或数据结构变化已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 2026-04-30:将 wuminapp 链下扫码支付业务文件迁入 `wuminapp/lib/offchain/`,
  保留钱包页和交易页入口 UI,移除 `wuminapp/lib/trade/offchain/` 业务目录。
- 2026-04-30:新增 `offchain/services/offchain_scan_flow.dart`,交易页不再直接持有
  清算行目录查询和付款页跳转细节。
- 2026-04-30:更新 `memory/05-modules/wuminapp/offchain/OFFCHAIN_DIRECTORY.md`
  与 Step 2c 文档,明确当前 offchain 目录真源。
- 2026-04-30:验证通过 `dart format`, `flutter analyze`, 以及
  `flutter test test/trade/payment_intent_golden_test.dart test/trade/clearing_bank_prefs_test.dart test/trade/clearing_bank_settings_page_test.dart`。
- 2026-04-30:补充全目录残留检查,删除未被任何入口引用的旧
  `wuminapp/lib/trade/onchain/trade_qr_scan_page.dart`,并清理空目录。

## 完成信息

- 完成时间：2026-04-30 10:05:11
- 完成摘要：已将 wuminapp 链下扫码支付业务收口到 lib/offchain，移除 lib/trade/offchain 业务目录，交易页和钱包页仅保留入口 UI，并通过 flutter analyze 与相关单测。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
