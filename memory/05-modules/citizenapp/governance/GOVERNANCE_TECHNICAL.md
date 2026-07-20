# Governance 治理模块技术文档（区块链规范版）

> 机构分类唯一真源 = CID 机构码（institution_code），见 [[ADR-025]]。

> ⚠️ **目录已迁移(2026-06-29,本文部分路径过时待重写)**:citizenapp 机构模块按"机构管理 vs 交易业务"重组——
> ① 机构身份/账户/管理员**只读**链访问 + ADR-028 统一机构模型 = `lib/citizen/institution/`(原 `lib/transaction/organization-manage/` 已删,服务改名 `InstitutionChainService`,storage 前缀经 `InstitutionPalletRouter` 路由 PublicManage(30)/PrivateManage(31),取代 OrganizationManage);
> ② 多签转账(公私个共用)= `lib/transaction/multisig-transfer/`(原 `lib/citizen/proposal/transaction/`);
> ③ 机构(公权+私权)创建/关闭已收归 onchina 控制台 + 冷钱包,citizenapp 不再发起;个人多签管理留 `lib/transaction/personal-manage/`。
> 本文下方出现的 `organization-manage` / `OrganizationManage` / `proposal/transaction` 路径按上述对应关系理解,待按新结构整篇重写。

## 1. 模块目标

`lib/citizen/` 负责 CitizenApp 底部“公民”Tab 及链上治理能力，覆盖：

- 提案（proposal）发起
- 投票（vote）提交
- 提案状态跟踪与结果展示

说明：

- 本文档定义的是“链上字段/格式/标准/流程”。
- 当前 App 已接入 runtime 升级、转账等主要治理路径，本文同时作为现有实现与后续扩展的对齐基线。

当前实现目录已经按公民、提案、交易管理与投票引擎边界重排：

```text
lib/8964/
  square_tab_page.dart
lib/citizen/
  citizen_tab_page.dart
  all/
  legislation/
  election/
  governance/
  public/
  proposal/
  shared/
lib/transaction/
  organization-manage/
  personal-manage/
lib/votingengine/
  internal-vote/
  joint-vote/
  legislation-vote/
```

`organization-manage` 代表注册机构多签账户的多签管理能力，归属 `lib/transaction/organization-manage/`，不作为公民提案三级目录预留。统一发起提案入口在 `lib/citizen/proposal/`，按 `ProposalSubject + ProposalCapabilityRegistry` 判断展示能力。

治理页本地状态边界：

- 治理机构、提案、管理员列表等链上信息不能因为本机钱包库短暂 busy 而整页“加载失败”。
- `ProposalContextResolver` 读取本地钱包失败时返回空管理员钱包列表，并保留链上机构/提案内容展示。
- 机构详情页单独读取冷钱包管理员匹配关系；该读取失败只影响“当前用户是否管理员”的本地提示，不影响机构余额、管理员名单和提案列表。
- 所有治理模块读写 Isar 必须走 `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()`，不得直接取 `WalletIsar.instance.db()`。
- 治理列表和详情页的展示数据分三层：本地静态机构常量、本机 Isar 持久化展示快照、链上 runtime 真值。页面首屏只能依赖前两层；链上读取放到后台 TTL 刷新、下拉刷新、返回刷新或提交前复核。
- `ProposalLocalStore` 保存提案列表摘要和机构详情页索引；公民-提案列表的可见范围由默认公共机构码和当前钱包订阅机构实时过滤，不使用全局治理索引。`ProposalDetailLocalStore` 保存转账提案、多签管理提案、Runtime 升级提案详情快照；两者都只服务展示，不得作为投票/执行/提交前校验的最终真相。
- `AdminAccountService` 只保存机构 `admins` 人员名册和本地签名钱包匹配短缓存，不产生业务权限。提交投票前，机构内部/联合提案必须重新读取 `EffectiveVoterSnapshot`、提案状态和对应账户投票记录；个人多签才读取 `AdminSnapshot`。
- 合格选民投票记录必须批量读取：内部投票走 `InternalVoteQueryService.fetchAdminVotesBatch()`，联合投票走 `RuntimeUpgradeService.fetchJointAdminVotesBatch()`；方法和 runtime storage 的稳定旧名不构成管理员授权语义。

## 2. 链上入口与权限边界

### 2.1 关键约束（必须遵守）

- `votingengine` 的 `create_internal_proposal`、`create_joint_proposal` 和 `internal_vote` 外部调用被禁用，直接调用会返回 `NoPermission`。
- 联合提案必须由业务治理 pallet 通过 `JointVoteEngine` trait 发起。
- 内部投票必须由业务治理 pallet 通过 `InternalVoteEngine` trait 转发。

### 2.2 可直接由交易发起的投票引擎入口

- `JointVote::cast_admin(proposal_id, actor_cid_number, approve)`（稳定 call/storage 旧名；资格来自岗位有效选民快照）
- `cast_referendum(proposal_id, approve)`

## 3. 通用字段与格式标准

### 3.1 基础类型

| 字段 | 链上类型 | App 传输规范 |
| --- | --- | --- |
| `account` | `AccountId32` | SS58 地址字符串（当前链 `ss58 = 2027`） |
| `institution` | `[u8; 48]` | `0x` + 96 hex（机构 pallet id） |
| `proposal_id` | `u64` | 全局单调主键(双层 ID v1)。展示号 `(year, seq_in_year)` 通过 `votingengine::ProposalDisplayId[id]` 反查表持有,App 渲染为 `2026000123` 风格(年份 + 6 位补零序号),与主键解耦 |
| `approve` | `bool` | `true/false` |

### 3.2 枚举与编码

- `institution_code`（CID 机构码，`[u8;4]`）：分类经谓词派生——`is_fixed_governance_code`（固定治理档：NRC/PRC/PRB/FRG/NJD）、`is_personal_code`（`PMUL` 个人多签）、`is_institution_code`（公权或私权法人机构账户码；链上不再区分公权/私权）。
- proposal kind：`0 = internal`，`1 = joint`。
- stage：`0 = internal`，`1 = joint`，`2 = citizen`。
- status：`0 = voting`，`1 = passed`，`2 = rejected`。

### 3.3 时效与阈值

- 单阶段投票时长：`VOTING_DURATION_BLOCKS`（当前为 30 天对应区块数）。
- 内部投票通过阈值：
  - NRC：`13`（硬编码）
  - PRC：`6`（硬编码）
  - PRB：`6`（硬编码）
  - 普通注册机构：链上 `internal-vote::ActiveInstitutionThresholds[cid_number]` 动态读取
  - 个人多签：链上 `internal-vote::ActivePersonalThresholds[personal_account]` 动态读取
- 联合投票权重：
  - NRC：`19`
  - 每个 PRC：`1`
  - 每个 PRB：`1`
  - 总票权：`105`
- 联合机构阈值（不是岗位阈值）：
  - NRC：`13`
  - PRC：`6`
  - PRB：`6`
