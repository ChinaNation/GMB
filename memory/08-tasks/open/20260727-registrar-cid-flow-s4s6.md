# 注册局 CID 流程改造(S4 链 + S6 onchina,合并一大步)

状态:open(四端功能及总任务第 5 步 OnChina finalized 查询/投影已完成；待第 6-7 步终审和第 8 步正式创世资产重生)
所属模块:Chain(citizenchain runtime)+ OnChina(注册局);**链与 onchina 强耦合,必须同批**([[no-compatibility]] 不留过渡)
母卡:`20260727-citizenapp-cid-identity-rootless-wallet.md`(S1-S3 已完成:CN 去地域化 CID、自助占号 `self_occupy_cid`、自助换绑 `self_rebind_cid_account_id`、`OP_SIGN_CID_REBIND` 签名域)

## 2026-07-29 当前定稿（取代本卡此前短载荷/匿名限定/本局限定决策）

- CID 是唯一身份主键；同一 CID 同时只能绑定一个有效 `account_id`，钱包只负责签名鉴权。
- 注册局首次绑定授权固定为
  `genesis_hash + cid + account_id + expected_binding_revision(0) + expires_at`，域 `0x12`；
  注册局换绑授权固定为
  `genesis_hash + cid + expected_old_account_id + new_account_id
  + expected_binding_revision + expires_at`，域 `0x1F`。CID 为最大 32 字节规范 SCALE
  BoundedVec，u64 均为小端；签名消息统一走 `blake2_256(GMB ‖ op_tag ‖ SCALE)`。
- 授权过期时间只取同一 finalized 快照的链上 `Timestamp.Now + 300s`；外层 QR `e` 必须
  等于内层过期时间。会话仍按服务端时间独立保存 600 秒。
- 匿名 CID 可由任一在册 CREG/FRG 办理；civic CID 可由投票身份居住城市的 CREG 或对应
  省 FRG 办理。换绑不要求旧钱包签名，个人多签不进入本流程。
- OnChina 不再要求“本局已有本地档案”才允许换绑；它从同一 finalized 状态读取
  `CidRegistry + AccountIdByCid + BindingRevisionByCid + CidByAccountId +
  VotingIdentityByCid + CandidateIdentityByCid + Timestamp.Now`，严格区分
  Missing/Active/Revoked 并校验正反闭环。
- call 6 固定为
  `actor_cid, actor_role, cid, account_id, expires_at, citizen_signature`；
  call 7 固定为
  `actor_cid, actor_role, cid, new_account_id, expected_binding_revision, expires_at,
  new_account_signature`。旧批量绑定调用已删除且不得恢复。
- 只有目标 extrinsic finalized 且有 `System.ExtrinsicSuccess`，并在同一 finalized 块
  精确核对目标状态后才可写本地投影。提交前保存 tx hash/锚点；断线分批回扫并保存游标；
  finalized 标记保存后重试只补投影，不得重提。
- 本节是当前验收合同。下方 2026-07-27/28 的“完成记录”只用于说明开发演进，其中被本节
  点名取代的短载荷、civic 拒绝、本局限定和“后期再做”口径均不再构成规范或兼容许可。

## 总任务第 5 步 OnChina finalized 闭环（2026-07-30，完成）

- `core/chain_citizen_identity.rs` 成为公民绑定、投票/竞选标志和 finalized 锚点的唯一
  链读取入口；办理、投影和全局查询共用，不再各自拼接 storage 读取。
- 注册局全局接口允许任一 FRG/CREG 查询链上公开 CID 绑定，不要求本地档案或办理地命中；
  链下护照和资料仍严格留在办理节点。
- 两个不同市注册局 OnChina 实例、两套独立 PostgreSQL 对同一 fresh 链 CID 返回完全一致
  的账户、revision=1 和 finalized 哈希；每套创世机构投影均为 49,593 个机构、99,232 个
  账户。
- 当前旧冻结链没有 `BindingRevisionByCid` 元数据，新读取器失败关闭。正式发布必须等待
  唯一正式创世重生后整体切换；本步未创世、未推送、未触发 CI。
- 创世法定代表人本地投影也已删除硬编码账户，统一读取 finalized 链上身份快照。后端
  179 项测试、严格 clippy、前端 TypeScript 和本步 Rust 格式检查通过；源码和编译包残留
  已清理。现有 HTTPS OnChina 已真实加载新哈希脚本，登录页完整渲染且控制台无错误。

## 已完成地基(S1-S3,新窗口直接复用)
- CID = CN 前缀去地域化、人主体(CTZN/NATP/SMTP)12 位号段(`primitives/cid`)。
- `self_occupy_cid`(call 5):公民本人自助占号+占即绑+匿名+自付费;类型 CTZN|NATP(`ensure_valid_self_registrable_cid`);SELF sentinel registrar + 空 residence;commitment=blake2_256(account_id)。
- `self_rebind_cid_account_id`(call 9):新账户 origin + 旧账户 detached 签名
  (`OP_SIGN_CID_REBIND`)，授权绑定创世、CID、预期旧/新账户、revision 和过期时间；
  `rebind_account_id` 原语维持 CID 不变并原子更新双向账户绑定。
- 复用 helper:`ensure_account_id_binding_available`/`bind_account_id`/`rebind_account_id`/`do_occupy_cid`(已剥离 CTZN 硬校验,类型校验下沉到调用方)。

