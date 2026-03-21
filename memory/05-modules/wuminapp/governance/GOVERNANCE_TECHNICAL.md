# Governance 治理模块技术文档（区块链规范版）

## 1. 模块目标

`lib/governance/` 负责 WuminApp 的链上治理能力规范，覆盖：

- 提案（proposal）发起
- 投票（vote）提交
- 提案状态跟踪与结果展示

说明：

- 本文档定义的是“链上字段/格式/标准/流程”。
- 当前 App 已接入 runtime 升级、转账等主要治理路径，本文同时作为现有实现与后续扩展的对齐基线。

## 2. 链上入口与权限边界

### 2.1 关键约束（必须遵守）

- `voting-engine-system` 的 `create_internal_proposal`、`create_joint_proposal` 和 `internal_vote` 外部调用被禁用，直接调用会返回 `NoPermission`。
- 联合提案必须由业务治理 pallet 通过 `JointVoteEngine` trait 发起。
- 内部投票必须由业务治理 pallet 通过 `InternalVoteEngine` trait 转发。

### 2.2 可直接由交易发起的投票引擎入口

- `joint_vote(proposal_id, institution, approve)`
- `citizen_vote(proposal_id, binding_id, nonce, signature, approve)`

## 3. 通用字段与格式标准

### 3.1 基础类型

| 字段 | 链上类型 | App 传输规范 |
| --- | --- | --- |
| `account` | `AccountId32` | SS58 地址字符串（当前链 `ss58 = 2027`） |
| `institution` | `[u8; 48]` | `0x` + 96 hex（机构 pallet id） |
| `proposal_id` | `u64` | 年份编码：`年份 × 1,000,000 + 年内计数器`，如 `2026000001`，App 显示为 `2026#1` |
| `approve` | `bool` | `true/false` |
| `nonce` | `BoundedVec<u8, 64>` | `0x` hex，解码后字节长度 `1..64` |
| `signature` | `BoundedVec<u8, 64>` | `0x` hex，解码后字节长度 `1..64` |
| `binding_id` | `Hash` | `0x` + 64 hex |

### 3.2 枚举与编码

- `org`：`0 = NRC(国储会)`，`1 = PRC(省储会)`，`2 = PRB(省储行)`，`3 = DUOQIAN(注册多签机构)`。
- proposal kind：`0 = internal`，`1 = joint`。
- stage：`0 = internal`，`1 = joint`，`2 = citizen`。
- status：`0 = voting`，`1 = passed`，`2 = rejected`。

### 3.3 时效与阈值

- 单阶段投票时长：`VOTING_DURATION_BLOCKS`（当前为 30 天对应区块数）。
- 内部投票通过阈值：
  - NRC：`13`（硬编码）
  - PRC：`6`（硬编码）
  - PRB：`6`（硬编码）
  - DUOQIAN：`用户注册时设定的 threshold`（链上 `DuoqianAccounts.threshold` 动态读取）
- 联合投票权重：
  - NRC：`19`
  - 每个 PRC：`1`
  - 每个 PRB：`1`
  - 总票权：`105`
- 联合机构内部管理员阈值：
  - NRC：`13`
  - PRC：`6`
  - PRB：`6`
- 联合投票阶段中，管理员直接上链投票：
  - 某机构赞成票达到该机构阈值时，链上自动形成该机构 `yes`
  - 若该机构剩余管理员已不足以让赞成票达到阈值，链上自动形成该机构 `no`
- 联合投票 `yes >= 105` 立即通过；任一机构形成 `no` 时立即转入公民投票；否则超时后进入公民投票阶段。
- 公民投票通过规则：`yes * 100 > eligible_total * 50`（严格大于 50%）。

## 4. 提案字段规范（按业务类型）

