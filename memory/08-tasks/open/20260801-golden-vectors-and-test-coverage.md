# 金标防漂移与测试覆盖补齐

状态：open（2026-08-01 立项，分四步，每步先出方案、确认后执行）

## 背景

2026-08-01 全仓审计结论：**密码学金标值本身没有漂移，但一致性靠人工维护、没有机制保证**；
另有一个 5.4 万行的模块零测试。

审计口径修正记录：中途曾按数组索引对齐比较 `signing_domain_vectors`，得出「冲突 45 处」，
系假阳性。按 `(op_tag, scale_payload_hex)` 重新对齐后，12 个共同向量 `message_hex` 全部一致。
**比较金标必须按语义键对齐，禁止按数组下标对齐。**

## 现状事实

### 金标向量分布

| 向量组 | chain | app | wallet | worker | 值一致性 |
| --- | --- | --- | --- | --- | --- |
| `account_derive_vectors` | ✅ | ✅ | — | — | 文件逐字节相同 |
| `binary_prefix_domain_vectors` | ✅ | ✅ | ✅ | — | 三方两两零冲突（`layout` 描述措辞不同） |
| `signing_domain_vectors` | ✅ 14 条 | ✅ 12 条 | ❌ | ❌ | 12 条共同向量全一致，app 为 chain 真子集 |

- 零软链接、零生成器，三份 `binary_prefix` 文件 hash 全不同（差异仅在 `layout` 人类可读字段）
- **CI 四条流水线无一校验金标一致性** → 改一端向量，另一端不会失败
- 加载方式：Rust 用 `env!("CARGO_MANIFEST_DIR")`；Dart 用相对 package 根路径

### 测试分布

| 模块 | 测试文件 | 测试行数 | 源码行数 | 比值 |
| --- | ---: | ---: | ---: | ---: |
| citizenchain 链 | 54 | 30,475 | 163,210 | 19% |
| citizenapp 客户端 | 174 | 32,026 | 111,705 | 29% |
| Worker | 32 | 8,529 | 23,336 | 37% |
| citizenwallet 冷钱包 | 23 | 7,120 | 19,454 | 37% |
| **onchina 控制台** | **0** | **0** | **54,579** | **0%** |

onchina 271 个源文件（134 `.rs` + 69 `.ts` + 68 `.tsx`），无 test 脚本。承担机构创建/关闭、
管理员变更、链上写提案、passkey + 冷签三档鉴权、作用域过滤 `*_in_scope`（fail-closed 安全边界）。

## 可复用的既有模式

仓库已有跨端一致性校验先例，**新增机制必须复用它，不另造**：

- `.github/scripts/check-pallet-registry-sync.mjs` — 校验 Dart pallet 注册表与链上
  `construct_runtime` 索引一致，退出码 0/1，CI 中作为独立步骤
- `.github/scripts/check-ai-guardrails.sh` — 文档、残留与注册表门禁

## 四步计划

### 第 1 步：金标防漂移机制 ✅ 已完成（2026-08-01）

单一真源 + CI 逐字段比对。消除「各端各存一份、靠人工同步」。

**落地内容**

- 新增 `.github/scripts/check-golden-vectors-sync.mjs`（纯 Node、无依赖、退出码 0/1）
  - 真源：`citizenchain/runtime/primitives/tests/fixtures/`
  - 按语义键对齐：`signing` 用 `(op_tag, scale_payload_hex)`、`binary_prefix` 用 `name`、
    `account_derive` 用 `(cid_number, kind)`
  - 只比密码学值，`layout` / `_comment` / `source` 等描述字段不比（各端措辞本就不同）
  - 镜像必须是真源子集；镜像自造向量即失败；未覆盖只提示不失败