## D4a 决策(注册局占号即绑账户 + 档案选填)
- **占号即绑钱包账户**:注册局「新增公民」弹窗**必填** = ①类型 CTZN|NATP ②用户钱包账户 ③用户签名(证账户控制)。
- **档案全改选填**:姓/名/居住省市镇/出生省市镇/出生日期/性别 等全部选填(可留空)。
- **一经填写不可改**:出生日期、出生省市镇、性别(可留空,但填后锁定)。链上 `CandidateIdentity.birth_date` 已有不可改先例,扩展到出生省市镇/性别。

## D4b 决策(注册局写入模型 + 换绑管辖)
- 写入模型:**注册局CID + 管理员岗位码 + 管理员钱包账户(origin)+ 用户CID + 用户钱包账户**。
- **匿名用户(仅上链 CID+账户,无投票/竞选)→ 任一注册局可新增/绑定/换绑**(无地域归属;admin_rebind 匿名 CID 不做辖区限制)。
- **已上链投票/竞选身份(civic)** → 当前必须支持注册局换绑：同市 CREG 或对应省 FRG；
  不得再列为后期功能。

## 链侧范围(S4)
1. **`occupy_cid` 改造(占即绑)**：参数固定加入 `account_id + expires_at +
   citizen_signature`；完整授权绑定创世/CID/revision=0/过期时间。占号即
   `bind_account_id`，commitment 固定为 `blake2_256(account_id)`；
   旧批量绑定调用已删除，不评估兼容或恢复。
2. **`register_voting_identity`/`upgrade_to_candidate_identity` 变纯升级**:账户已在占号绑定 → 用 `ensure_current_account_id_binding` 校验、**不再重绑**,只写 `VotingIdentityByCid`/`CandidateIdentityByCid`;仍 CTZN-only(NATP 永不可升,现有校验已保护)。出生省市镇/性别的不可改守卫扩展(D4a)。
3. **`admin_rebind_cid_account_id`**(call 7):注册局管理员换绑匿名或 civic CID；
   匿名由任一在册 CREG/FRG 办理，civic 由同市 CREG/对应省 FRG 办理；新账户签完整
   `CidRebindAuthorization`，不要求旧账户签名。
4. 费:注册局 call 走机构付费 `institution_onchain_route`(不变);admin_rebind 同。node 守卫已放行绑定变更(闭环一致即可)。
5. **测试**:occupy 占即绑 / 无账户拒 / 签名不符拒 / CTZN|NATP / register 纯升级不重绑 /
   admin_rebind 匿名任一注册局成功 + civic 同辖区成功/跨辖区拒 + 非注册局拒 +
   创世/旧账户/revision/过期时间任一改变签名拒。dev 期重新创世随母卡 S5。

## onchina 范围(S6)
- `src/domains/citizens/`:「新增公民」弹窗(`CitizenCreateModal`)改必填类型+账户+签名、档案选填;`occupy.rs` `prepare_citizen_occupy`/`encode_occupy_cid_call` 适配新 `occupy_cid` 签名(加 account + 用户签名);`admin_entry.rs` CID 生成同 CN(母卡 S1 已改 primitive,onchina wrapper 对齐)。
- 换绑入口:注册局管理员「换绑钱包」(匿名或本辖区 civic CID)→
  `admin_rebind_cid_account_id`；记录不存在于本地时同样从链上真源办理。
- **R5-split 适配**(母卡记)—— **④c 完成 ✅**:三处(`genesis_projection.rs::split_province_city`、`projection.rs::r5_province_city`、`main.rs::Db::scope_codes_from_cid`)全收敛到 primitives 权威单源 `cid_scope_codes`(自带人主体 fail-closed);程伟 live 误播种(CN 号 R5 读成省"CN")一并修正,省市改取基金会机构 CID。实际调用点是 `main.rs::Db::scope_codes_from_cid`(非母卡预估的 `docs/handler.rs`),喂机构文档 CID。详见④c段。
- 用户签名:注册局收集用户对占号/换绑的签名(扫码或输入),op_tag 复用/新增按链端定。

## 决策锁定(2026-07-27 用户拍板 A/B/C/D)
- **A 匿名鉴权** = 新 trait 方法 `can_manage_anonymous_cid(registrar, actor_cid, actor_role, action_code)`:任一在册 CREG/FRG 注册局持 citizen-identity 管理权即可,**不做辖区匹配**。因 `can_manage_voting_identity` 对空 residence fail-closed 且做辖区精确匹配,匿名 CID(无省市)不可复用。5 处 impl 同步(trait/`()`/configs Runtime/citizen-identity mock/citizen-issuance 集成 mock)。
- **B residence** = **删除** occupy_cid 的 `residence_province_code/residence_city_code` 参数。CID 为全国号、匿名无省市归属;居住地改存 onchina `citizens` 表(offchain)。CidRecord.residence 对注册局占的匿名 CID 留空(与 self_occupy 一致)。连带:`revoke_cid` 靠 rec.residence 辖区鉴权,匿名 CID(空 residence)暂无法经注册局吊销 → 归后期(不扩大 self_occupy 既有盲区)。
- **C 签名域** = `OP_SIGN_CID_OCCUPY = 0x12`，payload 固定为完整
  `CidOccupyAuthorization(genesis_hash,cid_number,account_id,revision=0,expires_at)`；
  四端逐字节同构并由金标向量锁定。
