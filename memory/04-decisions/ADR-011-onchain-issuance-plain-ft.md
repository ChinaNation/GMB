# ADR-011 链上发行代币(onchain-issuance Plain FT)协议规范

- 状态:Accepted(v3)
- 决议日期:2026-05-07(初稿)、2026-05-07 v2 修订(review 15 项)、2026-05-07 v3 修订(模块编号/call 同步对齐)
- 关联前置:ADR-010(SubjectKind 协议)、ADR-007(清算行三阶段)、ADR-008(SHENG 3-tier)
- 关联任务卡:`memory/08-tasks/open/20260507-onchain-issuance-plain-ft.md`

## 命名说明(v2 新增)

> 本 pallet 命名 `onchain-issuance`,与已有 `onchain-transaction` 在前缀字面有重叠,但语义不同:
> - `onchain-transaction`:链上交易手续费 pallet(已有)
> - `onchain-issuance`:**链上由用户主体(SFID 机构 / personal-manage 多签)发行代币**,"onchain" 在此特指"链上由用户在链端发行",不是"链上 vs 链下 issuance"对应。
>
> 与 `citizen-issuance / fullnode-issuance / resolution-issuance / shengbank-interest` 同处 `runtime/issuance/` 大类下,共属"GMB 发行业务族",但发行主体不同(citizen=公民认证 / fullnode=矿工铸块 / resolution=决议 / shengbank=省储行 / **onchain=用户**)。

---

## 背景

GMB 链当前只承载唯一原生币 GMB(`pallet_balances`)。业务侧出现"机构 / 个人多签发行自有代币"需求(积分、凭证、股权类等)。

业界公链(以太坊 ERC-20、Solana SPL/Token-2022)发币能力完全无监管,均由发行方自管。GMB 是国家级链,需要在链端结构性提供监管能力,但又不能让监管能力沦为发行方自治权。

## 决议

引入 **onchain-issuance** pallet 作为 **唯一外壳入口**,内核挂载 `pallet_assets`(Substrate 官方多资产模块),所有原生 extrinsic 由 `BaseCallFilter` 屏蔽。对外只暴露 SubjectId 入口(`SubjectKind = 0x04 OnchainAsset`),与 ADR-010 协议一致。

第一期范围 **Plain FT**(同质化代币,无锚定声明)。NFT / SFT / Pegged 留 Phase 2 单独 ADR。

---

### 一、发行人范围

| 主体类型 | SubjectKind | 是否允许发行 |
|---|---|---|
| Builtin(NRC/PRC/PRB 等内置主体) | 0x01 | 否(避免国家主体下场发币) |
| SfidInstitution(SFID 注册机构) | 0x02 | **是** |
| PersonalDuoqian(个人多签) | 0x03 | **是** |
| OnchainAsset(本协议位) | 0x04 | (仅 storage key 派生用,非发行人主体) |
| 裸账户(单签) | n/a | 否 |

链端在 `propose_issue` 入口 `ensure!(parse_subject_id(issuer).0 ∈ {0x02, 0x03})`,任何其他主体一律 reject。

### 二、协议位 SubjectKind 0x04 OnchainAsset(永久 ABI,v2 简化)

```
SubjectId(用户代币) = [u8; 48]
布局:
  byte[0]:    0x04                         (OnchainAsset)
  byte[1..5]: asset_id (u32 LE, 4B)        (pallet_assets 内部 AssetId)
  byte[5..48]: 43B 零填充(预留)
```

> v2 修订记录:**v1 曾设计 8B issuer_subject_short(blake2_128 摘要) + 4B asset_id**,review 时识别为冗余:
> 1. issuer_subject_short 8B 摘要不可逆,**无法**反查发行人完整身份;
> 2. asset_id(NextAssetId 自增)本身已全局唯一,SubjectId 已结构性互斥;
> 3. 真正反查发行人走 `Assets[SubjectId].issuer_subject_id` 字段(48B 完整保留)。
>
> 简化后 payload 只包含 4B asset_id,helper 函数无需引入哈希依赖。

`asset_id` 用于桥接 `pallet_assets` 内核;客户端无需哈希计算,从 SubjectId byte[1..5] 直接取 u32 LE 反推 asset_id。

### 三、资产种类 — Plain only(第一期)

```rust
pub enum AssetClass {
    Plain,    // 第一期唯一允许值
    // Pegged(PegDeclaration),  Phase 2 启用
}
```

链端在 `propose_issue` 入口强制 `class = Plain`,Pegged 路径 reject。

