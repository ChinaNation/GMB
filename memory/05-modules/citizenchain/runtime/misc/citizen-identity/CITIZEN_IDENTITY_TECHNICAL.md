# citizen-identity 技术说明

## 定位

`citizen-identity` 是链上中国 runtime 的公民身份真源模块。

模块只负责保存可被链上投票引擎读取的公民身份和四级人口数据，不保存本地档案全文，也不生成提案快照。OnChina 负责注册局录入和交易发起，runtime 负责最终授权、钱包签名校验和链上身份状态；投票引擎消费本模块的人口数据并形成自己的不可变提案快照。

## 身份级别

- 投票身份：用于公民参与投票，以永久 `cid_number` 为 storage key，身份值保存护照有效期、状态、居住省市镇；当前签名钱包由独立双向绑定保存。
- 参选身份：用于公民参选公职，在投票身份基础上增加出生省市镇、`family_name`、`given_name`、性别和**出生日期**（`birth_date`，YYYYMMDD 整数）。出生日期是注册局新增公民时必填、写入后不可修改的字段，链上凭此实时计算竞选公民年龄（见 `candidate_age`）。投票身份不含出生日期，姓名不得拼接成第三个字段。
- 公民逻辑主体统一为 `CitizenSubject { cid_number, wallet_account }`。该结构不新增 storage；读取时由钱包反向绑定取得永久 CID，再校验 CID 主键身份、CID 到钱包正向绑定、身份状态和 CID Active 状态。任一缺失、吊销或错配均 fail-closed。投票票据、候选快照、计票和当选结果必须恢复成完整公民主体。
- 竞选身份的 `family_name`、`given_name` 分开保存，各自最多 128 字节且必须非空；不保存合并姓名，也不保留带公民前缀的姓名别名。

## 授权边界

- 联邦注册局管理员可以在所管辖省份内登记、更新和注销公民链上身份。
- 市注册局管理员只能登记、更新和注销本市公民链上身份。
- 公民钱包签名是身份上链和修改的必要条件，注册局管理员不能绕过钱包签名替公民完成上链身份确认。

## 主要存储

正式创世目标和当前代码统一为 `StorageVersion = 0`。正式链尚未创世，不包含开发期迁移、旧字段双读或兼容分支，后续结构调整也不得递增版本。

- `VotingIdentityByCid`：永久公民 CID 到投票身份；身份更新只覆盖同一 CID 下的当前版本。
- `CandidateIdentityByCid`：永久公民 CID 到参选身份；更换当前签名钱包不得搬迁资料。
- `WalletAccountByCid`：永久公民 CID 到当前唯一签名钱包。
- `CidByWalletAccount`：当前签名钱包到永久公民 CID；必须与 `WalletAccountByCid` 严格闭环。
- `CountryVotingCount` / `ProvinceVotingCount` / `CityVotingCount` / `TownVotingCount`：按作用域维护就绪日期内状态正常且护照有效的投票人口。
- 人口日期变化计划与处理游标：有界处理护照生效、到期、吊销和迁居产生的人口变化；当天尚未就绪时拒绝提供新提案人口数据。
- `NextEligibilityRevision`：全局单调资格修订号，区分同一区块内多次身份写入。
- `VotingEligibilityVersionCount` / `VotingEligibilityVersions`：每个永久 CID 的不可变投票身份版本历史。
- `CountryVotingCount` 等人口计数与身份版本历史共同组成 `PopulationData` 的唯一真源；本模块没有 `PopulationSnapshots`、`NextSnapshotId` 或 proposal 绑定 storage。

## 主要交易

- `register_voting_identity`：注册投票身份。
- `upgrade_to_candidate_identity`：升级为参选身份。
- `update_voting_identity`：更新投票身份。
- `update_candidate_identity`：更新参选身份。
- `revoke_identity`：注销链上身份。

人口数据读取不是公开交易。已删除的 call index 5 永久留洞，不得复用；任何模块都不得在 citizen-identity 恢复提案快照或 snapshot_id。

## 投票引擎接入

投票引擎通过 `CitizenIdentityProvider` 读取链上状态：