- **D batch** = **删除旧批量绑定调用**（原 call 7，无 live 调用方）+ 连带 weights/benchmark/fee_route arm/tests + 死类型；`admin_rebind_cid_account_id`（小步③）**复用释放出的 `call_index(7)`**。

## 首小步执行(链侧,原①②合并)——**完成 ✅(2026-07-27,`cargo test --workspace` 81 批次 0 failed)**
> **原①(occupy 占即绑)与原②(register/upgrade 纯升级)必须合并**:占即绑后 CID 在 occupy 即绑账户,register 必须停止重绑改 `ensure_current_account_id_binding` 纯升级,否则 register 双重绑定 + 测试账户对不齐。二者是同一绑定模型翻转的两半,不可拆。
落地(仅链侧,主检出 `citizenchain/`):
- `occupy_cid` 当前签名
  `(origin, actor_cid, actor_role, cid_number, account_id, expires_at, citizen_signature)`：
  类型 CTZN|NATP → 注册局辖区授权 → 完整首次绑定授权验签 →
  `ensure_account_id_binding_available` → `do_occupy_cid`
  (actor_cid registrar/commitment=blake2_256(account_id))→ `bind_account_id`。
- register_voting_identity/upgrade_to_candidate_identity 改 `ensure_current_account_id_binding`(不重绑);出生锁定扩展 `ensure_candidate_locked_immutable`(birth_date=`BirthDateImmutable`,birth 省市镇+性别=新 `CandidateProfileImmutable`)。
- 删除旧批量绑定调用及其死类型、weights、benchmark、fee_route arm；`call_index(7)` 释放留 admin rebind。
- 新 trait 方法 5 处 impl 全同步:trait / `()` / configs Runtime(`can_manage_anonymous_cid`=FRG|CREG+is_authorized 无辖区匹配;`verify_occupy_signature`=OP_SIGN_CID_OCCUPY sr25519)/ citizen-identity mock / citizen-issuance 集成 mock。
- 签名域 `OP_SIGN_CID_OCCUPY=0x12` + `SIGN_OP_TAGS[13]` + 金标向量回填(`6611a18d…`)。**四端(Dart/TS)镜像留 S8**。
- **连带修复①(测试红暴露)`revoke_cid`**:居住地作用域改取自 `VotingIdentityByCid`(civic 有辖区精确匹配)/ 匿名 CID(无投票身份)走 `can_manage_anonymous_cid`——占即绑删 residence 后 CidRecord 无居住地,revoke_cid 原读 rec.residence 会 fail-closed。
- **连带修复②(consensus,待用户确认可否)节点守卫** `node_guard/cid_lifecycle.rs::validate_citizen_record`:`== CTZN` → `is_person_code`(放行 CTZN|NATP,与紧邻 `valid_province_city` 一致)。S1 备注「node 守卫=第二处必改点」;此坑自 S2 self_occupy NATP 潜伏,workspace 覆盖不到。**动了共识码,用户若否决即回退。**
- 测试语义变更:多占测试用不同账户(占即绑一账户一 CID);`AccountIdByCid[占号未登记CID]` None→Some(占号账户);occupy 幂等拒改判 `CidAccountIdBindingMismatch`;新增 occupy 占即绑/NATP/坏签名/CandidateProfileImmutable 用例。
- 门禁:`cargo test -p citizen-identity` 52 绿 / `-p citizen-issuance` 7 绿 / benchmark 编译 / `cargo test --workspace` **81 批次 0 failed**。

## 小步③ + 节点守卫一致性 —— **完成 ✅(2026-07-27,`cargo test --workspace` 全绿)**
> 用户加码原则(2026-07-27):**节点守卫必须保证用户能匿名**——投票/竞选身份是用户**可选项**而非必须,只凭钱包账户 +账户控制签名即可占号/换绑。守卫只背书「CN命名空间(CTZN\|NATP)+ 注册局非空 + 状态单调 + 绑定闭环」,绝不强制姓名/出生/居住/投票身份等任何身份信息。
### Part A — 节点守卫↔runtime 一致(node_guard/cid_lifecycle.rs)
- `validate_citizen_identity_binding` 投票身份**改为可选**(存在则校验、不存在则放行)——匿名 CID(账户绑定、无投票身份)合法。此校验是增量 `check_transition:698` + 全量 `validate_full_state:863/870/884` 的**共同咽喉**,一改双通。**此坑自 S2 self_occupy 潜伏**:测试辅助 `citizen_state` 永远配对身份+绑定+非空 residence,故无测试跑过匿名 CID→workspace 覆盖不到。
- 配合首小步的 `validate_citizen_record` `== CTZN` → `is_person_code`(放行 NATP)。两处合起 = 守卫完全接受「账户-only 匿名人主体 CID」。
- 新 node 测试 `anonymous_person_cid_binds_account_without_voting_identity`:NATP 匿名 CID(空居住地、无投票身份)过 check_transition + 直调 validate_citizen_identity_binding + 换绑迁移,锁死一致性(`validate_full_state` 不适用于最简 fixture=会 SingletonInstitutionMissing,改直调咽喉函数)。
### Part B — admin_rebind_cid_account_id(链 extrinsic,`call_index(7)`=删 batch 释放槽)
- `admin_rebind_cid_account_id(origin=管理员, actor_cid, actor_role, cid_number,
  new_account_id, expected_binding_revision, expires_at, new_account_signature)`：
  finalized 读取旧账户/revision → 匿名任一 CREG/FRG 或 civic 同市 CREG/对应省 FRG →
  新账户对完整 `CidRebindAuthorization` 走 `0x1F` 控制证明 → 一账户一 CID →
  `rebind_account_id` + `CidAccountIdRebound`。**无旧账户签名**(丢钥恢复目标)。
