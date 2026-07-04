# citizen-issuance 技术说明

## 定位

`citizen-issuance` 是公民轻节点认证奖励模块。

模块不提供外部交易，只在 `citizen-identity` 成功登记投票身份后，通过 `OnVotingIdentityRegistered` 回调发放一次性奖励。

## 触发链路

1. 注册局管理员在 OnChina 发起链上投票身份上链交易。
2. `citizen-identity` 校验注册局权限、公民钱包签名、身份号和居住地作用域。
3. 投票身份写入链上。
4. `citizen-identity` 调用 `OnVotingIdentityRegistered::on_voting_identity_registered(who, cid_number_hash)`。
5. `citizen-issuance` 按 `cid_number_hash` 去重后发放奖励。

## 存储

- `IdentityRewardClaimed`：已领取奖励的公民身份号哈希集合。
- `TotalIssued`：累计发行奖励金额。

## 事件

- `CertificationRewardIssued { who, cid_number_hash, reward }`
- `CertificationRewardSkipped { who, cid_number_hash, reason }`

## 去重规则

同一个 `cid_number_hash` 只能触发一次奖励。账户更换或重新登记不能让同一公民身份重复领奖。

## 验收

- `cargo test -p citizen-issuance`
- `cargo test -p citizen-identity -p citizen-issuance`
