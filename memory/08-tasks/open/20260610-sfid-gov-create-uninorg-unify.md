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

## 追加（2026-06-10 用户确认后落地的 UI 调整）

- 新增弹窗 label 去技术前缀:「SubjectProperty 主体属性」→「主体属性」、「P1 盈利属性」→「盈利属性」;机构详情页展示标签同步改「盈利属性」(全站用词统一)。
- 新增弹窗改双列布局(Row/Col span=12)压低高度:行1 主体属性|盈利属性、行2 省|市、行3 机构|学校名称/机构名称(两步式右侧留空)、行4 法定代表人姓名|证件照;所属法人(F)和法定代表人身份ID 因内容长(26 字符身份ID/机构名+省市)保持整行。三入口(私权/公权/教育)弹窗同时生效,纯 JSX 布局零逻辑改动。

## 追加 2（2026-06-10 用户两点指令）

- 公权 tab 下属非法人(F)的机构选项锁死「中国 (ZG)」,不再提供他国 TG(labels.ts GOV_UNINORG_INSTITUTION_ONLY,单项自动置灰);私权 F 仍 ZG/TG 可选,教育分校仍锁 JY。
- 「公安局」tab 标签改为「市公安局」(App.tsx label,路由 key/接口/缓存键不动)。

## 追加 3（2026-06-10 用户确认:系统代码不上前端）

- 机构详情(PrivateDetailLayout):「盈利属性」由 `1/盈利` 改纯中文 `盈利`;「机构代码」标签改「机构」、值由 `ZG/中国` 改纯中文 `中国`(映射缺失回退原代码仅作异常兜底)。GovDetailPage「机构类型」本就纯中文,不动。
- 新增弹窗下拉选项全部去括号代码:公法人/私法人/非法人、盈利/非盈利、政府/立法院/司法院/监察院、中国/他国、教育委员会、市名(原"合肥市 (001)"→"合肥市")共 18 处;value 仍是系统代码,前后端流转零变化。
- FRONTEND_LAYOUT.md 增设铁律:系统代码只在 value 与后端流转,前端展示与选项一律纯中文。

## 追加 4（2026-06-10 市公安局列表三状态列合一,A 方案）

- 市公安局列表删除「CPMS状态/安装码状态/身份码业务状态」三列,合成唯一「业务状态」列——三列本是同一条流水线的三个视角,身份码业务状态本就由另两者派生(main.rs identity_service_status)。
- 单轴六态:待生成安装码(新拆档,无 CPMS 站点记录)→ 待安装(安装码已生成待现场扫码)→ 待绑定身份码 → 可办理,+ 已禁用/已吊销;颜色 可办理=绿、等待态=橙、禁用/吊销=红。
- 后端派生逻辑细分 None→WAITING_INSTALL_CODE / PENDING→WAITING_INSTALL;cpms_status/install_token_status 仍作派生输入随行返回,前端不再展示;删 GovListTable 两个死 label 表 + statusTag 死颜色分支。

## 追加 5（2026-06-10 前端账户地址统一完整 SS58）

- 机构操作记录(GovDetailPage OperationRecords):「操作者」改「操作者账户」,actor_pubkey 由裸 0x hex+省略号改为完整 SS58(tryEncodeSs58,等宽小字+换行不截断)。
- 机构账户列表(AccountList)「账户地址」:已是 SS58 但截断(前12...后8),改完整显示;交易哈希不是账户,保持截断。
- 公民详情「投票账户」:wallet_address 缺失时原 fallback 直显 wallet_pubkey 裸 hex,改为转 SS58 兜底。
- 全仓扫描结论:其余展示点(省/市管理员列表与详情、扫码账户弹窗、公民列表 wallet_address 后端已给 canonical SS58)均已完整 SS58,无残留;FRONTEND_LAYOUT 增设「账户地址统一完整 SS58」铁律。

## 追加 6（2026-06-10 操作记录操作类型全量中文映射）

- 机构操作记录「操作」列加 AUDIT_ACTION_LABEL 全量中文映射(GovDetailPage)。**单一来源=后端 append_audit_log 各调用点的 action 字面量,经全仓扫描共 10 个**(11 个调用点全为字面量,AdminActionType.label() 走的是授权签名展示不入审计表):生成/重新生成 CPMS 安装码、CPMS 授权状态变更、删除 CPMS 授权、导入 CPMS 年度报告、CPMS 档案码核验、公民身份ID绑定、公开身份查询、App 选民人数查询、App 投票凭证签发。未知 action 回退显示原标识兜底;后端新增 action 须同步补映射(映射表注释已写明)。

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