- **专用签名域 `OP_SIGN_CID_ADMIN_REBIND=0x1F`(E1,用户拍板)**:防注册局把占号阶段(0x12)截获的签名重放成换绑,强制目标账户新鲜授权。`SIGN_OP_TAGS[14]` + 金标 `c8d1bf2d…`。四端镜像留 S8。
- 新 trait 方法 `verify_admin_rebind_signature`(6 处 impl 全同步)+ 错误 `InvalidAdminRebindSignature` + helper + weights(3)/benchmark。
- 测试覆盖：匿名换绑、civic 同市 CREG/对应省 FRG 换绑、跨辖区拒、未占拒、非注册局拒、
  坏签名拒、新账户已绑他 CID 拒，以及创世/旧账户/revision/过期时间篡改拒绝。
- 门禁:`cargo test -p citizen-identity` 58 绿 / node guard 4 绿 / `cargo test --workspace` 全绿。

## 小步④ 决策(用户 2026-07-27 定稿)
- **发号在 onchina 服务端**(注册局发起),钱包只签授权 + 给 account_id;**不由钱包生成 cid**。`citizen_cid_seed`→`cid_seed` 统一所有人主体登记发号(不造服务端随机种子轮子)。
- 流程**双方各签一次**:管理员点新增/更换→后端用同一 finalized 快照构造完整授权模板
  →用户(公民 App 或公民钱包)选择账户、原位填零槽并签完整授权→管理员回扫得
  `account_id+signature`→passkey+冷签 extrinsic→提交。
- 必填 = `account_id`(签名能力+占即绑主键)+ 类型 CTZN\|NATP;其余选填;不涉投票/竞选身份。
- CITIZEN_IDENTITY_PUSH(投票/竞选身份)**不动**(本来就好)。
- R5-split:**机构 cid 不动**(省市码照旧),只人主体(CN)改;注册局新建公民在 citizens 表有显式居住字段、不经 R5-split → ④c 收窄为 genesis 人(S5)+ 守卫。
- 命名铁律:全程 `account_id`/`accountId`；多账户授权必须使用
  `expected_old_account_id`/`new_account_id`，禁止 `account` 缩写和含糊同义字段
  [[wallet-account-naming-account-id]]。

### ④a-1 —— **完成 ✅(2026-07-27,onchina 145 tests 0 failed + byte-golden)**
- `encode_occupy_cid_call` 新字节布局:`[10][6] Compact+actor_cid ‖ Compact+actor_role ‖ Compact+cid ‖ account_id(32裸字节) ‖ Compact+occupy_signature`(删 commitment/residence)→ **字节契约恢复**;byte-golden `encode_occupy_cid_call_byte_golden` 钉死。
- 三段式:`prepare_citizen_occupy`(校验+`cid_seed`发号+pending 会话,返回 cid_number)→ 新 `submit_citizen_occupy`(验 `OP_SIGN_CID_OCCUPY` over `(cid_number,account_id)`+建 call+promote 会话+返回管理员冷签请求)→ `submit_chain_sign`(冷签+`persist_citizen_record` 写 account_id 占即绑落库)。新路由 `/api/v1/admin/citizens/occupy/submit`。
- `chain_sign_sessions` 加 pending purpose `CITIZEN_OCCUPY_PENDING` + `promote_chain_sign_session` 回填 call_data/nonce/signing_hash;`AdminCreateCitizenInput`/`ValidatedCitizenInput` 加 `cid_type`(CTZN\|NATP,必填);`cid_registry_lookup` 收敛为 `Result<bool>`(占用=存在,含墓碑)删 `OnChainCidRecord`(占即绑后 commitment 链上算,onchina 发号只判空闲)。
- **档案暂仍必填**(护照/DB 逻辑未动)。

### ④a-2 —— **完成 ✅(2026-07-27,onchina 145 tests 0 failed,0 warning)**
> 用户定稿:匿名占号只 cid+account_id,其他档案(含护照)全空、后期在详情页编辑完善(现有严格逻辑不动);投票/竞选身份仍走 CITIZEN_IDENTITY_PUSH 不碰。
> **citizens 表分区决策**:探查发现 citizens 与 subjects(机构也在内)**共用省分区**、且分区表 PK 必含分区键 → 真·去分区牵动 subjects/机构,过大。用户选 **B(保留省分区)**——理由:CID 全链公开,任一省注册局可从链上取号,onchina 每节点内嵌 PG、公民档案本地存(不做链投影),完善档案在办理注册局进行、必填出生/居住省市镇,区域天然分好;**保留分区不挡全国任何省市完善档案**(跨注册局完善=后期功能,分区不阻碍)。
落地(onchina,零表结构改):
- `AdminCreateCitizenInput` 收窄为 `{actor_role_code, cid_type}`;`validate_citizen_input` 改极简:类型 CTZN\|NATP 必填,**居住省/市取办理注册局作用域**(`get_visible_scope` locked_province/city;省级 FRG 市可空),其余档案一律空。
- `persist_citizen_record`:**不发护照**(passport_no/有效期空;护照随详情页填出生日期时由 `admin_update_citizen` 一次性签发),出生日期按字符串原样落库(空不 panic),写 `account_id`(占即绑)。
- `cid_seed`/`local_citizen_cid_seed`:出生日期改 `&str`(空不 panic);发号种子空档案靠 nonce 碰撞重试保唯一。
- `upsert_target_citizen_rows`(main.rs):省码必填、**市码可空**;`passport_no` 空则**跳过** passport_numbers 登记(避免空号主键冲突)。
- 匿名记录 province = 办理注册局省 → 落其省分区、正常出现在该局作用域列表(**无需改作用域安全过滤**)。
- 删净 4 个失活助手(`required_trimmed`/`parse_required_date`/`normalize_citizen_sex`/`resolve_citizen_scope`)。
- **编辑流 `admin_update_citizen` 完全未动**(必填完整、不可改项锁定、护照一次性签发保留)。
- 门禁:`cargo check -p onchina` 0 warning + `cargo test -p onchina` 145 绿。全工作区门禁跑中。

