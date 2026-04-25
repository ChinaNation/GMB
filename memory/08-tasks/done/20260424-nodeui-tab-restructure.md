# nodeui 顶部 tab 重构与网络内容下沉挖矿页

任务需求：
重构 citizenchain 节点 UI（Tauri 桌面端）的顶部 tab 布局和页面归属。

1. **顶部 tab 重构**
   - 新增 3 个顶级 tab：`国储会`、`省储会`、`省储行`（原治理 tab 下的子页抽出）
   - 删除 `治理` 顶级 tab 及其"提案"子 tab 入口（`ProposalListView` 整体下线）
   - 删除 `网络` 顶级 tab（内容下沉到挖矿页，详见第 2 点）
   - `开发升级` 从治理子 tab 迁入 `设置` 页
   - 最终顶部 tab 顺序（共 8 个）：首页 ｜ 挖矿 ｜ 国储会 ｜ 省储会 ｜ 省储行 ｜ 白皮书 ｜ 公民宪法 ｜ 设置

2. **网络内容下沉挖矿页**
   - 位置：插入在挖矿页 `资源监控` 与 `出块记录` 两 section 之间
   - 新 3×2 卡片网格布局：
     - 第 1 行：`总节点数` ｜ `在线节点`
     - 第 2 行：`治理节点`（单卡合并显示 `x ｜ xx ｜ xx`，分别对应 国储会/省储会/省储行 节点数） ｜ `清算节点`（暂时占位，前后端先返 0）
     - 第 3 行：`全节点` ｜ `轻节点`

3. **开发升级迁入设置页的 UI 收敛**
   - 功能逻辑完整保留（选文件 → 构建签名请求 → QR 展示 → 扫描回执 → 提交链 → 完成/错误 全流程不变）
   - form 阶段 UI 只保留 3 行：`选择文件`、`联合提案发起人管理员`、`生成签名请求` 按钮
   - 删除原页面 `<h2>开发期 Runtime 升级</h2>` 和 `.dev-upgrade-hint` 提示段（由设置页自身的小节标题承担）

所属模块：Blockchain Agent（citizenchain/node/frontend + node Tauri 后端）

## 输入文档

- `memory/07-ai/agent-rules.md`
- `memory/07-ai/chat-protocol.md`
- `memory/07-ai/task-card-template.md`
- `memory/08-tasks/templates/citizenchain-nodeui.md`
- `memory/08-tasks/done/20260324-nodeui-governance-tab.md`（治理 tab 引入历史，本次反向整改参考）

## 变更范围（文件级）

### 前端（citizenchain/node/frontend）

- `App.tsx`
  - `TabKey` 联合类型：去掉 `'governance'`/`'network'`，新增 `'nrc'`/`'prc'`/`'prb'`
  - top-nav 按钮按最终顺序重排
  - 路由分发：`governance`/`network` 分支删除；新增 `nrc`/`prc`/`prb` 分支分别渲染 `NrcSection`/`PrcSection`/`PrbSection`
- `governance/GovernanceSection.tsx`
  - 拆分为 3 个独立顶层 Section：`governance/NrcSection.tsx`、`governance/PrcSection.tsx`、`governance/PrbSection.tsx`（复用 `InstitutionDetailPage`、`InstitutionListView`、子视图路由）
  - 删除"提案"子 tab 和 `ProposalListView` 入口（各机构详情页 `InstitutionDetailPage.tsx:294-341` 已自带机构归属的提案列表，功能无损）
  - 删除"开发升级"子 tab（UI 迁入设置页）
  - `SubTab` 类型与 `backTab` 相关分支：去掉 `'proposals'`/`'dev-upgrade'`
  - `backToInstitutionParent` 按 `nrc`/`prc`/`prb` 3 个 Section 分别落地
  - 文件 `GovernanceSection.tsx` 若拆净后可删除，留 3 个独立 Section
- `governance/ProposalListView.tsx`：整体删除（入口无引用后）
- `governance/DeveloperUpgradePage.tsx` → **物理迁移**到 `settings/developer-upgrade/DeveloperUpgradePage.tsx`
  - 删除 `<h2>` 与 `.dev-upgrade-hint` 两个元素，仅保留 form 内 3 行 + 后续步骤的完整流程
  - 跨目录 import：`QrScanner` 和 `governance-types` 仍从 `governance/` 引用（保持最小改动）
