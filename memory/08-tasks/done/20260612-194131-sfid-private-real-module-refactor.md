# 任务卡:SFID 私权机构真实模块化返工

- 任务编号:20260612-194131
- 状态:done
- 创建时间:2026-06-12 19:41:31
- 模块:SFID

## 任务需求

此前私权机构拆分只形成目录和少量占位文件,实际业务仍集中在 `private/handler.rs`、
前端 `PrivateView`、`PrivateListTable`、`CreateInstitutionForm` 等聚合文件中,不符合
“个体经营、合伙企业、股权公司、股份公司、公益组织、注册协会”六类真实拆分目标。

本任务必须返工为真实模块化:

- 前端第一层可操作入口直接显示六类私权机构。
- 后端每类私权机构目录必须承担真实模型、校验、服务、仓储或路由职责。
- `private/` 根层只保留聚合路由和模块导出,不得继续吞掉六类业务逻辑。
- 清理空壳文件、旧注释、旧文档、旧 `sub_type/kind` 和 SFID 清算行业务残留。

## 身份规则

| 类型 | 代码 | 主体属性 | 法人资格 |
| --- | --- | --- | --- |
| 个体经营 | `GT` | `F` | 非法人 |
| 无限合伙 | `GP` | `F` | 非法人 |
| 有限合伙 | `LP` | `S` | 私法人 |
| 股权公司 | `GQ` | `S` | 私法人 |
| 股份公司 | `GF` | `S` | 私法人 |
| 公益组织 | `GY` | `S` | 私法人 |
| 注册协会 | `AS` | `S` | 私法人 |

`ZG/TG` 只属于人类主体来源分类,不得作为私权机构类型。

## 预计修改目录

| 目录 | 中文注释 |
| --- | --- |
| `sfid/backend/private/common/` | 私权机构共用模型、ID 规则、共用 DTO、共用仓储辅助;涉及代码,不得放类型特有业务 |
| `sfid/backend/private/sole/` | 个体经营真实后端模块;涉及代码,承接负责人、非法人规则和个体经营路由 |
| `sfid/backend/private/partnership/` | 合伙企业真实后端模块;涉及代码,承接 `GP/LP`、合伙类型和合伙人规则 |
| `sfid/backend/private/company/` | 股权公司真实后端模块;涉及代码,承接股东、出资和股权公司路由 |
| `sfid/backend/private/corporation/` | 股份公司真实后端模块;涉及代码,承接股份、股东和股份公司路由 |
| `sfid/backend/private/welfare/` | 公益组织真实后端模块;涉及代码,承接非营利属性、成员和公益组织路由 |
| `sfid/backend/private/association/` | 注册协会真实后端模块;涉及代码,承接 `AS` 私法人、会员和协会路由 |
| `sfid/backend/private/participants/` | 私权机构参与人关系模块;涉及代码,统一存储负责人、合伙人、股东、成员等关系 |
| `sfid/frontend/private/common/` | 私权机构前端共用类型、规则和基础组件;涉及代码,不得承载具体业务 API |
| `sfid/frontend/private/sole/` | 个体经营前端页面、表单、详情和 API;涉及代码 |
| `sfid/frontend/private/partnership/` | 合伙企业前端页面、合伙类型表单、合伙人视图和 API;涉及代码 |
| `sfid/frontend/private/company/` | 股权公司前端页面、股东视图和 API;涉及代码 |
| `sfid/frontend/private/corporation/` | 股份公司前端页面、股份/股东视图和 API;涉及代码 |
| `sfid/frontend/private/welfare/` | 公益组织前端页面、公益属性/成员视图和 API;涉及代码 |
| `sfid/frontend/private/association/` | 注册协会前端页面、协会属性/会员视图和 API;涉及代码 |
| `memory/01-architecture/sfid/` | 更新 SFID 总体架构;涉及文档,删除错误的私权大 tab 和旧清算行口径 |
| `memory/05-modules/sfid/` | 更新 SFID 前后端模块文档;涉及文档,写清六类真实模块职责 |
| `memory/08-tasks/` | 更新任务卡和任务索引;涉及文档 |

## 验收标准

- 六类目录不存在只有注释或空导出的占位文件。
- 后端六类模块各自提供真实类型定义、校验、服务/仓储或 handler。
- 前端六类模块各自提供真实页面、API 和类型边界。
- 私权机构 UI 第一层显示六类 tab,而不是一个“私权机构”业务 tab。
- 旧 `sub_type/kind` 不再作为业务字段出现,只允许数据库清理迁移中出现旧字段删除。
- SFID 清算行相关代码、入口和旧文档不得恢复。
- 后端 `cargo fmt && cargo check && cargo test --test institution_tests` 通过。
- 前端 `npm run build` 通过。
- 浏览器进入私权机构页面验证六类 tab 可见。

## 执行结果

- 后端 `private/handler.rs` 已删除,通用机构注册内核迁入 `subjects/registration.rs`。
- 后端 `private/sole|partnership|company|corporation|welfare|association` 均已具备真实 profile、校验、创建和列表 handler。
- 后端六类专属路由已挂载为 `/api/v1/private/sole|partnership|company|corporation|welfare|association`。
- 前端 `PrivateView` 已删除,改为 `PrivateShell` 只负责省市选择、当前私权类型页面和详情跳转。
- 前端六类目录均已具备 `types.ts / api.ts / Page.tsx / index.ts`。
- 前端顶层已删除单一私权机构大入口,直接显示个体经营、合伙企业、股权公司、股份公司、公益组织、注册协会六个 Tab。
- 旧根 `private/api.ts`、根列表、根创建弹窗已迁入 `private/common/`,各类业务 API 由六类目录显式调用。
- 空文件扫描通过,未发现 3 行以内的空壳文件。
- SFID 清算行代码和旧 `sub_type` 业务字段未恢复;`sub_type` 只保留在数据库清理迁移和旧字段不得存在的校验中。
- 后端 `cargo fmt && cargo check && cargo test --test institution_tests` 通过。
- 前端 `npm run build` 通过。
- dev server `http://127.0.0.1:5179/` 返回 200,打包产物包含六类 Tab 文案;本地环境未暴露 Playwright/browser 控制包,未做登录后交互截图。
