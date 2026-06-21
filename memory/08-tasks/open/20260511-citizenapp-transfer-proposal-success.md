# citizenapp 转账提案成功判定修复

## 任务需求

- 修复 citizenapp 发起治理机构主账户多签转账提案时，把交易哈希返回误判为提案创建成功的问题。
- 成功标准改为：交易进入区块，且该区块 `System.Events` 中存在 `DuoqianTransfer.TransferProposed` 事件。

## 建议模块

- `citizenapp/lib/rpc`：补充等待入块和按块查询事件的能力。
- `citizenapp/lib/transaction/duoqian-transfer`：转账提案提交后按事件确认真实 proposal_id。
- `memory/05-modules/citizenapp`：同步记录成功判定规则。

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
- 结果：citizenapp 发起普通多签转账提案时，必须等待交易入块并在该区块事件中匹配 `DuoqianTransfer.TransferProposed` 后才返回真实 `proposal_id`。
- 验证：已通过目标 Dart 静态检查；`dart test` 因项目未引入 `package:test` 无法直接运行。

## 2026-05-15 追加修复：主账户 hex 误按 SS58 解码

- 问题：普通多签转账提案提交前，`DuoqianTransferService.submitProposeTransfer` 把 `InstitutionInfo.mainAccount` 当成 SS58 地址传给 `Keyring().decodeAddress`，导致 64 位账户 hex 中的字符 `0` 被 Base58 校验拒绝。
- 修复：`mainAccount` 按 32 字节 AccountId hex 严格解码；用户输入的收款地址继续按 SS58 解码。
- 同类边界更正：普通链上转账从通讯录选择联系人、联系人详情页发起转账时，通讯录联系人地址属于用户展示 / 输入边界，直接保存并传递 SS58，不得再按内部 AccountId hex 转换。
- 残留清理：离线清算支付页 `toAddress` 删除 `0x` hex 双格式入口，只保留 SS58 扫码/展示边界。
- 文档：已在 `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md` 写明账户格式边界：runtime 为 `AccountId32`，App 内部为无 `0x` hex，用户展示/输入为 SS58。
- 残留检查：已扫描 `decodeAddress(...)` 调用；剩余调用均为 SS58 用户输入、SS58 钱包地址或明确的 SS58 本地字段。
- 验证：`dart analyze lib test`、`flutter test --concurrency=1`、`git diff --check` 均通过。