| 业务类型 | 提案入口 | 必填字段 | 发起权限 | 投票入口 |
| --- | --- | --- | --- | --- |
| 决议发行 | `propose_resolution_issuance` | `reason, total_amount, allocations[], eligible_total, snapshot_nonce, signature` | 国储会管理员（NRC） | 联合+公民 |
| Runtime 升级 | `propose_runtime_upgrade` | `reason, code, eligible_total, snapshot_nonce, signature` | 国储会管理员（NRC） | 联合+公民 |
| 管理员更换 | `propose_admin_replacement` | `org, institution, old_admin, new_admin` | 目标机构管理员 | 内部 |
| 决议销毁 | `propose_destroy` | `org, institution, amount` | 目标机构管理员 | 内部 |
| GRANDPA 密钥更换 | `propose_replace_grandpa_key` | `institution, new_key(32B)` | NRC/PRC 机构管理员 | 内部 |
| 省储行业务治理 | `propose_institution_rate / propose_verify_key / propose_sweep_to_main / propose_relay_submitters` | 见 4.4 | PRB 机构管理员 | 内部 |

### 4.1 联合提案额外字段标准（决议发行 / Runtime 升级）

- `eligible_total`：`u64`，必须 `> 0`。
- `snapshot_nonce`：`1..64` 字节。
- `signature`：`1..64` 字节，运行时当前要求 64 字节 `sr25519` 原始签名。

人口快照验签消息标准（runtime）：

```text
payload = (
  "GMB_SFID_POPULATION_V3",
  genesis_hash,
  who,
  eligible_total,
  snapshot_nonce
)
message = blake2_256(SCALE.encode(payload))
```

### 4.2 决议发行 allocations 约束

- `allocations` 不能为空。
- 每个 `recipient` 必须唯一，`amount > 0`。
- 分配接收者集合必须与链上 `AllowedRecipients` 完整一致（不能缺项、不能多项）。
- `sum(allocations.amount) == total_amount`。

### 4.3 GRANDPA 密钥更换约束

- `new_key` 不能全 0。
- `new_key` 必须是合法 Ed25519 压缩公钥（32 字节）。
- `new_key` 不得与当前 key 相同，不得与其他机构正在使用 key 冲突。
- 同一 `new_key` 不得被并发提案占用。

### 4.4 省储行业务治理约束（offchain-transaction-pos）

- `propose_institution_rate`：
  - `new_rate_bp` 范围 `1..10`（0.01%~0.1%）。
- `propose_verify_key`：
  - `new_key` 非空，长度不超过 `MaxVerifyKeyLen(当前 256)`。
- `propose_sweep_to_main`：
  - `amount > 0`；
  - 执行时还要满足保底与上限规则（保留费地址最低余额、单次最多提可用余额 80%）。
- `propose_relay_submitters`：
  - `submitters` 数量 `1..MaxRelaySubmitters(当前 8)`；
  - 账户不得重复。

## 5. 投票字段规范

### 5.1 内部投票（业务 pallet）

内部投票业务入口统一字段：

- `proposal_id: u64`
- `approve: bool`

典型函数：

- `vote_admin_replacement`
- `vote_destroy`
- `vote_replace_grandpa_key`
- `vote_institution_rate`
- `vote_verify_key`
- `vote_sweep_to_main`
- `vote_relay_submitters`

### 5.2 联合机构投票（投票引擎）

`joint_vote` 字段：

- `proposal_id: u64`
- `institution: [u8;48]`
- `approve: bool`

权限要求：

- 必须由“当前机构管理员个人钱包”直接提交，不能跨机构代投。
- 同一管理员对同一 `proposal_id + institution` 只能投一次。
- 链上按机构当前管理员门限自动结算机构结果，不再需要额外 `approvals proof` 或机构多签提交。

### 5.3 公民投票（投票引擎）

`citizen_vote` 字段：

- `proposal_id: u64`
- `binding_id: Hash`
- `nonce: BoundedVec<u8,64>`
- `signature: BoundedVec<u8,64>`
- `approve: bool`

运行时投票凭证验签消息标准：

```text
payload = (
  "GMB_SFID_VOTE_V3",
  genesis_hash,
  account,
  binding_id,
  proposal_id,
  nonce
)
message = blake2_256(SCALE.encode(payload))
```

防重放要求：

- 同一 `proposal_id + binding_id` 只能投一次。
- 同一 `proposal_id + binding_id + nonce` 不能重放。

## 6. 标准流程

### 6.1 提案发起流程（App 侧）