- `mining/mining-dashboard/MiningDashboardSection.tsx`
  - 在"资源监控" `</section>` 之后、"出块记录" `<section>` 之前，插入新的 `<NetworkInlineSection />`（可内嵌或抽子组件复用）
- `network/network-overview/NetworkOverviewSection.tsx`
  - 改版为 3×2 布局（或抽出 `NetworkInlineSection` 供挖矿页嵌入，旧 `NetworkOverviewSection` 可删）
  - `guochuhuiNodes`/`shengchuhuiNodes`/`shengchuhangNodes` 合并进单卡 `治理节点`，值格式化为 `${x} ｜ ${xx} ｜ ${xx}`（分隔符用全角竖线"｜"+ 两侧空格，和用户原文一致）
  - 新增 `清算节点` 卡片，读取 `network.clearingNodes`
- `settings/settings-panel/SettingsSection.tsx`
  - 新增"开发升级"小节（标题 `<h3>` 级），嵌入 `DeveloperUpgradePage`（裁剪后版本）
- `types.ts`
  - `NetworkOverview` 新增 `clearingNodes: number` 字段
- CSS（`App.css` 或对应 section 样式表）
  - `network-overview-grid` 改为强制 3 行 × 2 列布局（如 `grid-template-columns: repeat(2, 1fr)`）
  - 治理节点卡片的值样式（`x ｜ xx ｜ xx` 一行展示，必要时小字号）

### 后端（citizenchain/node Tauri 后端）

- `getNetworkOverview` 返回结构新增 `clearing_nodes: u32`（先写死为 0 占位，注明"清算节点统计口径待后续实现")
- 对应 Tauri command 序列化字段名保持 `clearingNodes`（和前端约定一致）

## 必须遵守

- 不可突破模块边界（本次只改 citizenchain/node 子项目，不动 runtime / wumin / wuminapp）
- 不可绕过既有契约：`InstitutionDetailPage` 内部已有的提案列表 / 管理员列表 / 创建提案 流程全部保留，不改行为
- 不可擅自修改安全红线：开发升级的 QR 签名 / payload / 链上提交链路 100% 保持，只动 form 阶段 UI
- 不清楚逻辑时先沟通
- 遵守 `feedback_no_compatibility.md`：不保留旧 tab 的任何过渡开关、兼容路由或"已删除"占位

## 最终决策（用户 2026-04-24 确认）

1. **清算节点占位显示**：数值位直接显示 `0`
2. **`GovernanceSection.tsx`**：直接**物理删除**，由 `App.tsx` 直接分发到 3 个独立 Section（`NrcSection`/`PrcSection`/`PrbSection`）
3. **`DeveloperUpgradePage.tsx`**：**物理迁移**到 `settings/developer-upgrade/DeveloperUpgradePage.tsx`（治理 tab 已删，该页不应留在 `governance/` 目录下，功能归属和目录归属必须一致）
4. **开发升级在设置页的小节标题**：用 `<h3>开发升级</h3>` 作为设置页内小节标题，不再复用原 `<h2>` 级标题
5. **NRC 单机构 Section 形态**：`NrcSection` 直接内嵌渲染 `InstitutionDetailPage`（`hideBackButton` 模式），与现行 NRC 子 tab 实现一致；PRC/PRB 保留"列表 → 详情"两级导航
6. **`governance/` 目录保留（仅限机构/提案相关）**：`AdminListPage`/`InstitutionDetailPage`/`InstitutionListView`/`CreateProposalPage`/`ProposalDetailPage`/`SafetyFundProposalPage`/`SweepProposalPage`/`RuntimeUpgradeProposalPage`/`QrScanner`/`governance-types` 仍放 `governance/` 目录，被 3 个新 Section 引用
7. **`QrScanner.tsx` 物理位置**：保留在 `governance/QrScanner.tsx`（被 governance 内其他提案页复用），迁移后的 `DeveloperUpgradePage` 跨目录 import（可接受）

## 输出物

