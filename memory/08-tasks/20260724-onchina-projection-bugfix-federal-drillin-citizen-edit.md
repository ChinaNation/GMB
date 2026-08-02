# OnChina 投影 bug 修复:联邦按市 drill-in 私权可见 + 公民编辑入口(不可变字段保护)

任务需求(用户报障 2 项):
1. 联邦注册局省组管理员进任何市 → 应能查看该市所有私权机构的链上字段。
   现状:M3 drill-in 端点存在但未接触发,联邦库拿不到该市私权机构 → 看不到。
   **修法:M3 drill-in(不是播种!)接到"联邦管理员按市查看私权机构"的读取路径**——
   联邦(Tier1)管理员请求某市私权机构列表时,先对该 (省,市) 做私权 drill-in 投影,再返回列表。
2. 加公民编辑入口,但**不可变字段一经初始化保存成功就永久锁定,不能再改**:
   - 不可变(initialize-once):family_name/given_name/citizen_sex/citizen_birth_date(出生日期)、
     birth_province/city/town_code(出生地)、passport_no(护照号)。
   - 可变:passport_valid_from/until(护照有效期)、residence 省/市/镇、voting_eligible。
   规则:某不可变字段本地现存非空 → 拒绝改动(保持原值);现存为空 → 允许设置(初始化),之后锁定。
   目的:让程伟这类"待补档"公民能补齐护照有效期等 → computed_identity_status 转 Normal → 可推链。

所属模块:citizenchain/onchina(后端 + 前端)

必须遵守:
- Bug1 用 M3 drill-in(已实现的 drill_in_project_scope 私权部分),不得播种;
- 不可变字段永久锁定(初始化后),防篡改出生日期/护照号/出生地;
- 编辑只动本地正本字段;链投影字段由 M1/M3 覆盖(保正本不变);
- 不破坏现有创建/上链/去重流程。

输出物:后端 drill-in 触发 + 公民编辑端点 + 前端编辑 Modal + 中文注释 + 测试 + 文档回写。

---

## 落地实况(2026-07-24 完成)

### Bug1:联邦按市 drill-in 私权可见
- `domains/projection.rs` 已有 `drill_in_project_private_scope(db, scope)`(遍历链上私权机构 CID,
  命中 (省,市) 即 `project_private_institution_by_cid`)。
- 接线点:`institution/subjects/registration.rs::list_institutions_inner`——当
  `filter==Private && is_tier1_registry(ctx.institution_code) && city_code.is_some()` 时,
  先 `drill_in_project_private_scope` 再 `list_institutions_exact`。链读失败只 `warn` 不阻断(fail-open 展示)。
- **不是播种**:联邦库仍不常驻私权,进哪个市投影哪个市。

### Bug2:公民编辑入口(字段可变性按"现实是否可变"定死)
- 端点:`POST /api/admin/citizens/:cid_number/edit` → `domains/citizens/admin_entry.rs::admin_update_citizen`
  (main.rs 已注册路由,位于 revoke/prepare 之前)。
- **字段分类订正(用户第二轮纠正)**:
  - **可变**:姓 family_name、名 given_name、居住市 city_code、居住镇 town_code、voting_eligible
    (人现实中会改名、会搬家)。姓名不进 `LockedIdentity`,handler 直接取 input(必填非空)。
  - **不可变(现实不可变,初始化后永久锁定)**:citizen_sex(性别)、citizen_birth_date(出生日期)、
    birth_province/city/town_code(出生地)、passport_no(护照号)。核心 `lock_immutable`
    (现存非空→拒改;空→初始化)+ `resolve_locked_identity` + `validate_locked_identity`。
  - **固定不动**:居住省 province_code(= CID 省 = 分区键)、公民 CID(主键)。
    **跨省居住迁移属"跨地区",涉及分区迁移+注册局交接,本入口暂不做(用户:先能改,跨地区后面再说)**。
    → 居住市/镇可改但限本省内(`validate_locked_identity` 按 existing.province_code 校验行政区归属)。
- **护照号/有效期**:服务端确定性签发(`allocate_passport_no` + 出生日期派生年限,与建档
  `persist_citizen_record` 同源),不接受前端直填;现存已签发则锁定原样保留;现存为空且出生
  日期就绪时以最终居住省市为命名空间签发一次。→ 首版误当可变入参已废弃(护照号唯一性/有效期口径)。
- 账户/状态/onchain_*/创建人/created_at 一律保留;archive_hash 重算。
- 前端:`citizens/EditCitizenModal.tsx`(姓名/居住市镇可编辑;性别/出生日期/出生地按现存值锁定
  非空只读、空放开级联录入;居住省只读;居住市为固定省下级联 Select)+
  `CitizenDetailPage.tsx` 标题右侧"编辑资料"入口 + `citizens/api.ts::editCitizen`。

### 验证
- onchina:`cargo check` 通过;`cargo test` 144 passed / 0 failed;新增 6 条 edit 单测
  (初始化/出生日期锁定拒改/姓名可变不锁定/一致或空保持/选举资格需出生日期/性别非法)。clippy 新代码仅 1 处
  `expect_used`(与同文件 persist_citizen_record 同模式,全仓 28 处既有)。
- 前端:`tsc --noEmit` 0 错;`npm run build`(tsc -b + vite build)通过。
- 浏览器可视化验证需完整后端(PG+链+管理员 passkey 登录+已建公民),此环境无法端到端触达,
  故以构建+类型+后端单测为准。

状态:完成(2026-07-24)
