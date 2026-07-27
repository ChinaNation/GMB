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

## 门禁
`cargo test --workspace` 全绿 + onchina 测试;链↔onchina occupy 调用字节契约一致;冷热链一致。
