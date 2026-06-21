任务需求：
修复 CID 公权机构/公安局详情页切换 tab 无反应、全系统提示分散且出现英文原始错误、注册局市注册局机构管理员视图标签和管理员目录展示不清晰的问题。

所属模块：
CID

必须遵守：
- CID 前端所有页面提示必须统一走一个入口，不能在业务组件中直接调用 `message.*`、`Modal.confirm`、`Modal.warning` 或 `alert`。
- 用户可见提示必须是中文，同一时刻只允许显示一个提示。
- 公权机构和公安局 tab 切换必须清理详情页状态，不能因为复用组件导致页面无反应。
- 注册局页面标签统一为“联邦注册局机构管理员列表”“市注册局机构管理员列表”，市注册局机构管理员只能只读查看本人所属省、市注册局机构管理员目录。
- 后端管理员列表查询必须按省市范围查询，不允许全量查询后内存过滤。
- 不涉及投票流程，不新增旧兼容目录或旧命名。

预计修改目录：
- `citizencode/frontend/App.tsx`：修复顶部 tab 切换状态重置，涉及代码。
- `citizencode/frontend/gov/`：修复公权机构和公安局详情状态串用，涉及代码。
- `citizencode/frontend/admins/`：修复注册局管理员目录、通行密钥提示和联邦注册局机构管理员/市注册局机构管理员标签，涉及代码。
- `citizencode/frontend/utils/`：新增统一中文单提示入口，涉及代码。
- `citizencode/frontend/citizens/`：自然人相关提示改走统一入口，涉及代码。
- `citizencode/frontend/private/`：私权机构提示改走统一入口，涉及代码。
- `citizencode/frontend/citizenpassport/`：CPMS 站点提示改走统一入口，涉及代码。
- `citizencode/frontend/accounts/`：账户提示改走统一入口，涉及代码。
- `citizencode/frontend/docs/`：资料库提示改走统一入口，涉及代码。
- `citizencode/backend/admins/`：管理员列表、计数、查找改为省市范围 SQL 查询，涉及代码。
- `memory/01-architecture/citizencode/`：同步 CID 前端提示、tab 状态和注册局目录设计，涉及文档。

验收标准：
- 从公权机构详情页点击公安局 tab 能立即切换到公安局页面。
- 从公安局详情页点击公权机构 tab 能立即切换到公权机构页面。
- `citizencode/frontend` 除统一提示入口外，不再直接调用 `message.*`、`Modal.confirm`、`Modal.warning` 或 `alert`。
- WebAuthn 取消、超时、浏览器不支持等错误均显示中文提示，不显示 W3C 英文原文。
- 市注册局机构管理员注册局页面只读显示本市市注册局机构管理员列表和本省联邦注册局机构管理员列表，标签正确。
- 管理员列表后端接口不再全量查询后内存过滤。
- 前端构建、后端编译通过，文档已更新，残留已清理。

执行记录：
- 已修复顶部 tab 切换状态: `App.tsx` 增加模块重置信号,`GovView` 在 `category/resetToken` 变化时清空详情、省市、搜索和新增弹窗状态。
- 已新增 `citizencode/frontend/utils/notice.ts` 作为 CID 前端唯一提示入口,统一中文提示、单提示显示、WebAuthn/网络/后端错误翻译和统一确认框。
- 已把 CID 前端业务组件中的 `message.*`、`Modal.confirm`、`Modal.warning` 调用迁移到 `notice`。
- 已在 `admin_security_api.ts` 归一 WebAuthn 取消、超时、浏览器不支持和安全错误,不再向页面暴露 W3C 英文原文。
- 已将注册局子页签统一为“联邦注册局机构管理员列表”“市注册局机构管理员列表”,市注册局机构管理员保持只读查看本人所属联邦注册局机构管理员/市注册局机构管理员目录。
- 已将管理员列表后端查询改为按省/市范围 SQL 查询,删除未使用的全量管理员列表函数。
- 已删除 `city_admin_scope` 初始化残留,市注册局机构管理员范围由 `admins.created_by + city` 解析。
- 已补充管理员查询索引:角色/城市、小写公钥、小写创建者和联邦注册局机构管理员省域索引。
- 已更新 CID 架构总览、技术架构和 5 万并发框架文档。
- 已运行 `cargo check` 通过。
- 已运行 `cargo check --tests` 通过。
- 已运行 `npm run build` 通过。
- 2026-06-05 复查发现后端 `passkey required` 通过 `ApiError.message` 泄露到提示层;原因是部分前端调用先取 `err.message` 再交给 `notice`,导致 `error_code` 丢失。
- 已为 `passkey required`、`security grant required` 增加稳定后端错误码,并在 `notice.ts` 中补充错误码和英文消息翻译。
- 已为 `notice.ts` 增加英文兜底规则:无法识别的纯英文错误不再原样展示,统一降级为中文兜底提示。
- 已把多处业务层 `err.message -> notice.error` 改为传入原始错误对象。
- 已再次运行 `npm run build`、`cargo check`、`cargo check --tests` 通过。