### ④b —— **完成 ✅(2026-07-27,onchina 146 tests 0 failed + rebind byte-golden)**
onchina admin_rebind 三段式入口(与 occupy 同构,全 `account_id`):
- `encode_admin_rebind_cid_account_id_call`(pallet 10/call 7,布局同 occupy)+ `verify_admin_rebind_signature`(域 `OP_SIGN_CID_ADMIN_REBIND`)+ `ADMIN_REBIND_CID_CALL_INDEX=7` + 两 purpose 常量。
- `prepare_citizen_rebind`(`{actor_role_code, cid_number}`) 直接读取 finalized 链上绑定真源
  并签发完整模板，不要求本地档案→`submit_citizen_rebind` 复查同一旧账户/revision/
  授权有效期后建 call+promote→`submit_chain_sign` 在 finalized 精确状态核验后更新存在的
  本地投影；本地无档案时链上换绑仍可完成。
- 路由 `/api/v1/admin/citizens/rebind/{prepare,submit}`;byte-golden `0a07…`。
- **本局限制已删除**：换绑资格只取链上 finalized CID 状态与注册局辖区授权，本地无档案
  不得成为拒绝条件。

### ④c —— **完成 ✅(2026-07-27,onchina 150 tests 0 failed + workspace 81 批次 0 failed)**
R5-split 三处收敛到 primitives 权威单源 + 修程伟 live 误播种:
- **发现**:primitives 已有 `cid::number::cid_scope_codes`(number.rs:84,文档明写「所有需按机构 CID 推作用域的模块**必须复用,不得自建第二真源**;人主体 CID 去地域化 → fail-closed 返回 Err」)。onchina 三处本地 R5 字符串切割正是它禁止的第二真源。
- **改**:`genesis_projection.rs::split_province_city`、`projection.rs::r5_province_city`、`main.rs::Db::scope_codes_from_cid` 全删自建切割、改调 `cid_scope_codes`(bytes→String 适配)。三者自动获人主体 CID fail-closed。
- **修 live bug**:程伟常量**已是 CN 号** `CN220-CTZN2-198805200-2026`(非 S5 才变;头注释 `GZ000-CTZN6` 是过期陈述,已订正)。旧 `split_province_city("CN220-…")` 把 R5 读成省"CN"/市"220" → 程伟被误播种到不存在的省"CN"。改 `seed_genesis_citizen_blocking`:程伟省市**均取自基金会机构 CID**(`GZ018-SFGYR`→GZ/018,创世同址「贵州/绥阳」),不再对程伟号调 split。
- **决策 K 复核**:机构 cid 路径零行为变化(R5 仍省2+市3);人主体号一律 fail-closed,不把号段误读成区划。
- 测试 +4:`split_province_city`/`r5_province_city` 各「基金会解析=GZ/018」+「人主体 CN 号 → Err」;`cid_scope_codes` 人主体拒绝已有 primitives 测试(number.rs:272-288)覆盖源头。
- 门禁:`cargo test -p onchina` 150 绿 / `cargo test --workspace` **81 批次 0 failed**。
- **第 4 处审计分区 —— 完成 ✅(用户 2026-07-28 拍板「在哪个注册局办理就归属哪个注册局,禁止兜底/退化/兼容」)**:`onchina/src/core/runtime_ops.rs::append_audit_log` 原按 `target_cid` R5 切省市 + "ZS" 兜底 → 人主体目标(CN 号)会算出省"CN",无 `audit_cn` 分区(分区按真实省预建)→ **INSERT 失败被 warn 吞掉 = 审计行静默丢失(live bug)**。改为**按办理该动作的本节点作用域**(`resolve_node_scope` 单源,与审计读侧 `admin_list_audit_logs` 的管理员 scope 同源)写 `province_code`/`city_code`,`target_cid` 仅存关联列;节点未绑定机构=非法调用,warn 丢弃不写(**不落错分区、无兜底**)。省/市编码与读侧过滤一致(已核 GZ/018)。
- **一处理论边角(记录,现无实际缺口)**:`delete_orphan_institutions_by_province`(main.rs:1292)按 `(province, target_cid)` 删孤儿机构审计;审计改办理局分区后,仅「联邦节点管辖跨省机构」场景审计会落联邦省而非机构省 → 该省清理漏删。但现创世联邦(NRC)与其管辖机构(基金会等)均在 GZ 同省,城市级机构办理局=同省,**无实际缺口**;真出现跨省联邦管辖时再硬化(去 province 限定,跨分区按 target_cid 删)。

