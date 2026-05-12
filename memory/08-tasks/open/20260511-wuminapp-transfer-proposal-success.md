# wuminapp 转账提案成功判定修复

## 任务需求

- 修复 wuminapp 发起治理机构主账户多签转账提案时，把交易哈希返回误判为提案创建成功的问题。
- 成功标准改为：交易进入区块，且该区块 `System.Events` 中存在 `DuoqianTransfer.TransferProposed` 事件。

## 建议模块

- `wuminapp/lib/rpc`：补充等待入块和按块查询事件的能力。
- `wuminapp/lib/transaction/duoqian-transfer`：转账提案提交后按事件确认真实 proposal_id。
- `memory/05-modules/wuminapp`：同步记录成功判定规则。

## 影响范围

- 只影响治理机构主账户多签转账提案的创建成功判定。
- 不修改 runtime 多签转账模块。
- 不修改收费模型。
- 不修改安全基金转账提案和手续费划转提案的业务规则。

## 主要风险点

- 交易已经签名但未入块时，页面等待时间会变长，必须给出真实失败状态。
- 同一区块可能存在多笔转账提案，必须按发起人、机构、收款人和金额匹配事件，不能只取第一个事件。
- 事件解析失败时不能写入本地提案历史。

## 执行状态

- 状态：完成
- 结果：wuminapp 发起普通多签转账提案时，必须等待交易入块并在该区块事件中匹配 `DuoqianTransfer.TransferProposed` 后才返回真实 `proposal_id`。
- 验证：已通过目标 Dart 静态检查；`dart test` 因项目未引入 `package:test` 无法直接运行。
