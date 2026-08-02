# 开发者升级与协议升级彻底分开 + 挖矿页轻节点改读链上 CID 计数

任务需求：
1. 删除区块链软件挖矿页「N 个引导节点未响应，节点数可能偏低」提示。
2. 开发者升级与协议升级彻底分开：公民控制台 `CitizenChain WASM` 卡片的「协议升级」
   按钮改名为「开发者升级」（功能不动，仍是 `developer_direct_upgrade`）；
   区块链软件国储会 tab 的「开发升级」功能整条删除，节点端只保留「协议升级」
   （`propose_runtime_upgrade`，走联合投票）。
3. `CitizenIdentity` 新增 `CidCount` 计数器（注册 +1 / 注销 −1），
   区块链软件挖矿页「在线节点」卡片的「轻节点」一格改读该计数。

所属模块：citizenchain/node、citizenchain/runtime（citizen-identity）、citizenconsole

输入文档：
- memory/07-ai/agent-rules.md
- memory/07-ai/unified-naming.md
- memory/05-modules/citizenchain/node/governance/GOVERNANCE_TECHNICAL.md
- memory/05-modules/citizenchain/runtime/governance/runtime-upgrade/RUNTIMEUPGRADE_TECHNICAL.md
- memory/07-ai/module-definition-of-done

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 链端 `runtime_upgrade::developer_direct_upgrade` 及其 QR 三处登记必须保留
  （公民控制台正在使用，删除会导致冷签两色 decodeFailed 红拒）
- 已上链存储结构变更走 runtime 升级 + 一次性 `OnRuntimeUpgrade` migration，不重新创世
- 挖矿页「在线节点」卡片固定三个值三个标签，布局不动

## 已定口径

- 公民控制台改名范围 = 5 处用户可见文案；代码层 `protocol-upgrade` id/mode、
  `rtupg` 前缀、CSS 类名、注释不动。
- 计数器命名 `CidCount`，`StorageValue<_, u64, ValueQuery>`，语义 = 当前 Active CID 数。
- 挖矿页公式保持 `在线节点 = 全节点 + 轻节点`，轻节点值改为链上 `CidCount`。

## 分步

| 步 | 内容 | 范围 | 状态 |
|---|---|---|---|
| 一 | 删挖矿页「引导节点未响应」提示 + 顺带清 `active_threads` 死代码 | `node/src/mining/network_overview.rs` | 完成 |
| 二 | 节点端删除开发升级全链路（含 `.dev-upgrade-*` 死 CSS） | citizenchain node 前后端 | 完成 |
| 三 | 控制台「协议升级」改名「开发者升级」 | citizenconsole | 完成 |
| 四 | 链端 `CidCount` 计数器 + migration | citizen-identity pallet + runtime | 完成 |
| 五 | 挖矿页轻节点改读 `CidCount` | `node/src/mining/network_overview.rs` | 完成 |

每步固定收尾：更新文档 → 完善注释 → 完善测试 → 清理残留 → 输出下一步技术方案，
用户确认后才进入下一步。

## 跨步遗留

- `HOME_TECHNICAL.md:43/47` 的「协议升级 / 开发升级」已随第三步改名为「开发者升级」，已处理。
- **本任务外的既有 fmt 红**：`citizenchain/node/src/core/node_guard/mod.rs:1739` 有一处工作区
  未提交改动（测试改名 `fee_behavior_probes` → `policy_behavior_probes`），手写换行不符合
  rustfmt，导致 node 的 `cargo fmt --check` 失败。非本任务改动，未动它（跑 `cargo fmt` 会
  改写他人在途工作）。本任务改的 5 个 node 源文件逐个 `rustfmt --check` 全部通过。
- **本任务外的既有失败**：`citizenconsole/test/production-security.test.mjs:141` 断言
  `citizenapp/cloudflare/schema/citizenapp.sql` 含 `SCHEMA VERSION: vX.Y.Z`，但工作区里
  该文件首行的 `-- SCHEMA VERSION: v1.0.0` 已被替换成一句重复描述（`git diff` 可见，
  非本任务改动）。控制台测试 48/49，唯一失败即此项。修法二选一：把版本行加回
  citizenapp.sql，或按新基线口径改测试断言。未在本任务内动，等用户指令。

输出物：
- 代码
- 中文注释
- 测试
- 文档更新
- 残留清理

验收标准：
- 功能可运行
- 测试通过
- 文档已更新
- 残留已清理
- Review 问题已处理
- 模块级完成标准已对照
