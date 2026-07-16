# CID 占号先行与吊销墓碑(占号体系 卡2)

> 设计真源:`memory/04-decisions/ADR-031-cid-occupy-registry.md`(D2/D3/D6/D7/D8)。2026-07-02 代码核查补充的硬约束:
> - onchina 现无任何自动提交交易通路(全部=裸 call_data→冷签→钱包提交),须按 D7 补「组装+dry-run+author_submitExtrinsic」后端提交骨架(参照 node/src/governance/signing.rs:612-683),QR 仍只签不提交;
> - 机构关闭现状=账户级物理删除、`Institutions` 永不删但状态从未置 Closed;须按 D3 落地 Closed 墓碑语义 + 堵 register(call 2)不查本 pallet Institutions 的重注册缺口;
> - citizens.onchain_tx_hash 等列现无写入者,须按 D8 经 indexer 事件回写闭环;
> - 新增 `occupy_cids_batch`(≤10,000 项/笔)供公民批量建档;占号/吊销费类 **Free(2026-07-02 已决,ADR-031 Q4)**,滥用由链上注册局授权门槛拦截。
> - 2026-07-03 修订:机构存量改全量创世直铸(卡3 D5),本卡占号先行流程只服务**运行期新增**(公民建档、未来新设机构);机构批量注册 extrinsic 本轮不实现。

## 设计定稿(用户已确认)

- 唯一性仲裁 = 链上写入时原子「验格式 + 查重 + 登记」;链下 RPC 预查仅作快速失败优化,不承担唯一性保证(查询防不住并发,写入才是仲裁)。
- 建档流程 = 本地生成号 → 提交占号交易(携档案承诺哈希)→ InBestBlock 成功 → 才写本地机构/公民档案。
- 占号即终身绑定:号码从此不可能发给任何其他主体。
- 落库失败恢复 = 幂等续用:占号记录含登记机构 + 档案承诺哈希,重试建档时发现「本注册局为同一档案承诺」已占 → 直接落库,不二次占号,孤号不产生。
- 清档 = 发吊销交易(链上状态 Active → Revoked 墓碑)+ 清本地档案;链上记录永不删除、号码永不复用(对齐 revoke 保留映射、行政区码墓碑 ADR-021)。
- 隐私边界(已认可):全部公民建档即占号,链上只有 cid_number + 承诺哈希 + 登记机构 + 块高,无姓名生日;链上可枚举每省建档总量。
- 建档依赖链活性,链不可用即建档失败(fail-closed)。
- runtime breaking → 重新创世,零兼容零残留。

## 目标

- `citizen-identity`(或独立归口)新增:
  - `CidRegistry` 存储:cid_number → { 登记机构, 承诺哈希, 状态 Active/Revoked, 块高 }。
  - 占号 extrinsic(注册局标准 extrinsic 签名,复用 `CitizenIdentityAuthority` 省市 scope 授权;遵守签名分层铁律,不引入 op_tag)。
  - 吊销 extrinsic(墓碑,不删除存储项)。
- `register_voting_identity` 前置要求:CID 已占号、状态 Active、归属一致。
- 机构侧:`public/private-manage` 注册即占号(现有 `Institutions` + sibling `cid_exists` 写入时查重即真源,不重复建表);核对 onchina 机构两步流严格「链上成功才转正」。
- onchina 建档流程改造:公民/机构统一占号先行;碰撞报错走 nonce 后缀重试(上限对齐 SFID n9 桶 1000 次重试规则,同时治愈确定性种子同名同生日碰撞即 409 无恢复的问题);清档接吊销交易;`onchain_tx_hash` 等字段回写。
- 占号/吊销费用归入 `primitives::fee_policy` 明确分支。

## 修改范围

- `citizenchain/runtime/otherpallet/citizen-identity/`
- `citizenchain/runtime/entity/public-manage/`、`private-manage/`(核对为主)
- `citizenchain/runtime/src/`(注册、费用分支、benchmark)
- `citizenchain/runtime/primitives/`(承诺哈希定义、fee_policy)
- `citizenchain/onchina/src/domains/citizens/`、`institution/`、`cid/`
- `citizenchain/onchina/frontend/`(建档/清档流程状态提示)

## 验收

- 并发占同号:一成一败(`CidAlreadyRegistered` 类错误)。
- 占号成功 + 本地落库失败 → 重试直接续用,链上不出现二次占号。
- 清档 → 链上 Revoked 墓碑,该号任何主体不可再占。
- 同名同生日同镇两位公民:第二位经 nonce 重试获得新号,建档成功。
- `cargo test` 相关 pallet 全绿;`cargo check -p onchina` 通过;`npm --prefix citizenchain/onchina/frontend run build` 通过。

## 进展

