# citizen-identity 技术说明

## 定位

`citizen-identity` 是链上中国 runtime 的公民身份真源模块。

模块只负责保存可被链上投票引擎读取的公民身份，不保存本地档案全文。OnChina 负责注册局录入和交易发起，runtime 负责最终授权、钱包签名校验、链上身份状态、人口统计和快照。

## 身份级别

- 投票身份：用于公民参与投票，链上保存 `cid_number`、钱包账户、护照有效期、状态、居住省市镇。
- 参选身份：用于公民参选公职，在投票身份基础上增加出生省市镇、姓名、性别和**出生日期**（`birth_date`，YYYYMMDD 整数）。出生日期是注册局新增公民时必填、写入后不可修改的字段，链上凭此实时计算竞选公民年龄（见 `candidate_age`）。投票身份不含出生日期。

## 授权边界

- 联邦注册局管理员可以在所管辖省份内登记、更新和注销公民链上身份。
- 市注册局管理员只能登记、更新和注销本市公民链上身份。
- 公民钱包签名是身份上链和修改的必要条件，注册局管理员不能绕过钱包签名替公民完成上链身份确认。

## 主要存储

当前 pallet 使用最终创世存储布局并显式声明 `StorageVersion = 1`；不包含开发期迁移、旧字段双读或兼容分支。

- `VotingIdentityByAccount`：账户到投票身份。
- `CandidateIdentityByAccount`：账户到参选身份。
- `AccountByCid`：公民身份号到链上账户。
- `CountryVotingCount` / `ProvinceVotingCount` / `CityVotingCount` / `TownVotingCount`：按作用域维护投票人口。
- `NextEligibilityRevision`：全局单调资格修订号，区分同一区块内多次身份写入。
- `VotingEligibilityVersionCount` / `VotingEligibilityVersions`：每账户不可变投票身份版本历史。
- `PopulationSnapshots`：作用域、分母、资格 revision、创建区块和护照判定日期的不可变快照元数据。

## 主要交易

- `register_voting_identity`：注册投票身份。
- `upgrade_to_candidate_identity`：升级为参选身份。
- `update_voting_identity`：更新投票身份。
- `update_candidate_identity`：更新参选身份。
- `revoke_identity`：注销链上身份。

人口快照不是公开交易。只有投票引擎可通过下述内部 trait 在提案创建事务中生成、
绑定和最终释放快照；已删除的 call index 5 永久留洞，不得复用。

## 投票引擎接入

投票引擎通过 `CitizenIdentityReader` 读取链上状态：

- `can_vote(who, scope)`：按账户和作用域判断投票资格。
- `can_be_candidate(who, scope)`：按账户和作用域判断参选资格。
- `candidate_age(account)`：读取参选身份 `birth_date` 并按链上当前日期（UTC+8）实时计算周岁；无参选身份、时间戳未初始化或出生日期落在未来返回 `None`（fail-closed）。任何调用方可据链上公开的出生日期计算竞选公民年龄。
- `population_count(scope)`：读取链上人口分母。
- `create_population_snapshot(scope)`：冻结分母、当前资格 revision 和 UTC+8 护照判定日期，返回 snapshot_id。
- `can_vote_at(who, snapshot_id)`：按 revision 二分定位账户创建时身份版本，再校验状态、护照日期和作用域。
- `release_population_snapshot(snapshot_id)`：提案历史清理后释放快照元数据；身份历史作为身份审计记录保留。

业务模块不得自行签发人口证明、资格证明或复刻投票资格逻辑。

## 快照一致性边界

- 提案创建后的人口增长、迁居、换号或撤销不会改变既有提案的成员资格；新提案读取更新后的 revision 和分母。
- 快照查询复杂度与单账户身份变更次数相关，为 `O(log versions)`，不遍历行政区全部公民。
- 当前人口计数器按 `CitizenStatus::Normal` 增量维护；护照过期没有独立链上定时事件，因此过期但尚未由注册局更新状态的公民仍计入宪法人口分母，但 `can_vote_at` 会按快照日期拒绝其投票。该口径不是“实时资格分子 + 旧人口分母”的漂移，而是公民人口分母与护照投票资格的既有区分。

## 发行回调

`citizen-issuance` 通过 `OnVotingIdentityRegistered` 回调发放一次性公民轻节点认证奖励。

奖励去重键为公民身份号哈希，避免同一公民身份重复领奖。

## CID 节点永久边界

`CidRegistry` 的正常写入口仍由本 pallet 执行；节点 `core/node_guard/cid_lifecycle.rs` 以 RAW storage 独立背书下列永久规则：

- CID 记录写入后不得删除；
- `registrar_account`、`commitment`、居住省市和 `registered_at` 不得通过 runtime 升级换主体；
- 只允许 `Active → Revoked`，`Revoked` 为不可恢复终态；
- CID 必须持续是合法 `CTZN` 家族号，登记/吊销高度不得指向未来。

因此 runtime 可以继续维护正常业务校验，但不能通过 `setCode` 恢复已吊销 CID 或复用号码。

## 验收

- `cargo test -p citizen-identity`
- `cargo test -p citizen-issuance`
- `cargo test -p votingengine -p joint-vote -p legislation-vote`
- `cargo test -p citizenchain`
