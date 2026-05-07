# 任务卡：Windows 控制台修复 + chainspec 冻结接入

- 任务编号：20260506-001500
- 状态：code-complete,等真机三平台 QA
- 负责人：当前主聊天入口（Blockchain Agent 主导节点客户端改造，user 负责一次性的链上导出动作）
- 关联前置：[20260505-220000-windows-macos-linux-installer-zero-dep.md](20260505-220000-windows-macos-linux-installer-zero-dep.md)（三平台打包，已交付，等真机 QA）
- 关联后续：无

## 1. 任务目标

修复两个 Windows 端启动后的关键缺陷：

1. **Windows 双击 exe 后会附带弹一个控制台窗口，且关掉控制台等于杀进程**
2. **Windows 端节点起来后不连主网，自己出一条孤链**（与 Mac/Linux 服务器互通失败）

铁律遵守：[feedback_chainspec_frozen.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_chainspec_frozen.md)、[feedback_desktop_is_miner.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_desktop_is_miner.md)。

## 2. 根因（已查实）

### 2.1 控制台窗口

[citizenchain/node/src/main.rs](../../citizenchain/node/src/main.rs) 缺少 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`。Rust MSVC 默认把可执行文件链接到 console subsystem，Windows 双击启动时会主动分配控制台作为 stdio 宿主，关闭控制台触发 CTRL_CLOSE_EVENT 杀进程。

### 2.2 孤链

[citizenchain/node/src/core/command.rs:40-47](../../citizenchain/node/src/core/command.rs:40) 的 `load_spec` 把 `--chain citizenchain` 路由到 [citizenchain/node/src/core/chain_spec.rs:83](../../citizenchain/node/src/core/chain_spec.rs:83) 的 `chain_config()`，而 `chain_config()` 用 `with_genesis_config_patch(genesis_config_presets::genesis_config())` **每次启动从当前源码现编创世**。

Substrate 的 genesis_hash 由完整创世 trie 算出，包括 `:code` 下的 runtime WASM。所以二进制嵌入哪一版 runtime 代码，就有哪一版 genesis_hash。Mac 端能连上 Linux 服务器是因为它们巧合用了同一份源码；Windows 端那次构建用的源码与当前主网不同，genesis_hash 不一致，libp2p substrate sub-protocol 在 handshake 阶段直接拒绝 → 永远 0 peer → PoW 默认全核开挖 → 出本地孤链。

仓库目前**没有任何冻结的 chainspec.json 文件**，违反 [feedback_chainspec_frozen.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_chainspec_frozen.md)。

## 3. 修复方案

### 3.1 控制台（1 行）

[citizenchain/node/src/main.rs](../../citizenchain/node/src/main.rs) 第 8 行 `#![warn(missing_docs)]` 之前加：

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

dev 构建保留控制台便于调试，release 构建走 windows subsystem 不弹控制台。

### 3.2 chainspec 冻结（两步走，user + Blockchain Agent 协作）

#### 步骤 A：user 一次性导出冻结 JSON（**只做一次，永不重做**）

主网现已发布。从任意一台**当前在线的主网权威节点**（e.g. `nrcgch.crcfrcn.com`，国储会权威节点）上导出 raw chainspec：

```bash
ssh ubuntu@nrcgch.crcfrcn.com
sudo -u citizenchain /usr/bin/citizenchain export-chain-spec \
    --chain citizenchain \
    --raw \
    > /tmp/citizenchain.raw.json
exit

scp ubuntu@nrcgch.crcfrcn.com:/tmp/citizenchain.raw.json \
    ~/GMB/citizenchain/node/chainspecs/citizenchain.raw.json
```

**注意**：

- `--raw` 是必须项,导出的是已扁平化的 key-value 对(含 `:code` 下的 runtime WASM 字节),不依赖任何 preset 函数
- 选权威节点而不是普通同步节点,避免数据库未跟齐导致导出失败
- 导出完成后**纳入版本库**,后续无论 runtime 怎么改、binary 怎么重编,这份 JSON **绝不再覆盖、绝不再重导**;runtime 升级一律走 [governance/runtime-upgrade](../../citizenchain/runtime/governance/runtime-upgrade/) 的链上 setCode

