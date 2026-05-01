# CITIZEN Issuance Technical Notes

## 0. 功能需求
### 0.1 核心职责
`citizen-issuance` 的功能需求是：
- 在 SFID 绑定成功后，按链上固定规则自动发放一次性“公民轻节点认证奖励”。
- 奖励发放逻辑必须与 `sfid-system` 解耦，由回调触发而非独立用户交易触发。
- 奖励结果必须可审计，成功与跳过都要有事件。

### 0.2 业务规则需求
- 同一 SFID 只能领奖一次。
- 同一账户只能领奖一次，即使后续换绑 SFID 也不能重复领奖。
- 奖励人数总量不得超过 `CITIZEN_ISSUANCE_MAX_COUNT`。
- 前 `CITIZEN_ISSUANCE_HIGH_REWARD_COUNT` 人与其后用户使用不同奖励档位。
- 模块不得提供人工补发、人工重试或人工改写领奖状态的外部治理入口，避免绕过一次性奖励约束。

### 0.3 计重与接线需求
- 模块不提供外部 extrinsic，全部逻辑由 `OnSfidBound` 回调驱动。
- 模块必须向上游暴露 `on_sfid_bound_weight()`，供 `sfid-system::bind_sfid` 叠加申报 weight。
- weight 的单一真相源必须来自本模块自身，而不是由上游模块手写估算。
- 上游只有在 SFID 绑定真正成功后才能触发奖励回调，模块自身不负责认证流程与身份校验。

---

## 1. 模块定位
`citizen-issuance` 是一个 FRAME pallet，用于在 SFID 绑定成功后发放“公民轻节点认证奖励”。