- 2026-07-03:**链端全部完成**(D2/D3/费类),受影响 crate 测试全绿:
  - citizen-identity 新增 `CidRegistry` 存储(registrar_account/commitment/居住地码/Active|Revoked 墓碑/块高)+ 三个 call:`occupy_cid`(幂等续用=同注册局+同承诺放行)/`occupy_cids_batch`(≤10,000,任一失败整笔回滚)/`revoke_cid`(墓碑+绑定身份联动吊销,作用域授权用占号时存的居住地防跨域);四个身份写入口新增 `ensure_cid_occupied_active` 前置;换号=旧号自动墓碑;`revoke_identity` 联动登记表墓碑;errors:CidAlreadyOccupied/CidNotOccupied/CidAlreadyRevoked;weights 手工保守上界三条。
  - 该旧整机构关闭口径已被 2026-07-15 改造废止：机构 CID 永久存在，只允许关闭自定义账户；协议账户、admins、岗位和阈值保持不变。
  - 费类:occupy/batch/revoke → Free(configs/mod.rs 穷尽 match per-call 分支),其余 CitizenIdentity 调用维持 VoteFlat。
  - 测试:citizen-identity 21/21(含幂等/冲突/批量回滚/无占号拒注册/吊销墓碑不可复用/换号旧号墓碑 6 个新用例);entity 34+34(关闭用例扩墓碑断言+重注册拒绝);citizen-issuance 12+5、runtime lib 30/30 占号前置改造后全绿。
  - 顺手修工作区既有 benchmark 断链 4 处:admins×2(CHINA_CB 导入+AdminProfile 结构升级)、resolution-issuance/runtime-upgrade(PreparedPopulationSnapshot.nonce_hash→scope)、runtime 基准清单摘除无基准的 citizen-identity;全 runtime `--features runtime-benchmarks` 编译过。
- 2026-07-03:**onchina 侧(D6/D7/D8)完成**,onchina 134 测试全绿、前端 tsc+build 通过、node crate 不受影响:
  - **D7 提交通路** `core/chain_submit.rs`:onchina 唯一 extrinsic 组装+提交模块(依赖 citizenchain/frame-system/pallet-transaction-payment/frame-metadata-hash-extension,与 node signing.rs 同源拼 SignedPayload);`prepare_signing`(实时 nonce+版本+创世哈希→冷签载荷+校验哈希)、`assemble_and_submit`(重建校验哈希防漂移→本地 sr25519 验签→system_dryRun 拒 Future/Stale→author_submitExtrinsic)、`wait_nonce_consumed`(accountNextIndex 越过=进块代理,95s 超时)、`find_extrinsic_block`(回溯 20 块比对 blake2 交易哈希);**QR 仍只签不提交**,冷钱包边界不变。2 单测绿。
  - **D6 建档流程** `domains/citizens/occupy.rs`:两阶段——prepare(校验→种子+nonce 0..999 碰撞重试+本地/链上双预查+链上同承诺幂等续用→`occupy_cid` 冷签会话)、submit(统一入口,按 purpose 分派:占号进块后落档案、吊销墓碑、身份上链回写);`chain_sign_sessions` 表(prepare 落库 submit 单次消费,携签名哈希防漂移)。`admin_entry.rs` 重构为 `validate_citizen_input`/`citizen_cid_seed`/`generate_citizen_cid_candidate`/`persist_citizen_record`/`create_output_from_record` 五段复用。清档走 `revoke_cid` + `mark_citizen_revoked`(本地 REVOKED 墓碑)。
  - **D8 回写闭环**:提交路径同步回写 `onchain_tx_hash/onchain_block_number/onchain_at`(update_citizen_onchain / persist 时写入),补齐"无写入者"缺口;`chain_runtime::cid_registry_lookup` 读链上 CidRegistry 供发号预查与幂等识别。
  - **chain_identity.rs**:complete 切 D7——QR 载荷改完整签名载荷(对齐钱包扩展尾解码器)、落 `CITIZEN_IDENTITY_PUSH` 会话,回签经统一 submit 提交。
  - **前端**:`core/useChainSign.tsx`(展示 sign_request QR→扫码→解析 signer_pubkey/signature);`api.ts` 改两阶段(prepareCitizenOccupy/submitCitizenChainSign/prepareCitizenRevoke);CreateModal 占号先行+冷签+进块落库;DetailPage 身份上链改 onchina 提交+新增"吊销身份(墓碑)"按钮;删旧"已提交链上交易"手动 QR 与死状态。
  - 踩坑:onchina 原 `parity-scale-codec` 与 runtime 传入的 `codec` 同 crate 双名冲突,统一去重为 workspace `codec`(9 文件改引用);submit 入参从 raw sign_response 改 signer_pubkey+signature(与前端 signWithScan 同形,后端按会话字节重新验签,安全不减)。

## 状态

- 2026-07-02:建卡。依赖卡1(链端校验先行)。
- 2026-07-03:链端(D2/D3/费类)+ onchina 侧(D6/D7/D8)全部完成。**卡2 完工**,下一步卡3(全量创世直铸 + 部署形态改造 + 重新创世)。