1. 选择业务类型并收集业务字段。
2. 校验当前钱包是否具备该机构管理员权限。
3. 若为联合提案，先获取 `eligible_total + snapshot_nonce + signature`。
4. 组装链上调用字段并签名提交。
5. 记录 `proposal_id` 与业务类型映射，订阅状态事件。

### 6.2 投票流程（App 侧）

1. 根据提案类型匹配投票入口（内部/联合/公民）。
2. 采集投票字段并做本地格式校验；联合投票与内部投票一样，直接由管理员个人钱包上链投票。
3. 发起签名并提交交易。
4. 监听事件刷新状态：
  - `InternalVoteCast / JointAdminVoteCast / JointInstitutionVoteFinalized / CitizenVoteCast`
  - `ProposalAdvancedToCitizen`
  - `ProposalFinalized`

#### 6.2.1 Runtime 升级提案在 App 里的联合投票实现

- `RuntimeUpgradeDetailPage` 从机构页进入时必须带上：
  - `institution`
  - `adminWallets`
- 页面会先按链上 `AdminsOriginGov.CurrentAdmins` 过滤当前仍有效的管理员钱包。
- 联合投票按钮只在以下条件全部满足时启用：
  - 提案仍处于 `joint` 阶段且状态为 `voting`
  - 当前机构尚未投票
- 当前用户已导入至少一个仍未投票的本机构管理员钱包
- App 直接使用所选管理员钱包提交 `joint_vote(proposal_id, institution, approve)`。
- 页面会读取：
  - `JointInstitutionTallies` 展示本机构当前赞成/反对管理员票数
  - `JointVotesByInstitution` 展示本机构是否已经形成最终机构结果
  - `JointVotesByAdmin` 判断当前导入管理员钱包是否已投票
- 页面展示的联合投票阈值不再写死 `3`，而是显示链上的联合权重阈值 `105`。
- 页面还会单独展示“本机构管理员投票进度 / 本机构阈值”，避免把联合权重阈值和机构内部门限混淆。

### 6.3 超时与补偿

- 投票引擎在 `on_initialize` 自动做到期结算（支持分桶分批）。
- 业务 pallet 允许在部分场景手动执行或重试（如执行失败补偿）。
- App 端必须支持”最终状态后停止轮询”，避免重复提交。

### 6.4 统一数据存储与自动清理

投票引擎统一存储所有提案数据（投票数据 + 业务数据），统一清理。业务模块不存储任何提案数据，不实现任何清理逻辑。

**统一存储：**

| Storage | 说明 | 写入方 |
| --- | --- | --- |
| `Proposals` | 提案基本信息（状态、起止区块） | 投票引擎 |
| `ProposalData` | 提案摘要层（序列化的 BoundedVec\<u8\>，默认上限 100KB） | 业务模块通过 `store_proposal_data()` |
| `ProposalObjectMeta` / `ProposalObject` | 提案对象层（大对象，例如 runtime wasm，默认上限 10MB） | 业务模块通过 `store_proposal_object()` |
| `ProposalMeta` | 辅助元数据（创建时间、通过时间） | 业务模块通过 `store_proposal_meta()` |
| `InternalTallies` / `JointInstitutionTallies` / `JointTallies` / `CitizenTallies` | 投票计数 | 投票引擎 |
| `InternalVotesByAccount` / `JointVotesByAdmin` / `JointVotesByInstitution` / `CitizenVotesByBindingId` | 投票记录 | 投票引擎 |
| `ActiveProposalsByInstitution` | 每机构活跃提案列表（上限 10） | 投票引擎 |

