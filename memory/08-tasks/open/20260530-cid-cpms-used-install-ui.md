任务需求：
调整 CID 机构详情页的 CPMS 安装授权面板。当 CPMS 授权状态为运行中且签发公钥已经绑定时，二维码区域不再显示“无二维码”，改为显示“安装码已使用”；该状态下不再显示“下载”按钮，只居中显示“禁用”和“吊销”两个按钮；“禁用”按钮改为黄色警示样式，“吊销”按钮保持红色危险样式。

所属模块：
CID 前端 / CPMS 安装授权管理面板。

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/workflow.md
- memory/07-ai/context-loading-order.md
- memory/07-ai/document-boundaries.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/task-card-template.md
- memory/07-ai/module-checklists/citizencode.md
- memory/07-ai/module-definition-of-done/citizencode.md
- memory/01-architecture/citizencode/CID_TECHNICAL.md

当前问题：
- CID 前端 `CpmsSitePanel` 在 `ACTIVE` 状态下已经不展示 INSTALL 二维码，但仍显示“无二维码”占位。
- `ACTIVE` 状态仍显示“下载”按钮，用户点击后会尝试下载一个不存在的二维码节点，容易误解为系统错误。
- `禁用` 与 `吊销` 同为红色危险按钮，不能清晰表达“暂停接收”和“废止授权”的风险等级差异。

目标状态：
- `PENDING`：继续显示 INSTALL 安装二维码，并保留下载按钮。
- `ACTIVE + cpms_pubkey_bound=true`：二维码区域显示“安装码已使用”，不显示下载按钮，只居中显示“禁用”和“吊销”。
- `ACTIVE` 下“禁用”使用黄色警示样式，“吊销”保持红色危险样式。
- `DISABLED`：继续显示启用和吊销，不改变后端状态语义。
- `REVOKED`：继续显示重发令牌，不改变重发安装码流程。

技术方案：
1. 在 `citizencode/frontend/citizenpassport/CpmsSitePanel.tsx` 中增加展示判断：
   - `installUsed = status === 'ACTIVE' && site.cpms_pubkey_bound`
   - `qrPayload` 仍只在 `PENDING` 状态取 `site.qr1_payload`
   - 当 `installUsed` 为真时，占位文案显示“安装码已使用”
2. 调整按钮渲染：
   - `ACTIVE` 状态移除下载按钮。
   - `ACTIVE` 状态按钮容器保持居中，且只包含禁用、吊销。
   - `PENDING` 状态继续保留下载按钮。
3. 调整按钮颜色：
   - 禁用按钮使用黄色/橙色警示样式，不使用 `danger`。
   - 吊销按钮继续使用 Ant Design `danger`。
4. 补充中文注释：
   - 在状态展示判断附近说明“安装码已使用”只表示 INSTALL 已完成闭环，不代表需要下载二维码。
5. 更新文档：
   - 如实现后发现现有 `CID_TECHNICAL.md` 的页面口径已经覆盖该行为，则仅在任务卡记录本次 UI 收口。
   - 如页面口径需要补充，更新 `memory/01-architecture/citizencode/CID_TECHNICAL.md` 的机构管理页面冻结口径。
6. 清理残留：
   - 检查是否仍存在 `ACTIVE` 状态下载按钮或无效下载路径。
   - 不保留临时变量、调试日志或无用样式。

预计修改目录：
- `citizencode/frontend/citizenpassport/`：修改 CPMS 安装授权面板展示逻辑。边界仅限前端 UI，不改接口、不改状态机、不改二维码协议；涉及代码和中文注释。
- `memory/01-architecture/citizencode/`：按实现结果决定是否补充 CID 机构管理页面口径。边界仅限文档口径说明；不改协议字段。
- `memory/08-tasks/open/`：保存当前任务卡和执行记录；仅文档记录。

不修改范围：
- 不修改 CID 后端 `CpmsSiteStatus` / `InstallTokenStatus` 状态流转。
- 不修改 INSTALL / ARCHIVE 二维码载荷。
- 不修改 CPMS 后端和 CPMS 前端。
- 不新增兼容旧流程或双轨展示。

必须遵守：
- 不可突破 CID 模块边界。
- 不可绕过既有 CPMS 授权状态契约。
- 不可擅自修改二维码协议和签名字段。
- 不保存原始实名。
- 关键展示判断补中文注释。
- 改代码后同步文档、清理残留并运行必要验证。

输出物：
- 代码：`citizencode/frontend/citizenpassport/CpmsSitePanel.tsx`
- 中文注释：解释已使用安装码状态的展示判断。
- 测试/验证：至少运行 `cd citizencode/frontend && npm run build`。
- 文档更新：按实现结果更新或确认 `memory/01-architecture/citizencode/CID_TECHNICAL.md` 已覆盖。
- 残留清理：删除无效 ACTIVE 下载入口，确认无调试残留。

验收标准：
- `PENDING` 状态仍能显示并下载 INSTALL 安装二维码。
- `ACTIVE + cpms_pubkey_bound=true` 状态显示“安装码已使用”。
- `ACTIVE + cpms_pubkey_bound=true` 状态不显示下载按钮。
- `ACTIVE` 状态只居中显示“禁用”“吊销”两个按钮。
- 禁用按钮为黄色警示样式，吊销按钮为红色危险样式。
- `DISABLED` / `REVOKED` 状态既有操作不回归。
- 前端构建通过。
- 文档和残留已收口。

执行记录：
- 已修改 `citizencode/frontend/citizenpassport/CpmsSitePanel.tsx`，`ACTIVE + cpms_pubkey_bound=true` 时显示“安装码已使用”，并移除运行态下载按钮。
- 已将 `ACTIVE` 状态的禁用按钮改为黄色警示样式，吊销按钮继续使用红色危险样式。
- 已更新 `memory/01-architecture/citizencode/CID_TECHNICAL.md`，补充 INSTALL 已使用后的机构详情页展示口径。
- 已运行 `cd citizencode/frontend && npm run build`，构建通过；Vite 输出大 chunk 提醒，为现有打包体积提示。