- 联合投票阶段中，`VotePlan` 对应岗位的有效选民直接上链投票：
  - 某机构赞成票达到该机构阈值时，链上自动形成该机构 `yes`
  - 若该机构剩余岗位快照选民已不足以让赞成票达到阈值，链上自动形成该机构 `no`
- 联合投票 `yes >= 105` 立即通过；任一机构形成 `no` 时立即转入联合公投；否则超时后进入联合公投阶段。
- 联合公投通过规则：`yes * 100 > eligible_total * 50`（严格大于 50%）。

## 4. 提案字段规范（按业务类型）

| 业务类型 | 提案入口 | 必填字段 | 发起权限 | 投票入口 |
| --- | --- | --- | --- | --- |
| 决议发行 | `propose_resolution_issuance` | `actor_cid_number, reason, total_amount, allocations[]` | NRC/43 PRC `COMMITTEE_MEMBER` 岗位有效任职人 | 联合+公民 |
| 协议升级 | `propose_runtime_upgrade` | `actor_cid_number, reason, code, new_pow_params` | NRC/43 PRC `COMMITTEE_MEMBER` 岗位有效任职人 | 联合+公民 |
| 个人多签管理员集合变更 | `propose_admin_set_change` | `institution_code=PMUL, personal_account, admins[], new_threshold` | 个人多签当前管理员 | 内部 |
| 决议销毁 | `propose_destroy` | `actor_cid_number, proposer_role_code, institution_account, amount` | 目标 CID 中拥有 `res-dst/0 Propose` 的岗位有效任职人 | 内部 |
| GRANDPA 密钥更换 | `propose_replace_grandpa_key` | `actor_cid_number, proposer_role_code, institution_account, new_key(32B)` | 目标 CID 中拥有 `gra-key/0 Propose` 的委员岗位有效任职人 | 内部 |
| 省储行业务治理(已下线) | ~~`propose_institution_rate / propose_verify_key / propose_sweep_to_main / propose_relay_submitters`~~ | Step 2b-iv-b 随老省储行清算 pallet 一起从 runtime 删除 | — | — |
| 清算行费率治理(新) | `propose_l2_fee_rate(call_index 40)` / `set_max_l2_fee_rate(call_index 41, Root)` | `bank, new_rate_bp` | 清算行管理员 / Root | — |

### 4.1 联合提案投票引擎字段标准

业务模块和客户端都不接收人口快照材料，也不提交独立 `PopulationScope` 参数。联合公投固定全国作用域；立法特别案由业务模块从 actor CID 推导国家/省/市作用域。投票引擎从 citizen-identity 读取一致的 `PopulationData` 后按 proposal_id 生成快照。

- `PopulationScope`：全国、省、市、镇四级内部作用域；人口数据由 `citizen-identity` 唯一提供，提案快照由投票引擎保存。
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

- `propose_l2_fee_rate(actor_cid_number, institution_account, new_rate_bp)`(call_index 40):
  - `institution_account` 必须是 `actor_cid_number` 下的清算行主账户；主账户仅为费率配置对象
  - 签名者必须属于 `AdminAccounts[actor_cid_number].admins`（`CidAccountQuery::is_institution_admin`）
  - `new_rate_bp` 范围 `[1, min(MaxL2FeeRateBp, 10)]`(默认上限 10 bp = 0.1%)
  - 成功后写 `L2FeeRateProposed[bank] = (rate, now + 1680 块)`（按六分钟平均目标换算的 7 天制度区块数）
  - `on_initialize` 每块扫描,到期后自动搬到 `L2FeeRateBp[bank]` 并发 `L2FeeRateActivated` 事件
- `set_max_l2_fee_rate(new_max)`(call_index 41,Root Origin):
  - 调整全局费率上限 `MaxL2FeeRateBp`,范围 `[1, 10]` bp
  - Step 2b 起将改为由联合投票回调(免费调用)
- `propose_verify_key` / `propose_sweep_to_main` / `propose_relay_submitters`
  在清算行体系下均无等价 Call。验签密钥由清算行多签管理员的 sr25519 私钥本地持有
  (offchain_keystore),不再走链上提案;手续费划转(sweep)仍由
  `multisig-transfer` pallet 的 `propose_sweep_to_main(call_index 5)` 治理。

## 5. 投票字段规范

### 5.1 内部投票（业务 pallet）

内部投票业务入口统一字段：

- `proposal_id: u64`
- `approve: bool`

统一函数：

- `InternalVote::cast(proposal_id, approve)`

说明：

- `admins-change` / `resolution-destro` / `grandpakey-change` 等业务 pallet 不再保留各自的 `vote_*` 投票入口。
- `multisig-transfer` 的转账、安全基金和划转提案也统一走 `InternalVote::cast`。
- ~~`vote_institution_rate` / `vote_verify_key` / `vote_relay_submitters`~~(Step 2b-iv-b 已下线,随老省储行 pallet 一起从 runtime 删除)

### 5.2 联合机构投票（投票引擎）

`joint_vote` 字段：

- `proposal_id: u64`
- `institution: [u8;48]`
- `approve: bool`

权限要求：

- 必须由“当前联邦注册局机构管理员个人钱包”直接提交，不能跨机构代投。
- 同一管理员对同一 `proposal_id + institution` 只能投一次。
- 链上按机构当前管理员门限自动结算机构结果，不再需要额外 `approvals proof` 或机构多签提交。

### 5.3 联合公投（联合投票第二阶段）

`cast_referendum` 字段：

- `proposal_id: u64`
- `approve: bool`

防重放要求：

- 同一 `proposal_id + account` 只能投一次。
- 投票资格由 runtime 从链上公民身份按人口作用域读取。

## 6. 标准流程

### 6.1 提案发起流程（App 侧）

1. 选择业务类型并收集业务字段。
2. App 只做签名钱包和岗位码的输入完整性校验；业务发起权限最终由 runtime 按 `CID + 岗位码 + BusinessActionId + Propose` 校验，管理员登录/激活态不能替代岗位权限。
3. 组装链上业务调用字段并签名提交；联合公投和立法特别案的人口作用域由 runtime
   固定或按 actor CID 推导，投票引擎在建案事务内读取人口数据并创建快照，App 不发送作用域或独立快照交易。
4. 记录 `proposal_id` 与业务类型映射，订阅状态事件。

### 6.2 投票流程（App 侧）

1. 根据提案类型匹配投票入口（内部/联合/选举/立法）。
2. 采集投票字段并做本地格式校验；机构联合/内部投票只允许提案创建时冻结的岗位有效选民钱包上链，个人多签内部投票使用独立管理员快照。
3. 发起签名并提交交易；交易 nonce 必须每次签名前实时读取 runtime `frame_system::Account.nonce`，App 不得缓存、自增、预占或回滚 nonce。
4. 投票是否成功必须由 runtime 投票引擎 storage 确认：
  - 内部投票读取 `InternalVote::InternalVotesByAccount(proposal_id, admin)`。
  - 联合投票读取 `JointVote::JointVotesByAdmin(proposal_id, institution, admin)`。
  - 联合公投读取 `JointVote::ReferendumVotesByAccount(proposal_id, account)`。
