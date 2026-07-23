# 奖励账户模块技术文档

## 0. 功能需求

- 页面支持绑定或变更手续费与铸块奖励收款地址，并展示当前已绑定地址。
- 修改前必须校验设备开机密码。
- 页面接收 SS58(2027) 展示地址，后端解析后只以小写
  `0x` 加 64 位十六进制 `account_id` 保存和提交链上。
- 模块为本机维护稳定的 `powr` 矿工签名账户，确保链上绑定主体与实际挖矿账户一致。
- 奖励账户必须不同于本机矿工账户。
- 模块需要在提交链上绑定交易前确认当前本地 RPC 端口确实属于目标链，避免把绑定操作误发到错误网络。
- 当链上绑定失败或超时时，模块需要明确告诉调用方“本地已保存，但链上未完成”。

## 1. 模块位置

- 后端路径：`node/src/settings/reward_account.rs`
- 前端路径：`node/frontend/settings/RewardAccountSection.tsx`
- 前端共享依赖：`node/frontend/shared/ss58.ts` 校验 SS58 地址，
  `node/frontend/shared/qr/AddressScanModal.tsx` 负责扫码填入地址。
- 对外命令：
  - `get_reward_account`
  - `set_reward_account`
- Node 私有 RPC：
  - `reward_bindAccount`
  - `reward_rebindAccount`

## 2. 模块职责

- 管理桌面节点的奖励账户本地配置。
- 从默认链 keystore 读取本机 `powr` 矿工账户，不建立第二套矿工身份真源。
- 将奖励账户变更同步为链上
  `FullnodeIssuance.bind_reward_account/rebind_reward_account` 交易。
- 保证奖励与手续费分成路由到绑定的奖励账户。

## 3. 数据模型

- `RewardAccount`
  - `account_id: Option<String>`：唯一账户标识。
  - `ss58_address: Option<String>`：从账户 ID 派生的展示地址。
- `StoredRewardAccount`
  - `account_id: String`：本地唯一持久化字段，必须符合
    `^0x[0-9a-f]{64}$`。

## 4. 存储设计

- 非密钥配置：`<app_data_dir>/reward-account.json`。
- 矿工签名密钥：仅存储在默认链 keystore 的 `powr` key 文件中。
- 不读取旧文件名、旧 JSON 字段或其他链目录，不提供 fallback。

## 5. 命令流程

### 5.1 `get_reward_account`

1. 读取 `reward-account.json`。
2. 文件不存在时返回空 `account_id` 和空 `ss58_address`。
3. 存在时严格校验 `account_id`，再派生 SS58(2027) 展示地址。

### 5.2 `set_reward_account`

1. 校验设备登录密码非空。
2. 通过 `node/src/settings/device_password.rs` 执行设备密码验证。
3. 严格校验 SS58(2027) 地址并解析为 32 字节账户 ID。
4. 若目标账户与本机 `powr` 矿工账户相同则拒绝。
5. 以小写 `0x` 加 64 位十六进制写入 `reward-account.json`。
6. 校验当前 RPC 链指纹（`ss58Format == 2027` 且 `system_name` 非空）。
7. 读取链上 `RewardAccountIdByMiner[miner]`：
   - 无值：调用 `reward_bindAccount(account_id)`。
   - 有值且不同：调用 `reward_rebindAccount(account_id)`。
   - 有值且相同：跳过提交。
8. 等待交易 finalized 后返回 `RewardAccount`。

## 6. 安全与边界

- 修改地址必须先通过设备密码验证。
- 签名算法为 `sr25519`，签名账户为本地 `powr` 矿工账户。
- 奖励账户必须不同于本地 `powr` 矿工账户；若相同则不提交链上交易。
- `account_id` 是存储、RPC 和授权值；`ss58_address` 仅用于输入和展示。
- 发起链上绑定前必须确认当前共享 RPC 端口属于目标链，避免误发交易。
- 链上读取与交易提交使用 `shared/rpc.rs` 维护的共享 RPC 端口来源，默认 9944，但可随运行时配置/检测结果切换。
- 若地址已保存但链上提交失败，命令返回错误，提示“本地已保存，但链上绑定失败”。
- `home::process::start_node` 会调用本模块同步函数，节点启动后自动补齐链上绑定。
- 矿工身份（`powr` 公钥）仅从默认链（`citizenchain`）的 keystore 目录读取，不遍历其他链目录，避免旧链残留 keystore 导致矿工身份判定错位。
- 节点数据目录通过 `shared/keystore::node_data_dir` 获取，默认链 keystore 路径通过 `shared/keystore::default_chain_keystore_dir` 获取。
- 返回前端的错误消息使用 `security::sanitize_path` 脱敏，仅保留文件名。
- SS58 地址校验和验证使用 blake2b-512（Substrate 标准），链上 storage key 派生使用 blake2b_128（与链上 Blake2_128Concat hasher 对齐）。
