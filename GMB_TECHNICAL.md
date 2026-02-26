# GMB 公民币区块链技术方案（基于当前实现）

## 1. 文档目的
- 基于 `/Users/rhett/GMB/README.md` 的制度与架构目标，结合当前 `citizenchain` 已实现代码，形成统一的技术落地方案。
- 统一“制度规则 -> 模块职责 -> 交易流程 -> 安全控制 -> 运维与验收”的执行口径。

## 2. 系统范围与目标
- 链名称：CitizenChain（GMB）。
- 运行框架：Substrate Runtime + Node。
- 核心目标：
  - 支持多层治理（国储会、省储会、省储行）。
  - 支持 PoW 出块与链上手续费分配。
  - 支持机构多签账户（duoqian）。
  - 支持省储行链下清算批量上链（offchain-transaction-pos）。
  - 支持 SFID 体系对机构与公民身份相关业务授权。

## 3. 总体架构
- 目录与职责：
  - `citizenchain/node`：链配置、网络、出块服务。
  - `citizenchain/runtime`：统一状态机与交易校验逻辑。
  - `citizenchain/governance`：治理提案与升级控制。
  - `citizenchain/issuance`：发行与奖励相关模块。
  - `citizenchain/transaction`：链上交易、链下打包、多签交易。
  - `primitives`：制度常量与中国机构常量集。
- 网络引导：
  - `bootNodes` 已统一内嵌在 `/Users/rhett/GMB/citizenchain/node/src/chain_spec.rs`，避免多处维护。

## 4. 关键实现现状（与代码一致）

### 4.1 Runtime 核心
- 交易扩展（TxExtension）已包含 `CheckNonKeylessSender`，用于禁止 keyless 地址直接发起交易签名。
- 货币最小单位采用“分”，`100 分 = 1 元`。
- ED、区块时间等基础参数从 `primitives` 统一读取。

### 4.2 PoW 出块
- 当前为固定难度 PoW 算法。
- 难度常量已统一在 `primitives::pow_const::POW_INITIAL_DIFFICULTY`。
- `node` 与 `runtime` 的交易扩展项已对齐，`benchmarking` 侧按 12 项扩展构造签名负载。

### 4.3 duoqian-transaction-pow（机构多签）
- 采用“SFID 先登记，后创建多签”模式：
  - `sfid_id` 由授权 SFID 账户登记上链；
  - `duoqian_address` 由链上固定算法从 `sfid_id` 派生（当前文档口径为 Blake3）；
  - 只有已登记机构才能创建多签账户。
- 创建规则：
  - `N >= 2`
  - `M >= ceil(N/2)` 且 `M <= N`
  - 管理员数量与公钥数量一致，且公钥不可重复
  - 创建金额不少于 `1.11 元`
  - 发起者必须为管理员之一，且签名满足阈值
- 支持注销后同地址重建（按当前业务规则）。

### 4.4 offchain-transaction-pos（省储行链下清算上链）
- 交易终态在链下确认时即固定：
  - payer 扣 `amount + fee`
  - 收款方加 `amount`
  - 省储行手续费账户加 `fee`
- 上链打包为独立动作：
  - 使用持久化队列 + 重试直到成功
  - 上链失败不回滚链下已确认终态
- 费率与机构权限由链上配置与校验控制。

## 5. 跨系统信任链方案（CPMS -> SFID -> Blockchain）
- 建议采用三层信任链：
  - 区块链保存 SFID 系统公钥
  - SFID 保存 CPMS 公钥
  - CPMS 离线签发二维码数据
- 推荐二维码载荷字段：
  - `archive_id`
  - `user_pubkey`
  - `biz_type`
  - `issued_at`
  - `expire_at`
  - `nonce`
  - `cpms_signature`
- SFID 验签通过后向链上提交授权交易（绑定认证/投票认证/人数快照等）。
- 链上仅接受 SFID 授权账户签名调用。

## 6. 安全控制基线
- 地址安全：
  - 关键保留地址、fee 地址、keyless 地址进入禁止抢注册集合。
  - keyless 账户禁止被普通签名交易直接转出。
- 交易安全：
  - duoqian 创建与金额/手续费校验原子执行，不满足条件整笔失败。
  - 错误码区分明确（参数、地址、阈值、重复、公钥、金额、手续费、签名、权限等）。
- 链下打包安全：
  - 打包提交者白名单校验；
  - 未授权提交不应改变队列重试状态；
  - 幂等键防重（建议维度：机构 + tx_id/batch_seq）。
- 密钥安全：
  - SFID 与治理根账户采用主备密钥轮换机制；
  - 高权限动作建议多签治理入口。

## 7. 部署与运维方案
- 节点部署：
  - 使用 `chain_spec.rs` 内建 bootNodes；
  - 域名可替换，但 `PeerId` 必须与实际 node key 对应。
- 监控指标：
  - 出块间隔、交易池、链下队列积压、批次重试次数、失败原因分布。
- 审计要求：
  - duoqian 创建/注销全事件落链；
  - 链下确认与上链摘要可双向追溯；
  - 关键治理操作保留 `code_hash` 与提案摘要。

## 8. 测试与验收
- 单元测试：
  - duoqian 规则边界（N/M、公钥重复、金额下限、权限签名）。
  - offchain 打包授权与幂等。
- 集成测试：
  - SFID 登记 -> duoqian 创建全流程。
  - 链下确认 -> 批次重试 -> 上链成功回写。
  - Runtime 升级后交易扩展兼容性检查。
- 回归测试：
  - `node`、`runtime`、`transaction` 三层编译与测试全绿。

## 9. 实施里程碑（建议）
1. 里程碑 A：稳定主链参数与保留地址规则（冻结常量接口）。
2. 里程碑 B：完成 CPMS->SFID->链上授权闭环与二维码防重放。
3. 里程碑 C：完成链下清算服务与持久化重试生产化。
4. 里程碑 D：完成治理升级流程硬化与审计报表。
5. 里程碑 E：压测与安全审计，形成上线基线版本。

## 10. 方案结论
- 当前代码基础已覆盖核心制度能力：治理、多签、PoW、链下打包上链。
- 上线前关键工作聚焦在三点：
  - 跨系统信任链的防重放与密钥轮换机制固化；
  - 链下清算持久化重试与监控告警完善；
  - 治理与运行时升级流程的最小权限化和审计闭环。