5. `author_submitExtrinsic` 返回 txHash、交易池 watch 的 `inBlock/finalized`、本地 pending 记录都不能单独代表“已投票”；内部投票和联合投票提交后必须回读对应 runtime 投票 storage。
6. 监听事件刷新状态：
  - `InternalVoteCast / JointAdminVoteCast / JointInstitutionVoteFinalized / ReferendumVoteCast`
  - `ProposalAdvancedToReferendum`
  - `ProposalFinalized`

待确认投票处理规则：

- 提交投票时，服务层先等待交易 `inBlock / finalized`，再回读 runtime 投票 storage；新成功流程不再写本地 pending，只清理旧残留 pending。
- 如果 runtime 已记录该选民投票，清除 pending，并把该合格选民显示为已投票。
- 如果交易池 watch 返回 `timeout / finalityTimeout / retracted / future / error`，不得直接清除 pending，也不得把选民恢复成未投票；必须继续以 runtime 投票 storage 为准。
- 如果 runtime 无投票记录且 pending 超过 20 分钟确认窗口，视为本地提交没有形成有效投票，清除 pending 并允许重新提交，不能让选民明细无限显示“投票中”。
- 服务层完成入块和 runtime 投票记录确认后，底部按钮停止 `submitting` 转圈；详情页立即把该选民显示为已投票，`_load()` 只后台刷新展示状态，不得把 txHash 当作投票成功。
- 联合投票读取 `JointVote` storage 时，机构参数必须使用统一 `AccountId` 编码；citizenapp 只能调用 `institutionIdentityToPalletId()`，不得在页面内手写 cid `[u8;48]` 编码。

#### 6.2.1 协议升级提案在 App 里的联合投票实现

- `RuntimeUpgradeDetailPage` 从机构页进入时必须带上：
  - `institution`
  - `adminWallets`
- 页面必须读取 `VotingEngine::EffectiveVoterSnapshot[(proposal_id, actor_cid_number)]`，再从本机已导入签名钱包中筛选快照成员；不得用当前 `AdminAccounts` 或当前岗位任职回算历史提案资格。
- 联合投票按钮只在以下条件全部满足时启用：
  - 提案仍处于 `joint` 阶段且状态为 `voting`
  - 当前机构尚未投票
- 当前用户已导入至少一个仍未投票的本机构岗位快照选民钱包
- App 使用所选岗位快照选民钱包提交 `JointVote::cast_admin(proposal_id, actor_cid_number, approve)`；稳定 call 名不表示 admins 获得授权。
- 页面会读取：
  - `JointInstitutionTallies` 展示本机构当前赞成/反对岗位选民票数
  - `JointVotesByInstitution` 展示本机构是否已经形成最终机构结果
  - `JointVotesByAdmin` 判断当前导入岗位选民钱包是否已投票（稳定 storage 旧名）
- 页面展示的联合投票阈值不再写死 `3`，而是显示链上的联合权重阈值 `105`。
- 页面还会单独展示“本机构岗位快照选民投票进度 / 本机构阈值”，避免把联合权重阈值、机构阈值和不存在的岗位阈值混淆。

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
| `InternalTallies` / `JointInstitutionTallies` / `JointTallies` / `ReferendumTallies` | 投票计数 | 投票引擎 |
| `InternalVotesByAccount` / `JointVotesByAdmin` / `JointVotesByInstitution` / `ReferendumVotesByAccount` | 投票记录 | 投票引擎 |
| `ActiveProposalsBySubject` | 每机构活跃提案列表（上限由 runtime 配置，当前生产值 10） | 投票引擎 |

**自动清理策略（统一清理路径）：**
- 提案完成（通过/拒绝/过期）时注册延迟清理：`schedule_cleanup(proposal_id, current_block)`
- 清理时间 = 完成时区块 + **90 天**区块数
- 如果目标区块的队列已满（50 个），自动顺延到下一个区块，保证不丢失
- 每区块 `on_initialize` 检查 `CleanupQueue[当前区块]`，到期后触发清理
- 每区块最多触发 **5 个**提案进入清理流程，未处理完的保留在队列中，下个区块继续
- 实际数据删除委托给 `PendingProposalCleanups` 分块状态机，保证大量投票记录（如联合公投上万条）能分多个区块完成
- 清理状态机阶段：`InternalVotes → JointAdminVotes → JointInstitutionVotes → JointInstitutionTallies → JointReferendumVotes → LegislationVotes → ElectionVotes → ProposalObject → FinalCleanup`
- 提案结束（通过/拒绝/过期）时，活跃提案名额在 `set_status_and_emit` 中**立即释放**，不依赖业务模块

### 6.5 App 侧链路失败展示约束

- 治理列表、机构详情、提案详情读取链上数据时，如果轻节点未初始化、未同步完成或链路降级，必须显示“加载失败 / 轻节点不可用”。
- 不允许把轻节点读取失败降级成“暂无提案”“暂无管理员”“机构不存在”这类空态。
- 提案相关页面可继续把“链上 key 确实不存在”解释为空数据，但必须与“轻节点不可用”严格区分。
- `on_initialize` weight 使用预估最大值（`cleanup_limit` 次读写），确保不超出声明的 weight

**清理范围（全部）：**

| Storage | 说明 |
| --- | --- |
| `Proposals` | 提案基本信息 |
| `ProposalData` | 业务摘要（转账、销毁、runtime 升级等提案类型） |
| `ProposalObjectMeta` / `ProposalObject` | 业务大对象（如 runtime wasm） |
| `ProposalMeta` | 辅助元数据 |
| `InternalTallies` / `JointInstitutionTallies` / `JointTallies` / `ReferendumTallies` | 投票计数 |
| `InternalVotesByAccount` / `JointVotesByAdmin` / `JointVotesByInstitution` / `ReferendumVotesByAccount` | 投票记录 |
| `PendingProposalCleanups` | 分块清理游标 |
| `ActiveProposalsBySubject`（兜底移除） | 活跃提案列表 |

**查询时效：**
- 90 天内：可查完整投票细节和业务详情
- 90 天后：仅可通过区块中的交易记录和事件查询
- 永久：区块中的交易记录和事件不受影响

