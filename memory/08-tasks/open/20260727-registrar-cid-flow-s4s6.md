# 注册局 CID 流程改造(S4 链 + S6 onchina,合并一大步)

状态:open(交接新窗口实现;用户 2026-07-27 拍板 D4a/D4b)
所属模块:Chain(citizenchain runtime)+ OnChina(注册局);**链与 onchina 强耦合,必须同批**([[no-compatibility]] 不留过渡)
母卡:`20260727-citizenapp-cid-identity-rootless-wallet.md`(S1-S3 已完成:CN 去地域化 CID、自助占号 `self_occupy_cid`、自助换绑 `self_rebind_cid_account`、`OP_SIGN_CID_REBIND` 签名域)

## 已完成地基(S1-S3,新窗口直接复用)
- CID = CN 前缀去地域化、人主体(CTZN/NATP/SMTP)12 位号段(`primitives/cid`)。
- `self_occupy_cid`(call 5):公民本人自助占号+占即绑+匿名+自付费;类型 CTZN|NATP(`ensure_valid_self_registrable_cid`);SELF sentinel registrar + 空 residence;commitment=blake2_256(account_id)。
- `self_rebind_cid_account`(call 9):新账户 origin + 旧账户 detached 签名(`OP_SIGN_CID_REBIND`);匿名限定(civic 拒);`rebind_account_id` 原语;`verify_rebind_signature`(4 处实现)。
- 复用 helper:`ensure_account_id_binding_available`/`bind_account_id`/`rebind_account_id`/`do_occupy_cid`(已剥离 CTZN 硬校验,类型校验下沉到调用方)。

## D4a 决策(注册局占号即绑账户 + 档案选填)
- **占号即绑钱包账户**:注册局「新增公民」弹窗**必填** = ①类型 CTZN|NATP ②用户钱包账户 ③用户签名(证账户控制)。
- **档案全改选填**:姓/名/居住省市镇/出生省市镇/出生日期/性别 等全部选填(可留空)。
- **一经填写不可改**:出生日期、出生省市镇、性别(可留空,但填后锁定)。链上 `CandidateIdentity.birth_date` 已有不可改先例,扩展到出生省市镇/性别。

## D4b 决策(注册局写入模型 + 换绑管辖)
- 写入模型:**注册局CID + 管理员岗位码 + 管理员钱包账户(origin)+ 用户CID + 用户钱包账户**。
- **匿名用户(仅上链 CID+账户,无投票/竞选)→ 任一注册局可新增/绑定/换绑**(无地域归属;admin_rebind 匿名 CID 不做辖区限制)。
- **已上链投票/竞选身份 → 必须在对应注册局改**(有链下数据)= **后期功能**,本步不做 civic 的 admin 换绑。

## 链侧范围(S4)
1. **`occupy_cid` 改造(占即绑)**:加 `account_id` + `citizen_signature` 参数;占号即 `bind_account_id`;类型校验 CTZN|NATP(`ensure_valid_self_registrable_cid`);residence 改可选(D4a)。commitment 基建议统一 `blake2_256(account_id)`(与 self_occupy 一致,因档案可选、不能再用档案种子)。`occupy_cids_batch` 同步或评估是否保留。
2. **`register_voting_identity`/`upgrade_to_candidate_identity` 变纯升级**:账户已在占号绑定 → 用 `ensure_current_account_id_binding` 校验、**不再重绑**,只写 `VotingIdentityByCid`/`CandidateIdentityByCid`;仍 CTZN-only(NATP 永不可升,现有校验已保护)。出生省市镇/性别的不可改守卫扩展(D4a)。
3. **新 `admin_rebind_cid_account`**(call 下一空位):注册局管理员换绑**匿名** CID;origin=管理员 + `T::CitizenIdentityAuthority::can_manage_voting_identity` 鉴权 + 新账户 `citizen_signature`(证新账户控制);匿名 CID 无 residence → 任一在册注册局可办(D4b,不做辖区匹配)。civic 换绑留后期。
4. 费:注册局 call 走机构付费 `institution_onchain_route`(不变);admin_rebind 同。node 守卫已放行绑定变更(闭环一致即可)。
5. **测试**:occupy 占即绑 / 无账户拒 / 签名不符拒 / CTZN|NATP / register 纯升级不重绑 / admin_rebind 匿名成功 + civic 拒 + 任一注册局可办。dev 期重新创世随母卡 S5。

