# 任务卡:runtime pallet 测试目录统一整改

- 任务编号:20260507-runtime-pallet-tests-restructure
- 状态:completed (2026-05-07)
- 负责人:当前主聊天入口(Blockchain Agent 主导)
- 关联前置:无
- 关联后续:organization-manage / personal-manage 单测重写 → 已并入本任务卡同日补做(2026-05-07,personal-manage 14 用例 + organization-manage 22 用例,见 §10)

## 完成结果

15 个 pallet 全部完成 src/tests/ 目录拆分,0 行为变化,258 个测试全部通过:

| Pallet | 测试数 | lib.rs 减重 |
|---|---|---|
| institution-asset | 1 | 105 → 61 |
| genesis-pallet | 8 | 313 → 191 |
| pow-difficulty | 9 | 457 → 213 |
| citizen-issuance | 12(+7 集成) | 590 → 226 |
| shengbank-interest | 19 | 780 → 414 |
| fullnode-issuance | 19 | 756 → 255 |
| resolution-destro | 14 | 931 → 290 |
| runtime-upgrade | 16 | 1015 → 342 |
| resolution-issuance | 16 | tests.rs 753 → tests/ 760 |
| offchain-transaction | 23 | tests.rs 769 → tests/ 774 |
| grandpakey-change | 17 | 1343 → 526 |
| sfid-system | 33 | tests.rs 1014 → tests/ 1019 |
| onchain-transaction | 20 | 1448 → 453 |
| admins-change | 31 | 2240 → 1023 |
| duoqian-transfer | 20 | 2331 → 1016 |

**形态最终落地**:小型单 mod.rs(3 个),其余 12 个一律 `mod.rs(mock+helper) + cases.rs(用例)` 双文件,严格对齐 internal-vote 样板。citizen-issuance 根目录原有的 tests/integration_bind_sfid.rs 保留不动(跨 pallet OnSfidBound 集成测试,单测覆盖不到的契约)。

## 1. 任务目标

把 runtime 下 15 个 pallet 的测试代码从 `lib.rs` 内联块或同级 `tests.rs` 单文件,统一搬迁到独立的 `src/tests/` 子目录,并按 mock 与用例分离原则拆分文件,达成以下目标结构:

```
<pallet>/src/
├── lib.rs              ← 仅业务代码,#[cfg(test)] mod tests; 一行声明
├── tests/
│   ├── mod.rs          ← #![cfg(test)] + mock runtime + 共用 helper
│   ├── mock.rs         ← (可选)若 mock 体积大独立成文件
│   └── cases_*.rs      ← 按主题拆分的测试用例
```

样板参考:`votingengine/internal-vote/src/tests/{mod.rs,cases.rs,dual_id.rs}` 已是目标形态。

**纯结构整改,零行为变化**:不增删 / 不改写任何测试用例,不调整断言,只搬代码 + 拆文件。

## 2. 影响范围

### 2.1 形态 A(测试内联在 lib.rs 末尾)— 12 个 pallet

| Pallet 路径 | lib.rs 总行 | 测试块行数 |
|---|---|---|
| `transaction/duoqian-transfer/src/lib.rs` | 2331 | 1317 |
| `governance/admins-change/src/lib.rs` | 2240 | 1219 |
| `transaction/onchain-transaction/src/lib.rs` | 1448 | 997 |
| `governance/grandpakey-change/src/lib.rs` | 1343 | 819 |
| `governance/runtime-upgrade/src/lib.rs` | 1015 | 675 |
| `governance/resolution-destro/src/lib.rs` | 931 | 643 |
| `issuance/fullnode-issuance/src/lib.rs` | 756 | 503 |
| `issuance/shengbank-interest/src/lib.rs` | 780 | 368 |
| `issuance/citizen-issuance/src/lib.rs` | 590 | 366 |
| `otherpallet/pow-difficulty/src/lib.rs` | 457 | 247 |
| `genesis/src/lib.rs` | 313 | 124 |
| `transaction/institution-asset/src/lib.rs` | 105 | 46 |

### 2.2 形态 B(已有同级 `tests.rs` 但 mock + 用例混在一起)— 3 个 pallet

