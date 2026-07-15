# 公民身份全链路:建档 → 上链 → 人口统计 → 投票引擎消费

- 更新日期:2026-07-14
- 事实源:本文是流程导读;字段与编码以代码为准
  - `citizenchain/onchina/src/domains/citizens/`(建档 + 上链准备)
  - `citizenchain/runtime/misc/citizen-identity/src/lib.rs`(链上身份 + 人口计数)
  - `citizenchain/runtime/votingengine/legislation-vote/src/lib.rs`(快照消费)
  - `memory/01-architecture/qr/qr-action-registry.md`(扫码动作登记)

## 1. 公民创建(本地建档,不碰链、不碰钱包)

- 注册局管理员登录 OnChina 控制台(链上 Active 管理员集合鉴权)创建公民档案,
  落节点本地 PG `citizens` 表:CID编号、护照号、姓名、性别、出生日期、
  居住地/出生地省市镇码、护照有效期、`citizen_status`、`voting_eligible`。
- 建档不要求钱包:未成年人、无钱包公民都可以先持有本地电子护照档案。
- 公民在 CitizenApp「我的 → 电子护照」选择一个热钱包作为投票账户,
  页面展示钱包地址二维码,供办理现场的操作员扫入。

## 2. 公民上链(录入钱包 + 双签名,`chain_identity.rs`)

前置:档案状态 NORMAL、`voting_eligible=true`、满 16 周岁、档案在本注册局辖区。

注册局上链操作一律最严档(`CITIZEN_ONCHAIN_PUSH` → PasskeyColdSign):
prepare 与 complete 前各需一次 WebAuthn passkey 断言 + 管理员冷钱包扫码签名,
换取绑定 `{cid_number, wallet_account}` 的一次性安全 grant,无 grant 一律 403。

1. **prepare**:操作员录入/扫描公民钱包账户 → 后端组
   `VotingIdentityPayload` SCALE 字节(9 字段:cid_number、wallet_account、
   citizen_age_years、passport_valid_from/until、citizen_status、
   居住地省/市/镇码)→ 打包 QR_V1 `k=1 a=2` 签名请求(180 秒有效)。
2. **公民签名(第一重签名)**:公民用 CitizenWallet 离线签名页或
   CitizenApp 电子护照扫码签名页扫码 → 两色识别独立解码载荷并展示中文字段,
   解不开一律拒签 → 本人确认后对
   `blake2_256(GMB || 0x10 || payload)`(`OP_SIGN_CITIZEN_IDENTITY`)
   做 sr25519 签名 → 展示 sign_response 二维码。
3. **complete**:操作员扫回执 → 后端验公钥一致 + 同域验签 → 通过后绑定钱包
   落库,并构造 `register_voting_identity(registrar_account, payload,
   citizen_signature)` call data,生成第二张二维码(链交易动作码 `0x0a00`)。
4. **管理员签名(第二重签名)**:注册局管理员用自己的钱包扫码冷签并提交
   extrinsic(标准 Substrate 交易签名)。
5. **链端执行**(pallet CitizenIdentity idx 10):
   - `ensure_signed` 管理员 origin;
   - 载荷合法性(年龄 ≥16 等);
   - `can_manage_voting_identity`:registrar 必须是 registrar_account 的链上
     Active 管理员,FRG 省组管本省、CREG 只管本市;
   - `ensure_citizen_signature`:链上对 `payload.encode()` 再验一次公民
     0x10 域签名——公民本人同意在链上可验证;
   - CID 唯一性(`AccountByCid` 一 CID 一钱包);
   - 写 `VotingIdentityByAccount` + `AccountByCid`,触发人口计数增量与
     `CitizenIssuance` 首次注册发行钩子(按档位/全局上限/CID+账户双重防重,
     把公民币 `deposit_creating` **直接铸入公民钱包账户**,无需领取动作),
     发 `VotingIdentityRegistered` 事件。
- 候选人身份 `upgrade_to_candidate_identity` 同构,链上公开档案多出生地三级码、
  公民姓名与性别(`citizen_sex`)。
