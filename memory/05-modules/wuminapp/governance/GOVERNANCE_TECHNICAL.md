# Governance 治理模块技术文档（区块链规范版）

## 1. 模块目标

`lib/citizen/` 负责 WuminApp 底部“公民”Tab 及链上治理能力，覆盖：

- 提案（proposal）发起
- 投票（vote）提交
- 提案状态跟踪与结果展示

说明：

- 本文档定义的是“链上字段/格式/标准/流程”。
- 当前 App 已接入 runtime 升级、转账等主要治理路径，本文同时作为现有实现与后续扩展的对齐基线。

当前实现目录已经按公民域重排：

```text
lib/citizen/
  citizen_tab_page.dart
  vote/
  governance/
  institution/
  shared/
  proposal/
    shared/
    transfer/
    runtime_upgrade/
    admin_change/
    resolution_issuance/
    resolution_destroy/
    grandpakey_change/
```

`organization-manage` 代表注册机构多签账户的多签管理能力，归属 `lib/governance/organization-manage/`，不作为公民提案三级目录预留。

## 2. 链上入口与权限边界

### 2.1 关键约束（必须遵守）

- `votingengine` 的 `create_internal_proposal`、`create_joint_proposal` 和 `internal_vote` 外部调用被禁用，直接调用会返回 `NoPermission`。
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
| `proposal_id` | `u64` | 全局单调主键(双层 ID v1)。展示号 `(year, seq_in_year)` 通过 `votingengine::ProposalDisplayId[id]` 反查表持有,App 渲染为 `2026000123` 风格(年份 + 6 位补零序号),与主键解耦 |
| `approve` | `bool` | `true/false` |
| `nonce` | `BoundedVec<u8, 64>` | `0x` hex，解码后字节长度 `1..64` |
| `signature` | `BoundedVec<u8, 64>` | `0x` hex，解码后字节长度 `1..64` |
| `binding_id` | `Hash` | `0x` + 64 hex |

### 3.2 枚举与编码

- `org`：`0 = NRC(国储会)`，`1 = PRC(省储会)`，`2 = PRB(省储行)`，`3 = REN(个人多签)`，`4 = PUP(公权机构账户)`，`5 = OTH(其他机构账户)`。
- proposal kind：`0 = internal`，`1 = joint`。
- stage：`0 = internal`，`1 = joint`，`2 = citizen`。
- status：`0 = voting`，`1 = passed`，`2 = rejected`。

### 3.3 时效与阈值

- 单阶段投票时长：`VOTING_DURATION_BLOCKS`（当前为 30 天对应区块数）。
- 内部投票通过阈值：
  - NRC：`13`（硬编码）
  - PRC：`6`（硬编码）
  - PRB：`6`（硬编码）
  - REN/PUP/OTH 动态账户：链上 `admins-change::Subjects.threshold` 动态读取
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
| 决议发行 | `propose_resolution_issuance` | `reason, total_amount, allocations[]` | 国储会 + 43 个省储会管理员 | 联合+公民 |
| 协议升级 | `propose_runtime_upgrade` | `reason, code` | 国储会 + 43 个省储会管理员 | 联合+公民 |
| 管理员集合变更 | `propose_admin_set_change` | `org, subject, new_admins[]` | 目标账户当前管理员 | 内部 |
| 决议销毁 | `propose_destroy` | `org, institution, amount` | 目标省级管理员 | 内部 |
| GRANDPA 密钥更换 | `propose_replace_grandpa_key` | `institution, new_key(32B)` | NRC/PRC 省级管理员 | 内部 |
| 省储行业务治理(已下线) | ~~`propose_institution_rate / propose_verify_key / propose_sweep_to_main / propose_relay_submitters`~~ | Step 2b-iv-b 随老省储行清算 pallet 一起从 runtime 删除 | — | — |
| 清算行费率治理(新) | `propose_l2_fee_rate(call_index 40)` / `set_max_l2_fee_rate(call_index 41, Root)` | `bank, new_rate_bp` | 清算行管理员 / Root | — |

### 4.1 联合提案投票引擎字段标准

业务模块不接收人口快照字段。`eligible_total / snapshot_nonce / signature / province / signer_admin_pubkey` 只属于投票引擎的联合投票人口快照准备流程。

- `eligible_total`：`u64`，必须 `> 0`。
- `snapshot_nonce`：`1..64` 字节。
- `signature`：`1..64` 字节，运行时当前要求 64 字节 `sr25519` 原始签名。