### 四、GMB 唯一性铁律

- 用户代币不可作 gas / 不可参与 fullnode-issuance / citizen-issuance / shengbank-interest 等任何系统发行流程。
- 所有 onchain-issuance 业务 propose 的计费**只收 GMB**。
- 用户代币的 metadata 内禁止含「自费率 / token gas」等字段。

---

### 五、监管六条铁律(v2 修订:NRC 主体识别 / propose origin / metadata 不可改)

#### 5.1 NRC 强制 monitor(不可关闭)— v2 字段去除

monitor 主体由链端**全局**强制为 NRC,**不在每个资产 storage 中存** `monitor_subject_id` 字段(v1 错误地在 OnchainAssetMeta 中冗余了一个 48B 字段,review 后去除)。

NRC 主体识别走 runtime 提供的 `NrcMainAccountProvider` trait(返回 `china_cb[0].main_address`),**与创建费收款方的 `NrcFeeAccountProvider`(返回 `china_cb[0].fee_address`)语义分离**。v1 错误复用同一个 trait,review 后拆为两个:

| trait | 返回值 | 用途 |
|---|---|---|
| `NrcMainAccountProvider` | `china_cb[0].main_address`(治理多签地址) | monitor 调用方校验 / 发起 JointVote 监管提案 |
| `NrcFeeAccountProvider`  | `china_cb[0].fee_address`(费用账户)     | 1000 GMB 创建费收款方 / GMB 转账目标 |

NRC 持有以下 5 项权力(任何代币不可关闭):
- `monitor_freeze_account(asset_id, who, reason_hash)`
- `monitor_unfreeze_account(asset_id, who, reason_hash)`
- `monitor_confiscate(asset_id, who, amount, reason_hash)`(强制 burn)
- `monitor_force_transfer(asset_id, from, to, amount, reason_hash)`
- `monitor_force_close(asset_id, reason_hash)`(整币封禁,30 天后销毁)

#### 5.2 第一期只允许 Plain 资产

`AssetClass::Pegged` 协议位预留,extrinsic 入口 reject。

#### 5.3 链端字符串黑名单

`name` / `symbol` / `description` 字段写入前过黑名单,命中任意词一律 reject:

| 类别 | 词例 |
|---|---|
| 法币 | 元 / RMB / CNY / ¥ / 人民币 / $ / USD / 美元 / 欧元 / 日元 / 港币 / HKD |
| 锚定 | 锚定 / 稳定 / stable / peg / 对标 / 等值 / 1:1 |
| 权威 | 央行 / 国家 / 官方 / official / authorized / 监管 |
| 数字货币 | 数字人民币 / DCEP / e-CNY / CBDC |

黑名单 storage 化(`Blacklist: StorageValue<BoundedVec<BoundedVec<u8, 32>, 256>>`),由 RuntimeUpgrade 投票添词/删词,不可 sudo。

#### 5.4 业务审批 — propose_X extrinsic + InternalVote 投票(v3 订正)

> v3 修订记录:**v2 误把"业务 pallet 不暴露 wrapper extrinsic"扩到 propose_X**,
> 与 GMB 现有架构(duoqian-transfer / personal-manage / organization-manage 全部都暴露
> propose_X extrinsic)严重背离。`unified_voting_entry_phase4` 铁律实际上是
> **删除 execute/cancel wrapper extrinsic**(由 VotingEngine 9.4/9.5 统一承载),
> 但 **propose_X 必须保留** — 它承载 proposer origin 校验、押金 reserve、业务参数透传等动作。

业务 pallet 暴露 5 个 `propose_X` extrinsic(call_index = 0..=4):

| 动作 | call_index | 引擎(底层) | propose origin 校验 |
|---|---|---|---|
| `propose_issue(IssueProposal)` | 0 | InternalVote(内部 admin 多签) | `ensure!(proposer ∈ admins(issuer_subject))`+ `Currency::reserve(1000 GMB)` |
| `propose_mint(MintProposal)` | 1 | InternalVote | `ensure!(proposer ∈ admins(asset.issuer_subject))` |
| `propose_burn(BurnProposal)` | 2 | InternalVote | 同上 |
| `propose_close(CloseProposal)` | 3 | InternalVote | 同上 |
| `propose_transfer(TransferProposal)` | 4 | InternalVote | 同上 |

`admins(subject)` 即 `admins-change::Subjects::get(subject_id)`,InternalVote::cast 阶段已校验 admin 投票身份,propose 入口额外 ensure 防止任意账户消耗 storage 提案位。

