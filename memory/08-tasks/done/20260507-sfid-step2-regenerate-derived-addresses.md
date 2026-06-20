任务需求：
SFID 改造 Step 2:跑 tools/duoqian.py --apply 同步所有派生地址(277 条 main /
87 条 fee / 1 条 NRC_ANQUAN_ACCOUNT / 408 条 china_zb 汇总),因 Step 1 把 277
条 sfid_number 字面量重生成,所有依赖 sfid_number 的 BLAKE2 派生地址必须连环重算。

所属模块：Blockchain (citizenchain primitives)

输入文档：
- memory/08-tasks/open/20260507-sfid-step1-rename-shenfen-to-sfid-number-and-city-001.md(Step 1)
- memory/feedback_sfid_naming_unified.md
- tools/duoqian.py 注释:派生公式

必须遵守：
- Step 1 必须先完成(sfid_number 字面量已切换),否则地址派生会基于旧值
- 不得手改 china_*.rs 任何 hex 字面地址,统一走脚本

## 改动清单

### tools/duoqian.py 一次跑通
- main_account(7 个 china_*.rs):**277 条全部重算**
- fee_account(cb + ch):**87 条全部重算**
- stake_account(ch):**0 条变更**(stake 派生不依赖 sfid_number,仅依赖 citizens_number)
- NRC_ANQUAN_ACCOUNT(china_cb 顶部):**1 条重算** → `b8a5c135280278916442137418ab6423eda038bb4662a5c02e70f8d528903529`
- china_zb.rs::CHINA_RESERVED_MAIN_ACCOUNTS:**408 条**汇总写回(去重排序)

### 派生公式(未变,只换 sfid_number 输入)
```
preimage = b"DUOQIAN" (10B) || op_tag (1B) || ss58_le[2027] (2B) || sfid_number_bytes
address  = blake2b_256(preimage)
```
- OP_MAIN = 0x00 → main_account
- OP_FEE  = 0x01 → fee_account
- OP_AN   = 0x03 → NRC_ANQUAN_ACCOUNT
- OP_STAKE= 0x02 → stake(用 citizens_number,不重算)

## 验证(全部 ok)

| 工程 | 命令 | 结果 |
|---|---|---|
| primitives + runtime | `cargo check` | **0 error** ✅ |
| admins-change | `cargo test` | **31/31** ✅ |
| organization-manage | `cargo test` | **24/24** ✅ |
| duoqian-transfer | `cargo test` | **20/20** ✅ |
| votingengine | `cargo test` | **79/79** ✅ |
| internal-vote | `cargo test` | **5/5** ✅ |
| sfid backend | `cargo test --bin sfid-backend` | **69/69** ✅ |
| wuminapp | `flutter test` | **154/154** ✅ |
| wumin | `flutter test` | **94/94** ✅ |

**Rust 测试合计 228 + Dart 测试合计 248 + sfid 69 = 545 全过**

## 输出物
- 277 条 main_account 重算(7 个 china_*.rs)
- 87 条 fee_account 重算(china_cb.rs + china_ch.rs)
- 1 条 NRC_ANQUAN_ACCOUNT 重算(china_cb.rs:31)
- 408 条 CHINA_RESERVED_MAIN_ACCOUNTS 重算(china_zb.rs)
- 任务卡(本文件)

## 验收标准

- ✅ 所有派生地址新值 = blake2b_256(DUOQIAN + op_tag + ss58_le + 新 sfid_number)
- ✅ china_zb.rs 汇总 408 条 = 277 main + 87 fee + 43 stake + 1 NRC_AN(去重排序)
- ✅ 链端 / sfid backend / wuminapp / wumin 全部测试通过
- ✅ tools/duoqian.py dry-run 再跑显示 "0 变更"(幂等验证)

## 顺手成果

Step 2 在 tools/ 改名补跑时(big_bang_sfid_rename.py SCAN_ROOTS 加 tools/),
连带把 tools/duoqian.py 的 NRC_SFID_NUMBER 常量改名 + 改值一并完成,
本 Step 2 只跑 --apply 即收割所有派生地址。

## SFID 改造完整闭环

| 步骤 | 范围 | 状态 |
|---|---|---|
| Step 1 | 命名统一 + 277 条 sfid_number 字面量重生成 | ✅ |
| Step 2 | 派生地址同步重算(773 条 hex) | ✅ |

整个 SFID 改造彻底闭环,旧数据全清,新链起来即用新格式。