**业务模块改造：**
- 所有模块的本地提案动作、创建块、通过块、单机构活跃提案等旧 Storage 已删除
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
2. 机构读取 `PublicAdmins / PrivateAdmins::AdminAccounts[cid_number]` 管理员人员集合；个人多签独立按 `personal_account` 读取 `PersonalAdmins`。两类人员每项均为 `admin_account + family_name + given_name`。
3. 机构 Storage key 格式：`twox_128(pallet_name) + twox_128("AdminAccounts") + blake2_128(cid_number) + SCALE(cid_number)`。
4. 机构管理员值严格解码为 `institution_code + Vec<Admin>`，`Admin` 顺序固定为账户、姓、名；CID 只在 storage key，不在 value；不含状态、`kind`、岗位、来源或创建时间。
5. App 再从对应 entity 的 `InstitutionRoles` 与 `InstitutionRoleAssignments` 读取岗位和有效任职，以管理员人员集合为左侧做联合展示；没有岗位的管理员不丢失。
6. 阈值不来自 `AdminAccounts`；固定治理机构使用制度阈值，普通机构从 `InternalVote.ActiveInstitutionThresholds[cid_number]` 读取。
7. 当前钱包属于机构 `admins` 只能被识别为可激活的机构签名人员；业务发起和投票授权必须继续读取完整岗位任职、岗位业务权限或提案岗位快照，姓名不参与授权。
8. 投票执行返回或下拉刷新时清除对应账户短缓存并重新读取 finalized 状态。

### 7.1.1 个人多签管理员更换协议

- 目录边界：`lib/citizen/proposal/admins-change/` 只承接个人多签管理员集合变更；机构创建、注销和个人多签生命周期仍归各自 entity 业务。
- 主体规则：只接受 `PersonalAccount + PMUL`；公权、私权、固定治理和非法人机构一律拒绝进入该流程。
- call data：`[PersonalAdmins pallet][propose_admin_set_change call][institution_code=PMUL][account_id:32][admins:Compact<Vec<Admin>>][new_threshold:u32_le]`，每个 `Admin` 按 `admin_account + family_name + given_name` 编码。
- 个人多签动态阈值校验为 `threshold * 2 > admins_len && threshold <= admins_len`。
- QR_V1 仍只携带 `b.a + b.d`；CitizenApp 产生三字段 call data，CitizenWallet 已按 `admin_account + family_name + given_name` 严格解码并在确认页合并显示姓名。旧纯账户与旧合并姓名布局均拒签，同一次更换只产生一次最终交易签名。
- 机构岗位或任职变化由具体业务模块形成治理结果后写入 entity，且只能引用既有管理员；岗位不得派生机构 `admins`。CitizenApp 当前不提供机构管理员集合直接变更入口。
- `institution_info_test.dart` 必须同时断言 `PMUL` 允许 `adminsChange`，且固定治理、公权、私权和非法人机构均不允许该能力，防止旧机构管理员直改入口回归。

### 7.2 机构详情页结构

机构详情页（`InstitutionDetailPage`）自上而下包含以下区域：

0. **机构账户信息卡**：身份 ID、主账户、制度账户类型和内部门槛均为本地固定数据，进入页面立即显示；主账户余额是链上 finalized 动态数据，独立后台读取并只更新余额字段。
1. **顶部机构卡片**：左侧机构图标 + 中间机构类型标签与管理员/阈值信息。
   - 机构类型、阈值来自本地制度常量。
   - 管理员人数来自链上对应管理员 pallet 的 `AdminAccounts`，读取中或读取失败时只更新副标题，不阻塞页面。
   - 已激活机构签名钱包：可进入提案类型页面并填写岗位码；是否能发起由 runtime 岗位授权最终裁决。
   - 未激活机构签名钱包：可查看入口，但提交按钮禁用并提示先激活签名钱包；不得表述为“激活管理员即有权限”。
2. **机构签名人员标识**：只说明当前钱包属于 `admins` 人员名册，不承诺任何业务权限；岗位和业务权限必须独立展示/校验。
3. **管理员列表入口**：所有用户可见，点击进入管理员列表页。
4. **投票事件列表**：所有用户可见，显示“本机构内部提案 + 所有机构都可见的联合投票提案”，按 ID 倒序展示。协议升级等联合投票提案必须在所有机构入口可见，不能只挂在国家储委会单一列表下。

### 7.2.1 机构详情页数据来源与刷新边界

| 数据 | 来源 | 刷新方式 | 首屏策略 |
| --- | --- | --- | --- |
| 机构全称/简称、类型、身份 ID | 本地静态注册表 | 随 App 版本更新 | 立即显示 |
| 主账户、费用账户、安全基金账户、永久质押账户 | 本地静态注册表 | 随 App 版本更新 | 立即显示 |
| 治理机构内部门槛 | 本地制度常量 | 随 App 版本更新 | 立即显示 |
| 管理员列表和管理员人数 | 链上对应管理员 pallet 的 `AdminAccounts` | 后台读取，30 秒内存短缓存，下拉刷新强制更新 | 显示“读取中/读取失败”副标题 |
| 当前用户管理员身份 | 本地钱包 + 本地激活记录 + 链上管理员列表 | 后台读取，激活/返回/下拉刷新后更新 | 显示身份确认中，不挡住页面 |
| 主账户余额 | finalized 链上账户余额 | 后台读取，短缓存，下拉刷新强制更新 | 余额字段显示“读取中” |
| 机构可见提案列表 | 链上机构提案索引 + 年度联合提案索引 | 后台读取，短缓存，下拉刷新/提案详情返回后强制更新 | 提案区局部加载 |
| 更多制度账户余额 | finalized 链上账户余额 | 用户展开后按需读取 | 不进入首屏请求 |

治理机构详情页不得使用单个 `_loading` 等待全部链上请求。管理员、余额、提案任一读取失败时，只能影响对应区域；固定本地数据必须始终可见。余额缓存只允许写入 finalized 读取结果，不得把 best 视图余额写入同一展示缓存。

### 7.2.2 提案列表本地持久化读库

治理机构详情页和公民-提案的提案列表使用 `ProposalLocalStore` 保存展示摘要；机构详情页继续使用机构索引，公民-提案按当前钱包可见范围重新过滤：

| 本地持久化内容 | 存储位置 | 用途 |
| --- | --- | --- |
| `LocalProposalSummary` | Isar `AppKvEntity(governance.proposal.summary.<proposal_id>)` | 提案卡片首屏展示摘要 |
| 单机构提案 ID 索引 | Isar `AppKvEntity(governance.proposal.index.institution.<cid_number>)` | 治理机构详情页提案列表 |

本地摘要包含 `proposalId / displayId / kind / stage / status / internalCode /
institutionBytes / subjectCidNumbers / cidFullName / title / subtitle / iconKind /
updatedAtMillis`。这些字段只用于列表展示和首屏恢复，不作为链上真相。

链上同步规则：

- 治理机构详情页先读本地机构索引和摘要；本地为空、索引超过 5 分钟、用户下拉刷新、发起/查看提案返回时，再读取当前年提案缓存并按 `subject_cid_numbers` 包含本机构 CID 过滤。
- 公民-提案从当前年提案缓存中过滤可见提案：默认机构码固定为 `NRC/NLG/NSN/NRP/NED/NJD/NSP/PRS`，订阅范围按当前热钱包订阅公权机构的 CID 精确命中 `subject_cid_numbers`；省储委会、省储行不在默认集合内，只有订阅对应机构时才展示。
- 新区块订阅只做节流检查，当前最短 60 秒刷新一次公民-提案可见 ID 列表；不得每个新区块都全量重算，也不得按订阅机构码放大到同类所有机构。
- 提案详情点击时，如果当前只有本地摘要，才按单个 `proposalId` 读取链上详情并回写本地摘要。
- `ProposalData` 和展示摘要创建后变化很少，优先持久化复用；`status`、红点、投票记录、票数进度等动态字段按短 TTL 或用户操作触发链上刷新。