通过后回调:
- 提案通过 → VotingEngine 调 onchain-issuance 内部 `execute_xxx` 函数(internal trait,不是 extrinsic)
- 提案否决/超时 → VotingEngine 调 `cancel_xxx`(internal trait)
- 失败重试 → 用户调 **VotingEngine::retry_passed_proposal(9.4)** 统一入口
- 取消 → 用户调 **VotingEngine::cancel_passed_proposal(9.5)** 统一入口

#### 5.5 可观测性

- 所有事件链上 emit:`AssetIssued / Minted / Burned / Closed / Transferred / MonitorFrozen / MonitorUnfrozen / MonitorConfiscated / MonitorForceTransferred / MonitorForceClosed`。
- NRC 监管 5 动作必须带 `reason_hash: H256`(链下文书 sha256),链下文书内容不上链。
- RPC `query_holders(asset_id, page)` 暴露持币账户列表(storage 本就公开,RPC 仅做分页便利)。

#### 5.6 退出 / 封禁 — JointVote + propose origin 校验(v2 新增)

- **发行方主动关闭**:走 InternalVote(机构/个人多签内部),关闭后销毁所有持仓余额,**不退还创建费**(创建费已转入 NRC fee_address,不是发行方押金)。
- **NRC 强制封禁**:走 **JointVote**(管理员多签 + 全民兜底),`monitor_force_close` 调用后 30 天内所有持仓冻结,30 天后由 `on_finalize` 通过 `ForceCloseSchedule` 队列触发自动销毁。
- 业务 pallet 暴露 5 个监管 `propose_monitor_X` extrinsic(call_index = 10..=14):

| 动作 | call_index | 引擎 | propose origin 校验 |
|---|---|---|---|
| `propose_monitor_freeze(MonitorFreezeProposal)` | 10 | JointVote | `ensure!(proposer ∈ admins(NRC_SUBJECT_ID))` |
| `propose_monitor_unfreeze(MonitorFreezeProposal)` | 11 | JointVote | 同上 |
| `propose_monitor_confiscate(MonitorConfiscateProposal)` | 12 | JointVote | 同上 |
| `propose_monitor_force_transfer(MonitorForceTransferProposal)` | 13 | JointVote | 同上 |
| `propose_monitor_force_close(MonitorForceCloseProposal)` | 14 | JointVote | 同上 |

- **强制销毁的实现**:`ForceCloseSchedule: StorageMap<BlockNumber, BoundedVec<u32 (asset_id), MaxScheduledPerBlock>>`,`on_finalize(n)` 只读取 `ForceCloseSchedule::take(n)`,不全表扫描 Assets。
- call_index 5..=9 留洞不复用(预留给未来业务 propose_X 扩展)。
- call_index 15+ 留洞,Phase 2 视需求扩展。

#### 5.7 metadata 永久不可改(v2 新增)

第一期发行后 `name / symbol / description` 永久锁定,不提供 `propose_set_metadata` 入口。如需改名,只能 close 旧代币重新发行。

设计理由:
- 防滥用(发行后改名换形态规避监管);
- 简化客户端(name/symbol 一次性写入,后续读 cache 即可);
- 与 GMB 一次性铸币、永不改名的语义一致。

Phase 2 视实际需求再开 set_metadata。

---

### 六、计费(v3 订正:每个 propose_X 走 OnchainTxAmountExtractor 自身分支)

| extrinsic | call_index | OnchainTxAmountExtractor 计费 | 业务内部额外动作 |
|---|---|---|---|
| `propose_issue` | 0 | VOTE_FLAT_FEE = 1 元 | propose 内部 `Currency::reserve(1000 GMB)` 押金,通过/否决 callback 中 transfer/refund |
| `propose_mint` | 1 | VOTE_FLAT_FEE = 1 元 | — |
| `propose_burn` | 2 | VOTE_FLAT_FEE = 1 元 | — |
| `propose_close` | 3 | VOTE_FLAT_FEE = 1 元 | — |
| `propose_transfer` | 4 | VOTE_FLAT_FEE = 1 元 | — |
| `propose_monitor_freeze` | 10 | VOTE_FLAT_FEE = 1 元(监管动作 propose 自身收 1 元防 spam) | — |
| `propose_monitor_unfreeze` | 11 | VOTE_FLAT_FEE = 1 元 | — |
| `propose_monitor_confiscate` | 12 | VOTE_FLAT_FEE = 1 元 | — |
| `propose_monitor_force_transfer` | 13 | VOTE_FLAT_FEE = 1 元 | — |
| `propose_monitor_force_close` | 14 | VOTE_FLAT_FEE = 1 元 | — |
| `InternalVote::cast` / `JointVote::cast_admin` / `JointVote::cast_referendum` | 22.0 / 23.0 / 23.1 | VOTE_FLAT_FEE = 1 元(每张管理员票或公民票) | — |