### 原剩余项收口
- **小步⑤(四模块:onchina后端+前端+citizenapp+citizenwallet)已完成 ✅**：占号/
  换绑两次扫码 UI、公民域签名 QR offchain 链路及授权分层均已落地，详见下方
  ⑤-1..⑤-5 和全面审计记录。
- **本卡当前仅剩发布与测试**：S5 正式创世资产重生完成后，再执行真实占号、换绑和
  冷热钱包扫码验收。
- **跨注册局换绑已纳入当前范围**：匿名 CID 任一在册 CREG/FRG 可办，civic CID 由
  同市 CREG/对应省 FRG 办理；不得以本地无档案为由拒绝。跨注册局补录档案仍是独立的
  本地档案流程，不得反向限制链上换绑。

## 钱包账户命名统一 account_id(2026-07-27,用户强制死规则 [[wallet-account-naming-account-id]])—— **完成 ✅**
> 用户拍板 A:onchina + 前端把「当账户用」的 `public_key` 命名全部 → `account_id`/`accountId`(对齐 Substrate `AccountId`);**节点加密签名层的 `sr25519::Public` 真公钥保留**(Substrate 标准 + [[desktop-is-miner]] 节点绝不动)。语义边界:账户身份=account_id,加密验签公钥=public_key。
- onchina 16 Rust 文件 + 20 前端文件:`actor_public_key`/`signer_public_key` → `account_id`/`accountId`;chain_submit 账户参数 `public_key` → `account_id`;`ChainSignSession`/`ChainSubmitInput`/QR/auth 全改。
- DB 列 `chain_sign_sessions.actor_public_key` → `account_id`(dev 期 IF NOT EXISTS,无 migration,重建库生效)。
- 前后端 DTO 同步(login complete / action commit / chain submit 三对 body 字段 + notice 错误码映射)。
- 保留:`sr25519::Public`/`parse_sr25519_public_key*`/`public_key_hex_to_b64`/WebAuthn `publicKey`;QR 线格式紧凑键 `u/s/b.u` 不变;primitives `GenerateCidNumberInput.public_key` 字段(值已是 account_id,字段名归 primitives 所有不改)。
- 门禁:`cargo check -p onchina` 0 warning + 前端 `tsc -b` 0 error;全工作区门禁复核中。
- **未动**:node/crates/citizenapp/citizenwallet。
- **链侧 extrinsic/事件命名补正(2026-07-27,用户揪出)**:S3/小步③ 曾使用违规的
  `_account` 后缀，现已彻底删除；当前唯一名称为
  `self_rebind_cid_account_id`、`admin_rebind_cid_account_id`(call 7)和
  `CidAccountIdRebound`，lib/weights/benchmarks/tests/configs 五文件全同步。
  **④b onchina 编码器名称为 `encode_admin_rebind_cid_account_id_call`**。新增链上
  extrinsic、事件和字段必须使用 `account_id`/`AccountId`，禁止 `_account` 同义命名
  ([[wallet-account-naming-account-id]])。

## 小步⑤(四模块:onchina后端 + onchina前端 + citizenapp + citizenwallet)——完成 ✅

### 用户决策(2026-07-28)
- **公民/新钱包内层签名挑战 = 后端下发完整授权模板，钱包原位填账户槽**：
  段 1 prepare 返回绑定创世/CID/预期旧账户/revision/过期时间的模板，钱包选择本账户、
  替换唯一 32 字节零槽后签名，一次扫码输出 `{account_id, signature}`。
- **范围 = 先出三模块完整方案再动手;citizenwallet 本会话一并做**(推翻母卡「citizenwallet 另一会话」的分工,因 occupy 要求钱包也能签 + 钱包 occupy_cid decoder 是必修隐患)。

### 关键结论(Explore 侦察定论)
- op_tag `OP_SIGN_CID_OCCUPY=0x12`/`OP_SIGN_CID_ADMIN_REBIND=0x1F` 已在 `SIGN_OP_TAGS`+金标(0x12);后端 `verify_occupy_signature`/`encode_occupy_cid_call`/三段 handler 已就绪。**链侧不动**。
- 缺口 = 「occupy 域签名 QR」这条 offchain 链路(后端构造+qr-protocol登记+钱包/App解码重算),完全不存在。模板 = 详情页身份上链的 citizen_signature(action 码 2,已端到端存在)。
- **必修隐患**:钱包 `_decodeOccupyCid`(citizenwallet/lib/signer/payload_decoder.dart:2578)仍旧 5 参(commitment+省+市),与新占即绑(account_id+occupy_signature)不符 → 管理员扫段3链交易 decodeFailed 两色红拒。不修 occupy 走不完。
- QR 协议真源 = `crates/qr-protocol/registry/*.yaml` + `export_registry` 生成器,同产 citizenapp/citizenwallet 两份 `qr_action_registry.g.dart`(一致性测试 `tests/registry_consistency.rs:209` 强制)。
- 列表 join `subjects.kind='CITIZEN'`,kind 域仅 CITIZEN/PUBLIC/PRIVATE → CTZN+NATP 都返回,「放开 NATP」纯 UI(标签+类型列),非查询改。

