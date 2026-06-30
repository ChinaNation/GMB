# citizen-identity 技术说明

## 定位

`citizen-identity` 是链上中国 runtime 的公民身份真源模块。

模块只负责保存可被链上投票引擎读取的公民身份，不保存本地档案全文。OnChina 负责注册局录入和交易发起，runtime 负责最终授权、钱包签名校验、链上身份状态、人口统计和快照。

## 身份级别

- 投票身份：用于公民参与投票，链上保存 `cid_number`、钱包账户、护照有效期、状态、居住省市镇。
- 参选身份：用于公民参选公职，在投票身份基础上增加出生省市镇和姓名。

## 授权边界

- 联邦注册局管理员可以在所管辖省份内登记、更新和注销公民链上身份。
- 市注册局管理员只能登记、更新和注销本市公民链上身份。
- 公民钱包签名是身份上链和修改的必要条件，注册局管理员不能绕过钱包签名替公民完成上链身份确认。

## 主要存储

- `VotingIdentityByAccount`：账户到投票身份。
- `CandidateIdentityByAccount`：账户到参选身份。
- `AccountByCid`：公民身份号到链上账户。
- `CountryVotingCount` / `ProvinceVotingCount` / `CityVotingCount` / `TownVotingCount`：按作用域维护投票人口。
- `PopulationSnapshots`：链上人口快照记录。

## 主要交易

- `register_voting_identity`：注册投票身份。
- `upgrade_to_candidate_identity`：升级为参选身份。
- `update_voting_identity`：更新投票身份。
- `update_candidate_identity`：更新参选身份。
- `revoke_identity`：注销链上身份。
- `start_population_snapshot`：按作用域生成链上人口快照。

## 投票引擎接入

投票引擎通过 `CitizenIdentityReader` 读取链上状态：

- `can_vote(who, scope)`：按账户和作用域判断投票资格。
- `can_be_candidate(who, scope)`：按账户和作用域判断参选资格。
- `population_count(scope)`：读取链上人口分母。

业务模块不得自行签发人口证明、资格证明或复刻投票资格逻辑。

## 发行回调

`citizen-issuance` 通过 `OnVotingIdentityRegistered` 回调发放一次性公民轻节点认证奖励。

奖励去重键为公民身份号哈希，避免同一公民身份重复领奖。

## 验收

- `cargo test -p citizen-identity`
- `cargo test -p citizen-issuance`
- `cargo test -p votingengine -p joint-vote -p legislation-vote`
- `cargo test -p citizenchain`