**自动清理策略（统一清理路径）：**
- 提案完成（通过/拒绝/过期）时注册延迟清理：`schedule_cleanup(proposal_id, current_block)`
- 清理时间 = 完成时区块 + **90 天**区块数
- 如果目标区块的队列已满（50 个），自动顺延到下一个区块，保证不丢失
- 每区块 `on_initialize` 检查 `CleanupQueue[当前区块]`，到期后触发清理
- 每区块最多触发 **5 个**提案进入清理流程，未处理完的保留在队列中，下个区块继续
- 实际数据删除委托给 `PendingProposalCleanups` 分块状态机，保证大量投票记录（如公民投票上万条）能分多个区块完成
- 清理状态机阶段：`InternalVotes → JointAdminVotes → JointInstitutionVotes → JointInstitutionTallies → CitizenVotes → VoteCredentials → ProposalObject → FinalCleanup`
- 提案结束（通过/拒绝/过期）时，活跃提案名额在 `set_status_and_emit` 中**立即释放**，不依赖业务模块
- `on_initialize` weight 使用预估最大值（`cleanup_limit` 次读写），确保不超出声明的 weight
- `UsedPopulationSnapshotNonce`（联合提案防重放）不清理（联合提案极少，累计存储量可忽略）

**清理范围（全部）：**

| Storage | 说明 |
| --- | --- |
| `Proposals` | 提案基本信息 |
| `ProposalData` | 业务摘要（转账/销毁/换管理员/runtime 升级摘要等所有类型） |
| `ProposalObjectMeta` / `ProposalObject` | 业务大对象（如 runtime wasm） |
| `ProposalMeta` | 辅助元数据 |
| `InternalTallies` / `JointInstitutionTallies` / `JointTallies` / `CitizenTallies` | 投票计数 |
| `InternalVotesByAccount` / `JointVotesByAdmin` / `JointVotesByInstitution` / `CitizenVotesByBindingId` | 投票记录 |
| `PendingProposalCleanups` | 分块清理游标 |
| `ActiveProposalsByInstitution`（兜底移除） | 活跃提案列表 |

**查询时效：**
- 90 天内：可查完整投票细节和业务详情
- 90 天后：仅可通过区块中的交易记录和事件查询
- 永久：区块中的交易记录和事件不受影响

**业务模块改造：**
- 所有模块的 `ProposalActions`、`ProposalCreatedAt`、`ProposalPassedAt`、`ActiveProposalByInstitution` 等 Storage 已删除
- 所有 `cancel_stale_*` extrinsic 已删除
- 所有 `cleanup_inactive_proposal` 函数已删除
- `resolution-issuance-gov` 和 `runtime-root-upgrade` 的独立 ID 体系（`NextProposalId`、`GovToJointVote`、`JointVoteToGov`）已删除，直接使用投票引擎 proposal_id

关键文件：
- `voting-engine-system/src/proposal_cleanup.rs`（清理逻辑）
- `voting-engine-system/src/active_proposal_limit.rs`（活跃提案限制）
- `voting-engine-system/src/lib.rs`（ProposalData/ProposalObject/ProposalMeta/CleanupQueue Storage + 公共接口）

### 6.4.1 Runtime 升级提案的摘要 / 对象分层

- `runtime-root-upgrade` 现在只把 `proposer + reason + code_hash + has_code + status` 编进 `ProposalData`
- 原始 wasm 不再塞进摘要层，而是统一进入 `ProposalObject(kind=runtime_wasm)`
- App 列表与详情页默认只读取摘要层，不主动拉取大对象
- 详情页里的 `hasCode` 含义已调整为“链上对象层 wasm 是否仍保留”，因此：
  - 投票中且 `hasCode=true`：显示“待执行”
  - 终态且 `hasCode=true`：显示“已归档”
  - `hasCode=false`：显示“已清理”

## 7. App 侧管理员权限检测与机构详情

### 7.1 管理员身份检测流程

1. 用户打开机构详情页，App 并行加载管理员列表和当前钱包信息。
2. 通过 `state_getStorage` 查询链上 `AdminsOriginGov.CurrentAdmins(institution_id)` 存储。
3. Storage key 格式：`twox_128("AdminsOriginGov") + twox_128("CurrentAdmins") + blake2_128(institution_48bytes) + institution_48bytes`。
4. 返回 SCALE 编码的 `BoundedVec<AccountId32, MaxAdminsPerInstitution>`（Compact 长度前缀 + N×32 字节公钥）。
5. 比对当前钱包 `pubkeyHex`（去 0x 前缀、小写）是否在列表中，确定管理员身份。
6. 查询结果内存缓存，下拉刷新时清除缓存重新查询。

### 7.2 机构详情页结构

机构详情页（`InstitutionDetailPage`）自上而下包含以下区域：