### 子步执行序(每子步先出聚焦方案待确认→实现→门禁)
- **⑤-1 qr-protocol 契约 —— 当前**：`citizen_occupy`(code 10) 必审
  `genesis_hash/cid_number/expected_binding_revision/expires_at`；
  `citizen_rebind`(code 11) 另必审 `expected_old_account_id`。账户由钱包填模板零槽；
  生成器同步产出 CitizenApp/CitizenWallet registry，禁止端侧第二真源。
- **⑤-2 OnChina 后端挑战下发 —— 当前契约**：
  `build_domain_sign_request_bytes` 的 `b.u` 留空，`b.d` 分别携完整
  `CidOccupyAuthorization`/`CidRebindAuthorization` 零账户槽模板；
  `action_occupy()`→10、`action_rebind()`→11。外层 QR `e` 必须等于模板内
  `expires_at`，钱包只允许原位填账户槽，不得尾部追加账户。
- **⑤-3 Mobile 必修 decoder + qr-protocol 链调用对齐 —— 完成 ✅(2026-07-28)**:
  - qr-protocol：`occupy_cid`（0x0a06）required_fields 省市→`account_id`；删除旧 0x0a07 批量动作并登记 `admin_rebind_cid_account_id`；孤儿 `cid_count` 从 fields.yaml 删除；重生两份 `.g.dart`。
  - citizenwallet：call 6 decoder 严格读取
    `cid + account_id[32] + expires_at:u64 + signature:Vec64`；call 7 严格读取
    `cid + new_account_id[32] + expected_binding_revision:u64 + expires_at:u64
    + signature:Vec64`，并输出 `new_account_id`。旧批量占号 decoder/常量已删除。
  - 测试：occupy_cid/admin_rebind decoder byte golden 钉死 revision/过期时间/签名长度与
    `new_account_id` 字段名；call 7 registry 断言同步。
  - 门禁:qr-protocol 一致性 7+1 绿 + `cargo test --workspace` 81 批次 0 failed + citizenwallet `dart analyze` 0 + signer 测试 +115 全绿。
  - 范围:只碰注册局 call 6/7;self_occupy(5)/self_rebind(9)不在 ⑤。
- **⑤-4a citizenwallet occupy 域签名路径 —— 完成 ✅(2026-07-28)**:
  - **后端 `b.d` 完整授权化**：CID 使用规范 `Compact(len)+cid`，其余创世、预期旧账户、
    账户零槽、revision 和过期时间均按冻结字段顺序编码；钱包填槽后直接对完整授权走
    对应 op-tag 签名，不再拼接或追加账户字节。
  - citizenwallet：`qr_protocols` 登记 `citizenOccupy(10)/citizenRebind(11)`；
    `payload_decoder` 严格解析两类完整授权模板、规范 Compact、固定字段顺序和无尾字节；
    `qr_signer` 只允许把所选账户原位写入零槽，再分别走 `0x12/0x1F`；缺账户、非零槽、
    新旧账户相同、revision/过期时间异常均拒签。响应 `b.u` 使用所选账户。
  - UI:`scan_page` occupy/rebind 走 `_pickBindingAccount`(bottom sheet 选本钱包账户);`offline_sign_page` 展示自选绑定账户。
  - 测试：两类完整授权模板与 materialize 后签名字节 golden、缺账户/非规范 Compact/
    非零槽/尾字节/revision/过期时间/新旧相同负例，以及 call 6/7 decoder golden。
  - 门禁:`cargo test --workspace` 81 批次 0 failed + citizenwallet `dart analyze lib` 0 + signer+ui `flutter test` 全绿。
- **⑤-4b citizenapp occupy 域签名路径 —— 完成 ✅(2026-07-28)**:
  - `signing.dart` 补 `kOpSignCidOccupy=0x12`/`kOpSignCidAdminRebind=0x1F`;`qr_protocols` 加 `citizenOccupy=10/citizenRebind=11` + `isSelfAccountDomainAction`(fromDecodedAction 已走 registry fallback 自动映射);`sign_request_body.fromJson` 放开 occupy/rebind 空 `u`。
  - `qr_signer.signingBytesForHex({...,selfAccountId})` 的 occupy/rebind 分支必须与冷钱包
    相同：严格解析模板、原位填零槽后走 `0x12/0x1F`，禁止尾部追加账户；缺账户返回空。
  - 新 `citizen_occupy_sign_service.dart`(三方校验反转=用户自选账户;含 `_readBoundedCid`/`_readCompact` 从 `d` 读 cid 展示;禁冷钱包代签);`scan_dispatch_flow` 加 occupy/rebind 分派 `_handleOccupySignRequest` + `_pickBindingWallet`(bottom sheet 选热账户)。
  - 金标:App fixture `signing_domain_vectors.json` 补 0x12/0x1F(message_hex 逐字节取自 Rust 源)+ signingBytesForHex occupy/rebind golden(与 citizenwallet ⑤-4a **同 CN220 向量,冷热逐字节一致**)。
  - 门禁:citizenapp `dart analyze lib` 0 + `flutter test test/signer/` 全绿(无回归)。