> v3 修订记录:
> - **v1**:错把 mint/transfer 计费按 OnchainTx 0.1% 计 → 与代币 amount 单位混淆
> - **v2**:订正为"由 InternalVote 分支统一 VOTE_FLAT_FEE",但同时错把 propose 也归入"业务 pallet 不暴露 wrapper extrinsic" → 与 GMB 架构背离
> - **v3**:订正为"业务 pallet 暴露 propose_X(各按 VOTE_FLAT_FEE)",每个 extrinsic 在 `OnchainTxAmountExtractor::RuntimeCall::OnchainIssuance(...)` 分支下单独归类

押金机制(propose_issue 专属):
- propose 时 `Currency::reserve(1000 GMB)`,写 `IssueDeposit[proposal_id] = (proposer, amount)`
- callback 通过 → `unreserve` + `transfer` 给 NRC fee_address
- callback 否决/过期 → `unreserve` 退还原 proposer
- 防 spam 同时不没收无辜提案者

`fee_policy` 单一权威源铁律不破:`ONCHAIN_ASSET_CREATE_FEE` 添入 `primitives::fee_policy`,所有引用从该路径导入,禁止散落 hardcode。

---

### 七、decimals

- 用户自定义,链端校验 `0..=18`(与 ERC-20 主流上限对齐);
- 越界 reject;
- 业界惯例(USDT=6 / USDC=6 / WBTC=8 / ETH=18)均落入此区间。

### 八、与 pallet_assets 内核映射(v2 注解 Freezer/Holder)

链端 `OnchainAssetMeta` storage 维护 `SubjectId(0x04) ↔ pallet_assets::AssetId(u32)` 双向映射,前端只感知 SubjectId,不感知 AssetId。

`pallet_assets::Config` 关键参数:
- `Currency = Balances`(GMB 押金币,但 deposit 系列常量统一 0,实际不锁 GMB);
- `Freezer = ()` / `Holder = ()`:框架阶段占位,业务实装监管 freeze 时需验证 `pallet_assets::freeze(asset_id, who)` 在 () 实现下行为是否符合"冻结特定持仓"语义。如果 () 仅 advisory,需自实装 `FrozenBalance` trait。
- `CreateOrigin = AsEnsureOriginWithArg<EnsureSigned>` / `ForceOrigin = EnsureRoot`:外部 extrinsic 全部被 RuntimeCallFilter reject,origin 设啥不影响实际入口,onchain-issuance 内部经 fungibles trait(Create / Mutate)直接调内核。

`pallet_assets` 全部原生 extrinsic 在 `BaseCallFilter` 中 reject:
- `create / start_destroy / destroy_accounts / destroy_approvals / finish_destroy / mint / burn / transfer / transfer_keep_alive / force_transfer / freeze / thaw / freeze_asset / thaw_asset / transfer_ownership / set_team / set_metadata / clear_metadata / force_set_metadata / force_clear_metadata / force_asset_status / approve_transfer / cancel_approval / force_cancel_approval / transfer_approved / touch / refund / set_min_balance / touch_other / refund_other / block`

**业务调用必须经由 `OnchainIssuance::propose_*` → InternalVote/JointVote 通过 → callback 回调 → `OnchainIssuance` 内部以 root 调用 `pallet_assets`**。

#### 8.1 双轨 storage 同步铁律(v2 关注点)

`OnchainIssuance::Assets`(SubjectId → OnchainAssetMeta)与 `pallet_assets::Asset`(AssetId → AssetDetails)是**双轨 storage**,任何状态变更必须 `with_transaction` 包裹保证原子性:
- close 时 `Assets.state = Closed` + `pallet_assets::start_destroy(asset_id)` 必须同事务;
- force_close 入 `ForceCloseSchedule` 与 `Assets.state = ForceClosed { close_block }` 必须同事务;
- mint/burn/transfer 通过 fungibles trait 调用,trait 内部已有 transaction 语义,但 onchain-issuance 写自己的 storage 时仍须包裹。

### 九、Pallet 索引 + Call 索引(v3 完整对齐)