1. **顶部机构卡片**：左侧机构图标 + 中间机构类型标签与管理员/阈值信息。
   - 管理员用户：卡片可点击，显示右箭头，点击进入提案类型页面。
   - 非管理员用户：卡片不可点击，不显示右箭头。
2. **管理员身份标识**（仅管理员可见）：绿色提示条"你是本机构管理员，点击上方卡片可发起提案"。
3. **管理员列表入口**：所有用户可见，点击进入管理员列表页。
4. **投票事件列表**：所有用户可见，显示“本机构内部提案 + 所有机构都可见的联合投票提案”，按 ID 倒序展示。Runtime 升级等联合投票提案必须在所有机构入口可见，不能只挂在国储会单一列表下。

### 7.2.1 全局提案列表（投票 tab）

公民页面的"投票"tab 展示全链所有提案（不分机构），按提案 ID 倒序（新的在上）。

**四层优化架构**：

| 层 | 说明 | 文件 |
| --- | --- | --- |
| WebSocket 订阅 | `chain_subscribeNewHeads` 监听新区块，自动检测新提案插入列表顶部 | `lib/rpc/chain_event_subscription.dart` |
| 本地内存缓存 | ProposalMeta / TransferProposalInfo / RuntimeUpgradeProposalInfo 缓存，避免重复 RPC | `lib/governance/proposal_cache.dart` |
| 批量查询 | `state_queryStorageAt` 一次 RPC 查多个 key，减少网络往返 | `chain_rpc.dart::fetchStorageBatch` |
| 分页加载 | 首屏 10 个，ScrollController 滚动触底加载更多 | `all_proposals_view.dart` |

**数据流**：首屏 → 分页取最新 10 个 ID → 缓存命中直接显示，未命中批量查 → 存缓存 → WebSocket 后台监听新区块 → 有新提案自动插入顶部。

**提案类型识别**：
- 内部提案：按转账等内部提案数据结构解码。
- 联合提案：按 `meta.kind == 1` 单独走联合提案解码链路，Runtime 升级提案进入 `runtime_upgrade_detail_page.dart`。
- 未接入专用详情页的联合提案，列表仍需可见，至少保留通用联合提案卡片。

**投票权判断**：
- 遍历用户所有钱包 → 比对每个提案所属机构的管理员列表
- 是管理员且未投票且状态=投票中 → 提案卡片显示红点
- 进入详情页后：有投票权显示投票按钮，无投票权不显示

**红点通知**：
- 提案卡片右上角：该提案需要用户投票 → 红点
- 底部"公民"tab 图标：汇总待投票数 → 带数字红点

**投票权类型（分阶段实现）**：

| 类型 | 判断依据 | 状态 |
| --- | --- | --- |
| 管理员投票 | 钱包是 NRC/PRC/PRB 管理员 | ✅ 已实现 |
| 公民投票 | 钱包绑定了 SFID | ⏭️ 后期 |
| 机构联合投票 | 钱包是当前机构管理员，且该管理员尚未对本机构投票 | ✅ 已实现 |

关键文件：`lib/governance/all_proposals_view.dart`

### 7.3 权限控制规则

| 用户身份 | 可访问页面/功能 |
| --- | --- |
| 管理员 | 机构详情页、管理员列表、投票事件列表、提案类型页面（发起提案） |
| 非管理员 | 机构详情页、管理员列表、投票事件列表 |

核心原则：**只有管理员才能进入提案类型页面发起提案**，非管理员用户只能查看机构信息、管理员列表和投票事件。

### 7.3.1 活跃提案数量限制

每个机构（`InstitutionPalletId`）同时最多允许 **10 个活跃提案**，不区分提案类型（转账、销毁、换管理员等），由投票引擎（`voting-engine-system::active_proposal_limit`）统一管控。

- 创建提案时：`try_add_active_proposal()` 检查并添加
- 提案完成时：`remove_active_proposal()` 在 `set_status_and_emit` 中立即释放（提案通过/拒绝/过期时）
- App 端发起提案前异步查询活跃数，达上限弹窗提示"提案数量已达上限"

关键文件：`voting-engine-system/src/active_proposal_limit.rs`