- 三条流水线各挂一处（缺一不可，路径路由决定）：
  - `citizenchain-ci.yml` → **新增独立 job `golden-vectors`，不设 if**
    （原 `guardrails` 只在 PR 跑，而真源改动常直接推 main，挂那里等于失效）
  - `citizenapp-ci.yml` → `check` job
  - `citizenwallet-ci.yml` → `build-apk` job
- 七份向量文件的 `_comment` 标注【真源】/【镜像】角色与同步义务

**破坏性验证**（改完即还原，工作区干净）

| 破坏方式 | 结果 |
| --- | --- |
| 篡改镜像 `message_hex` 一个字符 | 退出码 1，指出 `op_tag=0x10` 冲突并列出两侧值 |
| 镜像自造一条真源没有的向量 | 退出码 1，提示先在真源登记 |
| 改镜像顶层 `domain` | 退出码 1，指出 `GMB` vs `GMX` |

**回归**：citizenapp 金标 77 passed；citizenwallet signer 169 passed；
Rust `primitives` 78 + 金标 2 全通过。七份文件 diff 逐一验证「除 `_comment` 外与 HEAD 完全相同」。

### 第 2 步：Worker(TS) 补签名域金标 ✅ 已完成（2026-08-01）

**立项时的判断有误，执行时纠正**：审计说「Worker 侧零金标」，实际是向量**硬编码在
`test/signing_message.test.ts` 里**（11 条）。误判原因是只按文件名搜 `vector|fixture|golden`，
漏了内联常量数组。**结论：按文件名搜索不足以判定「有没有测试」，必须搜实现符号。**

先验证这 11 条内联向量与真源的一致性：**零不一致、零真源缺失**，未覆盖 `0x12/0x1e/0x1f`。
所以真问题不是「没有金标」，而是「金标内联、不受跨端校验保护」——与第 1 步同病。

**落地方式：直读真源，不建镜像**

沿用既有 `test/cross_end_contract.test.ts` 的做法（直接读另一端源文件，该文件注释记录了
`device_public_key` vs `device_public_key_hex` 键名漂移导致线上 100% 400 的真实事故）。
Worker 测试 `environment: 'node'`，`fs` 可用，CI 全仓 checkout，路径可达。

零副本 > 副本 + 比对：漂移在**物理上不可能**发生，而非「能被检测到」。因此 Worker
**不登记**进 `check-golden-vectors-sync.mjs` 的 mirrors，脚本注释已说明缘由。

**用例 11 → 19**

- 真源全部 14 条向量逐字节比对（含 Worker 当前未使用的域——`signingMessage` 是通用原语）
- 4 个 `OP_SIGN_*` 常量按真源 `name` 反查 `op_tag` 钉死
  （摘要算对不代表常量用对：常量写错会去验一个密码学合法、语义错误的签名）
- 1 条守卫断言真源可读、`domain == GMB`、向量非空
  （读空数组会一条用例都不生成而整体显示通过——金标静默失效）

**破坏性验证**（改真源，改完即还原）

| 破坏方式 | 结果 |
| --- | --- |
| 真源改一个 `message_hex` | TS 测试红，精确指出 `OP_SIGN_SQUARE_LOGIN (0x1b)` |
| 真源改 `op_tag` 登记值 | TS 测试红 2 条（向量比对 + 常量比对同时抓到） |

**回归**：Worker `typecheck` 通过，全量 **32 文件 / 228 用例全绿**；
真源 diff 验证「除 `_comment` 外与 HEAD 完全相同」；跨端门禁仍退出码 0。

**本步发现、未做的缺口（待定，属新增真源，不擅自扩大范围）**

`scaleCompact` / `scaleString` / `u64Le` 三个 SCALE 编码原语**没有独立金标**。
`device_subkey.test.ts` 虽引用了 `scaleString`/`u64Le`，但那是**用它们构造期望值**——
实现错了期望值同步错，测试照样绿，属于自证。`scaleCompact` 的分支边界
（63/64、16383/16384、2^30-1）是典型易错点，且它决定被签 payload 的字节。
补这一组需要在真源新增 SCALE 向量（Rust 侧 parity-scale-codec 生成），
不属于「补镜像」范围，单独决策。

