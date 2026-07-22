# citizen-issuance 技术说明

## 1. 定位

`citizen-issuance` 是公民投票身份首次认证奖励模块。模块不提供外部交易，只接收
`citizen-identity` 的 `OnVotingIdentityRegistered` 回调。

奖励金额、档位人数、总人数上限和一次性规则来自 `primitives::citizen_const`。这些规则
已同步实现于节点原生 `NodeGuard`，runtime 升级可以改变代码，但不能让遵守当前节点
二进制的节点接受突破永久规则的区块。

## 2. 同块两阶段结算

身份登记 extrinsic 与实际铸发必须在同一区块完成，但分为两个可独立验证阶段：

1. `citizen-identity` 校验注册局权限、公民钱包签名、CID 和居住地作用域，写入投票身份；
2. 回调校验永久与本块临时双重防重、人数上限和奖励非零，只写入待发队列；
3. 节点从 finalize 前视图读取完整队列，独立复核首次身份、CID 哈希、反向索引、领取资格和奖励档位；
4. runtime 在同块 `on_finalize` 逐项 `deposit_creating`，写入永久防重状态与累计人数并清空临时状态；
5. 节点把公民奖励和全节点 PoW 奖励汇总为同一个 finalize 发行计划，精确核对账户与总发行变化。

回调阶段不会提前改变余额。这样既保持“登记成功同块到账”的产品语义，又避免 extrinsic
内的手续费、转账或其他余额变化掩盖奖励金额。

## 3. 存储

永久状态：

- `RewardedCount`：累计成功领取人数，同时决定下一笔奖励所处档位；
- `IdentityRewardClaimed`：按 `cid_number_hash` 防止同一公民身份重复领取；
- `AccountRewarded`：按钱包账户防止换绑 CID 后重复领取。

仅在本块 finalize 前存在的临时状态：

- `PendingRewardCount`：本块待发数量；
- `PendingRewards[index]`：连续序号对应的 `(who, cid_number_hash)`；
- `PendingIdentityRewardClaimed`：本块 CID 哈希防重；
- `PendingAccountRewarded`：本块账户防重。

`on_finalize` 后四类临时状态必须全部删除；缺号、重复、残留、非规范 key 或提前改写永久
状态均会被节点拒绝。

## 4. 奖励与防重规则

- `index < CITIZEN_ISSUANCE_HIGH_REWARD_COUNT` 时使用高额奖励；之后使用常规奖励；
- `RewardedCount` 不得超过 `CITIZEN_ISSUANCE_MAX_COUNT`；
- 同一 CID 哈希和同一账户都只能成功领取一次；
- 同块重复登记由临时防重表拦截，跨块重复由永久防重表拦截；
- 节点从编译期常量和父状态累计数逐项推导金额，不信任 runtime 事件或 metadata；
- 同一账户同时是本块矿工和新公民时，两笔奖励必须按账户求和后精确到账。

## 5. 事件

- `CertificationRewardIssued { who, cid_number_hash, reward }`：实际铸发并写入永久状态后发出；
- `CertificationRewardSkipped { who, cid_number_hash, reason }`：回调因永久/临时重复、上限或金额转换失败而跳过。

`Balances::Issued` 在对应 `CertificationRewardIssued` 之前发生；事件只用于审计，不是节点守卫的信任输入。

## 6. Weight 与 benchmark

回调 weight 同时覆盖排队和同块 `on_finalize` 的最坏路径。`src/benchmarks.rs` 真实执行回调与
finalize，`src/weights.rs` 由 Substrate benchmark CLI 重新生成；当前测量为 7 次读取、8 次写入，
估算 proof size 3,593 bytes。不得用手写估值替代生成权重。

## 7. 节点永久守卫

`citizenchain/node/src/core/node_guard/citizen_issuance.rs` 使用 RAW storage key 和节点本地 SCALE
镜像，不读取 runtime metadata。它检查：

- 创世只能包含 FRAME 规范空状态：存储版本 0、两个计数的精确零值；
- 父状态不得残留待发队列，finalize 前队列必须连续且不超过本块 extrinsic 数；
- 身份必须首次以 `VotingIdentityByCid` 出现，CID 哈希、`WalletAccountByCid` 与 `CidByWalletAccount` 必须双向一致；
- 永久和临时双重防重、人数边界、档位金额必须逐项一致；
- finalize 后队列必须清空，永久标记和累计数必须精确推进；
- 未登记的 `CitizenIssuance` key 变化、收款账户变化或总发行变化一律 fail-closed。

## 8. 验收基线（2026-07-10）

- `citizen-issuance` 单元测试：13/13 通过；
- `integration_citizen_identity`：5/5 通过；
- `node_guard`：38/38 通过；`constitution`：38/38 通过；
- runtime benchmark feature 编译及 release benchmark 实跑通过；
- 当前源码 WASM 的隔离双节点真实登记 CID `GD000-CTZN6-616532784-2026`；矿工节点产出
  block#1，禁用挖矿的全节点通过 WSS 导入相同哈希
  `0x702e65e7b64ae7df80dbfb1e16e99ea9909ba302628c3c9d6fc722f6714050c5`；
- `RewardedCount=1`，待发计数不存在，身份、CID/账户永久防重标记均存在；
- Alice 同时是矿工和新公民：两笔奖励合计 1,999,800 分，扣身份登记制度费 100 分后，
  free balance 净增 1,999,700 分；
- 第二轮由 Alice 出块、Bob 作为新公民：block#1 双端哈希一致为
  `0x26d751b62ef23cc5d5884153c1782f67a5922b1d2246f16c5e610e5e034823a6`；Alice 获 PoW
  奖励并支付登记费后净增 999,800 分，Bob 新账户精确收到公民奖励 999,900 分；
- 临时 chainspec、节点数据库、测试签名代码和测试密钥材料已全部删除。