### 7.4 提案类型页面

提案类型页面（`ProposalTypesPage`）根据机构类型条件展示可发起的提案：

**通用提案（所有机构类型）：**
- 转账：从机构多签账户发起转账
- 换管理员：提议更换本机构管理员
- 决议销毁：提议销毁机构持有的资产

**国储会专属提案（仅 NRC）：**
- 决议发行：发起公民币发行决议，需联合投票+公民投票
- 验证密钥：更换 GRANDPA 共识验证密钥
- 状态升级：Runtime 升级，需联合投票+公民投票

提交成功后，提案类型页应把创建结果向上冒泡到机构详情页，触发列表刷新，避免“链上已创建但机构页仍停留旧状态”。

### 7.5 转账提案功能（已实现）

#### 7.5.1 链上模块

`duoqian-transfer-pow`（pallet_index=19）提供 2 个 extrinsic：

| Extrinsic | call_index | 说明 |
| --- | --- | --- |
| `propose_transfer(org, institution, beneficiary, amount, remark)` | 0 | 管理员发起转账提案 |
| `vote_transfer(proposal_id, approve)` | 1 | 管理员投票，达到阈值自动执行 |

投票通过后自动执行 `Currency::transfer(duoqian_address → beneficiary)`。
执行失败不回滚投票，发出 `TransferExecutionFailed` 事件，清理提案，管理员需重新发起。

#### 7.5.1.1 手续费模型

**提案提交和投票均免费**（`CallAmount` 返回 `NoAmount`），管理员个人账户零消耗，0 余额管理员也能操作。

**手续费仅在投票通过后执行转账时扣取**，从机构 `duoqian_address` 一次性扣除 `转账金额 + 手续费`。

手续费计算公式（与链上交易费一致）：
- 费率：`amount × 0.1%`
- 单笔最低：`10 分（0.1 元）`

手续费按制度规则三方分账：
| 接收方 | 比例 | 说明 |
| --- | --- | --- |
| 全节点矿工奖励钱包 | 80% | 通过 `MinerRewardWalletProvider` 查找当前区块矿工绑定钱包 |
| 国储会账户 | 10% | 通过 `NrcAccountProvider` 提供 |
| 销毁（黑洞） | 10% | 直接从流通中移除 |

分账在 pallet 内部的 `distribute_fee` 函数中完成，与 `PowOnchainFeeRouter` 规则一致。

#### 7.5.2 App 侧页面

| 页面 | 文件 | 说明 |
| --- | --- | --- |
| 转账表单 | `transfer_proposal_page.dart` | 填写收款地址、金额、备注，校验后签名提交 |
| 提案详情 | `transfer_proposal_detail_page.dart` | 查看提案信息（含备注折叠展开）、投票进度、管理员投票明细、投票操作 |

#### 7.5.2.1 签名方式

所有需要签名的操作（发起提案、投票、普通转账）统一检查钱包类型：
- **热钱包**（`signMode == 'local'`）：通过 `WalletManager.signWithWallet()` 本地签名，私钥不出类；签名前必须校验本地 seed 派生公钥与页面选中的管理员钱包 `pubkeyHex` 一致
- **冷钱包**（`signMode == 'external'`）：通过 `QrSigner` 协议（`WUMINAPP_QR_SIGN_V1`）发起扫码签名会话，导航到 `QrSignSessionPage` 展示请求二维码，用户用离线设备扫码签名后扫描回执二维码获取签名；回执中的 `pubkey` 必须与页面选中的管理员钱包一致

管理员钱包选择硬约束：
- 发起提案页面中选中的管理员钱包，是本次提案唯一允许的签名钱包
- 提案页面用于请求 SFID 人口快照的 `account_pubkey`，必须与最终链上 extrinsic 的签名钱包相同
- 任何“页面选中 A 钱包，但实际由 B 钱包签名”的情况，都必须在 App 侧直接拦截，不能继续提交

#### 7.5.3 App 侧服务

`TransferProposalService`（`transfer_proposal_service.dart`）封装：
- Extrinsic 编码（SCALE 编码 call data）和签名提交
- Storage 查询：活跃提案 ID、投票计数、提案状态、管理员投票记录
- 机构余额查询

