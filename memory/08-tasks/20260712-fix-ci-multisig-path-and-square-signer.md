# 修复两端 CI 失败:钱包 multisig 过期路径 + 公民 square 请求签名器夹具

任务需求：修复 CitizenWallet 与 CitizenApp 两条 CI 在提交「发布会员体系」后的失败，并同步更新文档、完善注释、清理残留。
所属模块：ci / citizenwallet / citizenapp（8964 广场）/ citizenchain（脚本与文档路径）

## 背景（基于当前 diff 的判断）

两条失败互相独立，都是这批「统一命名 + 发布会员体系」提交的连带遗漏：

- CitizenWallet CI 挂在「同步 runtime pallet/call 索引」步骤：`.github/workflows/citizenwallet-ci.yml`
  写死路径 `citizenchain/runtime/transaction/multisig-transfer/src/lib.rs`，但「统一命名」提交
  4ad7a9840 已把该 pallet crate 目录改名为 `multisig`（crate 包名现为 `multisig`）。步骤在
  `set -euo pipefail` 下 `grep` 找不到文件即 exit 1。
- CitizenApp CI 挂在「运行测试」(511 passed / 5 failed / 5 skipped)：`SquareApiClient._headers`
  在「发布会员体系」后新增硬校验——带 session 的请求若 `session.signRequest == null` 直接抛
  `设备请求签名器缺失，请重新登录`。5 个 square(8964)测试的 `SquareSession` 夹具漏注入签名器。

## 必须遵守（红线）

- 链上稳定标识**绝不改**：runtime 类型别名 `MultisigTransfer`（construct_runtime 名）、
  当时 MODULE_TAG 字节串仍为旧长名；2026-07-15 已在 breaking runtime 中统一为 `b"multisig"`，五端 ABI 同步且不兼容旧标签。
- CitizenApp 的 `citizenapp/lib/transaction/multisig-transfer/` 目录是独立产品当前命名、
  import 自洽，**不动**。
- 大面积命名注册表（unified-naming.md 545/560/573、模块技术文档、node 侧目录 doc）归属已存在的
  open 任务卡 `20260711-citizenchain-naming-cleanup.md`，本卡不越界重写，只清理与本次 CI 断因
  直接相关的坏路径。

## 输出物

- 代码/配置修复：
  - `.github/workflows/citizenwallet-ci.yml`：`multisig-transfer` → `multisig`（第 11、50 行路径）。
  - `citizenwallet/scripts/citizenwallet-run.sh`：本地版同款路径（第 28 行）。
  - `citizenchain/scripts/benchmark.sh`：benchmark 输出路径（multisig_transfer 行）。
  - `citizenchain/runtime/transaction/multisig/src/weights.rs`：文件头 `--output=` 注释路径。
  - CitizenApp 3 个测试文件的 `SquareSession` 夹具补 `signRequest` 假签名器。
- 文档更新：`memory/07-ai/unified-protocols.md` P-TX-005 的代码路径（真源 lib.rs + cargo manifest）。
- 中文注释：夹具处说明 `_headers` 现强制要求设备签名器。
- 残留清理：以上 citizenchain 侧漏改的过期 `multisig-transfer` 路径。

## 验收标准

- `flutter test` 两条 CI 相关测试本地通过（3 个 square 测试文件绿）。
- CI 索引同步 grep 逻辑对新路径能取到 `propose_transfer` 的 call_index(0)。
- 保留清单未被误改（`MultisigTransfer` 类型名、CitizenApp 业务目录）；链上 MODULE_TAG 后续已统一为 `b"multisig"`。
- 文档路径与实际目录一致；残留已清理。

## 执行结果（2026-07-12）

- CitizenWallet：`citizenwallet-ci.yml`(11/50)、`citizenwallet-run.sh`(28)、
  `citizenchain/scripts/benchmark.sh`(63)、`multisig/src/weights.rs` 头注释 的过期
  `multisig-transfer` 路径全部改为 `multisig`。本地模拟 CI「同步 runtime pallet/call 索引」
  步骤对新路径取值成功：OnchainTransaction=4 / MultisigTransfer=19 / VotingEngine=9，
  propose=0 / joint=0 / referendum=1，无空值 → 该步骤会通过。
- CitizenApp：3 个 square 测试文件的 `SquareSession` 夹具补 `signRequest` 假签名器
  （`square_feed_service_test` 抽共享 `_session()`；两个 profile 测试改共享 helper 一处）。
  `flutter test` 三文件 17 passed；`flutter analyze` 三文件 No issues。
