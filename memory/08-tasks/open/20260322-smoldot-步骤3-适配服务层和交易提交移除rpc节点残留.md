# 步骤3：适配服务层和交易提交，移除所有 RPC 节点残留

## 状态：已完成

## 背景

步骤1和步骤2完成后，ChainRpc 和 ChainEventSubscription 已切换到 smoldot 轻节点。本步骤确认所有上层服务在新架构下正常工作，执行全流程验证，并清理所有 RPC 节点相关残留代码和配置。

本任务是三步迁移的第三步，依赖步骤1和步骤2完成。

## 所属系列

smoldot 轻节点迁移（3/3）

## 前置依赖

- 步骤1完成：ChainRpc 底层替换为 smoldot
- 步骤2完成：ChainEventSubscription 替换为 smoldot 订阅

## 待完成项

### 1. 服务层验证与适配

逐一确认以下服务通过 smoldot 正常工作：

- [ ] `transfer_proposal_service.dart` — 转账提案查询 + 创建 + 投票
- [ ] `runtime_upgrade_service.dart` — 升级提案查询 + 创建 + 联合投票
- [ ] `institution_admin_service.dart` — 机构管理员列表查询
- [ ] `onchain_trade_service.dart` — 链上转账 + 确认轮询

如果 ChainRpc 接口不变，这些服务理论上无需改代码。但需要实际运行验证。

### 2. 交易提交验证

- [ ] `author_submitExtrinsic` 通过 smoldot 的 P2P 网络广播成功
- [ ] 交易在链上确认（通过 nonce 轮询验证）
- [ ] 交易哈希返回正确

### 3. 全流程端到端测试

- [ ] 余额查询（wallet_page）
- [ ] 链上转账发起 + 确认（onchain_trade_page）
- [ ] 转账提案创建（transfer_proposal_page）
- [ ] 转账提案投票（transfer_proposal_detail_page）
- [ ] 升级提案创建（runtime_upgrade_page）
- [ ] 联合投票（runtime_upgrade_detail_page）
- [ ] 新区块实时推送（all_proposals_view）

### 4. 清理 RPC 节点残留

- 删除 `WUMINAPP_RPC_URL` 环境变量的所有引用
- 删除启动脚本中的 RPC URL dart-define 参数
- 更新 memory 文档中关于 RPC 节点的描述
- 更新 wuminapp 技术架构文档

### 5. 更新文档

- 更新 `memory/05-modules/wuminapp/` 相关技术文档
- 在 `memory/04-adr/` 下创建 ADR：记录从 RPC 迁移到 smoldot 轻节点的决策和原因

## 涉及文件

- `wuminapp/lib/governance/transfer_proposal_service.dart` — 验证（可能不改）
- `wuminapp/lib/governance/runtime_upgrade_service.dart` — 验证（可能不改）
- `wuminapp/lib/governance/institution_admin_service.dart` — 验证（可能不改）
- `wuminapp/lib/trade/onchain/onchain_trade_service.dart` — 验证（可能不改）
- 启动脚本 — 移除 RPC URL 参数
- memory 文档 — 更新

## 不涉及文件

- `wuminapp/lib/wallet/capabilities/api_client.dart` — SFID HTTP 接口完全不受影响

## 风险点

- 交易广播依赖 smoldot 的 P2P 连接质量，如果轻节点连接的全节点较少，广播可能延迟
- 需要确保 citizenchain 的全节点开启了轻客户端协议支持
