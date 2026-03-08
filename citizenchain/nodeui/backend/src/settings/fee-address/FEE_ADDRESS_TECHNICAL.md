# Fee Address 模块技术文档

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
  - 以 `settings/security.rs` 的 AES-GCM 密文封装保存。
  - 首次缺失时，尝试从本地链 keystore 的 `powr` key 文件提取；仍不存在则生成随机 `0x<32bytes>` seed 并写入安全存储。
  - 旧版明文文件 `<app_data_dir>/miner-suri.txt` 仅用于一次性迁移，迁移后立即删除。

## 5. 命令流程

### 5.1 `get_reward_wallet`

1. 解析 `reward-wallet.json` 路径。
2. 文件不存在返回 `address = null`。
3. 存在则反序列化后返回地址。

### 5.2 `set_reward_wallet`

1. 校验设备登录密码非空。
2. 通过 `settings/security.rs` 执行设备密码验证（macOS/Linux/Windows）。
3. 调用 `normalize_wallet_address` 校验并标准化地址。
4. 写入 `reward-wallet.json`。
5. 确保安全存储中的 `miner-suri` 可用（必要时创建/迁移）。
6. 读取链上 `RewardWalletByMiner[miner]`：
   - 无值：提交 `bind_reward_wallet(wallet)`。
   - 有值且不同：提交 `rebind_reward_wallet(new_wallet)`。
   - 有值且相同：跳过提交。
7. 等待交易 finalized 成功后返回最新地址。

## 6. 安全与边界

- 修改地址必须先通过设备密码验证。
- 签名算法为 `sr25519`，签名账户为本地 `powr` 矿工账户。
- 若地址已保存但链上提交失败，命令返回错误，提示“本地已保存，但链上绑定失败”。
- `home::home_node::start_node` 会调用本模块同步函数，节点启动后自动补齐链上绑定。
