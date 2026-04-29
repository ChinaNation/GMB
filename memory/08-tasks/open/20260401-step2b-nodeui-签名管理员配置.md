# 第2步-B：nodeui 签名管理员配置 + node 链下清算启用

## 状态：open

## 前置依赖

- 第2步-A offchain pallet 简化密钥机制完成

## 任务目标

在节点 UI（nodeui）中实现签名管理员配置功能，用户在导入冷钱包时如果导入了省储行管理员公钥，可以将其设为签名管理员，节点自动启用链下清算功能。

## 设计

### nodeui 流程

1. 用户在 nodeui 导入冷钱包 → 导入管理员公钥
2. 系统检测该公钥是否是 CHINA_CH 中某省储行的 9 个 duoqian_admins 之一
3. 是 → 显示"设为签名管理员"按钮
4. 点击 → 弹窗输入该管理员对应的私钥
5. 私钥用用户设置的密码 AES-256 加密后写入本地存储（明文不落盘）
6. 成功后该管理员后面显示"签名账户"标签
7. 如需更换 → 导入另一个管理员 → 同样流程 → 最新写入的替换旧的

### node 启动流程

1. 节点启动 → 检查本地是否有加密的签名管理员私钥
2. 有 → nodeui 弹窗要求输入密码解锁
3. 解密后私钥保持在内存中（不落盘）
4. 启用链下清算 RPC（offchain_submitSignedTx 等）
5. 运行期间内存中的私钥用于自动签署 batch
6. 节点关闭 → 内存清零

### 私钥安全保障

- 磁盘上始终是 AES-256 加密状态
- 明文私钥仅在内存中存在
- 签名完成后尽快清零内存中的临时副本
- 每次节点启动需要密码解锁

## 改动范围

### nodeui

- 导入冷钱包时检测管理员身份
- 新增"设为签名管理员"按钮 + 私钥输入弹窗
- 管理员列表显示"签名账户"标签
- 节点启动时密码解锁弹窗

### node

- 新增加密私钥本地存储模块
- 启动时检测签名管理员 → 解锁 → 启用链下 RPC
- 链下 RPC：offchain_submitSignedTx、offchain_queryTxStatus
- 链下待结算账本（内存）
- 批量打包触发器（笔数 10 万 或 时间 10 个区块）

### 涉及文件

- `citizenchain/nodeui/` — 签名管理员 UI
- `citizenchain/node/src/rpc.rs` — 新增链下 RPC
- `citizenchain/node/src/service.rs` — 依赖注入
- `citizenchain/node/src/offchain/ledger.rs` — 清算行本地账本
- `citizenchain/node/src/offchain/settlement/packer.rs` — 批量打包器
- `citizenchain/node/src/offchain/keystore.rs` — 加密私钥存储

## 验收标准

- [ ] nodeui 导入管理员公钥后可设为签名管理员
- [ ] 私钥 AES-256 加密存储，明文不落盘
- [ ] 节点启动时密码解锁后启用链下 RPC
- [ ] wuminapp 可通过 WSS 提交签名交易到省储行节点
- [ ] 省储行节点自动批量打包上链
- [ ] 更换签名管理员功能正常