- 文档：`unified-protocols.md` P-TX-005 代码路径（真源 lib.rs、消费者、CI 同步名、cargo
  manifest）改为 `multisig`；保留链上名 `MultisigTransfer` 与仍存在的文档目录路径。
- 复查：citizenchain 代码/脚本、`.github/`、citizenwallet 三侧已无 `transaction/multisig-transfer`
  文件系统路径残留。

### 未纳入本卡（归属 `20260711-citizenchain-naming-cleanup.md`）

命名注册表与模块技术文档里仍以 `multisig-transfer` 记录 citizenchain 目录/命名的条目
（`unified-naming.md` 545/560/573、`CITIZENCHAIN_TECHNICAL.md`、`GOVERNANCE_TECHNICAL.md`、
各 `MULTISIG_TRANSFER_*` 模块文档等），属命名统一的注册表重写范畴，不在本次 CI 修复口径内，
留给该 open 卡统一收口，避免半状态改动。

## 追加：白皮书同批改名 follow-up（2026-07-12，用户追问后）

用户追问白皮书是否也漏改。核查 `citizenweb/src/whitepaper.md`（真源；`dist/*.js` 是构建产物、
`.claude/worktrees/` 副本不碰）确认第 5 章模块小节标签照抄 runtime 目录名，同批改名漏跟。已改：

- 5.8.1 标签 `onchain-transaction` → `onchain`
- 5.8.2 标签 `offchain-transaction` → `offchain`；正文（596 行，中/英）`offchain-transaction 模块` → `offchain 模块`
- 5.8.3 标签 `multisig-transfer` → `multisig`
- 5.9 标签 `otherpallet` → `misc`；架构树（368 行）`otherpallet/` → `misc/`
- 5.3.2 标签 `resolution-destro` → `resolution-destroy`；正文（490 行，中/英）同改

顺带（用户批准纳入）修 `citizenchain/scripts/benchmark.sh` 同批改名的坏路径/错 key（均对齐
`runtime/src/benchmarks.rs` 的 `define_benchmarks!`，`pow_difficulty` 印证 key 必须等于宏首 ident）：

- `citizen_identity` / `pow_difficulty` 输出路径 `runtime/otherpallet/…` → `runtime/misc/…`
- `resolution_destro:…/resolution-destro/…` → `resolution_destroy:…/resolution-destroy/…`（key+path）
- `multisig_transfer:` key → `multisig:`（补齐前一轮只改 path 的遗漏）

### 越界发现（不属本卡，另有归属，未动）

- `benchmark.sh` 的 `admins_change:runtime/governance/admins-change/src/weights.rs` 是**悬空条目**：
  admins 已拆成 `runtime/admins/{public,private,personal}-admins/`，该路径不存在且 `admins_change`
  未在 `define_benchmarks!` 注册——属 admins 拆分重构残留，与本次改名无关，留给 benchmark 专项卡
  （`20260623-benchmark-genesis-test-memory-sync.md`）。`citizen_identity` 亦为 benchmark 未注册项
  （其 weights 为手工上界，见注释），同属 benchmark-list 卫生，本卡不动。

## 追加：越界项清理 + 白皮书全面再检查（2026-07-12，用户续令）

- 清理越界项：删除 `benchmark.sh` 悬空条目 `admins_change:runtime/governance/admins-change/…`
  （admins 已拆成 `runtime/admins/{public,private,personal}-admins/`，且未在 `define_benchmarks!`
  注册）。删后最终校验：10 条 pallet 路径全真实、key 全注册（唯一 `citizen_identity` 是按设计
  不注册的已知例外，路径有效）。
- 白皮书 `citizenweb/src/whitepaper.md` 全面再检查：
  - ✅ 34 个章节英文标签全部对应真实 runtime 目录，零错位。
  - ✅ 无历史旧名（`wuminapp/wumin/WUMIN_/SFID/duoqian/ElectionCampaign/OrgType/admin-management/
    *_transaction` 等零命中）。
  - ✅ 反引号标识符 `ProposalOwner/MODULE_TAG/powr/citizenchain` 均真实存在。
  - ⚠→已修：603 行（中/英各一次）`submit_offchain_batch_v2` → `submit_offchain_batch`
    （真实 extrinsic 无 `_v2`，见 `offchain/src/lib.rs:475`）。属独立的过期版本后缀，非改名批次。
  - 复查：白皮书 src 已无 `_v[0-9]` 悬空后缀，snake_case 标识符全部对得上代码。
