# 任务卡：删除开发链，统一为正式链（remove-dev-chain）

- 任务编号：20260325-remove-dev-chain
- 状态：in-progress
- 所属模块：citizenchain-node, citizenchain-runtime, citizenchain-node
- 当前负责人：Claude (Blockchain Agent)
- 创建时间：2026-03-25

## 任务需求

删除所有开发链（dev chain）设计，只保留一条正式链（mainnet）。正式链分为开发期和运行期，开发期单节点即可 finalize。

## 技术方案

### 1. 删除 dev-chain feature flag

从 4 个 Cargo.toml 中删除 `dev-chain` feature 定义。

### 2. 删除 dev 链代码

- `chain_spec.rs`：删除 `dev_config()` 函数
- `genesis_config_presets.rs`：删除 `dev_config_genesis()` 和 DEV_RUNTIME_PRESET
- `service.rs`：删除 `ensure_dev_grandpa_key()` 和 `is_dev_chain` 变量
- `command.rs`：删除 dev 路由

### 3. 修正 mainnet 开发期 GRANDPA 配置

- `mainnet_config_genesis()`：GRANDPA 权威只放 NRC 第 1 把密钥
- `service.rs`：节点启动时检测 keystore 中是否有 GRANDPA 密钥，有则参与 finality（不区分链类型）

### 4. 清理 node 和脚本

- `node/backend/build.rs`：删除 dev-chain 条件编译
- `node/backend/src/home/process/mod.rs`：删除 `--chain dev`
- `scripts/run.sh`, `clean-dev.sh`：删除 `--features dev-chain`
- `scripts/citizenchain-node.service`：删除 `--chain dev`

### 改动范围

| 文件 | 改动 |
|------|------|
| `node/Cargo.toml` | 删 dev-chain feature |
| `runtime/Cargo.toml` | 删 dev-chain feature |
| `runtime/primitives/Cargo.toml` | 删 dev-chain feature |
| `node/backend/Cargo.toml` | 删 dev-chain feature |
| `node/src/chain_spec.rs` | 删 dev_config() |
| `node/src/command.rs` | 删 dev 路由 |
| `node/src/service.rs` | 删 ensure_dev_grandpa_key/is_dev_chain，改 GRANDPA 逻辑 |
| `runtime/src/genesis_config_presets.rs` | 删 dev_config_genesis，修 mainnet GRANDPA 单权威 |
| `node/backend/build.rs` | 删 dev-chain 条件 |
| `node/backend/src/home/process/mod.rs` | 删 --chain dev |
| `scripts/run.sh` | 删 --features dev-chain |
| `scripts/clean-dev.sh` | 删 --features dev-chain |
| `scripts/citizenchain-node.service` | 删 --chain dev |
