# Governance 治理模块技术文档（区块链规范版）

## 1. 模块目标

`lib/citizen/` 负责 WuminApp 底部“公民”Tab 及链上治理能力，覆盖：

- 提案（proposal）发起
- 投票（vote）提交
- 提案状态跟踪与结果展示

说明：

- 本文档定义的是“链上字段/格式/标准/流程”。
- 当前 App 已接入 runtime 升级、转账等主要治理路径，本文同时作为现有实现与后续扩展的对齐基线。

当前实现目录已经按公民、治理共享与投票引擎边界重排：

```text
lib/citizen/
  citizen_tab_page.dart
  public/
  vote/
lib/governance/
  admins-change/
  organization-manage/
  personal-manage/
  runtime-upgrade/
  shared/
    admin_institution_codec.dart
    institution_info.dart
    proposal/
lib/votingengine/
  internal-vote/
  joint-vote/
  citizen-vote/
```

`organization-manage` 代表注册机构多签账户的多签管理能力，归属 `lib/governance/organization-manage/`，不作为公民提案三级目录预留。

治理页本地状态边界：

- 治理机构、提案、管理员列表等链上信息不能因为本机钱包库短暂 busy 而整页“加载失败”。
- `ProposalContextResolver` 读取本地钱包失败时返回空管理员钱包列表，并保留链上机构/提案内容展示。
- 机构详情页单独读取冷钱包管理员匹配关系；该读取失败只影响“当前用户是否管理员”的本地提示，不影响机构余额、管理员名单和提案列表。
- 所有治理模块读写 Isar 必须走 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()`，不得直接取 `WalletIsar.instance.db()`。
- 治理列表和详情页的展示数据分三层：本地静态机构常量、本机 Isar 持久化展示快照、链上 runtime 真值。页面首屏只能依赖前两层；链上读取放到后台 TTL 刷新、下拉刷新、返回刷新或提交前复核。
- `ProposalLocalStore` 保存广场/机构提案列表摘要和索引；`ProposalDetailLocalStore` 保存转账提案、多签管理提案、Runtime 升级提案详情快照；两者都只服务展示，不得作为投票/执行/提交前校验的最终真相。
- `AdminSubjectService` 保存管理员主体持久化短缓存；提交投票前仍必须重新读取链上管理员快照、提案状态和对应管理员投票记录。
- 管理员投票记录必须批量读取：内部投票走 `InternalVoteQueryService.fetchAdminVotesBatch()`，联合投票走 `RuntimeUpgradeService.fetchJointAdminVotesBatch()`，避免 43 个管理员造成 43 次 storage RPC。

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
  - REN/PUP/OTH 动态账户：链上 `internal-vote::ActiveDynamicThresholds` 动态读取
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
| 管理员集合变更 | `propose_admin_set_change` | `org, subject, new_admins[], new_threshold` | 目标账户当前管理员 | 内部 |
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
3. 发起签名并提交交易；交易 nonce 必须每次签名前实时读取 runtime `frame_system::Account.nonce`，App 不得缓存、自增、预占或回滚 nonce。
4. 投票是否成功必须由 runtime 投票引擎 storage 确认：
  - 内部投票读取 `InternalVote::InternalVotesByAccount(proposal_id, admin)`。
  - 联合投票读取 `JointVote::JointVotesByAdmin(proposal_id, institution, admin)`。
  - 公民投票读取投票引擎对应的公民投票记录。
5. `author_submitExtrinsic` 返回 txHash、交易池 watch 的 `inBlock/finalized`、本地 pending 记录都不能单独代表“已投票”；内部投票和联合投票提交后必须回读对应 runtime 投票 storage。
6. 监听事件刷新状态：
  - `InternalVoteCast / JointAdminVoteCast / JointInstitutionVoteFinalized / CitizenVoteCast`
  - `ProposalAdvancedToCitizen`
  - `ProposalFinalized`

待确认投票处理规则：

- 提交投票时，服务层先等待交易 `inBlock / finalized`，再回读 runtime 投票 storage；新成功流程不再写本地 pending，只清理旧残留 pending。
- 如果 runtime 已记录该管理员投票，清除 pending，并把管理员显示为已投票。
- 如果交易池 watch 返回 `timeout / finalityTimeout / retracted / future / error`，不得直接清除 pending，也不得把管理员恢复成未投票；必须继续以 runtime 投票 storage 为准。
- 如果 runtime 无投票记录且 pending 超过 20 分钟确认窗口，视为本地提交没有形成有效投票，清除 pending 并允许重新提交，不能让管理员明细无限显示“投票中”。
- 服务层完成入块和 runtime 投票记录确认后，底部按钮停止 `submitting` 转圈；详情页立即把该管理员显示为已投票，`_load()` 只后台刷新展示状态，不得把 txHash 当作投票成功。
- 联合投票读取 `JointVote` storage 时，机构参数必须使用统一 `SubjectId` 编码；wuminapp 只能调用 `institutionIdentityToPalletId()`，不得在页面内手写 sfid `[u8;48]` 编码。

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

1. 用户打开机构详情页，App 先展示本地固定机构信息，再后台并行加载管理员列表和当前钱包信息。
2. 通过 `state_getStorage` 查询链上 `AdminsChange::Subjects(subject_id)` 存储。
3. Storage key 格式：`twox_128("AdminsChange") + twox_128("Subjects") + blake2_128(subject_id) + subject_id`。
4. `subject_id` 按主体类型派生：内置治理机构为 `0x01 + sfid_number`，个人多签为 `0x03 + AccountId32`，机构账户为 `0x05 + AccountId32`；`0x02 SfidInstitution` 只用于归属/检索，不进入管理员更换。
5. 返回 SCALE 编码的 `AdminSubject { org, kind, admins, creator, created_at, updated_at, status }`。
6. 阈值不再来自 `AdminsChange.Subjects`；治理机构用固定制度阈值，个人多签/机构账户从 `InternalVote.ActiveDynamicThresholds` 读取。
   `AdminsChange.Subjects` 的管理员列表后续字段是创建者和生命周期信息，不得作为阈值解码。
7. 比对当前钱包 `pubkeyHex`（去 0x 前缀、小写）是否在 `admins` 中，确定管理员身份。
8. 查询结果以内存 `subjectIdHex` 短缓存；提交管理员更换、投票执行返回、下拉刷新时清除缓存重新查询。

### 7.1.1 管理员更换手机端协议

- 目录边界：手机端管理员更换只在 `lib/governance/admins-change/` 内实现；机构/个人注册、注销仍归 `organization-manage` / `personal-manage`。
- 主体规则：`PersonalDuoqian` 必须使用 `ORG_REN`，`InstitutionAccount` 必须使用 `ORG_PUP / ORG_OTH`，`SfidInstitution` 不能作为管理员更换主体。
- QR call data：`[AdminsChange=12][call=0][org:u8][subject_id:48][new_admins:Compact<Vec<AccountId32>>][new_threshold:u32_le]`。
- 内置治理机构只读展示固定阈值，不展示输入框；提交管理员更换提案时 `new_threshold` 必须等于制度固定阈值。
- 个人多签和机构账户展示动态阈值输入框，校验公式为 `threshold * 2 > admin_count && threshold <= admin_count`。
- QR display 字段必须与 wumin 冷钱包 decoder 严格一致：`org`、`subject`、`new_admins`、`new_threshold`，其中 `subject/new_admins` 均使用 `0x` 小写 hex。
- 提交成功后必须按 `subjectIdHex` 清理 `AdminSubjectService` 缓存，避免页面继续展示旧管理员集合。

### 7.2 机构详情页结构

机构详情页（`InstitutionDetailPage`）自上而下包含以下区域：

0. **机构账户信息卡**：身份 ID、主账户地址、制度账户类型和内部门槛均为本地固定数据，进入页面立即显示；主账户余额是链上动态数据，独立后台读取并只更新余额字段。
1. **顶部机构卡片**：左侧机构图标 + 中间机构类型标签与管理员/阈值信息。
   - 机构类型、阈值来自本地制度常量。
   - 管理员人数来自链上 `AdminsChange::Subjects`，读取中或读取失败时只更新副标题，不阻塞页面。
   - 管理员用户：进入提案类型页面后可发起提案。
   - 非管理员用户：可进入提案类型页面查看入口，但发起按钮禁用并提示先激活管理员身份。
2. **管理员身份标识**（仅管理员可见）：绿色提示条"你是本省级管理员，点击上方卡片可发起提案"。
3. **管理员列表入口**：所有用户可见，点击进入管理员列表页。
4. **投票事件列表**：所有用户可见，显示“本机构内部提案 + 所有机构都可见的联合投票提案”，按 ID 倒序展示。协议升级等联合投票提案必须在所有机构入口可见，不能只挂在国储会单一列表下。

### 7.2.1 机构详情页数据来源与刷新边界

| 数据 | 来源 | 刷新方式 | 首屏策略 |
| --- | --- | --- | --- |
| 机构名称、类型、身份 ID | 本地静态注册表 | 随 App 版本更新 | 立即显示 |
| 主账户、费用账户、安全基金账户、质押账户地址 | 本地静态注册表 | 随 App 版本更新 | 立即显示 |
| 治理机构内部门槛 | 本地制度常量 | 随 App 版本更新 | 立即显示 |
| 管理员列表和管理员人数 | 链上 `AdminsChange::Subjects` | 后台读取，30 秒内存短缓存，下拉刷新强制更新 | 显示“读取中/读取失败”副标题 |
| 当前用户管理员身份 | 本地钱包 + 本地激活记录 + 链上管理员列表 | 后台读取，激活/返回/下拉刷新后更新 | 显示身份确认中，不挡住页面 |
| 主账户余额 | 链上账户余额 | 后台读取，短缓存，下拉刷新强制更新 | 余额字段显示“读取中” |
| 机构可见提案列表 | 链上机构提案索引 + 年度联合提案索引 | 后台读取，短缓存，下拉刷新/提案详情返回后强制更新 | 提案区局部加载 |
| 更多制度账户余额 | 链上账户余额 | 用户展开后按需读取 | 不进入首屏请求 |

治理机构详情页不得使用单个 `_loading` 等待全部链上请求。管理员、余额、提案任一读取失败时，只能影响对应区域；固定本地数据必须始终可见。

### 7.2.2 提案列表本地持久化读库

治理机构详情页和公民-广场的提案列表使用 `ProposalLocalStore` 作为本机持久化展示读库：

| 本地持久化内容 | 存储位置 | 用途 |
| --- | --- | --- |
| `LocalProposalSummary` | Isar `AppKvEntity(governance.proposal.summary.<proposal_id>)` | 提案卡片首屏展示摘要 |
| 全局治理提案 ID 索引 | Isar `AppKvEntity(governance.proposal.index.global)` | 公民-广场分页排序 |
| 单机构提案 ID 索引 | Isar `AppKvEntity(governance.proposal.index.institution.<sfid_number>)` | 治理机构详情页提案列表 |

本地摘要包含 `proposalId / displayId / kind / stage / status / internalOrg /
institutionBytes / institutionName / title / subtitle / iconKind /
updatedAtMillis`。这些字段只用于列表展示和首屏恢复，不作为链上真相。

链上同步规则：

- 治理机构详情页先读本地机构索引和摘要；本地为空、索引超过 5 分钟、用户下拉刷新、发起/查看提案返回时，再读取链上 `ProposalsByInstitution` 和年度联合提案索引。
- 公民-广场先读本地全局索引和摘要；本地为空、索引超过 5 分钟、用户下拉刷新、详情返回时，再读取 `ProposalsByOrg[0/1/2]`。
- 新区块订阅只做节流检查，当前最短 60 秒读取一次全局治理 ID 索引；不得每个新区块都全量读取三类 org 索引。
- 提案详情点击时，如果当前只有本地摘要，才按单个 `proposalId` 读取链上详情并回写本地摘要。
- `ProposalData` 和展示摘要创建后变化很少，优先持久化复用；`status`、红点、投票记录、票数进度等动态字段按短 TTL 或用户操作触发链上刷新。

硬边界：

- 本地持久化读库只服务列表展示，不允许替代 runtime storage。
- 投票按钮可用性、是否已投票、提交前状态校验、执行状态判断，仍必须读取链上 `VotingEngine / InternalVote / JointVote / AdminsChange` 相关 storage。

### 7.2.3 全局提案列表（广场 tab）

公民页面的"广场"tab 展示全链所有提案（不分机构），按提案 ID 倒序（新的在上）。

**四层优化架构**：

| 层 | 说明 | 文件 |
| --- | --- | --- |
| 新区块订阅 | `chain_subscribeNewHeads` 监听新区块，但对索引检查做 60 秒节流，避免每块全量读取 | `lib/rpc/chain_event_subscription.dart` |
| 本地持久化读库 | `LocalProposalSummary` + 全局/机构索引持久化，App 重启后可先显示本地列表 | `lib/governance/shared/proposal/proposal_local_store.dart` |
| 本地内存缓存 | ProposalMeta / TransferProposalInfo / RuntimeUpgradeProposalInfo 缓存，避免同一会话重复 RPC | `lib/governance/shared/proposal/proposal_cache.dart` |
| 批量查询 | `state_queryStorageAt` 一次 RPC 查多个 key，减少网络往返 | `chain_rpc.dart::fetchStorageBatch` |
| 分页加载 | 首屏 10 个，ScrollController 滚动触底加载更多 | `lib/citizen/vote/vote_view.dart` |

**数据流**：首屏先读本机 Isar 全局索引和摘要 → 本地可用则直接显示 → 索引过期/用户刷新/详情返回再读取链上三类 org 反向索引 → 首屏 10 条缺失详情批量查询 → 写回本地持久化读库和内存缓存 → 新区块后台节流检查，有新提案再插入顶部。

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

关键文件：`lib/citizen/vote/vote_view.dart`

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

治理机构名称、身份 ID、制度账户地址和治理机构固定阈值由
`tools/generate_wuminapp_governance_registry.mjs` 从 runtime primitives 生成到
`lib/institution/governance_institution_registry.generated.dart` 并由
`InstitutionInfo` 派生展示。管理员列表不写入静态注册表，必须动态读取链上
`AdminsChange::Subjects`。

治理机构列表页（`lib/governance/governance_list_page.dart`）只负责本机展示顺序：
国储会直接展示，单张卡片横跨整行显示到右侧边缘且高度与省储会/省储行卡片一致；省储会、省储行默认折叠，
标题行最右侧用线性右箭头/下箭头展开后按静态注册表顺序展示。
省储会分组标题图案使用 `assets/icons/government-line.svg`；省储行分组标题图案使用
`assets/icons/bank.svg`；机构卡片内部不显示名称左侧图标，只显示机构名称和右箭头。
用户长按省储会或省储行卡片拖拽时，只能在所属分组内排序；排序以
`SharedPreferences` 保存 `sfidNumber` 列表到本机，不写链、不跨设备同步。
本页不得再按管理员机构优先做 `_sorted()` 自动排序；管理员身份只影响卡片高亮，
不改变展示顺序。

治理机构详情页的账户信息区直接展示身份 ID 和主账户；主账户余额后台读取并仅更新余额
字段。更多制度账户不再进入二级页面，而是在当前账户信息卡内点击箭头展开。展开项按机构
实际存在的 `feeAddress / safetyFundAddress / stakeAddress` 懒加载链上余额，分别显示
费用账户、安全基金账户和质押账户。

治理机构详情页的提案列表先读取 `ProposalLocalStore` 中的机构索引和摘要；链上刷新成功后
回写本地摘要与机构索引。该列表展示的本地摘要允许短暂落后链上状态，但点击提案详情、投票、
执行和提交前校验必须重新读取链上详情和投票状态。

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
| `lib/citizen/citizen_tab_page.dart` | 公民 Tab 二级导航入口（投票 / 治理 / 机构） |
| `lib/citizen/vote/vote_view.dart` | 投票二级页，全局治理提案列表与待投票红点 |
| `lib/rpc/chain_event_subscription.dart` | WebSocket 链事件订阅（新区块通知 + 自动重连） |
| `lib/governance/shared/institution_info.dart + lib/governance/organization-manage/institution_registry.dart` | 87 个机构静态注册表 + `findInstitutionByPalletId` 反查 + `formatProposalId` 格式化 |
| `lib/governance/organization-manage/governance_institution_registry.generated.dart` | 从 runtime primitives 生成的治理机构身份 ID 与制度账户地址 |
| `lib/governance/admins-change/services/institution_admin_service.dart` | 管理员查询门面（委托 `AdminSubjectService` 读取 `AdminsChange::Subjects`） |
| `lib/governance/organization-manage/institution_detail_page.dart` | 机构详情页（管理员检测 + 账户信息内联展开 + 条件 UI + 投票事件列表） |
| `lib/governance/shared/proposal/proposal_context.dart` | 用户与提案关系解析（管理员 / 公民 / 查看者） |
| `lib/governance/shared/proposal/proposal_models.dart` | 多提案共用模型（ProposalMeta / ProposalWithDetail 等） |
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
| 阈值来源 | 投票引擎固定制度常量（13/6/6） | `internal-vote::ActiveDynamicThresholds`（注册或管理员变更时写入） |
| 管理员存储类型 | `AccountId` | `AccountId` |

投票引擎通过 `InternalAdminProvider` 查询管理员列表；动态阈值由 `internal-vote` 自己保存和读取。

### 8.4 Extrinsic

| Extrinsic | call_index | 说明 | 投票 |
| --- | --- | --- | --- |
| `OrganizationManage::propose_create_institution(..., admin_org, ...)` | 17.5 | 发起 SFID 机构多签账户创建提案；机构账户管理员 org 必须为 `ORG_PUP / ORG_OTH` | 投票引擎 |
| `OrganizationManage::propose_close(duoqian_address, beneficiary)` | 17.1 | 发起机构多签账户关闭提案 | 投票引擎 |
| `PersonalManage::propose_create(account_name, duoqian_admins, regular_threshold, amount)` | 7.0 | 发起个人多签账户创建提案；普通阈值用户输入且必须过半，注册阈值固定全员同意 | 投票引擎 |
| `PersonalManage::propose_close(duoqian_address, beneficiary)` | 7.1 | 发起个人多签账户关闭提案 | 投票引擎 |
| `InternalVote::cast(proposal_id, approve)` | 22.0 | 创建、关闭、转账等内部投票统一入口 | 统一投票入口 |

### 8.5 创建流程（Pending → Active）

1. App 创建前按链端口径校验发起钱包 free 余额覆盖 `初始资金 + 创建手续费 + ED`；
   创建手续费为 `max(初始资金 * 0.1%, 0.10 元)`，ED 当前为 `1.11 元`。
2. 管理员调用对应创建入口 → 写入机构或个人 pending storage + 投票引擎创建提案
3. App 不能把 txHash 当创建成功；必须等待交易入块，并在同一区块确认
   `PersonalManage.PersonalDuoqianProposed` 或
   `OrganizationManage.InstitutionCreateProposed` 后，才写本地记录。
4. 本地提案编号必须使用事件中的 `proposal_id`，不得预测
   `VotingEngine.NextProposalId`。
5. 发起人已自动记一票赞成，其他管理员调用 `InternalVote::cast` 补票
6. 创建投票全员同意后自动执行：`Currency::transfer` 转入资金 + 对应账户状态改为 Active
7. 投票超时/否决 → 清理 pending storage

### 8.6 关闭流程

1. 管理员调用 `propose_close` → 投票引擎创建提案
2. 发起人已自动记一票赞成，其他管理员调用 `InternalVote::cast` 补票
3. 注销投票全员同意后自动执行：扣链上手续费，把剩余 free 余额转入用户提供的收款地址，并删除对应个人/机构多签当前状态
4. 链上历史事件和历史提案不删除；wuminapp 本地继续在统一账户列表展示已注销账户，状态显示“已注销”，不显示余额；用户在详情页点击右上角“删除”后才清理本机数据。
5. Active 状态的个人/机构多签详情页右上角菜单只显示“关闭个人多签”或“关闭机构多签”文本项，不带删除图标；Closed 状态才显示“删除”并保留删除图标。

### 8.7 多签转账接入边界

多签账户管理目录只负责账户注册、创建、关闭、状态展示和管理员管理。
发起转账、转账详情、投票进度、余额提示、列表适配与详情跳转均由
`lib/transaction/duoqian-transfer/` 实现。

`lib/governance/organization-manage/` 不实现多签转账逻辑；账户详情页如需展示转账入口，
只允许挂载 `lib/transaction/duoqian-transfer/duoqian_transfer_entry.dart` 提供的入口组件。

### 8.8 手机端入口分流

2026-05-17 起，`wuminapp` 将多签账户入口从交易页迁入底部第 2 个 `多签` Tab，点击后直接显示统一多签账户列表，顶部标题为“多签”：

- 个人多签读取 `PersonalDuoqianEntity` 和 `PersonalDuoqianLocalState`。
- 机构多签读取 `DuoqianInstitutionEntity` 和 `InstitutionDuoqianLocalState`。
- 多签列表在首次点击 `多签` Tab 后构建，防止应用启动时提前触发多签账户发现。
- 多签列表首屏只读本机 Isar，不等待链上账户状态查询、不等待 `AdminsChange::Subjects`
  全量 discovery；链上刷新只能在后台更新局部状态。
- 本地状态使用 `AppKvEntity.stringValue` 保存 `active / pending / closed`，
  使用 `AppKvEntity.intValue` 记录最近成功链上同步时间。
- 多签账户详情页必须 local-first：个人详情页读取 `PersonalDuoqianEntity`、
  `PersonalDuoqianLocalState` 和 `personal_duoqian_detail:*`；机构详情页读取
  `DuoqianInstitutionEntity`、`InstitutionDuoqianLocalState` 和
  `institution_duoqian_detail:*`。这些都是本机持久化储存，不是内存缓存。
- 详情页本机快照可直接显示名称、地址、本地状态、管理员公钥列表、阈值和余额快照；
  链上只负责更新账户是否存在、Active/Pending 状态、管理员/阈值和 Active 余额。
- 详情页状态 TTL 与余额 TTL 不得混用。列表页批量状态刷新写入详情快照时，必须保留已有
  `balanceYuan` 和 `lastBalanceRefreshAtMillis`；Active 详情页若本地余额为空或余额
  TTL 过期，应只调用余额读取，不重复拉管理员/阈值。
- 详情页进入时不得用全屏转圈等待 `fetchPersonalAccount()`、
  `fetchDuoqianAccount()` 或 `InstitutionAdminService.fetchAdmins()`；
  自动刷新必须复用 `fetchPersonalAccountsBatch([address])` /
  `fetchDuoqianAccountsBatch([address])`，并受 TTL 控制。
- 详情页不展示“同步中”类 UI。下拉刷新、转账提案创建返回、投票返回、关闭返回
  才忽略 TTL 精准刷新当前账户；链上失败只保留本机已储存数据，不写成 Closed。
- Active 账户 60 分钟内不自动重复查链；Pending / Closed 账户 10 分钟内不自动重复查链；
  下拉刷新才忽略 TTL 强制刷新。
- 自动 discovery 只允许首次进入多签 Tab 或本机钱包 pubkey fingerprint 变化时触发；
  不做每日自动扫描，也不增加单独扫描按钮。
- 下拉刷新先强制刷新已知账户状态，再强制执行个人和机构 discovery。
- 创建、关闭、投票、删除返回时只刷新相关账户或本地记录，不重新扫描全部多签。
- 个人多签状态刷新由 `PersonalManageService.fetchPersonalAccountsBatch()` 分阶段批量读取
  `PersonalDuoqians / Subjects / ActiveDynamicThresholds / PendingDynamicThresholds`。
- 机构多签状态刷新由 `DuoqianManageService.fetchDuoqianAccountsBatch()` 分阶段批量读取
  `AddressRegisteredSfid / InstitutionAccounts / Subjects / ActiveDynamicThresholds / PendingDynamicThresholds`。
- 两条批量路径都通过 `ChainRpc.fetchStorageBatchChunked()` 分块读取 storage，列表页不得逐个账户循环调用详情查询。
- 右上角加号提供“新增个人多签 / 新增机构多签”两个入口。
- 原交易页中的多签入口删除，交易页只保留普通链上支付和扫码支付。

发起转账提案不删除，
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

2026-05-17 起，个人/机构多签创建类交易在入块后统一检查 `System.Events`：
先解析 `System.ExtrinsicFailed` 的 runtime 模块错误，再确认对应成功事件。
因此余额不足、管理员主体残留、机构账户索引占用等链上拒绝会显示真实
`PersonalManage / AdminsChange / OrganizationManage` 错误，不再落到“未找到成功事件”的泛化提示。

### 8.9 关键文件

| 文件 | 说明 |
| --- | --- |
| `lib/governance/duoqian_account_list_page.dart` | 个人 + 机构多签统一账户列表页 |
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
| `lib/governance/personal-manage/personal_manage_account_info_page.dart` | 个人多签账户详情页 |
| `lib/governance/personal-manage/personal_manage_discovery_service.dart` | 个人多签反向索引发现服务 |
| `lib/governance/personal-manage/personal_manage_service.dart` | PersonalManage 个人多签链上交互服务 |
| `lib/governance/personal-manage/personal_manage_storage_codec.dart` | PersonalManage storage key 与 SCALE 解码 |
| `lib/governance/personal-manage/personal_proposal_history_service.dart` | 个人多签提案历史聚合与 Isar 持久化 |
| `lib/isar/wallet_isar.dart` | 多签本地状态和最近链上同步时间复用 `AppKvEntity` |
| `lib/rpc/chain_rpc.dart` | 提供分块批量 storage 读取，供多签列表降低链上请求数量 |
| `organization-manage/src/lib.rs` | SFID 注册机构多签登记、创建、关闭业务逻辑 |
| `personal-manage/src/lib.rs` | 个人多签创建、关闭业务逻辑 |
| `duoqian-transfer/src/lib.rs` | 机构账户转账复用现有提案/投票/执行流程 |
| `votingengine/internal-vote/src/lib.rs` | 投票引擎（支持 ORG_REN / ORG_PUP / ORG_OTH 动态主体） |
| `votingengine/src/traits.rs` | `InternalVoteEngine` 语义化接口 |
| `runtime/src/configs/mod.rs` | `RuntimeInternalAdminProvider` + `RuntimeInternalAdminCountProvider` |

## 9. 源码对齐基线

- `lib/governance/shared/institution_info.dart + lib/governance/organization-manage/institution_registry.dart`
- `lib/governance/admins-change/services/institution_admin_service.dart`
- `lib/governance/organization-manage/institution_detail_page.dart`
- `lib/governance/governance_proposals_page.dart`
- `lib/governance/organization-manage/institution_admin_list_page.dart`
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