## onchina 范围(S6)
- `src/domains/citizens/`:「新增公民」弹窗(`CitizenCreateModal`)改必填类型+账户+签名、档案选填;`occupy.rs` `prepare_citizen_occupy`/`encode_occupy_cid_call` 适配新 `occupy_cid` 签名(加 account + 用户签名);`admin_entry.rs` CID 生成同 CN(母卡 S1 已改 primitive,onchina wrapper 对齐)。
- 换绑入口:注册局管理员「换绑钱包」(匿名 CID)→ `admin_rebind_cid_account`。
- **R5-split 适配**(母卡记):`genesis_projection.rs::split_province_city`、`projection.rs::r5_province_city`、`docs/handler.rs::scope_codes_from_cid` —— 公民 CN CID 不再从 R5 推省;省份取自 `citizens` 表居住字段。
- 用户签名:注册局收集用户对占号/换绑的签名(扫码或输入),op_tag 复用/新增按链端定。

## 决策锁定(2026-07-27 用户拍板 A/B/C/D)
- **A 匿名鉴权** = 新 trait 方法 `can_manage_anonymous_cid(registrar, actor_cid, actor_role, action_code)`:任一在册 CREG/FRG 注册局持 citizen-identity 管理权即可,**不做辖区匹配**。因 `can_manage_voting_identity` 对空 residence fail-closed 且做辖区精确匹配,匿名 CID(无省市)不可复用。5 处 impl 同步(trait/`()`/configs Runtime/citizen-identity mock/citizen-issuance 集成 mock)。
- **B residence** = **删除** occupy_cid 的 `residence_province_code/residence_city_code` 参数。CID 为全国号、匿名无省市归属;居住地改存 onchina `citizens` 表(offchain)。CidRecord.residence 对注册局占的匿名 CID 留空(与 self_occupy 一致)。连带:`revoke_cid` 靠 rec.residence 辖区鉴权,匿名 CID(空 residence)暂无法经注册局吊销 → 归后期(不扩大 self_occupy 既有盲区)。
- **C 签名域** = 新增 `OP_SIGN_CID_OCCUPY = 0x12`(哈希域),payload=`(cid_number, account_id).encode()`,新 `verify_occupy_signature`;`SIGN_OP_TAGS` 扩到 [13] + 金标向量补 0x12。四端(Dart/TS)镜像留 S8。
- **D batch** = **删除** `occupy_cids_batch`(call 7,无 live 调用方)+ 连带 weights/benchmark/fee_route arm/tests + 死类型 `CidOccupyItem/CidOccupyItemsBound/MAX_CID_OCCUPY_BATCH`;`admin_rebind_cid_account`(小步③)**复用释放出的 `call_index(7)`**。

## 首小步执行(链侧,原①②合并)——**完成 ✅(2026-07-27,`cargo test --workspace` 81 批次 0 failed)**
> **原①(occupy 占即绑)与原②(register/upgrade 纯升级)必须合并**:占即绑后 CID 在 occupy 即绑账户,register 必须停止重绑改 `ensure_current_account_id_binding` 纯升级,否则 register 双重绑定 + 测试账户对不齐。二者是同一绑定模型翻转的两半,不可拆。
落地(仅链侧,主检出 `citizenchain/`):
- `occupy_cid` 新签名 `(origin, actor_cid, actor_role, cid_number, account_id, citizen_signature)`:类型 CTZN|NATP → `can_manage_anonymous_cid` → `verify_occupy_signature` → `ensure_account_id_binding_available` → `do_occupy_cid`(actor_cid registrar/空 residence/commitment=blake2_256(account))→ `bind_account_id`。
- register_voting_identity/upgrade_to_candidate_identity 改 `ensure_current_account_id_binding`(不重绑);出生锁定扩展 `ensure_candidate_locked_immutable`(birth_date=`BirthDateImmutable`,birth 省市镇+性别=新 `CandidateProfileImmutable`)。
- 删 `occupy_cids_batch` + 死类型 `CidOccupyItem/CidOccupyItemsBound/MAX_CID_OCCUPY_BATCH` + weights(3)/benchmark/fee_route arm;`call_index(7)` 释放留 admin_rebind。
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
### Part B — admin_rebind_cid_account(链 extrinsic,`call_index(7)`=删 batch 释放槽)
- `admin_rebind_cid_account(origin=管理员, actor_cid, actor_role, cid_number, new_account_id, new_account_signature)`:老账户反查(无→`NotBoundToAnyCid`)→ 匿名限定(有投票身份→`CivicRebindRequiresRegistrar`)→ `can_manage_anonymous_cid`(任一注册局,action 7)→ 新账户 `OP_SIGN_CID_ADMIN_REBIND(0x1F)` 控制证明 → 一账户一 CID → `rebind_account_id` + `CidAccountRebound` 事件。**无旧账户签名**(用户丢钥、注册局代办 D4b);费=机构付费。
- **专用签名域 `OP_SIGN_CID_ADMIN_REBIND=0x1F`(E1,用户拍板)**:防注册局把占号阶段(0x12)截获的签名重放成换绑,强制目标账户新鲜授权。`SIGN_OP_TAGS[14]` + 金标 `c8d1bf2d…`。四端镜像留 S8。
- 新 trait 方法 `verify_admin_rebind_signature`(6 处 impl 全同步)+ 错误 `InvalidAdminRebindSignature` + helper + weights(3)/benchmark。
- 6 单测:成功换绑 / 未占拒 / civic 拒 / 非注册局拒 / 坏签名拒 / 新账户已绑他 CID 拒。
- 门禁:`cargo test -p citizen-identity` 58 绿 / node guard 4 绿 / `cargo test --workspace` 全绿。