硬边界：

- 本地持久化读库只服务列表展示，不允许替代 runtime storage。
- 投票按钮可用性、是否已投票、提交前状态校验、执行状态判断，仍必须读取链上 `VotingEngine / InternalVote / JointVote / 各管理员 pallet` 相关 storage。

### 7.2.3 公民提案列表

公民页面的"提案"子 tab 展示当前钱包可见的统一提案流，按提案 ID 倒序（新的在上）。默认机构码为 `NRC/NLG/NSN/NRP/NED/NJD/NSP/PRS`；其它公权机构只在当前钱包订阅对应机构时进入列表。

**四层优化架构**：

| 层 | 说明 | 文件 |
| --- | --- | --- |
| 新区块订阅 | `chain_subscribeNewHeads` 监听新区块，但对索引检查做 60 秒节流，避免每块全量读取 | `lib/rpc/chain_event_subscription.dart` |
| 本地持久化读库 | `LocalProposalSummary` + 机构索引持久化，App 重启后可复用摘要；公民-提案可见范围实时过滤 | `lib/citizen/shared/proposal/proposal_local_store.dart` |
| 本地内存缓存 | ProposalMeta / TransferProposalInfo / RuntimeUpgradeProposalInfo 缓存，避免同一会话重复 RPC | `lib/citizen/shared/proposal/proposal_cache.dart` |
| 批量查询 | `state_queryStorageAt` 一次 RPC 查多个 key，减少网络往返 | `chain_rpc.dart::fetchStorageBatch` |
| 分页加载 | 首屏 10 个，ScrollController 滚动触底加载更多 | `lib/citizen/all/proposal_view.dart` |

**数据流**：读取当前钱包订阅机构 CID → 从当前年提案缓存中过滤默认机构码和 `subject_cid_numbers` 命中的提案 → 首屏 10 条批量查询详情和上下文 → 写回本地摘要和内存缓存 → 新区块后台节流检查，有新提案再插入顶部。

**提案类型识别**：
- 内部提案：按转账等内部提案数据结构解码。
- 联合提案：按 `meta.kind == 1` 单独走联合提案解码链路，协议升级提案进入 `runtime_upgrade_detail_page.dart`。
- 未接入专用详情页的联合提案，列表仍需可见，至少保留通用联合提案卡片。

**投票权判断**：
- 遍历用户所有钱包 → 比对每个机构提案创建时的 `EffectiveVoterSnapshot`；个人多签才比对 `AdminSnapshot`
- 钱包属于对应快照、尚未投票且状态为投票中 → 提案卡片显示红点
- 进入详情页后：有投票权显示投票按钮，无投票权不显示

**红点通知**：
- 提案卡片右上角：该提案需要用户投票 → 红点
- 底部"公民"tab 图标：汇总待投票数 → 带数字红点

**投票权类型（分阶段实现）**：

| 类型 | 判断依据 | 状态 |
| --- | --- | --- |
| 机构岗位投票 | 钱包属于 NRC/PRC 委员或 PRB 董事岗位有效选民快照 | ✅ 已实现 |
| 联合公投 | 钱包具备链上投票资格 | ⏭️ 后期 |
| 机构联合投票 | 钱包属于当前机构的岗位有效选民快照，且尚未对本机构投票 | ✅ 已实现 |

关键文件：`lib/citizen/vote/vote_view.dart`

### 7.3 权限控制规则

| 用户身份 | 可访问页面/功能 |
| --- | --- |
| `admins` 名册成员且具有目标岗位权限 | 机构详情页、管理员列表、投票事件列表、提案类型页面；runtime 允许发起/投票 |
| `admins` 名册成员但无目标岗位权限 | 可浏览和激活签名钱包，runtime 拒绝目标业务发起/投票 |
| 非 `admins` 成员 | 机构详情页、管理员列表、投票事件列表 |

核心原则：`admins` 只限定机构人员和可任职范围，不授予业务权限；机构提案必须由完整 `CID + 岗位码` 对应的有效任职人发起，机构投票必须命中提案创建时岗位快照。

### 7.3.1 活跃提案数量限制

每个机构（`AccountId`）同时最多允许 runtime 配置数量的活跃提案（当前生产值为 **10**），不区分业务提案类型，由投票引擎（`votingengine::limit`，原 `active_proposal_limit`）统一管控。

- 创建提案时：`try_add_active_proposal()` 检查并添加
- 提案完成时：`remove_active_proposal()` 在 `set_status_and_emit` 中立即释放（提案通过/拒绝/过期时）
- App 端发起提案前异步查询活跃数，达上限弹窗提示"提案数量已达上限"

关键文件：`votingengine/src/limit.rs`

### 7.4 提案类型页面

提案类型页面（`ProposalTypesPage`）根据机构类型条件展示可发起的提案：

**通用提案（所有机构类型）：**
- 转账：从机构多签账户发起转账
- 决议销毁：提议销毁机构持有的资产

**国家储委会专属提案（仅 NRC）：**
- 决议发行：发起公民币发行决议，需联合投票，未全票通过或超时进入联合公投
- 验证密钥：更换 GRANDPA 共识验证密钥
- 协议升级：协议升级提案详情展示与投票入口，提案发起只在 node 管理端

提交成功后，提案类型页应把创建结果向上冒泡到机构详情页，触发列表刷新，避免“链上已创建但机构页仍停留旧状态”。

### 7.5 多签转账边界

多签转账不属于 governance/proposal 的实现范围。CitizenApp 端创建、详情、投票、
列表适配、余额提示、缓存和页面跳转统一由
`memory/05-modules/citizenapp/transaction/multisig-transfer/MULTISIG_TRANSFER_APP_TECHNICAL.md`
管理。

governance 侧只允许保留通用提案列表、机构详情页挂载点、投票上下文和
`InternalVote::cast` 共享能力；不得在 governance 文档或代码中重新描述
`MultisigTransfer::propose_*` 的字段、页面、service 或投票实现。

#### 7.5.1 转出资金账户（mainAccount / accounts）

`InstitutionInfo` 对治理机构使用 `InstitutionAccounts` 表达制度账户：
- `mainAccount`：主账户，转账提案的默认转出账户
- `feeAccount`：费用账户
- `safetyFundAccount`：安全基金账户，仅国家储委会显示
- `stakeAccount`：永久质押账户，仅省储行显示

个人多签和机构账户可通过 `InstitutionInfo` 传入账户，但业务侧统一读取
`InstitutionInfo.mainAccount`。治理机构不得再用其他字段表达主账户。
通过 `Keyring().encodeAddress(bytes, 2027)` 转为 SS58 地址展示。