- `citizen_subject(account)`：返回经过 CID↔钱包双向绑定、正常身份状态和 Active CID 校验的完整 `CitizenSubject`；本接口已落地，消费端不得从裸钱包自行拼接主体。
- 第 5、6 步将把投票引擎资格和票据接口切换到完整 `CitizenSubject`，不能只返回 bool 后让投票模块以裸钱包写票。
- 投票资格：由当前钱包解析永久 CID，再按双向绑定、状态、护照日期和作用域解析完整公民主体。
- 参选资格：在完整投票资格基础上校验参选身份必填字段并返回完整公民主体。
- `candidate_age(account)`：读取参选身份 `birth_date` 并按链上当前日期（UTC+8）实时计算周岁；无参选身份、时间戳未初始化或出生日期落在未来返回 `None`（fail-closed）。任何调用方可据链上公开的出生日期计算竞选公民年龄。
- `population_data(scope)`：返回作用域、当前人口分母、资格 revision 和 UTC+8 护照判定日期；这些字段必须在一次读取中保持一致。
- `can_vote_at(who, population_data)`：由当前钱包解析永久 CID，按 revision 二分定位该 CID 在提案创建时的身份版本，再校验状态、护照日期和作用域。

投票引擎收到 `PopulationData` 后自行写入 `ProposalPopulationSnapshots[proposal_id]`。提案清理只删除投票引擎快照；身份历史继续作为身份审计记录保留。

业务模块不得自行签发人口证明、资格证明或复刻投票资格逻辑。

## 快照一致性边界

- 提案创建后的人口增长、迁居、资料更新或撤销不会改变既有提案的成员资格；新提案读取更新后的 revision 和分母。
- 快照查询复杂度与单个永久 CID 的身份变更次数相关，为 `O(log versions)`，不遍历行政区全部公民。
- 最终人口计数器必须同时满足状态正常和护照在就绪日期有效。护照生效与到期通过有界日期变化计划维护；人口尚未推进到快照判定日期时必须拒绝建案。当前只按状态计数的开发期实现不是正式创世口径。

## 发行回调

`citizen-issuance` 通过 `OnVotingIdentityRegistered` 回调发放一次性公民轻节点认证奖励。

奖励去重键为公民身份号哈希，避免同一公民身份重复领奖。

## CID 节点永久边界

`CidRegistry` 的正常写入口仍由本 pallet 执行；节点 `core/node_guard/cid_lifecycle.rs` 以 RAW storage 独立背书下列永久规则：

- CID 记录写入后不得删除；
- `VotingIdentityByCid` 不得删除，身份资料及资格历史不得从一个 CID 迁移到另一个 CID；
- `WalletAccountByCid` 与 `CidByWalletAccount` 只表达当前签名钱包，必须一一对应并与 CID 主键身份闭环；
- `registrar_account`、`commitment`、居住省市和 `registered_at` 不得通过 runtime 升级换主体；
- 只允许 `Active → Revoked`，`Revoked` 为不可恢复终态；
- CID 必须持续是合法 `CTZN` 家族号，登记/吊销高度不得指向未来。

因此 runtime 可以继续维护正常业务校验，但不能通过 `setCode` 恢复已吊销 CID 或复用号码。

## 验收

- `cargo test -p citizen-identity`
- `cargo test -p citizen-issuance`
- `cargo test -p votingengine -p joint-vote -p legislation-vote`
- `cargo test -p citizenchain`

2026-07-21 第 3 步最终验收：`citizen-identity` 30 项、runtime 46 项、NodeGuard 公民发行 8 项和 CID 生命周期 3 项测试通过；`no_std`、`wasm32v1-none`、runtime benchmark/try-runtime 和 Rust 1.94.0 固定工具链 release Node 构建通过。当前源码 `citizenchain-fresh --tmp` 真实启动成功，block #0 为 `0x45144d74a7af61bb25cc08a803a19af1cdc946b007d22c774ce3acdeeebd7db4`，state root 为 `0xe916b283c7cd017aa87d2bfda2b835298195d2cbfc53c19536d0fddeae9874ea`，`peers=0`、`isSyncing=false`，runtime 六项项目版本均为 `0`，metadata 二进制 215,796 字节；节点已停止，未生成正式 chainspec。
