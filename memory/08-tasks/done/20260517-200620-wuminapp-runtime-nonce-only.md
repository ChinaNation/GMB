# 任务卡：wuminapp 统一改为 runtime nonce 唯一来源

## 任务需求

修复 wuminapp 投票交易因本地预占 nonce 导致 future nonce、交易不入块、runtime 投票不执行的问题。

## 模块归属

- Mobile Agent：`wuminapp`

## 目标状态

- nonce 只以 runtime `frame_system::Account.nonce` 为权威来源。
- wuminapp 不再缓存、自增、预占或回滚 nonce。
- 每次签名前实时读取链上 nonce，读取多少就使用多少。
- 投票类交易提交必须等待入块，再由 runtime 投票 storage 确认业务结果。
- 同一钱包已有待确认投票时，不允许再次提交同一提案投票。

## 预计修改目录

- `wuminapp/lib/rpc/`：清理本地 nonce 管理器，签名构造器直接读取 runtime nonce。
- `wuminapp/lib/votingengine/internal-vote/`：投票统一入口等待交易入块，保留 runtime storage 作为最终确认依据。
- `wuminapp/lib/transaction/duoqian-transfer/`：清理多签转账投票页 nonce reset 残留，修复未入块时的 UI 状态。
- `wuminapp/lib/governance/`：清理个人/机构多签与运行时升级投票相关 nonce 预占残留。
- `memory/05-modules/wuminapp/`：更新技术文档，明确 nonce 唯一来源规则。

## 风险点

- 投票等待入块会让按钮保持提交态更久，需要依赖超时和错误提示避免用户误解。
- 其他非投票交易仍可能引用旧签名入口，需要确认本地 nonce 管理器无残留。
- 旧 pending 数据里已存在 future nonce 记录，状态机必须能清理并允许用户重新提交。

## 验收标准

- `wuminapp/lib` 中无本地 nonce 管理器、预占、自增或回滚业务残留。
- 投票提交链路不再使用本地 nonce 预占。
- `dart analyze lib test` 通过，或明确记录环境阻塞。
- 文档更新 nonce 权威来源与客户端禁止预占规则。

## 执行记录

- 已删除 wuminapp 本地 nonce 管理器，统一签名构造器每次签名前实时读取 runtime nonce。
- 内部投票和协议升级联合投票已改为等待入块并回读投票引擎 storage。
- 新成功投票不再写本地 pending；待确认投票状态机仅清理旧残留，不再用 nonce 推断投票成功。