#### 步骤 B：Blockchain Agent 接入

新建目录 `citizenchain/node/chainspecs/`，把 user 拷过来的 `citizenchain.raw.json` 放进去。

改 [citizenchain/node/src/core/chain_spec.rs](../../citizenchain/node/src/core/chain_spec.rs)：

- 删 `chain_config()` 中所有 `with_genesis_config_patch` / `with_chain_type` / `with_protocol_id` / `chain_properties` / `reserve_boot_nodes` / `CHAIN_SPEC_BOOTNODES` 现编路径（这些已在 raw JSON 里固化）
- 改成：

```rust
const CHAIN_SPEC_RAW: &[u8] = include_bytes!("../../chainspecs/citizenchain.raw.json");

pub fn chain_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(CHAIN_SPEC_RAW.to_vec())
        .map_err(|e| format!("加载冻结 chainspec 失败: {e}"))
}
```

- 保留 `pub type ChainSpec = sc_service::GenericChainSpec<NoExtension>;`
- `WASM_BINARY` 不再被本文件引用(genesis WASM 已在 JSON 里),可继续保留供其它路径(e.g. setCode proposal 构造)使用

[citizenchain/node/src/core/command.rs:42](../../citizenchain/node/src/core/command.rs:42) 的 `load_spec` 不需要改 —— `"" | "citizenchain"` 仍走 `chain_spec::chain_config()`,只是这函数语义从「现编」变「读 JSON」。

## 4. 必须遵守

- 不可突破模块边界：本任务仅动 `citizenchain/node/src/main.rs` 一行 + `citizenchain/node/src/core/chain_spec.rs` 重写 + 新增 `citizenchain/node/chainspecs/citizenchain.raw.json`。runtime / 业务 0 改动
- 遵守 [feedback_chainspec_frozen.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_chainspec_frozen.md)：JSON 一旦提交,**永不重导、永不覆盖**
- 遵守 [feedback_desktop_is_miner.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_desktop_is_miner.md)：桌面端默认全核挖矿不动,[home/process/mod.rs:106](../../citizenchain/node/src/home/process/mod.rs:106) 那段 `available_parallelism()` 不许改
- 遵守 [feedback_no_chain_changes.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_no_chain_changes.md)：本任务零 runtime / 链上代码改动
- 遵守 [feedback_no_compatibility.md](../../../.claude/projects/-Users-rhett-GMB/memory/feedback_no_compatibility.md)：`chain_config()` 直接切换到读 JSON,不留旧的 `with_genesis_config_patch` 现编路径
- `chainspecs/` 目录下**只有这一个文件**,不留 dev / staging / 多版本 JSON

## 5. 输出物

- [citizenchain/node/src/main.rs](../../citizenchain/node/src/main.rs)：加 `windows_subsystem` 属性
- [citizenchain/node/src/core/chain_spec.rs](../../citizenchain/node/src/core/chain_spec.rs)：重写为 `include_bytes!` + `from_json_bytes`,删现编路径
- `citizenchain/node/chainspecs/citizenchain.raw.json`（新建,user 提供内容）
- 中文注释（`chain_spec.rs` 顶端写明「这是冻结 chainspec,不要再加 build path」）
- 残留清理：`CHAIN_SPEC_BOOTNODES` / `chain_properties` / `reserve_boot_nodes` / `ChainSpecBootnode` 全删,保留 `pub type ChainSpec`
- 文档更新：[memory/05-modules/citizenchain/node/NODE_TECHNICAL.md](../05-modules/citizenchain/node/NODE_TECHNICAL.md) 补一节「冻结 chainspec 流程」

## 6. 验收标准

### 6.1 控制台修复（Windows 真机 QA）

- [ ] 在 Windows 上 `cargo tauri build --release` 出 NSIS exe
- [ ] 双击桌面快捷方式启动,**不弹任何 cmd / powershell 黑窗口**
- [ ] dev 模式 `cargo tauri dev` 在 Windows 上仍弹控制台便于看 log

