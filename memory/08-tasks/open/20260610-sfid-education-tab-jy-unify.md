# 任务卡：SFID 新增「教育机构」Tab（JY 学校机构统一收口）

状态：代码完成（待用户端到端 QA）
修订（2026-06-10，由 20260610-sfid-gov-create-uninorg-unify 卡落地）：
- 分校(F)的"上级法人属性/上级盈利属性"两个推导选择器已删除，替换为真实"所属法人(学校本部)"
  搜索选择器（本市本部，盈利属性自动继承，创建即挂 parent_sfid_number）。
- 公权机构页面新增按钮恢复（新语义：G 公权机构 ZF/LF/SF/JC + F 公权下属非法人，与本卡删除的
  JY-only 旧按钮不同）；本卡"公权新增按钮整个删除"一项被新需求取代。
创建：2026-06-10
完成：2026-06-10
模块：SFID（frontend + backend）
来源：用户需求 + 两轮边界确认 + 计划批准。

## 需求（已确认）

在「私权机构」和「公权机构」tab 之间新增「教育机构」tab，专管教育委员会（institution_code=JY）类学校机构：

1. 所有手动注册的 JY 机构（G 公法人=公立学校、S 私法人=私立学校、F 非法人=分校）统一只在教育机构 tab 管理；私权/公权列表不再显示、不再能新增 JY。
2. 自动生成的监管机构本体（国家教育委员会/公民教育委员会，gov 表内、org_code=CITY_EDU/NATIONAL_EDU）留在公权机构目录不动。区分依据：手动创建只写 subjects 且 org_code=None → `institution_code='JY' AND org_code IS NULL` 精确圈定手动学校。
3. 教育机构新增表单：主体属性 G/S/F 三选；G → p1 锁「0 非盈利」；S → p1 可选 0/1；F → 先选「上级法人属性」(G/S)：上级=G 锁 0，上级=S 再选上级盈利属性、F 的 p1 跟随。上级法人属性仅推导 p1，不在创建时关联具体机构（详情页 search-parents 流程不动）。机构锁 JY，学校名称/法定代表人必填。
4. 公权页面新增按钮整个删除（手动新增本来只有 JY）；私权机构选项删 JY（剩 ZG/TG）。

后端创建链路零改动（JY 市级管理员限制、G 强制 p1="0" 由 generator.rs 硬规则保证）；唯一后端改动是 /api/v1/institution/list 过滤。

## 实施清单

### 后端（sfid/backend）
- [x] subjects/model.rs：新增 `InstitutionListFilter` 枚举（Private=排除 JY / Gov=排除手动 JY / Education=JY AND org_code IS NULL 跨两 category），静态 SQL 子句无注入面；subjects/mod.rs 导出
- [x] private/handler.rs list_institutions：category 接受 PRIVATE_INSTITUTION/GOV_INSTITUTION/EDUCATION_INSTITUTION 三值映射枚举（统一切换，无兼容值）
- [x] main.rs list_institutions_exact：签名 `Option<&str>` → 枚举，SQL 用 format! 拼静态子句，参数 $1..$5 整体前移（accounts 子查询 $2/$3 → $1/$2）

### 前端（sfid/frontend）
- [x] subjects/labels.ts：删 G_NONPROFIT_GOV；PRIVATE_INSTITUTIONS 删 JY（剩 ZG/TG）；新增 `CreateFormCategory`（PRIVATE/EDUCATION 双值，locksForCategory 收窄入参并删 PUBLIC_SECURITY/GOV_INSTITUTION 死 case）+ `educationP1Locks` 联动函数；删 InstitutionFieldLocks.lockedInstitutionName（恒 null 死字段）
- [x] core/institution/CreateInstitutionForm.tsx：删 isPublicGov/isPublicSecurity 全部分支；EDUCATION 模式（G/S/F、机构锁 JY、学校名称必填查重、F 渲染「上级法人属性」+上级=S 时「上级盈利属性」，切主体属性/切上级属性/切上级盈利三条路径都 setFieldsValue({p1}) 重算）；G 查重带 city（后端 G 分支同市查重），S/F 全国
- [x] education/ 新模块：api.ts（listEducationInstitutions=category=EDUCATION_INSTITUTION + 创建三件套复制 private）/ EducationCreateModal / EducationListTable（去清算行列、加主体属性列含盈利标注）/ EducationView（精确搜索形态，详情复用 gov/GovDetailPage 调度：S/F→PrivateDetailLayout 可编辑、G→只读；创建成功跳详情）
- [x] 删 gov/GovCreateModal.tsx；GovView 删新增按钮/createOpen/createLabel；gov/api.ts 删 checkInstitutionName/createInstitution/uploadLegalRepresentativePhoto 及 grant 相关 import
- [x] private/PrivateCreateModal 写死 category="PRIVATE_INSTITUTION" 删透传；PrivateView 同步简化
- [x] App.tsx：education tab 插 private 与 gov 之间 + routedView 分支（387 行 ≤400）；AuthContext 加 canViewEducation（联邦/市管理员）
- [x] tsconfig.json include 加 education

### 文档
- [x] memory/05-modules/sfid/frontend/FRONTEND_LAYOUT.md：目录树加 education/、API 规则、表单唯一实现改 private/education、JY 归属新规则、私权选项只剩 ZG/TG、tsconfig 清单

## 验证记录

- 后端 `cargo check` 0 error + `cargo test` 52/52；本卡触碰文件 `cargo fmt --check` 干净（cpms/handler.rs、subjects/admin.rs 的 fmt 差异属上一任务卡未提交改动，private/clearing.rs 为历史存量，均不在本卡范围）。
- 前端 `npm run build`（tsc -b + vite build）0 error。
- **列表 SQL 三分支已用开发库实测**（事务内造 G/S/F 三条手动 JY 测试行后 ROLLBACK，不落库）：
  - Education 分支命中手动 G 公立学校 + S 私立学校 + F 分校（跨 GOV/PRIVATE 两存储 category）✓
  - Private 分支搜 JY 学校名 0 行（排除生效）✓
  - Gov 分支搜手动 JY 学校 0 行、搜「合肥市公民教育委员会」（org_code=CITY_EDU 监管本体）命中 ✓
- **数据不变量已核**：开发库 3,186 条 JY 行全部为自动监管本体（org_code=CITY_EDU 无一为空），`org_code IS NULL` 圈定手动学校无误杀风险。
- JY 残留扫描：education 模块之外仅剩 INSTITUTION_CODE_LABEL 展示映射与指向教育 tab 的注释,无功能入口残留。

## 待用户端到端 QA

1. tab 顺序：首页 → 私权机构 → **教育机构** → 公权机构 → 公安局 → 市注册局 → 联邦注册局。
2. 教育 tab 新增（市级管理员）：G 锁非盈利；S 可选 0/1；F 选上级=公法人锁 0、上级=私法人时 p1 跟随上级盈利属性；机构锁「教育委员会 (JY)」；学校名称查重（G 需先选市）+ 法定代表人必填；创建成功跳详情（S 补企业类型、F 关联所属法人、G 只读）。
3. 教育 tab 搜索：按学校名/SFID 命中新建 G/S/F 学校。
4. 私权 tab：新增选项只剩 ZG/TG；搜 JY 学校名返回空。
5. 公权 tab：市详情页无「新增」按钮；公民教育委员会/国家教育委员会监管本体仍在目录。
6. 公安局 tab 回归不受影响。
