# 任务卡：公权机构手动创建放开 + 非法人创建统一（所属法人创建即挂 + 地域规则）

状态：代码完成（待用户端到端 QA）
创建：2026-06-10
完成：2026-06-10
模块：SFID（frontend + backend）
来源：用户三轮需求确认（公权新增按钮恢复两能力 / 统一非法人组件流程 / 非法人地域规则）。
前置：20260610-sfid-education-tab-jy-unify（教育机构 tab 收口，本卡修订其分校上级选择方式）。

## 已确认口径

1. **公权机构手动创建放开**：G + 政府ZF/立法院LF/司法院SF/监察院JC 四类（排除中央银行CB——省公民储备银行每省唯一已生成；排除JY——归教育 tab）；公安局命名守卫保留；机构名称必填同市查重；法定代表人必填；p1 锁非盈利；市级管理员注册本市。手动公权机构**进浏览目录**（official 查询并入手动行，reconcile 自动目录机制不动）。
2. **非法人(F)统一流程**（私权/公权/教育三入口同一组件）：创建时**必选所属法人**（真实父级，创建即写 parent_sfid_number），盈利属性继承所属法人（公法人父→锁0；私法人父→继承其 p1）；"非法人必须从属法人"立为后端硬规则（开发库 F 存量=0，零迁移）。教育 tab 上一卡的"上级法人属性/上级盈利属性"推导选择器删除，换真实本部学校选择器。详情页关联所属法人保留，语义=改挂，同套校验。
3. **列表按所属法人分流**：父=私法人→私权；父=公法人→公权（含浏览目录）；父=教育机构（JY 学校）→教育。F+JY ⇔ 父级必须是 JY 学校（代码一致性校验）。
4. **地域规则**：教育本部→分校同市；公法人父按层级（org_code 前缀）：市/镇级及手动公权行→同市，PROVINCE_→同省，国家级（NATIONAL_/MINISTRY_/FEDERAL_）→全国，未知前缀防御性同市；私法人父→全国不限。三处同源：创建校验 + 改挂校验 + 所属法人搜索预过滤。

## 实施清单（全部完成）

### 后端（sfid/backend）
- [x] subjects/uninorg：单一权威源——`parent_locality_rule`（org_code 前缀判层级）+ `code_consistency_violation`（F+JY⇔学校本部）+ `locality_violation` + `inherited_p1` + 6 条单元测试；模块头注释写明"三处同源缺一有绕过口"
- [x] subjects/model.rs：CreateInstitutionInput 加 parent_sfid_number；ParentInstitutionRow 加 p1
- [x] private/handler.rs create_institution：放开 G 非 JY（仅 ZF/LF/SF/JC，CB/其它拒绝）；手动 G 统一市级管理员（教育学校/公权机构分别提示）；F 必传 parent + 存在/属性/代码一致/地域/p1 继承五连校验 + 创建即写 parent；grant payload 加 parent_sfid_number（前后端同步）
- [x] subjects/admin.rs update_institution 改挂：补与创建同源的代码一致性+地域+p1 继承校验（p1 烧死在号段，改挂只能挂继承值一致的父级）
- [x] subjects/admin.rs search_parent_institutions：f_institution/province/city/parent_property 四参数，省市必传（缺参拒绝不退化全国搜索）；分校模式=本市学校本部；普通模式=S 全国 ∪ G 按层级地域（split_part 判前缀）；SELECT 加 p1 + status=ACTIVE
- [x] InstitutionListFilter 父级感知：list_institutions_exact 加 `LEFT JOIN subjects par`（按 sfid_number 关联不限分区，父级允许跨省）；Private 排除父=G 的 F（父缺失防御性兜底私权可见）；Gov 并入父=G 的 F
- [x] main.rs list_official_institutions_scope：INNER→LEFT JOIN gov + LEFT JOIN par；目录 = 自动目录(排公安局) + 手动 G(org_code 空非 JY) + 父=G 的 F；kind 扩 PRIVATE