#### 7.5.4 Extrinsic SCALE 编码

**propose_transfer**: `[0x13][0x00][org:u8][institution:48B][beneficiary:32B][amount:u128_le_16B][Vec remark]`
**vote_transfer**: `[0x13][0x01][proposal_id:u64_le][approve:bool]`

#### 7.5.5 机构 duoqian_address

每个 `InstitutionInfo` 包含 `duoqianAddress` 字段（32 字节 hex），来源于 `primitives` 中的 `duoqian_address`。
通过 `Keyring().encodeAddress(bytes, 2027)` 转为 SS58 地址展示。

### 7.6 管理员列表页面

管理员列表页面（`AdminListPage`）展示：
- 机构名称与类型标签
- 管理员总数与通过阈值
- 每位管理员的完整 SS58 地址（format 2027），当前用户标记"我"
- 地址一键复制功能

### 7.6 机构标识编码

shenfen_id 来源于 `primitives/china/china_cb.rs`（NRC + PRC）和 `primitives/china/china_ch.rs`（PRB），
编码为 48 字节固定长度（UTF-8 右补零），与链上 `InstitutionPalletId` 一致。

### 7.7 关键文件

| 文件 | 说明 |
| --- | --- |
| `lib/governance/all_proposals_view.dart` | 全局提案列表（分页 + 缓存 + WebSocket + 红点通知） |
| `lib/governance/proposal_cache.dart` | 提案内存缓存（Meta + Transfer Detail + Runtime Upgrade Detail） |
| `lib/rpc/chain_event_subscription.dart` | WebSocket 链事件订阅（新区块通知 + 自动重连） |
| `lib/governance/institution_data.dart` | 87 个机构静态注册表 + `findInstitutionByPalletId` 反查 + `formatProposalId` 格式化 |
| `lib/governance/institution_admin_service.dart` | 链上管理员查询服务（RPC + SCALE 解码 + 缓存） |
| `lib/governance/institution_detail_page.dart` | 机构详情页（管理员检测 + 条件 UI + 投票事件列表） |
| `lib/governance/proposal_types_page.dart` | 提案类型选择页（转账已接入真实页面） |
| `lib/governance/runtime_upgrade_page.dart` | Runtime 升级提案创建页（人口快照 + WASM 上传 + 签名提交） |
| `lib/governance/runtime_upgrade_detail_page.dart` | Runtime 升级提案详情页（联合投票/公民投票进度） |
| `lib/governance/admin_list_page.dart` | 管理员列表页（SS58 地址展示） |
| `lib/governance/transfer_proposal_page.dart` | 转账提案创建页（表单 + 校验 + 签名提交） |
| `lib/governance/transfer_proposal_detail_page.dart` | 转账提案详情页（投票进度 + 管理员明细 + 投票操作） |
| `lib/governance/transfer_proposal_service.dart` | 提案列表装配服务（转账提案 + 联合提案查询、分页与机构页事件聚合） |
| `lib/rpc/chain_rpc.dart` | RPC 服务（含 `fetchStorage` 公开方法） |
| `lib/main.dart` | 机构列表结构化（`InstitutionInfo`）+ 卡片点击跳转 |

## 8. 注册多签机构（duoqian-transaction-pow）

### 8.1 概述

`duoqian-transaction-pow` 模块为非治理机构提供多人管理的公共支出账户。所有操作（创建、关闭）通过投票引擎的内部投票机制执行，与治理机构（NRC/PRC/PRB）使用同一套投票、存储、清理基础设施。

### 8.2 机构类型

注册多签机构使用 `org = 3`（`ORG_DUOQIAN`），与治理机构 org 0/1/2 并列。

`InstitutionPalletId`（48 字节）= `duoqian_address`（32 字节 AccountId）+ 16 字节零填充。

### 8.3 动态阈值与管理员

| 项目 | 治理机构（NRC/PRC/PRB） | 注册多签机构（DUOQIAN） |
| --- | --- | --- |
| 管理员来源 | `admins_origin_gov::CurrentAdmins`（创世/治理替换） | `DuoqianAccounts.duoqian_admins`（注册时设定） |
| 阈值来源 | 硬编码（13/6/6） | `DuoqianAccounts.threshold`（注册时设定） |
| 管理员存储类型 | `AccountId` | `AccountId` |