- **⑤-5 onchina 前端 —— 完成 ✅(2026-07-28)**:
  - `citizens/api.ts`:`CreateCitizenInput`→**`{actor_role_code, cid_type:'CTZN'|'NATP'}`**(删全档案字段;**订正**:后端占号 prepare 只收类型+岗位码,档案占号后经「编辑资料」补,故创建弹窗极简化);`PrepareCitizenOccupyResult.sign_request`→`citizen_sign_request`;新增 `submitCitizenOccupy`(`/occupy/submit`)/`prepareCitizenRebind`(`/rebind/prepare`)/`submitCitizenRebind`(`/rebind/submit`)+ DTO。
  - `CitizenCreateModal` 重写为两次扫码(类型+岗位码;`useChainSign`×2:公民扫 `citizen_sign_request` → `submitCitizenOccupy` → 管理员扫 → `submitChainSign`)。
  - `CitizenDetailPage` 加「换绑钱包」区(岗位码 + Popconfirm;新钱包扫 `new_wallet_sign_request` → `submitCitizenRebind` → 管理员冷签)。
  - `CitizensView` 标题/按钮「公民/居民」+ 加「类型」列(从 CID 机构段派生 CTZN=公民/NATP=居民)。
  - 门禁:`tsc -b --force` 0 error(无前端单测)。

## ⑤ 全部完成 —— 注册局 CID 流程四端闭环 ✅(2026-07-28)
occupy 占即绑两次扫码(公民占号签名 + 管理员冷签)+ admin_rebind 换绑,链侧→
OnChina 后端三段 → qr-protocol 契约 → citizenwallet/citizenapp 域签名
(冷热逐字节一致)→ OnChina 前端 UI。2026-07-29 已把完整防重放授权、civic 辖区换绑、
跨注册局链上办理与 finalized 两阶段恢复纳入当前完成标准；创世前必须完成真实联调验收。

### occupy 两次扫码时序
prepare→同一 finalized 快照构造完整授权模板 → 公民/新钱包填账户槽并签 `0x12/0x1F`
→ domain submit 严格验签与竞态复查 → 管理员扫链交易 → chain/submit →
目标 extrinsic finalized+ExtrinsicSuccess → 同块精确状态核验 → 保存 finalized 标记 →
本地投影。投影失败重试只补投影。

## 全面审计(2026-07-28,security-reviewer + code-reviewer 产线索 → 逐条回原文核验)
- **[CRITICAL] 已修 ✅**:citizenwallet `offline_sign_page.dart` 占号/换绑请求(b.u 空)展示行无条件求值 `signerPublicKeyHex` getter,而 citizenwallet `_b64ToBytes` 对空串 `throw`(citizenapp 版不抛)→ 整页 FormatException 红屏、占号/换绑冷钱包端 100% 崩溃。修:改用 `QrActions.isSelfAccountDomainAction(action)` 判断绕开会抛的 getter + 加占号渲染回归 widget 测试(`test/ui/offline_sign_page_test.dart`)。dart analyze 0 + 钱包全套绿。
- **[HIGH] 授权分层倒挂 —— 已修 ✅(用户揪出:违反硬规则「查看=登录态/本地写=passkey/链上写=passkey+冷签」[[onchina-three-tier-write-auth]])**:occupy/rebind 是**链上写**(发 occupy_cid/admin_rebind extrinsic),原只有段3 冷签、漏了 prepare 的 passkey(revoke 有、它俩没有)。修:`prepare_citizen_occupy`/`prepare_citizen_rebind` 补 `require_admin_security_grant(CitizenOnchainPush)`(occupy.rs)+ 前端 `prepareCitizenOccupy`/`prepareCitizenRebind` 补 `assertPasskey`+`PASSKEY_ASSERTION_HEADER`(api.ts)。现三档合规:占号/换绑 = prepare passkey + 段3 冷签。
- **[设计非 bug] admin_rebind 无旧账户签名 + 全国无辖区**=D4b 既定(丢钥代办,靠注册局线下核验);链上有独立第二道验签(configs.rs sr25519)兜底密码学层。
- **[MEDIUM 记录] 审计失败静默丢弃**:`append_audit_log` fire-and-forget(既有框架模式,非本次回归);安全敏感事件建议独立告警通道。`consumed_at` 永不置位靠删行防重放(既有模式)。
- **测试缺口 —— 已补 ✅**:①`verify_occupy_signature`/`verify_admin_rebind_signature` 加 sign→verify 往返 + 域分离拒绝测试(occupy.rs,onchina 154 绿);②新 `citizen_occupy_sign_service_test.dart`(citizenapp 6 项:prepare 解 CID/换绑 isOccupy/非占号拒/冷钱包拒/畸形 d 拒 + **硬编码常量 vs registry 防漂移断言**);③CRITICAL 的 occupy 渲染回归 widget 测试(citizenwallet)。
- **已核验无问题（排除误判）**：旧批量动作与 `cid_count` 已从当前协议删除；命名全为 `account_id`；op 域 0x11/0x12/0x1F 严格分离；冷热四端签名字节逐字节一致（golden 兜底）；段2/段3 会话防重放（已消费检查）+`same_account_id` 防越权+验签失败即拒；空 b.u 放开仅限 occupy/rebind；旧全档案字段/getCidMeta 等无孤儿。
- **门禁**:`cargo test --workspace` 81 批次 0 + citizenwallet/citizenapp `dart analyze` 0 + 两端 flutter test 全绿 + 前端 tsc 0。

## 门禁
`cargo test --workspace` 全绿 + onchina 测试;链↔onchina occupy 调用字节契约一致;冷热链一致。