- 代码：上述前后端改动
- 中文注释：新 Section、`NetworkInlineSection`、新增 `clearingNodes` 字段处的中文说明
- 测试：
  - 手工验收顶部 tab 可切换、8 个 tab 顺序正确
  - 手工验收挖矿页 3 个 section 顺序正确、网络 3×2 卡片布局正确
  - 手工验收设置页"开发升级"子区 3 行可点击、QR/扫码/提交链路完整
  - 若项目含 RTL / Vitest 快照，补齐对应快照更新
- 文档更新：
  - `memory/02-modules/citizenchain/` 下 UI 相关说明（若有）
  - 删除 `GovernanceSection` 相关历史文档中已失效的 tab 描述
- 残留清理：
  - `GovernanceSection.tsx` / `ProposalListView.tsx` 若删除，对应 import / 样式类 / 测试文件一并清掉
  - `App.tsx` 中 `governance`/`network` 相关 `TabKey` 字面量零残留
  - CSS 中无引用的 `governance-sub-tabs` 等类若已无引用则清理

## 验收标准

- 功能可运行：顶部 8 个 tab 正常切换；挖矿页内嵌网络卡片数据刷新正常；设置页开发升级 QR 签名全流程可提交成功
- 测试通过：项目 `npm test` / `cargo test` 保持绿
- 文档已更新
- 残留已清理（Grep 无 `'governance'` / `'network'` 顶级 tab 字面量、无 `ProposalListView` 引用、无 `dev-upgrade-hint` 残留）
- Review 问题已处理
- 模块级完成标准已对照（`memory/08-tasks/templates/citizenchain-nodeui.md`）

## 开工状态

- 2026-04-24：三项待确认细节已全部拍板（见"最终决策"），进入执行阶段
- 下一步：主入口分派 Blockchain Agent 按顺序实施前端拆分 → 后端字段补齐 → 残留清理 → 手工验收

- 状态：done

## 完成信息

- 完成时间：2026-04-24 16:16:22
- 完成摘要：顶部 tab 重构完成：首页/挖矿/国储会/省储会/省储行/白皮书/公民宪法/设置 8 tab 平铺；网络 3×2 卡片下沉到挖矿页（治理节点合并显示国储会｜省储会｜省储行，清算节点占位 0）；开发升级迁入设置页（UI 收敛为选文件/管理员/生成签名请求 3 行）；提案子 tab 删除（各机构详情页自带提案列表）；前端 tsc/vite build 通过，后端 cargo check -p node + ui::network 测试 9/9 通过
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md

## 追加调整（2026-04-24 同日 UI 微调）

用户验收时提出 3 处 UI 细节调整，已全部完成并通过 vite build：

1. **治理节点卡片 3 列子网格**：从单格字符串拼接 `${x} ｜ ${xx} ｜ ${xx}` 改为 3 列子网格，每列独立显示数字 + 名称（国储会/省储会/省储行），数字与名称严格上下对齐。新增 `.governance-node-grid` / `.governance-node-col` / `.governance-node-name` CSS，删除 `.metric-sublabel`。
2. **挖矿页标题** `挖矿实际收益` → `挖矿收益`。
3. **开发升级 UI 横排重构**：复用 `.bootnode-inline` 3 列 grid 框架 + 背景框样式：左侧一组 `h2 开发升级 + 选择文件按钮 + 文件名`、中间管理员下拉、右侧"生成签名请求"按钮（与"上传私钥"视觉一致，均无特殊类、走默认 button 样式）。移除 `<label>Runtime WASM 文件</label>`、`<label>联合提案发起人管理员</label>`、`.dev-upgrade-form` / `.dev-upgrade-field` / `.dev-upgrade-file-row` / `.dev-upgrade-pick-file` 等废弃 CSS。`SettingsSection` 移除外层 `<section><h3>开发升级</h3>` 包装，由 `DeveloperUpgradePage` 自身承载 section 容器。
4. **机构详情页文案微调**（`InstitutionDetailPage.tsx:159-177`）：第 1 卡"机构类型" → "机构类型 /身份ID"；第 2 卡"机构主账户" → "主账户"；联合投票权重数字后追加"票"字（与内部投票阈值格式对齐）。NRC/PRC/PRB 全部生效（共用同一详情组件）。

