# 任务卡：wumin 冷钱包机构名注册表合并

- 状态：done（2026-04-15）
- 归属：Mobile Agent（wumin）

## 背景 / 根因

国储会发起"手续费划转提案"后，wumin 冷钱包扫二维码签名时报：

> 警告：交易内容与摘要不符，禁止签名

服务端 `citizenchain/node/src/ui/governance/signing.rs` 调用
`find_entry(shenfen_id)`（在 `mod.rs` 的 NATIONAL_COUNCILS + PROVINCIAL_COUNCILS
+ PROVINCIAL_BANKS 三个数组中查找）把 `shenfen_id` 还原为中文名，写入
`display.fields.institution`。

冷钱包 `wumin/lib/signer/payload_decoder.dart` 用
`clearingBankName(shenfen_id) ?? shenfen_id` 还原机构名并塞入
`decoded.fields.institution`。但 `clearing_banks.dart` 只有 43 条
`SFR-` 省储行数据，缺：
- 1 条国储会（`GFR-LN001-CB0C-...`，`国家储备委员会`）
- 43 条省储会（`GFR-...`）

结果：国储或省储会发起提案时 `clearingBankName()` 返回 null，
`bankName` 退化成原始的 `GFR-LN001-CB0C-617776487-20260222` 字符串，
与服务端 `国家储备委员会` 不匹配 → `DisplayMatchStatus.mismatched`
→ `交易内容与摘要不符,拒绝签名`。

## 修复方案

**合并为单一机构注册表**。冷钱包与服务端一一对齐 87 条机构
（1 NRC + 43 PRC + 43 PRB），对比即命中，冷钱包签名页直接显示中文名。

### 改动

| 文件 | 动作 | 说明 |
|---|---|---|
| `wumin/lib/chain/institutions.dart` | 新建 | 3 个常量列表 + `institutionName()` 查找函数，注释标注事实源 `citizenchain/node/src/ui/governance/mod.rs` |
| `wumin/lib/chain/clearing_banks.dart` | 删除 | 按"不搞兼容/保留/过渡"铁律 |
| `wumin/lib/signer/payload_decoder.dart` | 改 4 处 | `import` 换文件；`clearingBankName` → `institutionName`（行 7、468、500、750） |
| `wumin/test/signer/payload_decoder_test.dart` | 加 2 测试 | 国储会 / 省储会 `propose_sweep_to_main` 应分别还原为 `国家储备委员会` / `中枢省储备委员会` |
| `memory/08-tasks/open/20260405-offchain-deposit-model-redesign.md` | 改 1 行 | 旧文件名 `clearing_banks.dart` 改为 `institutions.dart` |

### HA000 省份名统一

发现服务端权威源历史上对 HA000 省份在两种机构角色下使用了不同中文名
（PRB 用 `滨海省`，其他所有模块都用 `海滨省`）。本次一并统一为 `海滨省`。

改动点：
- `citizenchain/runtime/primitives/china/china_ch.rs:522`
- `citizenchain/node/src/ui/governance/mod.rs:108`
- `wumin/lib/chain/institutions.dart`
- `wuminapp/lib/trade/offchain/clearing_banks.dart:45`
- `wuminapp/lib/governance/institution_data.dart:706`

`shenfen_id` 未变（仍为 `SFR-HA000-CH1N-832919801-20260222`），链上标识不受影响；
`shenfen_name` 仅用于展示，runtime 升级即可生效。

## 验证

- `flutter analyze --no-fatal-infos` 通过
- `flutter test test/signer/payload_decoder_test.dart` 新增 2 个测试通过
- 端到端：国储会手续费划转重扫二维码（人工验证）

## 后续

长期应由代码生成器从 `citizenchain/node/src/ui/governance/mod.rs`
（或更底层的 `runtime/primitives/china/*.rs`）产出 Dart 文件，
消除三处手抄（链上 primitives / 节点 UI mod.rs / wumin Dart）。
此项单独立项，不在本任务卡范围。
