# 公民身份全链路:建档 → 上链 → 人口统计 → 投票引擎消费

- 更新日期:2026-07-22
- 事实源:本文是流程导读;字段与编码以代码为准
  - `citizenchain/onchina/src/domains/citizens/`(建档 + 上链准备)
  - `citizenchain/runtime/misc/citizen-identity/src/lib.rs`(链上身份 + 人口计数)
  - `citizenchain/runtime/votingengine/legislation-vote/src/lib.rs`(快照消费)
  - `memory/01-architecture/qr/qr-action-registry.md`(扫码动作登记)

## 1. 公民创建(本地建档,不碰链、不碰钱包)

- 注册局管理员登录 OnChina 控制台(链上 Active 管理员集合鉴权)创建公民档案,
  落节点本地 PG `citizens` 表:CID编号、护照号、`family_name`、`given_name`、性别、出生日期、
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
   落库,并构造 `register_voting_identity(actor_cid_number, actor_role_code,
   payload, citizen_signature)` call data,生成第二张二维码(链交易动作码 `0x0a00`)。
4. **管理员签名(第二重签名)**:注册局管理员用自己的钱包扫码冷签并提交
   extrinsic(标准 Substrate 交易签名)。
5. **链端执行**(pallet CitizenIdentity idx 10):
   - `ensure_signed` 管理员 origin;
   - 载荷合法性(年龄 ≥16 等);
   - `can_manage_voting_identity`:签名钱包必须是 `actor_cid_number + actor_role_code`
     对应注册局岗位的有效任职管理员,FRG 省专员管本省、CREG 专员只管本市;
   - `ensure_citizen_signature`:链上对 `payload.encode()` 再验一次公民
     0x10 域签名——公民本人同意在链上可验证;
   - CID 永久唯一；`VotingIdentityByCid` 以 CID 保存身份，`WalletAccountByCid` 与
     `CidByWalletAccount` 保存当前唯一签名钱包的双向绑定；
   - 写入上述 CID 主键身份与当前钱包绑定，触发人口计数增量与
     `CitizenIssuance` 首次注册发行钩子(按档位/全局上限/CID+账户双重防重,
     把公民币 `deposit_creating` **直接铸入公民钱包账户**,无需领取动作),
     发 `VotingIdentityRegistered` 事件。
- 候选人身份 `upgrade_to_candidate_identity` 同构,链上公开档案多出生地三级码、
  `family_name`、`given_name`、性别(`citizen_sex`)和出生日期。姓名结构不得再拼接或另造同义字段。
- 链上身份字段定稿:投票公民 = CID号(身份存储键)+ 当前绑定钱包账户 + 护照有效期起止 +
  身份状态 + 居住地省/市/镇码;参选公民另加出生地省/市/镇码 + `family_name` +
  `given_name` + 性别 + 出生日期。
  年龄只作注册门槛(≥16)校验,不进链上状态。

2026-07-22 已完成投票引擎公民主体接入：资格接口返回 `CitizenSubject { cid_number, wallet_account }`；联合公投、立法公投和选举普选均按永久 CID 去重并在票据值中保存完整主体。候选快照、候选人计票和当选结果也已统一为完整 `CitizenSubject`，不得恢复裸钱包主体。

## 3. 公民人口数据(citizen-identity pallet)

- 每次注册/更新/撤销身份时同步维护四级有效选民计数器:
  `CountryVotingCount` / `ProvinceVotingCount(省)` /
  `CityVotingCount(省,市)` / `TownVotingCount(省,市,镇)`,
  最终只统计状态正常且在当前人口就绪日期护照有效的身份。
- 护照未来生效、到期、身份吊销和迁居必须由 citizen-identity 维护有界日期变化计划；
  当天人口变化尚未处理完成时，新提案人口数据 fail-closed，不得用不完整分母建案。
- `PopulationReadyDate` 保存四级人口完整推进至的 UTC+8 日期；护照生效和到期转换按
  `(date, index)` 独立存储并携带永久 CID、身份 revision 和转换类型，不复制钱包或身份全文。
  `on_idle` 每块最多推进 366 个日期、处理 2,048 个转换项，且最多使用区块权重的 1/8；
  实际处理量继续受剩余权重限制。
- 身份写入只在 `PopulationReadyDate == 当前日期` 且没有人口维护故障时执行。某日转换
  分块处理中，内部计数即使已经部分改变也不会通过 provider 对外发布，身份变更同样暂停，
  直到整日转换全部完成。
- 取数入口 `CitizenIdentityProvider`:
  - `citizen_subject(who)`:先由 `CidByWalletAccount` 取得永久 CID，再校验 `WalletAccountByCid`、CID 主键身份、身份状态和 CID 状态后返回完整公民主体，任何错配均 fail-closed;
  - `population_data(scope)`:仅在当前日期人口完整就绪时返回 `Some(PopulationData)`，O(1)
    读取对应作用域计数、当前资格 revision 和判定日期；未就绪或维护故障返回 `None`；
  - `voting_subject(who, scope)`:当前身份、护照、作用域和 CID↔钱包全部有效时返回完整投票公民主体;
  - `candidate_subject(who, scope)`:voting_subject 有效且持有候选人身份时返回完整竞选公民主体;
  - `voting_subject_at(who, population_data)`:由当前签名钱包解析永久 CID，按该 CID 的不可变身份历史校验快照时资格并返回完整主体。