- `OnchainIssuance` = `pallet_index = 25`
- `Assets`(pallet_assets) = `pallet_index = 26`(原生 extrinsic 全部被 RuntimeCallFilter reject)

**OnchainIssuance call_index 分配(v3):**

| call_index | extrinsic | 类别 |
|---|---|---|
| 0 | `propose_issue` | 业务(InternalVote) |
| 1 | `propose_mint` | 业务 |
| 2 | `propose_burn` | 业务 |
| 3 | `propose_close` | 业务 |
| 4 | `propose_transfer` | 业务 |
| 5..=9 | 留洞,未来业务扩展 | — |
| 10 | `propose_monitor_freeze` | 监管(JointVote) |
| 11 | `propose_monitor_unfreeze` | 监管 |
| 12 | `propose_monitor_confiscate` | 监管 |
| 13 | `propose_monitor_force_transfer` | 监管 |
| 14 | `propose_monitor_force_close` | 监管 |
| 15+ | 留洞,未来扩展 | — |

`call_index` 一经上线即永久 ABI(同 GMB 现有所有业务 pallet 实践),5..=9 / 15+ 留洞不复用。

**客户端硬编码同步要求**:
- `wumin/lib/signer/pallet_registry.dart` 必须加 `onchainIssuancePallet=25` + 10 个 call_index 常量
- `wumin/lib/signer/payload_decoder.dart` 必须加 OnchainIssuance(25) 路由分支 + 10 个 `_decodeProposeOnchainXxx` SCALE 解码器
- `wuminapp/lib/asset/shared/onchain_asset_constants.dart` 必须加 pallet_index 与 call_index 常量(用于业务模块 service 风格的 RuntimeCall 构造)

### 十、ACTION 常量(VotingEngine ProposalData 业务标签)

```rust
pub const MODULE_TAG: &[u8] = b"onc-iss";

// 业务 5 ACTION(InternalVote)— propose origin 校验:proposer ∈ issuer admins
pub const ACTION_ONCHAIN_ASSET_ISSUE:    [u8; 4] = *b"OAIS";
pub const ACTION_ONCHAIN_ASSET_MINT:     [u8; 4] = *b"OAMT";
pub const ACTION_ONCHAIN_ASSET_BURN:     [u8; 4] = *b"OABN";
pub const ACTION_ONCHAIN_ASSET_CLOSE:    [u8; 4] = *b"OACL";
pub const ACTION_ONCHAIN_ASSET_TRANSFER: [u8; 4] = *b"OATR";

// 监管 5 ACTION(JointVote)— propose origin 校验:proposer ∈ NRC admins
pub const ACTION_ONCHAIN_ASSET_MONITOR_FREEZE:          [u8; 4] = *b"OMFZ";
pub const ACTION_ONCHAIN_ASSET_MONITOR_UNFREEZE:        [u8; 4] = *b"OMUF";
pub const ACTION_ONCHAIN_ASSET_MONITOR_CONFISCATE:      [u8; 4] = *b"OMCF";
pub const ACTION_ONCHAIN_ASSET_MONITOR_FORCE_TRANSFER:  [u8; 4] = *b"OMFT";
pub const ACTION_ONCHAIN_ASSET_MONITOR_FORCE_CLOSE:     [u8; 4] = *b"OMFC";
```

冷钱包 `wumin` decoder 必须补 10 个 OnchainIssuance call_index 路由分支(基于 `payload_decoder.dart` 的 SCALE 解码框架,而非 QR envelope body 类型),沿用 `project_qr_signing_two_color` 两色识别铁律,禁止盲签。

> v3 修订记录:**v2 错误地把 10 个 ACTION 实现为 `wumin/lib/qr/bodies/onchain_asset_*_body.dart` QR envelope 顶层 body 类型**,与 wumin 现有"sign_request envelope 中 payload_hex 走 SCALE RuntimeCall 解码"机制不符。v3 删除这 10 个 body.dart,改在 `payload_decoder.dart` 加 OnchainIssuance(25) 路由分支(与 DuoqianTransfer 19 / OrganizationManage 17 等同款 SCALE 解码风格)。

## 决议附注

- 不写 storage migration,随用户单独"重新创世"操作带过去。
- spec_version 不在本任务内 bump,由用户单独处理。
- 本 ADR 第一期(Plain FT)落地后,Pegged 资产、NFT、SFT 各自独立 ADR(分别预留 ADR-012 / 013 / 014 编号位)。

## v3 修订追溯(2026-05-07,模块编号/call 同步对齐)