核心目标：
- 与 `sfid-system` 解耦：本模块只负责奖励发行逻辑。
- 奖励规则链上可验证：阶段奖励、总量上限、一次性约束。
- 审计可观测：成功发奖与跳过原因均有事件。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/issuance/citizen-issuance/src/lib.rs`

---

## 2. 上游依赖与接线
上游模块：
- `/Users/rhett/GMB/citizenchain/runtime/otherpallet/sfid-system/src/lib.rs`

接线方式：
- `sfid-system::bind_sfid` 成功后调用 `T::OnSfidBound::on_sfid_bound(&who, binding_id)`。
- Runtime 中配置 `type OnSfidBound = CitizenIssuance`：
  - `/Users/rhett/GMB/citizenchain/runtime/src/configs/mod.rs`
  - `type WeightInfo = citizen_issuance::weights::SubstrateWeight<Runtime>`

Weight 集成：
- `sfid-system` 在 `bind_sfid` 的 weight 中，叠加 `T::OnSfidBound::on_sfid_bound_weight()`。
- 本模块实现 `OnSfidBoundWeight` 并公开估算值，避免回调开销遗漏。

---

## 3. 规则常量（来源）
来源文件：
- `/Users/rhett/GMB/citizenchain/runtime/primitives/src/citizen_const.rs`

关键常量：
- `CITIZEN_ISSUANCE_MAX_COUNT`：可获奖励公民轻节点总上限
- `CITIZEN_ISSUANCE_HIGH_REWARD_COUNT = 14_436_417`
- `CITIZEN_ISSUANCE_HIGH_REWARD = 999_900`（单位：分）
- `CITIZEN_ISSUANCE_NORMAL_REWARD = 99_900`（单位：分）
- `CITIZEN_ISSUANCE_ONE_TIME_ONLY = true`
  - 代码中使用编译期断言强制该值恒为 `true`，运行时不存在“关闭一次性奖励”的分支。

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
- `CertificationRewardIssued { who, binding_id, reward }`
- `CertificationRewardSkipped { who, binding_id, reason }`

`SkipReason`：
- `DuplicateBindingId`
- `MaxCountReached`
- `AccountAlreadyRewarded`
- `ZeroRewardConfigured` — 奖励常量已由编译期断言锁定非零，该原因仅作为 `Balance` 类型转换后的防御性兜底。

错误：
- `Error<T>` 当前为空（模块无外部可调用 extrinsic，核心流程由回调驱动）。

---

## 6. 核心流程
入口：
- `OnSfidBound::on_sfid_bound(who, binding_id)`

执行逻辑：
1. 先进行 SFID 去重检查（`RewardClaimed`）。
2. 再进行账户去重检查（`AccountRewarded`）。
3. 读取 `RewardedCount`，若达到 `MAX_COUNT` 则跳过。
4. 根据 `RewardedCount` 与 `HIGH_REWARD_COUNT` 决定高额/常规奖励。
5. 将奖励常量转换为 `BalanceOf<T>`；高额/常规奖励常量在编译期断言必须非零，若类型转换后仍得到 0，则返回 `ZeroRewardConfigured` 并跳过，不推进状态。
6. `Currency::deposit_creating` 铸币到 `who`。
   - 这是模块设计内的主动增发，返回的 `PositiveImbalance` 会被有意丢弃。
7. 写回 `RewardedCount += 1`，并写入两级去重标记。
8. 发事件：
   - 成功发 `CertificationRewardIssued`
   - 跳过发 `CertificationRewardSkipped`

---

## 7. Weight 策略
本模块暴露：
- `Pallet::on_sfid_bound_weight() -> Weight`
- `OnSfidBoundWeight::on_sfid_bound_weight()`（委托到上述方法，单一真相源）
- `WeightInfo::on_sfid_bound()`（由 runtime 注入）

当前状态：
- `WeightInfo::on_sfid_bound()` 已由 Substrate benchmark CLI 自动生成（见 `src/weights.rs` 文件头，日期 2026-03-17）。

说明：
- 该值用于上游 `bind_sfid` 申报 weight 叠加。
- benchmark 入口当前覆盖首次成功发奖路径；重复领取、达到上限、零奖励配置等跳过路径未单独申报。
- 由于跳过路径的存储读写与状态推进都少于成功路径，当前成功路径 weight 可视为保守上界。

---

## 8. Benchmark 设计
- benchmark 入口：`on_sfid_bound`
- 覆盖口径：首次成功发奖路径（含 `deposit_creating`、计数写回、双重去重标记与事件）
- 未单独覆盖：重复领取、达到上限、零奖励配置等跳过路径
- 目的：校准上游 `bind_sfid` 叠加的回调 weight，而不是测用户交易本身
- Cargo feature：`runtime-benchmarks` 会向 `pallet-balances` 与 `sfid-system` 传播；`primitives` 当前不暴露 benchmark feature，不在传播列表中。

---

## 9. 安全性与制度语义
- 双重防重：
  - SFID 级：同一 SFID 只能领取一次。
  - 账户级：同一账户只能领取一次（即使后续换绑 SFID）。
- 总量约束：
  - 超过 `CITIZEN_ISSUANCE_MAX_COUNT` 不再发放。
- 可审计性：
  - 跳过路径均有链上原因事件，不依赖链下日志推断。
- 配置健壮性：
  - 高额/常规奖励常量通过编译期断言锁定非零；`ZeroRewardConfigured` 只保留为运行时 `Balance` 转换异常的防御性事件，而不是常规配置漂移路径。

---

## 10. 测试覆盖（当前）

### 10.1 单元测试
`cargo test -p citizen-issuance` 覆盖：
- 首次绑定发放高额奖励
- 达到 `MAX_COUNT` 后停止发放
- 相同 SFID 不重复发放
- `HIGH_REWARD_COUNT` 边界切换到常规奖励
- `HIGH_REWARD_COUNT - 1` 仍发高额奖励
- 成功发放事件断言
- 跳过事件断言（重复 SFID / 达上限 / 账户已领奖）
- 不同账户不同 SFID 独立发放
- 同一账户不同 SFID 仅首笔发放

### 10.2 集成测试
`cargo test -p citizen-issuance --test integration_bind_sfid` 覆盖：
- `bind_sfid` extrinsic → `OnSfidBound` → 奖励发放完整链路
- 同一账户换绑时奖励被跳过但绑定成功
- 不同账户独立绑定独立领奖
- 达到发放上限后绑定成功但奖励被跳过
- `bind_sfid` weight 声明包含回调 weight（非零）

---

## 11. 运维与审计建议
- 核查奖励问题时优先查看事件：
  - `CertificationRewardIssued`
  - `CertificationRewardSkipped`
- 若出现大量 `AccountAlreadyRewarded`，通常表示用户存在重绑行为或上游业务流程变化。
- 若需调整奖励规模或阶段阈值，应先更新 `primitives::citizen_const`，并同步评估：
  - 经济参数影响
  - 链上状态增长
  - Weight 估算是否仍保守
