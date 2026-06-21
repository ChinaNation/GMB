# node duoqian-transfer 技术说明

## 模块边界

node 端多签转账拆为独立目录：

- `citizenchain/node/src/duoqian_transfer/`：Tauri 命令、签名构造、call_data、AccountId 编码、提案详情解码和独立 storage 查询。
- `citizenchain/node/frontend/duoqian-transfer/`：多签转账、安全基金转账、手续费划转的创建页面、详情展示组件、API 和类型。

`governance` 不再注册 `build_propose_transfer_request / submit_propose_transfer`、安全基金转账、手续费划转等 pallet=19 创建/提交命令；前端也不再在 `governance/api.ts` 中暴露这些 API。

`governance/proposal` 只负责提案通用聚合，可调用 `duoqian_transfer::proposal` 的解码结果并通过 JSON flatten 保持前端兼容；不得在 governance 目录重新定义多签转账详情结构、SCALE 解码或 `DuoqianTransfer::*ProposalActions` 查询。

多签转账、安全基金转账、手续费划转提案详情中的金额字段按 finalized storage 读取；best/latest 只允许用于交易进度和链状态提示，不作为金额展示口径。

`offchain` 不再声明 `duoqian_transfer` 模块，清算行目录只负责清算行功能。

## 支持范围

node 端只支持：

- `0x01 Builtin`：治理机构主体。
- `InstitutionAccount AccountId`：注册机构多签账户主体。

node 端明确拒绝：

- `PersonalAccount AccountId`：个人多签转账，该能力只在 citizenapp 端实现。

## QR 字段

`propose_transfer` 冷钱包展示字段统一为：

```json
{
  "action": "propose_transfer",
  "fields": ["institution", "beneficiary", "amount_yuan", "remark"]
}
```

禁止恢复旧 `org` 展示字段。