### 6.2 chainspec 冻结（三平台同源验证）

- [ ] 在 Mac / Windows / Linux 各编一份 binary,启动后日志输出的 `Genesis: 0x...` **三平台完全一致**,且与 `nrcgch.crcfrcn.com` 主网当前 genesis 一致
- [ ] Windows 端启动后 5 分钟内 peer count > 0,且能从主网拉到当前最高块
- [ ] Windows 端 `system_chain_getBlockHash` RPC 拿 #0 与主网 `nrcgch.crcfrcn.com` 拿 #0 字节相同
- [ ] 三平台节点都能在 5 分钟内追到主网 best,本地不出现先于主网的孤块

### 6.3 残留清理

- [ ] `grep -rn "with_genesis_config_patch\|CHAIN_SPEC_BOOTNODES\|reserve_boot_nodes" citizenchain/node/src/` 零结果
- [ ] `genesis_config_presets` 在 node 目录下不再被 use
- [ ] `Cargo.lock` 未受本任务影响

## 7. 风险与回滚

- **风险 1**：JSON 文件大（含 raw runtime WASM，预计 1~3 MB）。`include_bytes!` 会把它整段嵌入二进制 → exe / dmg / deb / AppImage 都会增大对应字节。可接受
- **风险 2**：步骤 A 导出时 user 误用了非主网节点（如本机 dev 节点），冻结的 JSON 与真主网 genesis 不符 → 回滚后所有节点需统一切换到这份新 JSON。**回滚办法**：若发现导出错了，立刻在仓库 revert JSON commit 与代码 commit；服务器侧不动（Linux 服务器跑的还是 deb 老版本，没切到新冻结代码）
- **风险 3**：`from_json_bytes` 加载耗时（启动慢 100~300 ms）。可接受
- **回滚整体**：`git revert` 这一个 PR,服务器 deb 不变,Mac/Win 用户重装老版本即可

## 8. 分工与执行顺序

| 顺序 | 谁 | 做什么 | 状态 |
|---|---|---|---|
| 1 | Blockchain Agent(代 user 执行) | SSH 到 `nrcgch.crcfrcn.com`,跑 `export-chain-spec --raw`,scp 回本地 `citizenchain/node/chainspecs/citizenchain.raw.json`(1.32 MB,sha256 `2b9f46e4aefb66f892d5dc170b2c2bfc33b6b12a88192617b06c18e8ea38a2db`) | ✅ done(2026-05-06) |
| 2 | Blockchain Agent | 写 `windows_subsystem` 属性([main.rs:8](../../citizenchain/node/src/main.rs:8)) | ✅ done |
| 3 | Blockchain Agent | 重写 [chain_spec.rs](../../citizenchain/node/src/core/chain_spec.rs) 为 `include_bytes!` + `from_json_bytes`,删 `CHAIN_SPEC_BOOTNODES`/`reserve_boot_nodes`/`chain_properties`/`ChainSpecBootnode` 现编路径,文件 97 行 → 24 行 | ✅ done |
| 4 | Blockchain Agent | 更新 [NODE_TECHNICAL.md](../05-modules/citizenchain/node/NODE_TECHNICAL.md) §4 重写为「冻结铁律」并补冻结流程,行数表同步 | ✅ done |
| 5 | Blockchain Agent | `cargo check -p node`(WASM_FILE 用 target/wasm 缓存)零本次改动相关警告;改动文件零警告;耗时 34.5s | ✅ done |
| 6 | Blockchain Agent | 任务卡回写 + memory 索引落 [project_chainspec_frozen_2026_05_06.md](../../../.claude/projects/-Users-rhett-GMB/memory/project_chainspec_frozen_2026_05_06.md) | ✅ done |
| 7 | **user** | 三平台真机出包,跑 §6 全套 QA(Win 不弹控制台、三平台 genesis 一致、Windows peer count > 0、5 分钟内追到主网 best) | pending |