治理机构全称/简称、身份 ID、制度账户和治理机构固定阈值由
`scripts/generate_citizenapp_governance_registry.mjs` 从 runtime primitives 生成到
`lib/transaction/organization-manage/governance_institution_registry.generated.dart` 并由
`InstitutionInfo` 派生展示。管理员列表不写入静态注册表，必须动态读取链上
对应管理员 pallet 的 `AdminAccounts`。

治理机构列表页（`lib/citizen/governance/governance_tab.dart`）只负责本机展示顺序：
国家储委会直接展示，单张卡片横跨整行显示到右侧边缘且高度与省储委会/省储行卡片一致；省储委会、省储行默认折叠，
标题行最右侧用线性右箭头/下箭头展开后按静态注册表顺序展示。
省储委会分组标题图案使用 `assets/icons/government-line.svg`；省储行分组标题图案使用
`assets/icons/bank.svg`；机构卡片内部不显示全称/简称左侧图标，只显示机构简称和右箭头。
用户长按省储委会或省储行卡片拖拽时，只能在所属分组内排序；排序以
`SharedPreferences` 保存 `cidNumber` 列表到本机，不写链、不跨设备同步。
本页不得再按管理员机构优先做 `_sorted()` 自动排序；管理员身份只影响卡片高亮，
不改变展示顺序。

治理机构详情页的账户信息区直接展示身份 ID 和主账户；主账户 finalized 余额后台读取并仅更新余额
字段。更多制度账户不再进入二级页面，而是在当前账户信息卡内点击箭头展开。展开项按机构
实际存在的 `feeAccount / safetyFundAccount / stakeAccount` 懒加载 finalized 链上余额，分别显示
费用账户、安全基金账户和永久质押账户。

治理机构详情页的提案列表先读取 `ProposalLocalStore` 中的机构索引和摘要；链上刷新成功后
回写本地摘要与机构索引。该列表展示的本地摘要允许短暂落后链上状态，但点击提案详情、投票、
执行和提交前校验必须重新读取链上详情和投票状态。

### 7.6 管理员列表页面

管理员列表页面（`AdminListPage`）展示：
- 机构简称与类型标签
- 管理员总数与通过阈值
- 每位管理员的完整 SS58 地址（format 2027），当前用户标记"我"
- 地址一键复制功能

### 7.6 机构标识编码

`institution_data.dart` 统一输出 D/ADR-015 `AccountId32`：内置治理机构为 `0x01 + cid_number UTF-8 + 右零填充`；个人多签为 `0x03 + AccountId32 + 15B 零填充`；注册机构账户为 `0x05 + AccountId32 + 15B 零填充`。`0x02 注册机构归属关系` 只保留给同一 CID 机构下多账户归属/检索，不作为转账支出主体。

### 7.7 关键文件

| 文件 | 说明 |
| --- | --- |
| `lib/citizen/citizen_tab_page.dart` | 公民 Tab 二级导航入口（投票 / 治理 / 机构） |
| `lib/citizen/vote/vote_view.dart` | 投票二级页，全局治理提案列表与待投票红点 |
| `lib/rpc/chain_event_subscription.dart` | WebSocket 链事件订阅（新区块通知 + 自动重连） |
| `lib/citizen/shared/institution_info.dart + lib/transaction/organization-manage/institution_registry.dart` | 治理机构静态注册表 + `findInstitutionByPalletId` 反查 + `formatProposalId` 格式化 |
| `lib/transaction/organization-manage/governance_institution_registry.generated.dart` | 从 runtime primitives 生成的治理机构身份 ID 与制度账户 |
| `lib/citizen/proposal/admins-change/services/institution_admin_service.dart` | 机构管理员只读门面：联合读取 admins 钱包集合与 entity 岗位任职 |
| `lib/citizen/institution/institution_detail_page.dart` | 统一机构详情页（管理员检测 + 账户信息内联展开 + 条件 UI + 投票事件列表） |
| `lib/citizen/shared/proposal/proposal_context.dart` | 用户与提案关系解析（管理员 / 公民 / 查看者） |
| `lib/citizen/shared/proposal/proposal_models.dart` | 多提案共用模型（ProposalMeta / ProposalWithDetail 等） |
| `lib/votingengine/internal-vote/internal_vote_service.dart` | 多提案共用内部投票提交服务 |
| `lib/votingengine/internal-vote/pending_vote_store.dart` | 多提案共用待确认投票记录 |
| `lib/votingengine/internal-vote/proposal_vote_widgets.dart` | 多提案共用投票 UI 组件 |
| `lib/citizen/shared/institution_manage_detail_page.dart` | 跨个人/机构的多签管理提案详情页 |
| `lib/citizen/proposal/proposal_entry_page.dart` | 统一提案类型选择页 |
| `lib/citizen/proposal/runtime-upgrade/runtime_upgrade_page.dart` | 协议升级说明页（不发起提案、不选择 WASM、不获取人口快照） |
| `lib/citizen/proposal/runtime-upgrade/runtime_upgrade_detail_page.dart` | 协议升级提案详情页（联合投票/联合公投进度） |
| `lib/citizen/proposal/runtime-upgrade/runtime_upgrade_service.dart` | 协议升级提案链上交互服务 |
| `lib/citizen/institution/institution_admin_list_page.dart` | 机构管理员列表页：按钱包聚合岗位、任期、来源和余额 |
| `lib/transaction/organization-manage/` | 机构多签层（机构账户创建、关闭、详情、机构管理服务、机构 storage codec、机构发现服务） |
| `lib/transaction/personal-manage/` | 个人多签层（个人列表、详情、发现、创建、关闭、管理员激活、提案历史、PersonalAdmins 链上编解码） |
| `lib/rpc/chain_rpc.dart` | RPC 服务（含 `fetchStorage` 公开方法） |
| `lib/main.dart` | App 壳、应用锁与底部导航；不再内联公民 Tab 业务页面 |

## 8. 注册多签账户（organization-manage / personal-manage）

### 8.1 概述

`organization-manage` 负责 CID 注册机构多签，`personal-manage` 负责个人多签。两者都复用投票引擎的内部投票机制，与固定治理档使用同一套投票、存储、清理基础设施。

### 8.2 机构类型

注册个人账户使用个人多签码（PMUL，`is_personal_code`）；注册机构账户使用机构账户码（`is_institution_code`，公权或私权法人），与固定治理档机构码（`is_fixed_governance_code`，NRC/PRC/PRB/FRG/NJD）并列。

个人多签主体是 32 字节 `personal_account`；机构主体是最大 32 字节的 CID。两者使用不同枚举分支，禁止把机构账户地址填入 CID 分支或以账户回落机构身份。

### 8.3 动态阈值与管理员

