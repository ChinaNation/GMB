# Fee Address 模块技术文档

## 0. 功能需求

- 页面需要支持绑定或变更手续费收款地址，并展示当前已绑定地址。
- 模块需要校验设备开机密码后才允许修改地址。
- 模块需要同时支持 `0x + 64 hex` 和 SS58(2027) 两种地址输入格式，并进行标准化。
- 模块需要为本机维护稳定的 `powr` 矿工签名账户，确保奖励地址绑定和实际挖矿账户一致。
- 模块需要拒绝把本机 `powr` 矿工账户本身设置为奖励钱包，要求使用独立收款钱包。
- 模块需要把矿工签名密钥安全存储，并同步写入本地节点 keystore，且清理旧的 `powr` key 文件。
- 模块需要在提交链上绑定交易前确认当前本地 RPC 端口确实属于目标链，避免把绑定操作误发到错误网络。
- 当链上绑定失败或超时时，模块需要明确告诉调用方“本地已保存，但链上未完成”。

## 1. 模块位置

- 路径：`nodeui/backend/src/settings/fee-address/mod.rs`
- 对外命令：
  - `get_reward_wallet`
  - `set_reward_wallet`

## 2. 模块职责

- 管理桌面节点 UI 的“手续费收款地址”本地配置。
- 管理全节点挖矿签名密钥（`POWR_MINER_SURI`）本地持久化。
- 将地址变更同步为链上 `FullnodePowReward.bind_reward_wallet/rebind_reward_wallet` 交易。
- 确保本地节点启动时使用同一 `powr` 矿工签名账户，保证奖励与手续费分成可路由到绑定地址。
- 启动前同步 `powr` keystore（并清理旧 `powr` key 文件），避免通过环境变量传递明文密钥。

## 3. 数据模型

- `RewardWallet`
  - `address: Option<String>`
  - 返回给前端的展示结构。
- `StoredWallet`
  - `address: String`
  - 本地文件持久化结构。

## 4. 存储设计

- 地址配置：`<app_data_dir>/reward-wallet.json`
- 矿工签名 URI：系统安全存储（Keychain/Keyring）键 `powr-miner-suri`
  - 以 `nodeui/backend/src/shared/security.rs` 的 AES-GCM 密文封装保存。
  - 首次缺失时，尝试从本地链 keystore 的 `powr` key 文件提取；仍不存在则生成随机 `0x<32bytes>` seed 并写入安全存储。
  - 旧版明文文件 `<app_data_dir>/miner-suri.txt` 仅用于一次性迁移；安全优先：先删除明文旧文件，再写入安全存储，即使安全存储写入失败也不会在磁盘上残留明文密钥。

## 5. 命令流程

### 5.1 `get_reward_wallet`

1. 解析 `reward-wallet.json` 路径。
2. 文件不存在返回 `address = null`。
3. 存在则反序列化后返回地址。

### 5.2 `set_reward_wallet`

1. 校验设备登录密码非空。
2. 通过 `nodeui/backend/src/settings/device-password/mod.rs` 执行设备密码验证（macOS/Linux/Windows）。
3. 调用 `normalize_wallet_address` 校验并标准化地址。
4. 解析目标地址；若与本机 `powr` 矿工账户相同则直接拒绝。
5. 写入 `reward-wallet.json`。
6. 确保安全存储中的 `miner-suri` 可用（必要时创建/迁移）。
7. 校验当前 RPC 目标链指纹（`ss58Format == 2027` 且 `system_name` 非空）。
8. 读取链上 `RewardWalletByMiner[miner]`：
   - 无值：提交 `bind_reward_wallet(wallet)`。
   - 有值且不同：提交 `rebind_reward_wallet(new_wallet)`。
   - 有值且相同：跳过提交。
9. 等待交易 finalized 成功后返回最新地址。

## 6. 安全与边界

- 修改地址必须先通过设备密码验证。
- 签名算法为 `sr25519`，签名账户为本地 `powr` 矿工账户。
- 奖励钱包必须不同于本地 `powr` 矿工账户；若相同则在本地直接拒绝，不提交链上交易。
- 发起链上绑定前必须确认当前共享 RPC 端口属于目标链，避免误发交易。
- 链上读取与交易提交使用 `shared/rpc.rs` 维护的共享 RPC 端口来源，默认 9944，但可随运行时配置/检测结果切换。
- 若地址已保存但链上提交失败，命令返回错误，提示“本地已保存，但链上绑定失败”。
- `home::process::start_node` 会调用本模块同步函数，节点启动后自动补齐链上绑定。
- 节点数据目录通过 `shared/keystore::node_data_dir` 获取，避免重复定义路径逻辑。
- 返回前端的错误消息使用 `security::sanitize_path` 脱敏，仅保留文件名。
- SS58 地址校验和验证使用 blake2b-512（Substrate 标准），链上 storage key 派生使用 blake2b_128（与链上 Blake2_128Concat hasher 对齐）。
