# 任务卡：治理 NRC 返回导航错位修复

- 状态：done（2026-04-15）
- 归属：Blockchain Agent（citizenchain node UI frontend）

## 现象

用户从"国储会"tab 点击"手续费划转"按钮进入提案页，提交或取消后点击返回，
落地页看上去像国储会详情，但：

1. 左上角多了一个"← 返回机构列表"按钮
2. 点击"安全基金转账"、"运行时升级"按钮无反应
3. 必须再点一次"返回机构列表"才能回到"真正"的国储会详情页

费率提案页 (`propose-fee-rate`) 同样路径有相同问题。

## 根因

`citizenchain/node/frontend/governance/GovernanceSection.tsx` 把国储会详情页
渲染成了**两套不同的 `<InstitutionDetailPage>` 实例**：

### A. "真"国储会页（`activeTab === 'nrc'`，第 190-214 行）

- 嵌入在治理子 Tab 栏之下（顶部"提案/国储会/省储会/省储行/开发升级"仍在屏幕上）
- `hideBackButton={true}` — 不显示"返回机构列表"
- 挂齐了 `onCreateSafetyFund` / `onCreateRuntimeUpgrade` / `onCreateSweep` 三个 NRC 专属 handler

### B. "通用"机构详情视图（`view.page === 'institution-detail'`，第 148-170 行）

- 全屏覆盖视图，替换了整条治理子 Tab 栏
- 不传 `hideBackButton` — 显示"返回机构列表"（这对 PRC/PRB 是正确的）
- 只挂 `onCreateSweep` / `onCreateFeeRate` / `onCreateProposal` —— 为 PRC/PRB 场景设计，
  未挂 `onCreateSafetyFund` / `onCreateRuntimeUpgrade`（这两个是 NRC 专属）

### 触发路径

`propose-sweep` / `propose-fee-rate` 两个提案页的 `onBack` / `onSuccess`
**不区分 backTab** 一律跳转到视图 B：

```tsx
onBack={() => setView({ page: 'institution-detail', shenfenId: view.shenfenId, backTab: view.backTab })}
```

对 PRC/PRB（backTab='prc'|'prb'，来时即通过视图 B）是对的；
对 NRC（backTab='nrc'，来时是 tab 内联渲染 A）则跳到了 B，表现为上述 3 个现象。

## 修复

**方案 1：按 backTab 分发返回父级**。NRC 的父级是 tab 状态 `{ page: 'nrc' }`，
PRC/PRB 的父级是 `{ page: 'institution-detail' }`。

抽 `backToInstitutionParent(backTab, shenfenId)` helper：

```tsx
const backToInstitutionParent = (backTab: SubTab, shenfenId: string) => {
  if (backTab === 'nrc') {
    setView({ page: 'nrc' });
  } else {
    setView({ page: 'institution-detail', shenfenId, backTab });
  }
};
```

`propose-sweep` / `propose-fee-rate` 的 `onBack` / `onSuccess` 都改为
`() => backToInstitutionParent(view.backTab, view.shenfenId)`。

### 为什么不选"方案 2"（把视图 B 改成自动适配 NRC）

视图 A 嵌入在治理子 Tab 栏下，视图 B 是全屏覆盖——**结构本身不同**。
即使视图 B 自动 `hideBackButton` + 挂齐 handler，用户看到的 NRC 页仍然
**没有子 Tab 栏**，还是"另一个假页"。要彻底统一，得把 NRC tab 的内联渲染
也搬进 view.page 系统，牵动面更大。

方案 1 用 backTab 判定父级结构，4 处小改动解决，无架构损伤。

## 改动

| 文件 | 改动 |
|---|---|
| `citizenchain/node/frontend/governance/GovernanceSection.tsx` | 新增 `backToInstitutionParent` helper；`propose-sweep` / `propose-fee-rate` 的 `onBack` / `onSuccess` 改为调用它；在 `GovernanceView` 类型定义上方加回归防护注释 |

其他 `propose-safety-fund` / `propose-upgrade` 不受影响——这两个只从 NRC 入口触发，
原本的 `setView({ page: view.backTab })` 就是对的。

## 验证

- `npx tsc -b` — ✅ 通过
- 手动验证预期路径（需配合 `./citizenchain/scripts/run.sh` 启动节点）：
  - NRC → 手续费划转 → 返回 → ✅ 回带子 Tab 栏的 NRC 页，无"返回机构列表"按钮，各按钮可点
  - NRC → 费率设置 → 返回 → ✅ 同上
  - PRC 列表 → 某机构 → 手续费划转 → 返回 → ✅ 回该机构详情页（带"返回机构列表"符合预期）
  - PRB 同样

## 回归防护

`GovernanceView` 类型定义上方加中文注释，说明 NRC 返回路径的特殊性，
防止后续加新 NRC 入口提案时重复此 bug。

## 相关任务卡

- `20260415-proposal-action-resolver.md` — 治理提案解码统一入口（同日完成）
- `20260415-wumin-institutions-unified-registry.md` — wumin 冷钱包机构注册表合并（同日完成）
