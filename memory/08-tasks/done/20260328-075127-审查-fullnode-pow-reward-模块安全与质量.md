# 任务卡：全面仔细的检查一遍 fullnode-pow-reward 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

- 任务编号：20260328-075127
- 状态：done
- 所属模块：citizenchain/issuance
- 当前负责人：Codex
- 创建时间：2026-03-28 07:51:27

## 任务需求

全面仔细的检查一遍 fullnode-pow-reward 模块有没有安全漏洞、有没有需要改进的地方、功能需求是否严格实现、中文注释技术文档是否完整、有没有要清理的残留

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
- citizenchain/runtime/README.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- 必要时联动 `primitives`

### 先沟通条件

- 修改 runtime 存储结构
- 修改资格模型
- 修改提案、投票、发行核心规则


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`、`nodeui` 或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 冷钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 冷钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
- 文档与残留必须一起收口

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenchain.md

# CitizenChain 完成标准

- 改动范围和所属模块清晰
- 关键逻辑已补中文注释
- 文档已同步更新
- 影响链规则、存储或发布行为的点都已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已读取启动协议要求文档、模块代码、weights、benchmarks、runtime 配置、node RPC、nodeui 绑定与挖矿面板代码。
- 已执行验证：
  - `cargo test -p fullnode-pow-reward`
  - `cargo test -p onchain-transaction-pow`
  - `cargo check -p node`
  - `cargo check -p nodeui-desktop-shell`

## 审查结论

### 主要发现

1. 协议层作者身份缺少强约束，`fullnode-pow-reward` 当前把 PoW pre-runtime digest 中的 `AccountId` 直接当作矿工身份使用，但 `citizenchain/node/src/service.rs` 中的 `SimplePow::verify` 只校验该字段“能解码为账户”，并未校验它与实际出块实体存在密码学绑定；`citizenchain/runtime/src/configs/mod.rs` 中 `PowDigestAuthor` 也只是直接解码该 digest。结果是：奖励与手续费分账依赖“诚实 node 会把本地 keystore 公钥写进 digest”这一实现约定，而不是链上协议保证，制度上要求的“矿工身份账户 / 奖励钱包分离”可以被定制矿工程序绕过。
2. `nodeui/backend/src/settings/fee-address/mod.rs` 的 `set_reward_wallet` 没有在本地保存前拒绝“奖励钱包等于矿工账户”，而是先写入 `reward-wallet.json` 再异步调用 `sync_saved_reward_wallet_inner`；同时该命令会先返回成功，再通过事件异步回传链上绑定失败或超时。这与技术文档中“本地直接拒绝矿工自身账户”“链上绑定失败时命令返回错误”的功能需求不一致，也会留下一个本地已保存但链上永远无法绑定的错误配置。
3. `nodeui/backend/src/settings/fee-address/mod.rs` 中 `local_powr_miner_account_hex` 会扫描 `node-data/chains/*/keystore` 下所有 `powr` 文件并返回排序后的第一个；但 node 侧 `reward_bindWallet/reward_rebindWallet` 实际使用的是当前链 keystore 中的第一个 `powr` 密钥。若本机存在旧链目录或陈旧 `powr` 文件，nodeui 查询绑定状态、拒绝自绑定校验、挖矿收益归属判断都可能与 node 的真实签名矿工身份不一致。

### 需求实现与文档判断

- pallet 本体的核心制度路径基本落地：固定奖励常量、起止高度、`on_finalize` 结算、未绑定回退到矿工自身账户、`bind/rebind` 基本约束、事件审计和基础测试都已实现。
- 中文注释在 pallet、runtime 配置、node RPC 关键路径上基本完整，没有看到明显调试残留进入正式逻辑。
- 技术文档不算完整：
  - `memory/05-modules/citizenchain/runtime/issuance/fullnode-pow-reward/FULLNODE_TECHNICAL.md` 第 10 节把“出块身份与绑定身份一致”描述成既成事实，但现状更接近 node 约定而非协议保证。
  - `memory/05-modules/citizenchain/nodeui/settings/fee-address/FEE_ADDRESS_TECHNICAL.md` 仍描述了“失败直接返回错误”“确保 miner-suri/清理旧 powr key 文件”等旧口径，与当前实现不完全一致。
- 权重侧还存在工程改进空间：`bind/rebind` 已 benchmark，`on_finalize` 仍依赖 `on_initialize` 手工预申报 `reads_writes(3,3)`，当前未看到专门的最坏路径计重验证。

### 残留判断

- 未发现明显临时调试逻辑、废弃分支或无用事件残留。
- 但 nodeui 奖励钱包链路存在“旧密钥目录/陈旧本地配置”带来的状态残留风险，属于后续应收口的问题。

## 完成信息

- 完成时间：2026-03-28 07:56:33
- 完成摘要：完成 fullnode-pow-reward 审查；确认 pallet 核心功能与测试基本成立，但发现 3 个需要后续处理的问题：PoW 作者身份仅靠 digest 约定、nodeui 奖励钱包保存/失败反馈不符合文档口径、旧 powr 密钥可能导致 nodeui 与 node 身份错位。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