| Pallet 路径 | tests.rs 行数 |
|---|---|
| `otherpallet/sfid-system/src/tests.rs` | 1014 |
| `transaction/offchain-transaction/src/tests.rs` | 769 |
| `issuance/resolution-issuance/src/tests.rs` | 753 |

### 2.3 不在本次范围

**已是目标形态(2 个,无需动)**:
- `votingengine/internal-vote/src/tests/`
- `votingengine/joint-vote/src/tests/`

**就地单元测试(11 处,留在原处不搬)**:贴着被测函数的小型 `#[cfg(test)] mod tests`,搬走反而失去就近阅读价值

pallet 业务文件中的就地单测(5 处,共 18 个 #[test]):
- `transaction/offchain-transaction/src/bank_check.rs`(5)
- `transaction/offchain-transaction/src/batch_item.rs`(5)
- `votingengine/internal-vote/src/migrations/v1.rs`(3)
- `votingengine/joint-vote/src/migrations/v1.rs`(3)
- `otherpallet/sfid-system/src/duoqian_info/{mod.rs, tests.rs}`(2,子模块自带 tests.rs)

primitives crate 内的就地单测(6 处,共 19 个 #[test]):primitives 是常量/类型/纯函数库,每个文件最多 1-7 个测试,各自紧贴自家被测函数,搬到 src/tests/ 反而增加无谓间接层。
- `primitives/china/china_ch.rs`(3) — 国库行常量数据校验
- `primitives/src/derive.rs`(7) — `account_id_from_*` 派生函数
- `primitives/src/fee_policy.rs`(6) — 费率规则
- `primitives/src/count_const.rs`(1) — 常量一致性
- `primitives/src/genesis.rs`(1) — 创世常量
- `primitives/src/pow_const.rs`(1) — 块时间常量

**业务代码中的 `#[cfg(test)]` test-only helper(不是测试用例本身,必须留在原处)**:
- `votingengine/src/data.rs` — 4 处 `#[cfg(test)] pub fn store_proposal_data / remove_proposal_object / ...`
- `votingengine/src/lib.rs` — 1 处 `#[cfg(test)] pub fn set_callback_execution_result`
- `votingengine/joint-vote/src/jointinternal.rs` — 1 处 `#[cfg(test)] { ... }` 内联 test-only 业务分支

**原零测试 pallet(2 个,2026-05-07 同日补做,见 §10)**:
- `governance/organization-manage` → 22 用例 ✅
- `governance/personal-manage` → 14 用例 ✅

## 3. 拆分原则

### 3.1 通用规则

1. **`lib.rs` 末尾留一行**:`#[cfg(test)] mod tests;`
2. **`tests/mod.rs`**:文件首行 `#![cfg(test)]`,内含 mock runtime + Test 类型 + parameter_types + Config impl + 共用 fixture/helper;末尾 `mod cases_*;` 声明
3. **`tests/mock.rs`(可选)**:当 mock runtime 体积超过 200 行或测试用例 > 10 个时,把 mock 单独成文件,`tests/mod.rs` 仅做 `pub mod mock; pub use mock::*;`
4. **`tests/cases_<主题>.rs`**:按测试用例的语义主题拆分,主题命名遵循"用户字面命名照抄"(memory:`feedback_user_naming_literal`),不强造分类
5. **零行为变化**:不动用例顺序、不动断言、不动测试名;`use super::*;` 改成对应路径

### 3.2 巨型 pallet 拆分建议(三个 1000+ 行的)

仅作建议,实际文件名以拆分时观察到的语义簇为准:

- **duoqian-transfer**(1317):mock + 三类 propose / 三类 finalize / cleanup / fee
  → `mock.rs / cases_propose.rs / cases_finalize.rs / cases_fee.rs / cases_cleanup.rs`
- **admins-change**(1219):mock + Institutions storage + 提案/投票/cleanup
  → `mock.rs / cases_storage.rs / cases_proposal.rs / cases_cleanup.rs`
- **onchain-transaction**(997):mock + 转账 / 限额 / 错误路径
  → `mock.rs / cases_transfer.rs / cases_limit.rs / cases_error.rs`

中型(300~800 行) pallet 不强求拆 cases:`mod.rs(mock) + cases.rs(用例)` 两文件即可。

小型(< 200 行) pallet:`mod.rs` 单文件搞定。

## 4. 执行顺序

按"风险递增 + 行数递减"顺序串行处理,每个 pallet 一次提交:

**Step 1:小型试水(3 个,< 200 行)**
1. `transaction/institution-asset`(46)
2. `genesis`(124)
3. `otherpallet/pow-difficulty`(247)

**Step 2:中型批量(8 个,300~800 行)**
4. `issuance/citizen-issuance`(366)
5. `issuance/shengbank-interest`(368)
6. `issuance/fullnode-issuance`(503)
7. `governance/resolution-destro`(643)
8. `governance/runtime-upgrade`(675)
9. `issuance/resolution-issuance`(753,形态 B)
10. `transaction/offchain-transaction`(769,形态 B)
11. `governance/grandpakey-change`(819)

**Step 3:大型(4 个,1000+ 行)**
12. `otherpallet/sfid-system`(1014,形态 B)
13. `transaction/onchain-transaction`(997)
14. `governance/admins-change`(1219)
15. `transaction/duoqian-transfer`(1317)

每完成一个 pallet 立即跑 `cargo test -p <pallet>` 确认 0 失败再进下一个。

## 5. 验收清单

- [ ] 15 个 pallet 全部完成结构搬迁
- [ ] 每个 pallet 的 `lib.rs` 中已无 `#[cfg(test)] mod tests { ... }` 大块,只剩 `#[cfg(test)] mod tests;` 单行声明
- [ ] 形态 B 的 3 个 `src/tests.rs` 文件已删除,改为 `src/tests/` 目录
- [ ] `cargo test -p <pallet>` 在每个 pallet 通过(数量与改动前一致)
- [ ] `cargo test --workspace` 全部通过
- [ ] 业务代码 0 改动:`git diff` 中 `lib.rs / mock /benchmarks` 等业务侧除"删除测试块 + 加一行 mod 声明"外无其他变化
- [ ] 测试用例数量与改动前完全一致(改动前总数:见下表,执行时实测对账)
- [ ] git mv 保 blame:每个用例文件的 `git log --follow` 能追回到原 lib.rs

## 6. 改动前测试用例基线(grep 实测)

| Pallet | `#[test]` 数 |
|---|---|
| genesis | 6 |
| governance/admins-change | 29 |
| governance/grandpakey-change | 15 |
| governance/resolution-destro | 12 |
| governance/runtime-upgrade | 14 |
| issuance/citizen-issuance | 10 |
| issuance/fullnode-issuance | 17 |
| issuance/resolution-issuance | (tests.rs 中,需点数) |
| issuance/shengbank-interest | 17 |
| otherpallet/pow-difficulty | 7 |
| otherpallet/sfid-system | (tests.rs 中,需点数) |
| transaction/duoqian-transfer | 18 |
| transaction/institution-asset | 1 |
| transaction/offchain-transaction | (tests.rs 中,需点数;另 bank_check 5 + batch_item 5 不动) |
| transaction/onchain-transaction | 18 |

**待统计**:形态 B 的 3 个 `tests.rs` 各自的 `#[test]` 数量,执行第一步时点数补登。

## 7. 风险与回退

**主要风险**:
1. **`use super::*;` 路径失效**:测试块从 lib.rs 内层嵌套搬到 `tests/mod.rs` 后,层级少一层,测试中所有 `super::*`、`crate::Pallet`、`crate::Error` 等引用都要改 → 逐 pallet 编译验证
2. **mock runtime `frame_support::runtime` 宏展开**:mock runtime 内部的 `mod runtime { ... }` 块可能持有对外部 `Test` 类型的引用,搬动时注意 `pub` 可见性 → 对照 internal-vote 样板做
3. **`derive_impl` / `parameter_types` 宏**:可能依赖 `use frame_system as system;` 等本地 alias,搬到新文件后要把 alias 一并带过去
4. **测试块内的 `mod xxx { ... }` 子模块**:某些 pallet(如 admins-change 1219 行)测试块内可能有嵌套 mod,搬动时保持嵌套结构,不展开

**回退**:每个 pallet 一次独立提交,任一 pallet 出问题可单独 `git revert` 该提交,不影响其他 pallet。

## 8. PR 策略

按用户决策"15 个一锅端 1 个 PR",但内部分 15 次提交(每 pallet 一次),便于 review 时按 pallet 逐个审。

PR 标题:`runtime: 统一 pallet 测试目录结构 (15 pallets)`

Commit message 模板:
```
runtime/<pallet>: 拆 tests/ 目录

- src/lib.rs 测试块搬到 src/tests/{mod.rs, cases_*.rs}
- 形态 X(A/B) → 形态 C(目标)
- mock runtime 拆到 mock.rs(若适用)
- 0 行为变化,N 个 #[test] 全部通过
```

## 9. 不在本任务范围

- 修复 / 改写已有测试用例
- 优化 mock runtime 设计
- 调整 benchmarks.rs 结构(本次只动 tests)
- 调整 migrations/v1.rs 中的就地单元测试(形态 D 不动)

## 10. 增量补做(2026-05-07 同日延伸)

主任务 15 个 pallet 完工后,顺势补做 2 个原零测试 pallet:

| Pallet | 测试数 | mod.rs | cases.rs | 用例分类 |
|---|---|---|---|---|
| personal-manage | 14 用例(+2 frame 自动 = 16 passed) | 423 行(mock + 12 helper) | 460 行 | propose_create×7 + propose_close×4 + cleanup×1 + 边界×2 |
| organization-manage | 22 用例(+2 frame 自动 = 24 passed) | 441 行(mock + 14 helper) | 716 行 | SFID 登记×5 + 创建×8 + 关闭×5 + 边界×4 |

合计新增 36 用例 / 2040 行测试代码,0 失败。两 pallet 共用形态 C 双文件结构(`tests/{mod.rs, cases.rs}`),mock 各自独立,无任何跨 pallet 共享 helper 抽象。

至此 runtime 下 17 个 pallet 全部具备测试覆盖(总计 298 passed)。

## 11. 增量补做(2026-05-07 同日延伸,runtime 主 crate)

`runtime/src/`(citizenchain runtime crate,**不是 pallet**)的 26 个测试同样按本次形态搬到 `runtime/src/tests/{mod.rs, cases.rs}`:

| 原位置 | 测试数 | 性质 | 搬到 |
|---|---|---|---|
| `runtime/src/lib.rs:385-466`(82 行) | 4 | runtime 整体自检(常量/VERSION/MODULE_TAG 全局唯一/fee_payer 路由) | cases.rs 簇 1 |
| `runtime/src/configs/mod.rs:1485-2553`(1067 行) | 18 | runtime 装配集成测试(SFID 双层签名/admins/joint vote/population snapshot) | cases.rs 簇 2 |
| `runtime/src/configs/mod.rs:2740-2844`(104 行) | 4 | 机构资金白名单允许矩阵 | cases.rs 簇 3 |

合计 26 用例 / 1083 行 cases.rs + 185 行 mod.rs(共用 helper:new_test_ext / setup_step3_test_admins / build_bind_credential / build_vote_signature / build_pop_signature / 4 个 asset helper)。

**关键决策修正**:之前误判 configs/mod.rs 末尾两个测试块为"形态 D 就地单测",实际它们都是 runtime 装配后的端到端契约,跟 lib.rs 的 4 个用例同性质,统一搬出。

**副作用**:configs/mod.rs 内一个私有 `fn is_nrc_admin` 改为 `pub(crate)`(测试搬到兄弟模块后无法访问私有项)。这不算"行为变化",仍仅供 crate 内部使用。

**验收**:`cargo test -p citizenchain --lib`(需设 `WASM_FILE` 因 build.rs 强制) 37 passed 0 failed(含其他 11 个原 genesis_config_presets 等模块测试)。本地未跑过 cargo test 时,`cargo check -p citizenchain --tests` 可用作语法/类型保险。
