任务需求：
SFID 号格式日期段从 D8(YYYY) 缩为 D4(YYYY),整体长度 32→28 字符;
同时给 SFID 生成的三条业务路径统一加 1000 次碰撞重试保护栏,
取代原本不一致的处理(路径 1 5 次 / 路径 2 单次/ 路径 3 0 次)。

旧数据全部删除,无需 migration。

所属模块：SFID

输入文档：
- memory/feedback_no_compatibility.md(死规则:绝不搞兼容)
- sfid/backend/sfid/validator.rs(原 D8 格式定义)
- sfid/backend/sfid/generator.rs(SFID 唯一生成入口)

必须遵守：
- 不可突破模块边界(链端、wuminapp、sfid-frontend 不动)
- 不留 D8 兼容代码,旧数据全清
- 所有 SFID 生成必须走 sfid::generate_sfid_code 单一入口

## 改动清单

### 格式 D8 → D4
| 文件 | 改动 |
|---|---|
| sfid/backend/sfid/validator.rs | `SFID_NUMBER_SEGMENT_D8_LEN = 8` → `SFID_NUMBER_SEGMENT_D4_LEN = 4`;校验段 5 长度 8→4;文档 + 测试 fixture |
| sfid/backend/sfid/generator.rs | `Utc::now().format("%Y%m%d")` → `format("%Y")` |
| sfid/backend/sfid/mod.rs | 重导出常量名 D8_LEN → D4_LEN |

### 1000 次碰撞重试
| 文件 | 路径 | 旧行为 | 新行为 |
|---|---|---|---|
| sfid/backend/institutions/handler.rs:459 | 路径 1 CPMS 机构创建 | 5 次重试 + UUID account_pubkey | 1000 次重试 + UUID |
| sfid/backend/institutions/service.rs:352 | 路径 2 公安局 reconcile | 单次生成,撞了 continue 跳过城市 | 1000 次重试,nonce = `PS-{省}-{市}#{retry}` |
| sfid/backend/citizens/binding.rs:258 | 路径 3 公民绑定 | **0 重试**(撞了直接覆盖 store 数据) | 1000 次重试,nonce = `{pubkey}#{retry}` |

### 顺手修 pre-existing bug
| 文件 | 改动 |
|---|---|
| sfid/backend/login/mod.rs | 加 `pub(crate) use model::{AdminQrChallengeInput,AdminQrCompleteInput,AdminQrResultQuery}`(原 main_tests.rs 编译失败) |

## 容量分析

n9 桶 = 10⁹,单 (a3, 省, 市, 机构, year) 5 元组共享。

| 路径 | 桶最大填充 | 1000 次都撞概率 |
|---|---|---|
| 路径 1 CPMS 机构 | 全国一年新增几万个机构 | ≈ 0 |
| 路径 2 公安局 reconcile | 全国 ~几百市 | ≈ 0 |
| 路径 3 公民绑定 | 单省 1.5 亿人(15% 桶填充) | 0.15¹⁰⁰⁰ ≈ 10⁻⁸²⁴ |

→ 实际不可达,1000 次保护栏只防极端饱和与代码 bug。

## 验证
- ✅ `cargo check` 0 error 0 warning(SFID 相关)
- ✅ `cargo test --bin sfid-backend` 69/69 全过
  - sfid 模块 16/16(validator + generator + a3 + cities + category + institution_code)
  - 其他模块 53/53(login + cpms + main_tests 等)

## 输出物
- 代码 + 中文注释
- 测试通过
- 任务卡(本文件)
- memory 固化:n9 桶容量分析 + 1000 次保护栏

## 验收标准
- 新生成 SFID 末段 4 字符纯数字年(如 `2026`)
- 三条业务路径都走 1000 次保护栏,撞了能换
- 旧 D8 数据按死规则全清,启动新表后自然全是 D4
- 链端 / wuminapp / sfid-frontend 不动(grep 确认无解析日期段处)