### 第 3 步：冷钱包补哈希域金标 ✅ 已完成（2026-08-01）

**核实结果**：冷钱包哈希域 op_tag 为 **0x10 / 0x12 / 0x1F**
（`qr_signer.dart` 的 `_opSignCitizenIdentity` / `_opSignCidOccupy` / `_opSignCidAdminRebind`）。

**真缺口是「自证」而非「无测试」**：`qr_signer_test.dart` 第 336 / 361 / 386 行三处签名域断言，
都是测试自己用 `Blake2bDigest` 算一遍当 expected 再与实现比。这只证明「实现调用了
pointycastle」，没证明「结果与链端一致」——若 pointycastle 的 `Blake2b(digestSize:32)`
与 Rust `blake2_256` 在任何参数上有差异，或两处同时用错，测试照样全绿。
而冷钱包是**真正拿私钥出签的那一端**，签出链端不认的签名要到线上才暴露。

**落地方式：直读真源**（与 Worker 同策略，实测 `flutter test` cwd 为 package 根，
`../citizenchain/...` 可达）。新增 `test/signer/signing_domain_golden_test.dart`，**17 个用例**：

- 真源全部 14 条向量：`blake2_256(GMB || op_tag || payload)` 逐字节比对
  **期望值来自 Rust 生成的真源，自证由此变他证**；首次验证了
  pointycastle `Blake2b(digestSize:32)` ≡ Rust `blake2_256`
- 公开出签路径：`QrSigner.signingBytesFor` 走 `OP_SIGN_CITIZEN_IDENTITY` 端到端比对，
  并钉死「签的是摘要不是原文」
- 三个哈希域 op_tag 在真源的登记值（`0x10`/`0x12`/`0x1f`）

**只覆盖 0x10 的公开路径**：另两域要求 payload 是结构化 Authorization 模板并原位替换
账户槽，真源的任意字节 payload 套不进该结构。其槽位与 op_tag 组装由 `qr_signer_test.dart`
覆盖，摘要正确性已由上一组独立锁住。双向锁成立：链端改域编号 → 新文件红；
冷钱包改常量 → `qr_signer_test.dart` 红。

**未改** `qr_signer_test.dart`：两个方向都已被拦，无需再动（避免无谓改动）。

**破坏性验证**（改真源，改完即还原）

| 破坏方式 | 结果 |
| --- | --- |
| 真源改 `0x10` 的 `message_hex` | 红 2 条（原语比对 + 公开路径比对） |
| 真源把 `cid_occupy` 改成 `0x22` | 红 2 条（原语比对 + op_tag 登记断言） |

**回归**：`dart analyze` 无问题；冷钱包全量 **271 passed / 0 failed**；
真源验证「向量内容与 HEAD 完全相同」；跨端门禁仍全绿。

**未新增镜像文件**，故 `check-golden-vectors-sync.mjs` 的 `GROUPS[0].mirrors` 保持不变
（脚本注释已更新说明冷钱包与 Worker 均走直读）。

### 第 4 步：onchina 补三档鉴权测试 ✅ 已完成（2026-08-01）

**审计结论被推翻**：本卡开头写的「onchina 零测试 / 0%」是**错的**。

| 项 | 审计说 | 实际 |
| --- | --- | --- |
| Rust `#[test]` | 0 | **182 个** |
| 含测试的 Rust 文件 | 0 | **43 / 134（32%）** |
| Rust 测试代码 | 0 行 | **3,959 行（占比 10%）** |
| 前端 ts/tsx | — | 确实零测试（137 文件，无 test 脚本、无测试依赖） |

错因：按**文件名**判断（`grep -icE "\.test\.|/tests?/"`），而 Rust 标准做法是内联
`#[cfg(test)] mod tests`，文件名里没有 "test"。**这是同一个坑的第四次**（第 2 步是内联
金标向量）。死规则：**判断「有没有 X」必须搜实现符号；文件名搜索只能用于定位，不能用于否定。**