### 前端（sfid/frontend）
- [x] subjects/api.ts：CreateInstitutionInput.parent_sfid_number + ParentInstitutionRow.p1 + SearchParentsOptions 共享类型
- [x] subjects/labels.ts：CreateFormCategory 三值；GOV case（G/F 双属性 + GOV_MANUAL_INSTITUTIONS=ZF/LF/SF/JC）；统一 `p1LocksForSubject`（G 锁0 / S 可选默认1 / F 继承父级未选置空）+ `inheritedP1` + `institutionChoicesFor`；删 educationP1Locks / dynamicLocksForSubjectProperty / InstitutionFieldLocks.p1Choices（残留扫描零命中）
- [x] CreateInstitutionForm 整体改造：F 统一"所属法人"搜索选择器（必须从结果中选定，手填不放行；换市清空重选）；p1 随父级选定自动继承锁死；GOV 模式（G 名称必填同市查重 / F 两步式）；教育删两个推导选择器
- [x] 三模块 api.ts：searchParentInstitutions 各自包装（私权 parentProperty=S / 公权 =G / 教育分校模式）；createInstitution grant payload 全部加 parent_sfid_number
- [x] gov/api.ts 恢复创建三件套；GovCreateModal 重建（G+F 双能力）；GovView 新增按钮/弹窗恢复（仅公权分支，公安局无）
- [x] PrivateDetailLayout 改挂搜索改传 fInstitution/province/city（与创建同源预过滤）

### 文档/收尾
- [x] FRONTEND_LAYOUT.md：表单唯一实现三入口、公权两能力、非法人统一流程/分流规则/地域规则全部改写
- [x] 上一卡（education-tab-jy-unify）补修订记录；EducationView 过时注释改"改挂"
- [x] 残留扫描：educationP1Locks/dynamicLocksForSubjectProperty/parent_subject_property/parent_p1 全仓零命中
- [x] cargo fmt 本卡触碰文件干净（cpms/handler.rs、private/clearing.rs 的 fmt 差异属上一任务/历史存量，不在本卡范围）

## 验证记录

- 后端 `cargo check` 0 error + `cargo test` 58/58（含 uninorg 6 条新单测：学校本部同市/私法人全国/公法人层级三档+未知前缀收紧/地域校验/分校代码一致性/p1 继承）。
- 前端 `npm run build`（tsc -b + vite build）0 error。
- **开发库实测（事务内造数后 ROLLBACK，不落库）**：
  - 精确搜索三分支：私企+私权下属F→私权 ✓；公立研究院(手动G)+公权下属F→公权 ✓；本部+分校→教育 ✓；交叉零泄漏 ✓
  - 公权浏览目录：自动目录保留 + 手动 G + 公权下属 F 并入 ✓；JY 本部/分校/私企不进 ✓
  - 所属法人搜索：分校模式只命中本市本部（他市学校/监管本体/私企排除）✓；私权模式私企命中、学校本部排除 ✓；公权模式省级同省命中（广东省联邦政府被排除）、市级同市、国家级不限 ✓
- **数据事实**：org_code 前缀分布 TOWN/CITY/PROVINCE/MINISTRY/FEDERAL/NATIONAL 六类、零 PUBLIC_ORG 残量；F 机构存量 0（必挂规则零迁移生效）；省市字段存储带后缀（安徽省/合肥市），前后端同源传值。

## 待用户端到端 QA

1. 公权 tab：市详情页「+ 新增」恢复；G 可选 政府/立法院/司法院/监察院 四类（无央行/教育委员会），名称查重同市，建成后出现在公权浏览目录；F 选所属法人只能搜到本市市级、本省省级、国家级公权机构，盈利属性锁非盈利。
2. 教育 tab：分校(F)必选本市学校本部为所属法人，盈利属性继承本部；不再出现"上级法人属性"两个手选项。
3. 私权 tab：F 必选私法人所属法人（全国可选），盈利属性继承。
4. 跨域负例：他省机构名搜所属法人(公权/教育入口)返回空；改挂到他市市级机构被后端拒绝。
5. 公安局 tab 无新增按钮、回归不变。