- 每次身份注册、资料更新、迁居或撤销都会递增全局 revision,关闭同一永久 CID 的旧版本并写入
  `VotingEligibilityVersions`;同一区块多次写入也有确定顺序。CID 不得修改、替换、删除或复用。

## 4. 投票引擎消费(legislation-vote 特别案公投)

- **建案事务内快照**:特别案创建时由 `legislation-vote` 先从发起机构
  `actor_cid_number` 的唯一 CID 解析国家/省/市作用域，再在同一存储事务内调用
  `VotingEngine::create_population_snapshot(proposal_id, scope)`。投票引擎只从
  `CitizenIdentityReader::population_data(scope)` 取得人口数据，再写入自身
  `ProposalPopulationSnapshots[proposal_id]`。人口为零或后续建案失败时整笔事务回滚，
  不存在公开准备交易、调用者缓存、独立 snapshot_id 或待消费中转存储；普通案和重大案不创建人口快照。
- **分母与成员资格同源冻结**:`cast_referendum_vote` 对每张票把该提案人口快照传给
  `voting_subject_at(who, population_data)`；提案创建后的新增、迁居、资料更新或撤销不改变已有提案。
- **消费端全量校验**(`voting_subject_at`):由当前钱包绑定解析永久 CID，按 revision 定位该 CID 在创建时的身份版本 + 状态 NORMAL + **护照有效期窗口内**(链上时间戳按 UTC+8
  冻结 YYYYMMDD,过期或未生效即拒,时间戳缺失 fail-closed)+ 居住地在作用域内;
  钱包签名由投票 extrinsic 本身在交易层强制。
- **分母口径约束**:人口分母与单人资格同为“永久 CID Active + CID↔钱包绑定完整 +
  状态正常 + 快照日期护照有效 + 行政区匹配”。2026-07-22 起四级计数已经按该口径维护，
  过期或尚未生效的护照不再进入分母。
- 客户端对齐:CitizenApp、CitizenWallet 和 OnChina 不构造、签名或解码独立快照
  交易；扫码只确认业务提案，作用域和快照均由链端按 actor CID 内联确定。

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

## 2026-07-22 第 5 步验收

- 当前、竞选和历史人口快照资格接口全部返回 `CitizenSubject { cid_number, wallet_account }`，不再用 bool 或裸钱包表达公民授权主体。
- 联合公投与立法公投按永久 CID 去重并保存完整票据；人口快照仍只冻结作用域、有效总数、资格 revision、判定日期和创建区块。
- 五个投票 crate、runtime 46 项及受影响业务模块测试，全 workspace 测试目标，`no_std`、WASM、benchmark/try-runtime 和 release Node 构建均通过。
- `citizenchain-fresh --tmp` 真实节点 block #0 为 `0x69b4a0025356d050004cff3ef176167a6520b59c9086c9ac6b9a45c4b9e9c0e6`，state root 为 `0x0b066c3567ed25c15cfa96b7d249b6235df4746a253144db21c87dfd2ed2333e`，metadata 二进制 220,197 字节，runtime 六项项目版本均为 `0`；验收节点已停止。

## 2026-07-22 第 6 步验收

- 选举元数据只保留唯一 `actor_cid_number + role_code`，发起机构就是拟任职机构；`target_cid_number`、`office_code`、`rule_id` 已从 `election-vote` 清零。
- 候选快照、普选票据、候选计票和当选结果全部保存完整 `CitizenSubject`；普选按永久 CID 去重，互选按机构 CID + 岗位码 + 钱包票据去重。
- `election-vote` 17 项、votingengine 4 项、runtime 46 项、全 workspace 测试目标、`no_std`、WASM、benchmark/try-runtime 和 release Node 构建通过。
- `citizenchain-fresh --tmp` 真实节点 block #0 为 `0x285ca7f4ab0f24771baff6a6fc10141ee281fbbd6ce1a8f9dcd1d7676501a41b`，state root 为 `0x27ecdc5b73ce195df4bdfe6c05fe68ef0b682c58751f5f145868a69a1f4672bd`，metadata 二进制 220,398 字节，runtime 六项项目版本均为 `0`；验收节点已停止。

## 2026-07-22 第 8 步客户端协议验收

- CitizenApp 与 Cloudflare 读取公民身份时，先由钱包取得永久 CID，再在同一个 finalized 区块校验 CID Active、CID↔钱包双向绑定和 CID 主键身份；任何缺失、截断、尾随或错配都不承认身份。
- `VotingIdentityByCid` 值只承载护照有效期、状态、居住省/市/镇码和更新时间；CID 只在 storage key。`CandidateIdentityByCid` 只增加出生省/市/镇码、`family_name`、`given_name`、性别、出生日期和更新时间。
- OnChina PostgreSQL 最终 schema 已用真实临时数据库初始化并核验；数据库不再在启动期读取、回填或删除旧合并姓名、旧有效期、旧竞选范围和重复居住区字段。注册局办理身份的双签业务边界没有改变。
- 第 8 步没有修改 runtime；全端完整 fresh Node 与链上调用验收统一留在第 9 步。当前发现的护宪守卫 `LawDecodeFailed` 必须从源码嵌入 WASM 与节点解码结构查明，不得通过关闭或绕过守卫处理。