- 链上身份字段定稿:投票公民 = CID号 + 钱包账户(存储键)+ 护照有效期起止 +
  身份状态 + 居住地省/市/镇码;参选公民另加出生地省/市/镇码 + 姓名 + 性别。
  年龄只作注册门槛(≥16)校验,不进链上状态。

## 3. 公民快照统计(citizen-identity pallet 增量计数)

- 每次注册/更新/撤销身份时同步维护四级合格选民计数器:
  `CountryVotingCount` / `ProvinceVotingCount(省)` /
  `CityVotingCount(省,市)` / `TownVotingCount(省,市,镇)`,
  只统计 `identity_counts_as_voter`(状态 NORMAL)的身份。
- 取数入口 `CitizenIdentityProvider`:
  - `population_count(scope)`:O(1) 读对应作用域计数器,不遍历公民;
  - `can_vote(who, scope)`:有链上投票身份 + 状态正常 + 居住地落在作用域内;
  - `can_be_candidate(who, scope)`:can_vote 且持有候选人身份;
  - `create_population_snapshot(scope)`:冻结分母、全局资格 revision 和护照判定日期;
  - `can_vote_at(who, snapshot_id)`:按账户不可变身份历史校验创建时资格。
- 每次身份注册、更新、迁居、换号或撤销都会递增全局 revision,关闭旧版本并写入
  `VotingEligibilityVersions`;同一区块多次写入也有确定顺序。

## 4. 投票引擎消费(legislation-vote 特别案公投)

- **两步快照、同块强约束**:
  1. `prepare_population_snapshot(scope)`:经
     `CitizenIdentityReader::create_population_snapshot(scope)` 创建身份真源快照(必须 >0),
     存 `PendingPopulationSnapshots[发起人]{snapshot_id, eligible_total,
     prepared_at=当前块}`;
  2. 同块创建特别案(`vote_type=SPECIAL`)时消费:`prepared_at != now` 报
     `PopulationSnapshotNotCurrent` 并清除——保证公投分母就是建案当块的
     链上人口数。`eligible_total` 固化进 `Proposal.citizen_eligible_total`,
     `snapshot_id` 绑定到 `ProposalPopulationSnapshotIds`。
- **分母与成员资格同源冻结**:`cast_referendum_vote` 对每张票调用
  `can_vote_at(who, snapshot_id)`;提案创建后的新增、迁居、换号或撤销不改变已有提案。
- **消费端全量校验**(`can_vote_at`):按 revision 定位创建时身份(注册时已验公民签名并锁定
  CID↔钱包一对一)+ 状态 NORMAL + **护照有效期窗口内**(链上时间戳按 UTC+8
  冻结 YYYYMMDD,过期或未生效即拒,时间戳缺失 fail-closed)+ 居住地在作用域内;
  钱包签名由投票 extrinsic 本身在交易层强制。
- **分母口径约束**:人口计数器按状态增量维护,链上没有"护照到期"事件,
  过期公民在注册局更新状态前仍计入分母,但投票被 `can_vote` 拦截。
- 客户端对齐:扫码确认页只展示作用域("人口快照只携带作用域,链端直接读取
  citizen-identity",`citizenwallet payload_decoder.dart`);OnChina 立法
  控制台发起入口在 `domains/legislation/law/chain_vote.rs`。

## 签名域三端一致性纪律

`blake2_256(GMB || 0x10 || payload)` 的构造分别在:

| 端 | 位置 |
|---|---|
| runtime 真源 | `primitives::sign::signing_message` |
| OnChina 验签 | `domains/citizens/chain_identity.rs` |
| CitizenWallet 签名 | `lib/signer/qr_signer.dart::signingBytesFor` |
| CitizenApp 签名 | `lib/signer/signing.dart::signingMessage`(经 `qr_signer.dart::signingBytesForHex`) |

任何一端改动必须四处同步,并刷新对应测试
(`citizenwallet/test/signer/qr_signer_test.dart`、
`citizenapp/test/signer/qr_signer_test.dart` 0x10 用例、
`citizenapp/test/my/myid/voting_identity_payload_test.dart`)。
现场设备必须使用 2026-06-30 提交 5c8374185 之后的构建,
旧构建对 payload 直签会被后端域验签拒绝。