且 `cargo test --workspace` 早已在 CI（`citizenchain-ci.yml:199`），onchina 在 workspace 内——
这 182 个测试一直在跑，测试基础设施不需要搭。

**真实缺口（按安全权重）**

| 目录 | 生产行 | 测试行 | 占比 |
| --- | ---: | ---: | ---: |
| `auth` | 5,426 | 285 | **5%** |
| `institution` | 3,813 | 209 | **5%** |
| `src` 顶层（含 `main.rs` 的 `*_in_scope`） | 2,951 | 0 | **0%** |
| `indexer` | 899 | 0 | **0%** |
| `domains` | 13,001 | 2,205 | 17% |
| `scope`（规则派生，作用域过滤已下沉 SQL） | 220 | 157 | 71% |

用户选定 **A 方案：只补 `auth`**（安全权重最高、零新依赖、纯函数可测）。

**落地内容**：`src/auth/operation_auth.rs` 内联测试 4 → **11 个**，新增 7 个：

- `expected_tier()` **穷尽 match 登记表** —— 新增 `AdminActionType` 变体时**编译失败**，
  强制登记档位。比运行期断言强：`auth_type()` 用 `|` 按组归档，新变体极易被顺手并进
  错误的组；登记表逐个列举，形态不同，两处同时错的概率极低
- `every_action_tier_matches_the_registry` —— 20 个变体逐一比对
- `no_write_action_falls_back_to_session_tier` —— 写动作落到只读档 `Session` = 完全绕过
  passkey，最严重的降档失败模式，单独立断言
- `governance_and_signing_actions_must_require_cold_sign` —— **独立于登记表的第二道锁**，
  按动作语义（`PROPOSE_*`/`CAST_*`/`*_SIGN`/`GUARD_VOTE`）推导，不看 `auth_type()` 实现
- `tier_membership_counts_are_pinned` —— Passkey 档恰好 4、ColdSign 档恰好 16。
  逐变体断言可能连同登记表一起被改而静默通过，计数锁迫使档位人数变化成为显式决策
- `action_type_string_mapping_round_trips_for_every_variant` —— `as_str()` ⇄
  `parse_action_type()` 全 20 变体往返（线格式漏改会把动作解析成另一个，可能连带换掉鉴权档）
- `all_actions_list_covers_every_variant` —— 表内无重复且每项都过穷尽 match

**破坏性验证**（改完即还原）

| 破坏方式 | 结果 |
| --- | --- |
| 把 `InstitutionCreate` 从链上写降档为本地写 | 红 2 条（逐变体断言 + 计数锁） |
| `as_str` 改 `GUARD_VOTE` → `GUARD_VOTE_V2`（映射不对称） | 红 2 条（往返测试 + 既有立法测试） |

**回归**：`cargo fmt` 无差异；onchina 全量 **189 passed / 0 failed**；改动仅
`src/auth/operation_auth.rs` 一个文件。

**发现但未修（既有问题，不属本步范围）**

`cargo clippy -p onchina --all-targets` 报 1 个 warning：
`onchina/src/domains/citizens/occupy.rs:2474` 对 `Result` 用 `expect()`。
用 `git stash` 验证 **HEAD 版本同样存在**，非本次引入。CI 用
`cargo clippy --workspace --all-targets --locked -- -D warnings`，**该 warning 会让 CI 变红**。
需单独处理。

## 执行约定

每步：**先出技术方案 → 用户确认 → 执行 → 更新文档 + 完善注释 + 清理残留 → 输出下一步方案**。

## 验收

- 任一端单方面修改金标向量值，CI 必须失败
- 签名域四端（chain / app / wallet / worker）全部被金标锁住
- onchina 两条安全边界有测试，且能在 CI 跑