| 项目 | 固定治理机构 | 普通注册机构 / 个人多签 |
| --- | --- | --- |
| 管理员来源 | `public-admins::AdminAccounts[cid_number]` | 机构为 `public/private-admins::AdminAccounts[cid_number]`；个人为 `personal-admins::AdminAccounts[personal_account]` |
| 阈值来源 | 投票引擎固定制度常量 | 机构为 `ActiveInstitutionThresholds[cid_number]`；个人为 `ActivePersonalThresholds[personal_account]` |
| 管理员存储类型 | `AccountId` | `AccountId` |

投票引擎通过 `InternalAdminProvider` 查询管理员列表；动态阈值由 `internal-vote` 自己保存和读取。

### 8.4 Extrinsic

| Extrinsic | call_index | 说明 | 投票 |
| --- | --- | --- | --- |
| `OrganizationManage::propose_create_institution(..., institution_code, ...)` | 17.5 | 发起 CID 机构多签账户创建提案；机构账户管理员 institution_code 必须为机构账户码（`is_institution_code`） | 投票引擎 |
| `OrganizationManage::propose_close(account, beneficiary)` | 17.1 | 发起机构多签账户关闭提案 | 投票引擎 |
| `PersonalManage::propose_create(account_name, admins, regular_threshold, amount)` | 7.0 | 发起个人多签账户创建提案；普通阈值用户输入且必须过半，注册阈值固定全员同意 | 投票引擎 |
| `PersonalManage::propose_close(account, beneficiary)` | 7.1 | 发起个人多签账户关闭提案 | 投票引擎 |
| `PersonalAdmins::propose_admin_set_change(institution_code, account, admins, new_threshold)` | 29.0 | 发起个人多签管理员集合变更提案 | 投票引擎 |
| `InternalVote::cast(proposal_id, approve)` | 20.0 | 创建、关闭、转账等内部投票统一入口 | 统一投票入口 |

### 8.5 创建流程（Pending → Active）

1. App 创建前按链端口径校验发起钱包 free 余额覆盖 `初始资金 + 创建手续费 + ED`；
   创建手续费为 `max(初始资金 * 0.1%, 0.10 元)`，ED 当前为 `1.11 元`。
2. 管理员调用对应创建入口 → 写入机构或个人 pending storage + 投票引擎创建提案
3. App 不能把 txHash 当创建成功；必须等待交易入块，并在同一区块确认
   `PersonalAdmins.PersonalAccountProposed` 或
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
4. 链上历史事件和历史提案不删除；CitizenApp 本地继续在统一账户列表展示已注销账户，状态显示“已注销”，不显示余额；用户在详情页点击右上角“删除”后才清理本机数据。
5. Active 状态的个人/机构多签详情页右上角菜单只显示“关闭个人多签”或“关闭机构多签”文本项，不带删除图标；Closed 状态才显示“删除”并保留删除图标。

### 8.7 多签转账接入边界

多签账户管理目录只负责账户注册、创建、关闭、状态展示和管理员管理。
发起转账、转账详情、投票进度、余额提示、列表适配与详情跳转均由
`lib/transaction/multisig-transfer/` 实现。

`lib/transaction/organization-manage/` 不实现多签转账逻辑；账户详情页如需展示转账入口，
只允许挂载 `lib/transaction/multisig-transfer/multisig_transfer_entry.dart` 提供的入口组件。

### 8.8 手机端入口分流

2026-07-02 起，CitizenApp 删除底部 `多签` Tab，原位置改为底部 `广场` Tab。个人多签入口迁回交易 Tab：

- 交易 Tab 在链上支付表单上方显示一行双入口：左侧“扫码支付”，右侧“多签账户”。
- “扫码支付”保持现有扫码支付流程，不显示右箭头。
- “多签账户”进入 `lib/transaction/personal-manage/personal_account_list_page.dart`，顶部标题为“多签账户”。
- 个人多签账户列表只读取 `PersonalAccountEntity` 和 `PersonalAccountLocalState`，不读取或展示机构账户。
- 个人多签 discovery 只扫描 `PersonalAdmins.AdminAccounts`，并按 `kind=Personal`、`institution_code=PMUL`、本机管理员钱包过滤。
- 机构(公权/私权)注册、创建、关闭由 OnChina 注册局控制台 + 冷钱包处理；CitizenApp 不再提供机构多签注册、发现或列表展示入口。
- 个人多签详情页必须 local-first：读取 `PersonalAccountEntity`、`PersonalAccountLocalState` 和 `personal_account_detail:*`。这些都是本机持久化储存，不是内存缓存。
- 详情页本机快照可直接显示名称、地址、本地状态、管理员公钥列表、阈值和余额快照；链上只负责更新账户是否存在、Active/Pending 状态、管理员/阈值和 Active finalized 余额。
- 详情页状态 TTL 与余额 TTL 不得混用。列表页批量状态刷新写入详情快照时，必须保留已有 `balanceYuan` 和 `lastBalanceRefreshAtMillis`；Active 详情页若本地余额为空或余额 TTL 过期，应只调用余额读取，不重复拉管理员/阈值。
- 详情页进入时不得用全屏转圈等待 `fetchPersonalAccount()`；自动刷新必须复用 `fetchPersonalAccountsBatch([account])`，并受 TTL 控制。
- 详情页不展示“同步中”类 UI。下拉刷新、转账提案创建返回、投票返回、关闭返回才忽略 TTL 精准刷新当前账户；链上失败只保留本机已储存数据，不写成 Closed。
- Active 账户 60 分钟内不自动重复查链；Pending / Closed 账户 10 分钟内不自动重复查链；下拉刷新才忽略 TTL 强制刷新。
- 自动 discovery 只允许首次进入“多签账户”列表或本机钱包 pubkey fingerprint 变化时触发；不做每日自动扫描，也不增加单独扫描按钮。
- 下拉刷新先强制刷新已知个人多签账户状态，再强制执行个人多签 discovery。
- 创建、关闭、投票、删除返回时只刷新相关个人多签账户或本地记录，不重新扫描全部个人多签。
- 个人多签状态刷新由 `PersonalManageService.fetchPersonalAccountsBatch()` 分阶段批量读取 `PersonalAccounts / PersonalAdmins::AdminAccounts / ActivePersonalThresholds`；Pending 阈值按提案 ID 单独读取，不作账户回落。
- 批量路径通过 `ChainRpc.fetchStorageBatchChunked()` 分块读取 storage，列表页不得逐个账户循环调用详情查询。
- 个人多签账户列表右上角加号直接进入 `personal_account_create_page.dart`，不再弹出个人/机构选择。

发起转账提案不删除，
入口由 `lib/transaction/multisig-transfer/multisig_transfer_entry.dart` 提供，
具体页面和链上构造仍归 `lib/transaction/multisig-transfer/`。

2026-04-30 第二轮收口只迁移纯多签文件：账户管理模型/服务、账户详情、创建、
关闭、账户列表、账户详情和机构发现归入 `lib/organization-manage`；跨个人/机构的
多签管理提案详情当前收口到 `lib/citizen/shared/institution_manage_detail_page.dart`。
QR 协议、Isar schema、钱包流水、治理聚合页、机构通用服务和内部投票通用服务仍留在原模块目录；
多签转账相关文件统一归 `lib/transaction/multisig-transfer/`。