通过 `InternalThresholdProvider` trait 和 `InternalAdminProvider` trait，投票引擎动态查询阈值和管理员列表。

### 8.4 Extrinsic

| Extrinsic | call_index | 说明 | 投票 |
| --- | --- | --- | --- |
| `register_sfid_institution(sfid_id)` | 2 | SFID 系统登记机构，派生多签地址 | 不需要 |
| `propose_create(sfid_id, admin_count, admins, threshold, amount)` | 0 | 发起"创建多签账户"提案 | 投票引擎 |
| `vote_create(proposal_id, approve)` | 3 | 创建提案投票，达标自动激活账户并转入资金 | 投票引擎 |
| `propose_close(duoqian_address, beneficiary)` | 1 | 发起"关闭多签账户"提案 | 投票引擎 |
| `vote_close(proposal_id, approve)` | 4 | 关闭提案投票，达标自动转出余额并删除账户 | 投票引擎 |

### 8.5 创建流程（Pending → Active）

1. 管理员调用 `propose_create` → 写入 `DuoqianAccounts`（status=Pending）+ 投票引擎创建提案
2. 其他管理员调用 `vote_create` → 投票引擎记票
3. 达到 threshold → 自动执行：`Currency::transfer` 转入资金 + `DuoqianAccounts.status` 改为 Active
4. 投票超时/否决 → 删除 Pending 状态的 `DuoqianAccounts`

### 8.6 关闭流程

1. 管理员调用 `propose_close` → 投票引擎创建提案
2. 其他管理员调用 `vote_close` → 投票引擎记票
3. 达到 threshold → 自动执行：`Currency::transfer` 转出全部余额 + 删除 `DuoqianAccounts`

### 8.7 关键文件

| 文件 | 说明 |
| --- | --- |
| `duoqian-transaction-pow/src/lib.rs` | 注册、创建、关闭业务逻辑 |
| `voting-engine-system/src/internal_vote.rs` | 投票引擎（含 ORG_DUOQIAN 支持） |
| `voting-engine-system/src/lib.rs` | InternalThresholdProvider trait |
| `runtime/src/configs/mod.rs` | RuntimeInternalThresholdProvider + RuntimeInternalAdminProvider |

## 9. 源码对齐基线

- `lib/governance/institution_data.dart`
- `lib/governance/institution_admin_service.dart`
- `lib/governance/institution_detail_page.dart`
- `lib/governance/proposal_types_page.dart`
- `lib/governance/admin_list_page.dart`
- `lib/governance/transfer_proposal_page.dart`
- `lib/governance/transfer_proposal_detail_page.dart`
- `lib/governance/transfer_proposal_service.dart`
- `lib/rpc/chain_rpc.dart`
- `citizenchain/transaction/duoqian-transfer-pow/src/lib.rs`
- `citizenchain/transaction/duoqian-transaction-pow/src/lib.rs`
- `citizenchain/governance/voting-engine-system/src/lib.rs`
- `citizenchain/governance/voting-engine-system/src/internal_vote.rs`
- `citizenchain/governance/voting-engine-system/src/joint_vote.rs`
- `citizenchain/governance/voting-engine-system/src/citizen_vote.rs`
- `citizenchain/governance/voting-engine-system/src/proposal_cleanup.rs`
- `citizenchain/governance/voting-engine-system/src/active_proposal_limit.rs`
- `citizenchain/governance/resolution-issuance-gov/src/lib.rs`
- `citizenchain/governance/runtime-root-upgrade/src/lib.rs`
- `citizenchain/governance/admins-origin-gov/src/lib.rs`
- `citizenchain/governance/resolution-destro-gov/src/lib.rs`
- `citizenchain/governance/grandpa-key-gov/src/lib.rs`
- `citizenchain/transaction/offchain-transaction-pos/src/lib.rs`
- `citizenchain/runtime/src/configs/mod.rs`
- `primitives/src/count_const.rs`