## 小步④ 决策(用户 2026-07-27 定稿)
- **发号在 onchina 服务端**(注册局发起),钱包只签授权 + 给 account_id;**不由钱包生成 cid**。`citizen_cid_seed`→`cid_seed` 统一所有人主体登记发号(不造服务端随机种子轮子)。
- 流程**双方各签一次**:管理员点新增/更换→弹窗用户签名请求二维码→用户(公民app 或 公民钱包皆可)签 `(cid_number, account_id)`→管理员回扫得 account_id+签名→passkey+冷签 extrinsic→提交。
- 必填 = `account_id`(签名能力+占即绑主键)+ 类型 CTZN\|NATP;其余选填;不涉投票/竞选身份。
- CITIZEN_IDENTITY_PUSH(投票/竞选身份)**不动**(本来就好)。
- R5-split:**机构 cid 不动**(省市码照旧),只人主体(CN)改;注册局新建公民在 citizens 表有显式居住字段、不经 R5-split → ④c 收窄为 genesis 人(S5)+ 守卫。
- 命名铁律:全程 `account_id`/`accountId`,签名 payload `(cid_number, account_id)`,禁 `account` 缩写 [[wallet-account-naming-account-id]]。

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

### 剩余
- ④b:admin_rebind 入口(三段同构,`OP_SIGN_CID_ADMIN_REBIND`,call 7,新 purpose)。
- ④c:R5-split genesis 人(随 S5 重生)+ 守卫;机构不动。
- **跨注册局完善(后期)**:在非创建局完善档案 = 该局从链上取 CID+账户 → 在本局 PG 建/完善本地记录(现有编辑要求本地已有记录,需新流程)。

## 钱包账户命名统一 account_id(2026-07-27,用户强制死规则 [[wallet-account-naming-account-id]])—— **完成 ✅**
> 用户拍板 A:onchina + 前端把「当账户用」的 `public_key` 命名全部 → `account_id`/`accountId`(对齐 Substrate `AccountId`);**节点加密签名层的 `sr25519::Public` 真公钥保留**(Substrate 标准 + [[desktop-is-miner]] 节点绝不动)。语义边界:账户身份=account_id,加密验签公钥=public_key。
- onchina 16 Rust 文件 + 20 前端文件:`actor_public_key`/`signer_public_key` → `account_id`/`accountId`;chain_submit 账户参数 `public_key` → `account_id`;`ChainSignSession`/`ChainSubmitInput`/QR/auth 全改。
- DB 列 `chain_sign_sessions.actor_public_key` → `account_id`(dev 期 IF NOT EXISTS,无 migration,重建库生效)。
- 前后端 DTO 同步(login complete / action commit / chain submit 三对 body 字段 + notice 错误码映射)。
- 保留:`sr25519::Public`/`parse_sr25519_public_key*`/`public_key_hex_to_b64`/WebAuthn `publicKey`;QR 线格式紧凑键 `u/s/b.u` 不变;primitives `GenerateCidNumberInput.public_key` 字段(值已是 account_id,字段名归 primitives 所有不改)。
- 门禁:`cargo check -p onchina` 0 warning + 前端 `tsc -b` 0 error;全工作区门禁复核中。
- **未动**:node/runtime/crates/citizenapp/citizenwallet。

## 小步⑤(onchina 前端)
`CitizenCreateModal` 必填类型+account_id+签名、档案选填 + 「换绑钱包」入口;公民 tab/列表改「公民/居民」覆盖 CTZN+NATP,后端列表查询放开 NATP。

## 门禁
`cargo test --workspace` 全绿 + onchina 测试;链↔onchina occupy 调用字节契约一致;冷热链一致。