2026-04-30 第三轮收口删除治理提案类型页中的“创建多签/关闭多签”入口。多签创建
只能从 `机构多签` 或 `个人多签` 列表右上角进入；多签关闭只能从具体多签账户详情
页进入。机构多签关闭与个人多签关闭分别使用独立页面，不能再共用一个关闭入口。

2026-06-26 起，CitizenApp 个人多签主业务与 runtime `personal-admins` 对齐：
个人创建、关闭、管理员更换、管理员激活、待创建提案反查、提案历史、PersonalAdmins call data、
PersonalAdmins ProposalData 解码、`PersonalAdmins::PersonalAccounts` storage codec
统一放入 `lib/transaction/personal-manage/`。`lib/transaction/organization-manage/` 不再承载这些个人主业务；
机构多签创建、关闭、详情、发现和 OrganizationManage storage codec 统一放入 `lib/transaction/organization-manage/`。
个人/机构共用的详情、账户列表、提案上下文和链上管理员读取能力放入 `lib/citizen/shared/` 或 `lib/citizen/proposal/`。

2026-05-17 起，个人/机构多签创建类交易在入块后统一检查 `System.Events`：
先解析 `System.ExtrinsicFailed` 的 runtime 模块错误，再确认对应成功事件。
因此余额不足、管理员主体残留、机构账户索引占用等链上拒绝会显示真实
`PersonalAdmins / PublicAdmins / PrivateAdmins / OrganizationManage` 错误，不再落到“未找到成功事件”的泛化提示。

### 8.9 关键文件

| 文件 | 说明 |
| --- | --- |
| `lib/8964/square_tab_page.dart` | 底部广场 Tab 当前入口页 |
| `lib/citizen/all/proposal_view.dart` | 公民 Tab 内“提案”统一列表页：默认公共机构 + 当前钱包订阅公权机构 |
| `lib/transaction/transaction_tab_page.dart` | 交易 Tab 双入口编排：扫码支付 / 多签账户 |
| `lib/transaction/personal-manage/personal_account_list_page.dart` | 个人多签账户列表页 |
| `lib/citizen/shared/institution_manage_detail_page.dart` | 个人/机构多签管理提案共用投票详情页；业务解码委托对应 manage 服务 |
| `lib/transaction/personal-manage/personal_account_create_page.dart` | 个人多签创建表单 |
| `lib/transaction/personal-manage/personal_account_close_page.dart` | 个人多签关闭表单 |
| `lib/transaction/personal-manage/personal_admin_list_page.dart` | 个人多签管理员激活列表 |
| `lib/transaction/personal-manage/personal_manage_account_info_page.dart` | 个人多签账户详情页 |
| `lib/transaction/personal-manage/personal_manage_discovery_service.dart` | 个人多签反向索引发现服务 |
| `lib/transaction/personal-manage/personal_manage_service.dart` | PersonalAdmins 个人多签链上交互服务 |
| `lib/transaction/personal-manage/personal_manage_storage_codec.dart` | PersonalAdmins storage key 与 SCALE 解码 |
| `lib/transaction/personal-manage/personal_proposal_history_service.dart` | 个人多签提案历史聚合与 Isar 持久化 |
| `lib/isar/wallet_isar.dart` | 多签本地状态和最近链上同步时间复用 `AppKvEntity` |
| `lib/rpc/chain_rpc.dart` | 提供分块批量 storage 读取，供多签列表降低链上请求数量 |
| `organization-manage/src/lib.rs` | CID 注册机构多签登记、创建、关闭业务逻辑 |
| `personal-admins/src/lib.rs` | 个人多签创建、关闭、管理员更换业务逻辑 |
| `multisig-transfer/src/lib.rs` | 机构账户转账复用现有提案/投票/执行流程 |
| `votingengine/internal-vote/src/lib.rs` | 投票引擎（支持个人多签码 PMUL（`is_personal_code`）/ 机构账户码（`is_institution_code`）动态主体） |
| `votingengine/src/traits.rs` | `InternalVoteEngine` 语义化接口 |
| `runtime/src/configs/mod.rs` | `RuntimeInternalAdminProvider` + `RuntimeInternalAdminsLenProvider` |

## 9. 源码对齐基线

- `lib/citizen/shared/institution_info.dart + lib/transaction/organization-manage/institution_registry.dart`
- `lib/citizen/proposal/admins-change/services/institution_admin_service.dart`
- `lib/citizen/institution/institution_detail_page.dart`
- `lib/citizen/proposal/proposal_entry_page.dart`
- `lib/transaction/organization-manage/institution_admin_list_page.dart`
- `lib/citizen/shared/institution_manage_detail_page.dart`
- `lib/transaction/transaction_tab_page.dart`
- `lib/transaction/personal-manage/personal_account_list_page.dart`
- `lib/transaction/personal-manage/personal_account_create_page.dart`
- `lib/transaction/personal-manage/personal_account_close_page.dart`
- `lib/transaction/personal-manage/personal_admin_list_page.dart`
- `lib/transaction/personal-manage/personal_manage_account_info_page.dart`
- `lib/transaction/personal-manage/personal_manage_discovery_service.dart`
- `lib/transaction/personal-manage/personal_manage_service.dart`
- `lib/transaction/personal-manage/personal_manage_storage_codec.dart`
- `lib/transaction/personal-manage/personal_proposal_history_service.dart`
- `lib/rpc/chain_rpc.dart`
- `citizenchain/runtime/transaction/multisig-transfer/src/lib.rs`
- `citizenchain/runtime/private/organization-manage/src/lib.rs`
- `citizenchain/runtime/admins/personal-admins/src/lib.rs`
- `citizenchain/runtime/votingengine/src/lib.rs`
- `citizenchain/runtime/votingengine/src/internal_vote.rs`
- `citizenchain/runtime/votingengine/src/joint_vote.rs`
- `citizenchain/runtime/votingengine/joint-vote/src/jointreferendum.rs`
- `citizenchain/runtime/votingengine/src/cleanup.rs`
- `citizenchain/runtime/votingengine/src/limit.rs`
- `citizenchain/runtime/issuance/resolution-issuance/src/lib.rs`
- `citizenchain/runtime/governance/runtime-upgrade/src/lib.rs`
- `citizenchain/runtime/admins/public-admins/src/lib.rs`
- `citizenchain/runtime/admins/private-admins/src/lib.rs`
- `citizenchain/runtime/governance/resolution-destro/src/lib.rs`
- `citizenchain/runtime/governance/grandpakey-change/src/lib.rs`
- `citizenchain/runtime/transaction/offchain-transaction/src/lib.rs`
- `citizenchain/runtime/src/configs/mod.rs`
- `primitives/src/count_const.rs`
