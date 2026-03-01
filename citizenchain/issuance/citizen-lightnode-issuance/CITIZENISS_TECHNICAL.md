# CITIZEN Lightnode Issuance Technical Notes

## 1. 模块定位
`citizen-lightnode-issuance` 是一个 FRAME pallet，用于在 SFID 绑定成功后发放“公民轻节点认证奖励”。

核心目标：
- 与 `sfid-code-auth` 解耦：本模块只负责奖励发行逻辑。
- 奖励规则链上可验证：阶段奖励、总量上限、一次性约束。
- 审计可观测：成功发奖与跳过原因均有事件。

代码位置：
- `/Users/rhett/GMB/citizenchain/issuance/citizen-lightnode-issuance/src/lib.rs`

---

## 2. 上游依赖与接线
上游模块：
- `/Users/rhett/GMB/citizenchain/otherpallet/sfid-code-auth/src/lib.rs`

接线方式：
- `sfid-code-auth::bind_sfid` 成功后调用 `T::OnSfidBound::on_sfid_bound(&who, sfid_hash)`。
- Runtime 中配置 `type OnSfidBound = CitizenLightnodeIssuance`：
  - `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`

Weight 集成：
- `sfid-code-auth` 在 `bind_sfid` 的 weight 中，叠加 `T::OnSfidBound::on_sfid_bound_weight()`。
- 本模块实现 `OnSfidBoundWeight` 并公开估算值，避免回调开销遗漏。

---

## 3. 规则常量（来源）
来源文件：
- `/Users/rhett/GMB/primitives/src/citizen_const.rs`

关键常量：
- `CITIZEN_LIGHTNODE_MAX_COUNT`：可获奖励公民轻节点总上限
- `CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT = 14_436_417`
- `CITIZEN_LIGHTNODE_HIGH_REWARD = 999_900`（单位：分）
- `CITIZEN_LIGHTNODE_NORMAL_REWARD = 99_900`（单位：分）
- `CITIZEN_LIGHTNODE_ONE_TIME_ONLY = true`

---

## 4. 存储结构
- `RewardedCount: u64`
  - 已发放奖励人数计数（全局累计）。
- `RewardClaimed: Map<Hash -> ()>`
  - SFID 级去重标记（存在即已领）。
- `AccountRewarded: Map<AccountId -> ()>`
  - 账户级去重标记（纵深防御，避免同账户换 SFID 重复领）。

说明：
- `RewardClaimed` 采用 `()` 而非 `bool`，以减少状态冗余。
- 检查去重使用 `contains_key`。

---

## 5. 事件与错误
事件：
- `CertificationRewardIssued { who, sfid_hash, reward }`
- `CertificationRewardSkipped { who, sfid_hash, reason }`

`SkipReason`：
- `DuplicateSfid`
- `MaxCountReached`
- `AccountAlreadyRewarded`

错误：
- `Error<T>` 当前为空（模块无外部可调用 extrinsic，核心流程由回调驱动）。

---

## 6. 核心流程
入口：
- `OnSfidBound::on_sfid_bound(who, sfid_hash)`

执行逻辑：
1. 先进行 SFID 去重检查（`RewardClaimed`）。
2. 再进行账户去重检查（`AccountRewarded`）。
3. 读取 `RewardedCount`，若达到 `MAX_COUNT` 则跳过。
4. 根据 `RewardedCount` 与 `HIGH_REWARD_COUNT` 决定高额/常规奖励。
5. `Currency::deposit_creating` 铸币到 `who`。
6. 写回 `RewardedCount += 1`，并写入两级去重标记。
7. 发事件：
   - 成功发 `CertificationRewardIssued`
   - 跳过发 `CertificationRewardSkipped`

---

## 7. Weight 策略
本模块暴露：
- `Pallet::on_sfid_bound_weight() -> Weight`
- `OnSfidBoundWeight::on_sfid_bound_weight()`（委托到上述方法，单一真相源）

当前估算：
- `T::DbWeight::get().reads_writes(5, 5)`（按最重成功路径保守估算）

说明：
- 该值用于上游 `bind_sfid` 申报 weight 叠加。
- 当前为估算法，后续可引入 benchmark 做精确化。

---

## 8. 安全性与制度语义
- 双重防重：
  - SFID 级：同一 SFID 只能领取一次。
  - 账户级：同一账户只能领取一次（即使后续换绑 SFID）。
- 总量约束：
  - 超过 `CITIZEN_LIGHTNODE_MAX_COUNT` 不再发放。
- 可审计性：
  - 跳过路径均有链上原因事件，不依赖链下日志推断。

---

## 9. 测试覆盖（当前）
`cargo test -p citizen-lightnode-issuance` 覆盖：
- 首次绑定发放高额奖励
- 达到 `MAX_COUNT` 后停止发放
- 相同 SFID 不重复发放
- `HIGH_REWARD_COUNT` 边界切换到常规奖励
- `HIGH_REWARD_COUNT - 1` 仍发高额奖励
- 成功发放事件断言
- 跳过事件断言（重复 SFID / 达上限 / 账户已领奖）
- 不同账户不同 SFID 独立发放
- 同一账户不同 SFID 仅首笔发放

---

## 10. 运维与审计建议
- 核查奖励问题时优先查看事件：
  - `CertificationRewardIssued`
  - `CertificationRewardSkipped`
- 若出现大量 `AccountAlreadyRewarded`，通常表示用户存在重绑行为或上游业务流程变化。
- 若需调整奖励规模或阶段阈值，应先更新 `primitives::citizen_const`，并同步评估：
  - 经济参数影响
  - 链上状态增长
  - Weight 估算是否仍保守