人口快照验签消息标准（runtime）：

```text
payload = (
  DUOQIAN_DOMAIN,
  OP_SIGN_POP,
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

### 4.4 清算行(L2)费率治理(当前,替代原省储行治理)

Step 2b-iv-b 删除了原省储行 `propose_institution_rate / propose_verify_key /
propose_sweep_to_main / propose_relay_submitters` 四项治理 Call。新清算行体系
下的等价机制:

- `propose_l2_fee_rate(bank, new_rate_bp)`(call_index 40):
  - 签名者必须是该清算行主账户的多签管理员之一(`SfidAccountQuery::is_admin_of`)
  - `new_rate_bp` 范围 `[1, min(MaxL2FeeRateBp, 10)]`(默认上限 10 bp = 0.1%)
  - 成功后写 `L2FeeRateProposed[bank] = (rate, now + 20160 块)`(7 天延迟)
  - `on_initialize` 每块扫描,到期后自动搬到 `L2FeeRateBp[bank]` 并发 `L2FeeRateActivated` 事件
- `set_max_l2_fee_rate(new_max)`(call_index 41,Root Origin):
  - 调整全局费率上限 `MaxL2FeeRateBp`,范围 `[1, 10]` bp
  - Step 2b 起将改为由联合投票回调(免费调用)
- `propose_verify_key` / `propose_sweep_to_main` / `propose_relay_submitters`
  在清算行体系下均无等价 Call。验签密钥由清算行多签管理员的 sr25519 私钥本地持有
  (offchain_keystore),不再走链上提案;手续费划转(sweep)仍由
  `duoqian-transfer` pallet 的 `propose_sweep_to_main(call_index 5)` 治理。

## 5. 投票字段规范

### 5.1 内部投票（业务 pallet）

内部投票业务入口统一字段：

- `proposal_id: u64`
- `approve: bool`

统一函数：

- `InternalVote::cast(proposal_id, approve)`

说明：

- `admins-change` / `resolution-destro` / `grandpakey-change` 等业务 pallet 不再保留各自的 `vote_*` 投票入口。
- `duoqian-transfer` 的转账、安全基金和划转提案也统一走 `InternalVote::cast`。
- ~~`vote_institution_rate` / `vote_verify_key` / `vote_relay_submitters`~~(Step 2b-iv-b 已下线,随老省储行 pallet 一起从 runtime 删除)

### 5.2 联合机构投票（投票引擎）

`joint_vote` 字段：

- `proposal_id: u64`
- `institution: [u8;48]`
- `approve: bool`

权限要求：

- 必须由“当前省级管理员个人钱包”直接提交，不能跨机构代投。
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
  DUOQIAN_DOMAIN,
  OP_SIGN_VOTE,
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
2. 校验当前钱包是否具备该省级管理员权限。
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

#### 6.2.1 协议升级提案在 App 里的联合投票实现

- `RuntimeUpgradeDetailPage` 从机构页进入时必须带上：
  - `institution`
  - `adminWallets`
- 页面会先按链上 `AdminsChange.Subjects` 过滤当前仍有效的管理员钱包。
- 联合投票按钮只在以下条件全部满足时启用：
  - 提案仍处于 `joint` 阶段且状态为 `voting`
  - 当前机构尚未投票
- 当前用户已导入至少一个仍未投票的本省级管理员钱包
- App 直接使用所选管理员钱包提交 `joint_vote(proposal_id, institution, approve)`。
- 页面会读取：
  - `JointInstitutionTallies` 展示本机构当前赞成/反对管理员票数
  - `JointVotesByInstitution` 展示本机构是否已经形成最终机构结果
  - `JointVotesByAdmin` 判断当前导入管理员钱包是否已投票
- 页面展示的联合投票阈值不再写死 `3`，而是显示链上的联合权重阈值 `105`。
- 页面还会单独展示“本省级管理员投票进度 / 本机构阈值”，避免把联合权重阈值和机构内部门限混淆。

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
| `ActiveProposalsByInstitution` | 每机构活跃提案列表（上限由 runtime 配置，当前生产值 10） | 投票引擎 |

**自动清理策略（统一清理路径）：**
- 提案完成（通过/拒绝/过期）时注册延迟清理：`schedule_cleanup(proposal_id, current_block)`
- 清理时间 = 完成时区块 + **90 天**区块数
- 如果目标区块的队列已满（50 个），自动顺延到下一个区块，保证不丢失
- 每区块 `on_initialize` 检查 `CleanupQueue[当前区块]`，到期后触发清理
- 每区块最多触发 **5 个**提案进入清理流程，未处理完的保留在队列中，下个区块继续
- 实际数据删除委托给 `PendingProposalCleanups` 分块状态机，保证大量投票记录（如公民投票上万条）能分多个区块完成
- 清理状态机阶段：`InternalVotes → JointAdminVotes → JointInstitutionVotes → JointInstitutionTallies → CitizenVotes → VoteCredentials → ProposalObject → FinalCleanup`
- 提案结束（通过/拒绝/过期）时，活跃提案名额在 `set_status_and_emit` 中**立即释放**，不依赖业务模块

### 6.5 App 侧链路失败展示约束

- 治理列表、机构详情、提案详情读取链上数据时，如果轻节点未初始化、未同步完成或链路降级，必须显示“加载失败 / 轻节点不可用”。
- 不允许把轻节点读取失败降级成“暂无提案”“暂无管理员”“机构不存在”这类空态。
- 提案相关页面可继续把“链上 key 确实不存在”解释为空数据，但必须与“轻节点不可用”严格区分。
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
- `resolution-issuance` 和 `runtime-upgrade` 的独立 ID 体系（`NextProposalId`、`GovToJointVote`、`JointVoteToGov`）已删除，直接使用投票引擎 proposal_id

关键文件：
- `votingengine/src/cleanup.rs`（清理逻辑）
- `votingengine/src/limit.rs`（活跃提案限制）
- `votingengine/src/lib.rs`（ProposalData/ProposalObject/ProposalMeta/CleanupQueue Storage + 公共接口）

### 6.4.1 协议升级提案的摘要 / 对象分层

- `runtime-upgrade` 现在只把 `proposer + reason + code_hash` 编进 `ProposalData`
- 原始 wasm 不再塞进摘要层，而是统一进入 `ProposalObject(kind=runtime_wasm)`
- App 列表与详情页默认只读取摘要层，不主动拉取大对象
- App 展示协议升级提案真实结果时，只读取 `VotingEngine::Proposals.status`。

## 7. App 侧管理员权限检测与机构详情

### 7.1 管理员身份检测流程

1. 用户打开机构详情页，App 并行加载管理员列表和当前钱包信息。
2. 通过 `state_getStorage` 查询链上 `AdminsChange::Subjects(subject_id)` 存储。
3. Storage key 格式：`twox_128("AdminsChange") + twox_128("Subjects") + blake2_128(subject_id) + subject_id`。
4. `subject_id` 按主体类型派生：内置治理机构为 `0x01 + sfid_number`，个人多签为 `0x03 + AccountId32`，机构账户为 `0x05 + AccountId32`；`0x02 SfidInstitution` 只用于归属/检索，不进入管理员更换。
5. 返回 SCALE 编码的 `AdminSubject { org, kind, admins, threshold, creator, created_at, updated_at, status }`。
6. 比对当前钱包 `pubkeyHex`（去 0x 前缀、小写）是否在 `admins` 中，确定管理员身份。
7. 查询结果以内存 `subjectIdHex` 缓存；提交管理员更换、投票执行返回、下拉刷新时清除缓存重新查询。

### 7.1.1 管理员更换手机端协议

- 目录边界：手机端管理员更换只在 `lib/governance/admins-change/` 内实现；机构/个人注册、注销仍归 `organization-manage` / `personal-manage`。
- 主体规则：`PersonalDuoqian` 必须使用 `ORG_REN`，`InstitutionAccount` 必须使用 `ORG_PUP / ORG_OTH`，`SfidInstitution` 不能作为管理员更换主体。
- QR call data：`[AdminsChange=12][call=0][org:u8][subject_id:48][new_admins:Compact<Vec<AccountId32>>]`。
- QR display 字段必须与 wumin 冷钱包 decoder 严格一致：`org`、`subject`、`new_admins`，其中 `subject/new_admins` 均使用 `0x` 小写 hex。
- 提交成功后必须按 `subjectIdHex` 清理 `AdminSubjectService` 缓存，避免页面继续展示旧管理员集合。

### 7.2 机构详情页结构

机构详情页（`InstitutionDetailPage`）自上而下包含以下区域：

1. **顶部机构卡片**：左侧机构图标 + 中间机构类型标签与管理员/阈值信息。
   - 管理员用户：卡片可点击，显示右箭头，点击进入提案类型页面。
   - 非管理员用户：卡片不可点击，不显示右箭头。
2. **管理员身份标识**（仅管理员可见）：绿色提示条"你是本省级管理员，点击上方卡片可发起提案"。
3. **管理员列表入口**：所有用户可见，点击进入管理员列表页。
4. **投票事件列表**：所有用户可见，显示“本机构内部提案 + 所有机构都可见的联合投票提案”，按 ID 倒序展示。协议升级等联合投票提案必须在所有机构入口可见，不能只挂在国储会单一列表下。

### 7.2.1 全局提案列表（投票 tab）

公民页面的"投票"tab 展示全链所有提案（不分机构），按提案 ID 倒序（新的在上）。

**四层优化架构**：

| 层 | 说明 | 文件 |
| --- | --- | --- |
| WebSocket 订阅 | `chain_subscribeNewHeads` 监听新区块，自动检测新提案插入列表顶部 | `lib/rpc/chain_event_subscription.dart` |
| 本地内存缓存 | ProposalMeta / TransferProposalInfo / RuntimeUpgradeProposalInfo 缓存，避免重复 RPC | `lib/citizen/governance/proposal_cache.dart` |
| 批量查询 | `state_queryStorageAt` 一次 RPC 查多个 key，减少网络往返 | `chain_rpc.dart::fetchStorageBatch` |
| 分页加载 | 首屏 10 个，ScrollController 滚动触底加载更多 | `all_proposals_view.dart` |

**数据流**：首屏 → 分页取最新 10 个 ID → 缓存命中直接显示，未命中批量查 → 存缓存 → WebSocket 后台监听新区块 → 有新提案自动插入顶部。

**提案类型识别**：
- 内部提案：按转账等内部提案数据结构解码。
- 联合提案：按 `meta.kind == 1` 单独走联合提案解码链路，协议升级提案进入 `runtime_upgrade_detail_page.dart`。
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
| 机构联合投票 | 钱包是当前省级管理员，且该管理员尚未对本机构投票 | ✅ 已实现 |

关键文件：`lib/citizen/governance/all_proposals_view.dart`

### 7.3 权限控制规则

| 用户身份 | 可访问页面/功能 |
| --- | --- |
| 管理员 | 机构详情页、管理员列表、投票事件列表、提案类型页面（发起提案） |
| 非管理员 | 机构详情页、管理员列表、投票事件列表 |

核心原则：**只有管理员才能进入提案类型页面发起提案**，非管理员用户只能查看机构信息、管理员列表和投票事件。

### 7.3.1 活跃提案数量限制

每个机构（`SubjectId`）同时最多允许 runtime 配置数量的活跃提案（当前生产值为 **10**），不区分提案类型（转账、销毁、换管理员等），由投票引擎（`votingengine::limit`，原 `active_proposal_limit`）统一管控。

- 创建提案时：`try_add_active_proposal()` 检查并添加
- 提案完成时：`remove_active_proposal()` 在 `set_status_and_emit` 中立即释放（提案通过/拒绝/过期时）
- App 端发起提案前异步查询活跃数，达上限弹窗提示"提案数量已达上限"

关键文件：`votingengine/src/limit.rs`

### 7.4 提案类型页面

提案类型页面（`ProposalTypesPage`）根据机构类型条件展示可发起的提案：

**通用提案（所有机构类型）：**
- 转账：从机构多签账户发起转账
- 换管理员：提议更换本省级管理员
- 决议销毁：提议销毁机构持有的资产

**国储会专属提案（仅 NRC）：**
- 决议发行：发起公民币发行决议，需联合投票+公民投票
- 验证密钥：更换 GRANDPA 共识验证密钥
- 协议升级：协议升级提案详情展示与投票入口，提案发起只在 node 管理端

提交成功后，提案类型页应把创建结果向上冒泡到机构详情页，触发列表刷新，避免“链上已创建但机构页仍停留旧状态”。

### 7.5 多签转账边界

多签转账不属于 governance/proposal 的实现范围。wuminapp 端创建、详情、投票、
列表适配、余额提示、缓存和页面跳转统一由
`memory/05-modules/wuminapp/duoqian-transfer/DUOQIAN_TRANSFER_APP_TECHNICAL.md`
管理。

governance 侧只允许保留通用提案列表、机构详情页挂载点、投票上下文和
`InternalVote::cast` 共享能力；不得在 governance 文档或代码中重新描述
`DuoqianTransfer::propose_*` 的字段、页面、service 或投票实现。

#### 7.5.1 转出资金账户（mainAddress / accounts）

`InstitutionInfo` 对治理机构使用 `InstitutionAccounts` 表达制度账户：
- `mainAddress`：主账户，转账提案的默认转出账户
- `feeAddress`：费用账户
- `safetyFundAddress`：安全基金账户，仅国储会显示
- `stakeAddress`：质押账户，仅省储行显示

个人多签和机构账户可通过 `InstitutionInfo` 传入账户地址，但业务侧统一读取
`InstitutionInfo.mainAddress`。治理机构不得再使用 `duoqianAddress` 表达主账户。
通过 `Keyring().encodeAddress(bytes, 2027)` 转为 SS58 地址展示。

治理机构名称、身份 ID 和制度账户地址由
`tools/generate_wuminapp_governance_registry.mjs` 从 runtime primitives 生成到
`lib/institution/governance_institution_registry.generated.dart`。管理员列表与阈值不写入
静态注册表，必须动态读取链上 `AdminsChange::Subjects`。

治理机构详情页的账户信息区直接展示身份 ID、主账户和主账户余额；更多制度账户不再
进入二级页面，而是在当前账户信息卡内点击箭头展开。展开项按机构实际存在的
`feeAddress / safetyFundAddress / stakeAddress` 懒加载链上余额，分别显示费用账户、
安全基金账户和质押账户。

### 7.6 管理员列表页面

管理员列表页面（`AdminListPage`）展示：
- 机构名称与类型标签
- 管理员总数与通过阈值
- 每位管理员的完整 SS58 地址（format 2027），当前用户标记"我"
- 地址一键复制功能

### 7.6 机构标识编码

`institution_data.dart` 统一输出 D/ADR-015 `SubjectId(48)`：内置治理机构为 `0x01 + sfid_number UTF-8 + 右零填充`；个人多签为 `0x03 + AccountId32 + 15B 零填充`；注册机构账户为 `0x05 + AccountId32 + 15B 零填充`。`0x02 SfidInstitution` 只保留给同一 SFID 机构下多账户归属/检索，不作为转账支出主体。

### 7.7 关键文件

| 文件 | 说明 |
| --- | --- |
| `lib/citizen/governance/all_proposals_view.dart` | 全局提案列表（分页 + 缓存 + WebSocket + 红点通知） |
| `lib/citizen/governance/proposal_cache.dart` | 提案内存缓存（Meta + Transfer Detail + Runtime Upgrade Detail） |
| `lib/citizen/citizen_tab_page.dart` | 公民 Tab 二级导航入口（投票 / 治理 / 机构） |
| `lib/vote/vote_view.dart` | 投票二级页，全局治理提案列表与待投票红点 |
| `lib/rpc/chain_event_subscription.dart` | WebSocket 链事件订阅（新区块通知 + 自动重连） |
| `lib/common/institution_info.dart + lib/organization-manage/institution_registry.dart` | 87 个机构静态注册表 + `findInstitutionByPalletId` 反查 + `formatProposalId` 格式化 |
| `lib/institution/governance_institution_registry.generated.dart` | 从 runtime primitives 生成的治理机构身份 ID 与制度账户地址 |
| `lib/institution/institution_list_page.dart` | 机构分类列表（国储会 / 省储会 / 省储行） |
| `lib/governance/admins-change/services/institution_admin_service.dart` | 管理员查询门面（委托 `AdminSubjectService` 读取 `AdminsChange::Subjects`） |
| `lib/governance/organization-manage/institution_detail_page.dart` | 机构详情页（管理员检测 + 账户信息内联展开 + 条件 UI + 投票事件列表） |
| `lib/common/proposal/proposal_context.dart` | 用户与提案关系解析（管理员 / 公民 / 查看者） |
| `lib/common/proposal/proposal_models.dart` | 多提案共用模型（ProposalMeta / ProposalWithDetail 等） |
| `lib/votingengine/internal-vote/internal_vote_service.dart` | 多提案共用内部投票提交服务 |
| `lib/votingengine/internal-vote/pending_vote_store.dart` | 多提案共用待确认投票记录 |
| `lib/votingengine/internal-vote/proposal_vote_widgets.dart` | 多提案共用投票 UI 组件 |
| `lib/governance/duoqian_manage_detail_page.dart` | 跨个人/机构的多签管理提案详情页 |
| `lib/governance/governance_proposals_page.dart` | 提案类型选择页 |
| `lib/governance/runtime-upgrade/runtime_upgrade_page.dart` | 协议升级说明页（不发起提案、不选择 WASM、不获取人口快照） |
| `lib/governance/runtime-upgrade/runtime_upgrade_detail_page.dart` | 协议升级提案详情页（联合投票/公民投票进度） |
| `lib/governance/runtime-upgrade/runtime_upgrade_service.dart` | 协议升级提案链上交互服务 |
| `lib/governance/organization-manage/institution_admin_list_page.dart` | 管理员列表页（SS58 地址展示） |
| `lib/governance/organization-manage/shared` | 机构多签层（机构账户列表、机构详情、机构管理服务、机构 storage codec、机构发现服务） |
| `lib/governance/organization-manage/institution` | 机构多签层（机构列表入口、机构创建表单） |
| `lib/personal-manage` | 个人多签层（个人列表、详情、发现、创建、关闭、管理员激活、提案历史、PersonalManage 链上编解码） |
| `lib/rpc/chain_rpc.dart` | RPC 服务（含 `fetchStorage` 公开方法） |
| `lib/main.dart` | App 壳、应用锁与底部导航；不再内联公民 Tab 业务页面 |

## 8. 注册多签账户（organization-manage / personal-manage）

### 8.1 概述

`organization-manage` 负责 SFID 注册机构多签，`personal-manage` 负责个人多签。两者都复用投票引擎的内部投票机制，与治理机构（NRC/PRC/PRB）使用同一套投票、存储、清理基础设施。

### 8.2 机构类型

注册个人账户使用 `org = 3`（`ORG_REN`）；注册机构账户按类型使用 `org = 4`（`ORG_PUP`，公权机构账户）或 `org = 5`（`ORG_OTH`，其他机构账户），与治理机构 org 0/1/2 并列。

`SubjectId`（48 字节）使用 SubjectKind 协议：个人账户为 `0x03 PersonalDuoqian`，机构账户为 `0x05 InstitutionAccount`，payload 均为账户 `AccountId` 前 32 字节并右填零。

### 8.3 动态阈值与管理员

| 项目 | 治理机构（NRC/PRC/PRB） | 注册多签账户（REN/PUP/OTH） |
| --- | --- | --- |
| 管理员来源 | `admins_change::Subjects`（创世/治理替换） | `admins_change::Subjects`（注册时写入，治理替换后更新） |
| 阈值来源 | `admins_change::Subjects.threshold`（创世写入 13/6/6） | `admins_change::Subjects.threshold`（链端按管理员数量派生） |
| 管理员存储类型 | `AccountId` | `AccountId` |

通过 `InternalThresholdProvider` trait 和 `InternalAdminProvider` trait，投票引擎动态查询阈值和管理员列表。

### 8.4 Extrinsic

| Extrinsic | call_index | 说明 | 投票 |
| --- | --- | --- | --- |
| `OrganizationManage::propose_create_institution(..., admin_org, ...)` | 17.5 | 发起 SFID 机构多签账户创建提案；机构账户管理员 org 必须为 `ORG_PUP / ORG_OTH` | 投票引擎 |
| `OrganizationManage::propose_close(duoqian_address, beneficiary)` | 17.1 | 发起机构多签账户关闭提案 | 投票引擎 |
| `PersonalManage::propose_create(account_name, duoqian_admins, amount)` | 7.0 | 发起个人多签账户创建提案 | 投票引擎 |
| `PersonalManage::propose_close(duoqian_address, beneficiary)` | 7.1 | 发起个人多签账户关闭提案 | 投票引擎 |
| `InternalVote::cast(proposal_id, approve)` | 22.0 | 创建、关闭、转账等内部投票统一入口 | 统一投票入口 |

### 8.5 创建流程（Pending → Active）

1. 管理员调用对应创建入口 → 写入机构或个人 pending storage + 投票引擎创建提案
2. 其他管理员调用 `InternalVote::cast` → 投票引擎记票
3. 达到 threshold → 自动执行：`Currency::transfer` 转入资金 + 对应账户状态改为 Active
4. 投票超时/否决 → 清理 pending storage

### 8.6 关闭流程

1. 管理员调用 `propose_close` → 投票引擎创建提案
2. 其他管理员调用 `InternalVote::cast` → 投票引擎记票
3. 达到 threshold → 自动执行：`Currency::transfer` 转出全部余额 + 关闭对应机构或个人多签账户

### 8.7 多签转账接入边界

多签账户管理目录只负责账户注册、创建、关闭、状态展示和管理员管理。
发起转账、转账详情、投票进度、余额提示、列表适配与详情跳转均由
`lib/transaction/duoqian-transfer/` 实现。

`lib/governance/organization-manage/` 不实现多签转账逻辑；账户详情页如需展示转账入口，
只允许挂载 `lib/transaction/duoqian-transfer/duoqian_transfer_entry.dart` 提供的入口组件。

### 8.8 手机端入口分流

2026-04-30 起，`wuminapp` 将多签账户入口从“我的”页迁入交易页，并拆成
两个单类型入口：

- `机构多签`：只读取 `DuoqianInstitutionEntity`，右上角提供“创建机构多签账户”和链上自动发现/刷新入口。
- `个人多签`：只读取 `PersonalDuoqianEntity`，右上角提供“创建个人多签账户”和链上自动发现/刷新入口。

旧的 `lib/trade/duoqian/duoqian_trade_page.dart` 聚合页已删除。发起转账提案不删除，
入口由 `lib/transaction/duoqian-transfer/duoqian_transfer_entry.dart` 提供，
具体页面和链上构造仍归 `lib/transaction/duoqian-transfer/`。

2026-04-30 第二轮收口只迁移纯多签文件：账户管理模型/服务、账户详情、创建、
关闭、账户列表、账户详情和机构发现归入 `lib/organization-manage`；跨个人/机构的
多签管理提案详情留在 `lib/governance/duoqian_manage_detail_page.dart`。
QR 协议、Isar schema、钱包流水、治理聚合页、机构通用服务和内部投票通用服务仍留在原模块目录；
多签转账相关文件统一归 `lib/transaction/duoqian-transfer/`。

2026-04-30 第三轮收口删除治理提案类型页中的“创建多签/关闭多签”入口。多签创建
只能从 `机构多签` 或 `个人多签` 列表右上角进入；多签关闭只能从具体多签账户详情
页进入。机构多签关闭与个人多签关闭分别使用独立页面，不能再共用一个关闭入口。

2026-05-09 起，wuminapp 个人多签主业务与 runtime `personal-manage` 对齐：
个人创建、关闭、管理员激活、待创建提案反查、提案历史、PersonalManage call data、
PersonalManage ProposalData 解码、`PersonalManage::PersonalDuoqians` storage codec
统一放入 `lib/governance/personal-manage/`。`lib/governance/organization-manage/` 不再承载这些个人主业务；
目前仅保留机构多签能力和 `AdminInstitutionCodec` 这类个人/机构都要读取的底层 Subject 解码能力。

### 8.9 关键文件

| 文件 | 说明 |
| --- | --- |
| `lib/governance/organization-manage/duoqian_account_list_page.dart` | 机构多签账户列表页 |
| `lib/governance/organization-manage/duoqian_account_info_page.dart` | 机构多签账户详情页 |
| `lib/governance/organization-manage/duoqian_discovery_service.dart` | 机构多签反向索引发现服务 |
| `lib/governance/organization-manage/duoqian_manage_models.dart` | 机构关闭提案模型与机构账户状态模型 |
| `lib/governance/organization-manage/duoqian_manage_service.dart` | OrganizationManage 机构多签链上交互服务 |
| `lib/governance/duoqian_manage_detail_page.dart` | 个人/机构多签管理提案共用投票详情页；业务解码委托对应 manage 服务 |
| `lib/governance/organization-manage/institution_duoqian_create_page.dart` | 机构多签创建表单 |
| `lib/governance/organization-manage/institution_duoqian_close_page.dart` | 机构多签关闭表单 |
| `lib/governance/personal-manage/personal_duoqian_create_page.dart` | 个人多签创建表单 |
| `lib/governance/personal-manage/personal_duoqian_close_page.dart` | 个人多签关闭表单 |
| `lib/governance/personal-manage/personal_admin_list_page.dart` | 个人多签管理员激活列表 |
| `lib/governance/personal-manage/personal_manage_account_list_page.dart` | 个人多签账户列表页 |
| `lib/governance/personal-manage/personal_manage_account_info_page.dart` | 个人多签账户详情页 |
| `lib/governance/personal-manage/personal_manage_discovery_service.dart` | 个人多签反向索引发现服务 |
| `lib/governance/personal-manage/personal_manage_service.dart` | PersonalManage 个人多签链上交互服务 |
| `lib/governance/personal-manage/personal_manage_storage_codec.dart` | PersonalManage storage key 与 SCALE 解码 |
| `lib/governance/personal-manage/personal_proposal_history_service.dart` | 个人多签提案历史聚合与 Isar 持久化 |
| `organization-manage/src/lib.rs` | SFID 注册机构多签登记、创建、关闭业务逻辑 |
| `personal-manage/src/lib.rs` | 个人多签创建、关闭业务逻辑 |
| `duoqian-transfer/src/lib.rs` | 机构账户转账复用现有提案/投票/执行流程 |
| `votingengine/internal-vote/src/lib.rs` | 投票引擎（支持 ORG_REN / ORG_PUP / ORG_OTH 动态主体） |
| `votingengine/src/lib.rs` | InternalThresholdProvider trait |
| `runtime/src/configs/mod.rs` | RuntimeInternalThresholdProvider + RuntimeInternalAdminProvider |

## 9. 源码对齐基线

- `lib/common/institution_info.dart + lib/organization-manage/institution_registry.dart`
- `lib/governance/admins-change/services/institution_admin_service.dart`
- `lib/governance/organization-manage/institution_detail_page.dart`
- `lib/governance/governance_proposals_page.dart`
- `lib/governance/organization-manage/institution_admin_list_page.dart`
- `lib/governance/organization-manage/duoqian_account_list_page.dart`
- `lib/governance/organization-manage/duoqian_account_info_page.dart`
- `lib/governance/organization-manage/duoqian_discovery_service.dart`
- `lib/governance/organization-manage/duoqian_manage_models.dart`
- `lib/governance/organization-manage/duoqian_manage_service.dart`
- `lib/governance/duoqian_manage_detail_page.dart`
- `lib/governance/organization-manage/institution_duoqian_create_page.dart`
- `lib/governance/organization-manage/institution_duoqian_close_page.dart`
- `lib/governance/personal-manage/personal_duoqian_create_page.dart`
- `lib/governance/personal-manage/personal_duoqian_close_page.dart`
- `lib/governance/personal-manage/personal_admin_list_page.dart`
- `lib/governance/personal-manage/personal_manage_account_list_page.dart`
- `lib/governance/personal-manage/personal_manage_account_info_page.dart`
- `lib/governance/personal-manage/personal_manage_discovery_service.dart`
- `lib/governance/personal-manage/personal_manage_service.dart`
- `lib/governance/personal-manage/personal_manage_storage_codec.dart`
- `lib/governance/personal-manage/personal_proposal_history_service.dart`
- `lib/rpc/chain_rpc.dart`
- `citizenchain/runtime/transaction/duoqian-transfer/src/lib.rs`
- `citizenchain/runtime/governance/organization-manage/src/lib.rs`
- `citizenchain/runtime/governance/personal-manage/src/lib.rs`
- `citizenchain/runtime/votingengine/src/lib.rs`
- `citizenchain/runtime/votingengine/src/internal_vote.rs`
- `citizenchain/runtime/votingengine/src/joint_vote.rs`
- `citizenchain/runtime/votingengine/src/citizen_vote.rs`
- `citizenchain/runtime/votingengine/src/cleanup.rs`
- `citizenchain/runtime/votingengine/src/limit.rs`
- `citizenchain/runtime/issuance/resolution-issuance/src/lib.rs`
- `citizenchain/runtime/governance/runtime-upgrade/src/lib.rs`
- `citizenchain/runtime/governance/admins-change/src/lib.rs`
- `citizenchain/runtime/governance/resolution-destro/src/lib.rs`
- `citizenchain/runtime/governance/grandpakey-change/src/lib.rs`
- `citizenchain/runtime/transaction/offchain-transaction/src/lib.rs`
- `citizenchain/runtime/src/configs/mod.rs`
- `primitives/src/count_const.rs`