| # | 修订内容 |
|---|---|
| v3-1 | 业务 pallet 暴露 5 个业务 propose_X(call_index 0..=4)+ 5 个监管 propose_monitor_X(call_index 10..=14)。订正 v2 对 unified_voting_entry phase 4 的过度解读 |
| v3-2 | 删除 v2 在 `wumin/lib/qr/bodies/` 下创建的 10 个 `onchain_asset_*_body.dart`(设计错位,QR envelope 顶层 body 不应承载 RuntimeCall 业务)|
| v3-3 | 在 `wumin/lib/signer/pallet_registry.dart` 加 `onchainIssuancePallet=25` + 10 个 call_index 常量 |
| v3-4 | 在 `wumin/lib/signer/payload_decoder.dart` 加 OnchainIssuance(25) 路由分支 + 10 个 `_decodeProposeOnchainXxx` SCALE 解码占位 |
| v3-5 | 在 `wuminapp/lib/asset/shared/onchain_asset_constants.dart` 加 pallet_index / call_index 常量 |
| v3-6 | configs/mod.rs OnchainTxAmountExtractor 把 `RuntimeCall::OnchainIssuance(_)` 细分到 10 个 propose_X 各按 VOTE_FLAT_FEE |
| v3-7 | configs/mod.rs RuntimeCallFilter 不再屏蔽 OnchainIssuance(propose_X 是合法入口);Assets 仍全 reject |
| v3-8 | onchain-issuance lib.rs `#[pallet::call]` 不再为空,实装 10 个 propose_X 框架(fn 签名 + ensure stub) |

## v2 修订追溯(2026-05-07,本次 review 后)

15 项 review 修订全部纳入 v2:

| # | 修订内容 |
|---|---|
| 1 | NrcAccountProvider 拆 `NrcMainAccountProvider` + `NrcFeeAccountProvider`,语义分离 |
| 2 | propose 阶段 origin 校验铁律写入(业务 ACTION:proposer∈issuer admins;监管 ACTION:proposer∈NRC admins) |
| 3 | 1000 GMB 创建费改为 propose 时 reserve / 通过时 transfer / 否决时退还 |
| 4 | 计费表订正:mint/burn/transfer/close 全部 `VOTE_FLAT_FEE = 1 元`(由 OnchainTxAmountExtractor 在 InternalVote 分支统一计费),不再写"OnchainTx 0.1%" |
| 5 | ForceClose 30 天倒计时改为 `ForceCloseSchedule: StorageMap<BlockNumber, BoundedVec<u32>>`,on_finalize O(1) take 而非 O(N) 全表扫描 |
| 6 | onchain-issuance/Cargo.toml `try-runtime` feature 传播给依赖 |
| 7 | SubjectId 0x04 payload 简化:去掉 8B issuer_subject_short,改为 4B asset_id LE + 43B 零填充 |
| 8 | 因 #7 简化,helper 函数无需哈希依赖,问题自动消失 |
| 9 | OnchainAssetMeta 去掉 `monitor_subject_id` 字段(每条都是同一个 NRC SubjectId,冗余);monitor 主体走全局 trait 解析 |
| 10 | OnchainAssetMeta 去掉 `asset_id` 字段(SubjectId byte[1..5] 即 asset_id LE,无需重复存),保留 `AssetIdIndex` 反向索引(pallet_assets 事件回调反查 SubjectId 用) |
| 11 | metadata 永久不可改铁律写入 5.7 节(第一期不开 set_metadata) |
| 12 | fee.rs 死代码 `let _ = (...)` 清理 |
| 13 | 顶部加命名说明,区分 onchain-issuance 与 onchain-transaction 的语义 |
| 14 | pallet_assets Freezer/Holder=() 注解(8 节)— 业务实装时验证 |
| 15 | 双轨 storage 同步铁律写入 8.1 节 |

## 风险与回滚

- pallet_assets 接入失败回滚:删除 `runtime/issuance/onchain-issuance` crate 与 workspace member,恢复 construct_runtime / configs / Cargo 改动。
- 黑名单初始词表过严导致正常发币被拒:RuntimeUpgrade 投票删词,无需停链。
- monitor 主体 NRC 私钥风险:NRC 由 `china_cb` 顶级机构 + 国储会 + 联合投票兜底覆盖,与 GMB 链整体安全模型一致,本协议不引入新风险面。
- v2 修订因协议位 0x04 还未上线,SubjectId payload 简化(去掉 issuer_subject_short)零迁移成本。
