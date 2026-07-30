# 身份页「当前身份」徽章：纯访客不显示

状态：done（2026-07-29 落地并验证）

## 实现

`myid_page.dart` `_PassportIdentityCard` 新增 `_showsCurrentBadge` 判据
（`current && (tier != visitor || registeredCid 非空)`）。徽章渲染
`if (current)` → `if (_showsCurrentBadge)`；标题右内边距 `current ? 88 : 0`
→ `_showsCurrentBadge ? 88 : 0`（徽章不挂时不留空）。加粗边框/阴影仍挂 `current` 未动。

测试 4 处同步：纯访客用例（`访客当前卡排第一…`、`没有默认热钱包…`、`身份账户变化…`）
改断言「无徽章」；匿名已注册用例补断言「有『当前身份』徽章」。

## 验证

| 项 | 结果 |
|---|---|
| `flutter analyze`（全 app） | 0 问题 |
| `flutter test test/myid_page_test.dart` | 20 passed |

行为终态：纯访客无徽章；匿名已注册/投票/竞选显示「当前身份」；链读失败三卡无徽章（回归）。

## 任务需求

公民 App「我的 → 身份」页，右上角「当前身份」徽章改为按 CID 注册态显示：

- 未注册 CID（纯访客）→ **不显示任何徽章**（既不是「未注册」，也不是「当前身份」）
- 有 CID 但无投票/竞选身份（匿名已注册）→ 显示「当前身份」
- 有投票/竞选身份 → 在对应卡显示「当前身份」

等价规则：「当前身份」徽章照旧显示在当前卡，**唯独当前卡是纯访客（无 CID）时不显示**。

## 已确认边界

- 真源：`MyIdState.isAnonymousRegistered`（[myid_service.dart:95](citizenapp/lib/my/myid/myid_service.dart:95)）
  = `tier==visitor 且 cidNumber 非空`。投票/竞选必有 CID，匿名已注册有 CID，仅纯访客无 CID。
- 只改渲染门槛，不引入「未注册」文案，不动「匿名」小药丸（`_AnonymousTag`）。
- **不变量**：链读失败（`queryFailed`）时不得显示任何徽章——现状靠
  `_isCurrent` 已带 `!_isQueryFailed`，改动必须继续挂在 `current` 之内，不新开旁路。
  「不显示徽章」不能把「没读到链」冒充成「没注册」。
- 卡片排序（当前卡置顶）不变——排序用 `_state.tier`/`_isCurrent`，与徽章无关。
- 纯访客卡的加粗边框/阴影（`current` 驱动）保留不动：卡片仍是当前身份卡，
  只是不挂徽章。徽章预留的 88px 右内边距需随徽章一起撤，避免标题行留空。

## 预计修改

- `citizenapp/lib/my/myid/myid_page.dart`：`_PassportIdentityCard` 新增
  `_showsCurrentBadge` 判据（`current && (tier!=visitor || 已注册CID)`），
  徽章渲染（当前 `if (current)`，行 520）与标题右内边距（行 460 `current ? 88 : 0`）
  改挂 `_showsCurrentBadge`。
- `citizenapp/test/myid_page_test.dart`：纯访客用例改断言「无徽章」；补匿名已注册
  用例锁「有『当前身份』徽章」；账户切换用例的初始纯访客态改断言无徽章。

## 主要风险

- 现有 3 处测试断言纯访客有徽章（[myid_page_test.dart:82,177,235](citizenapp/test/myid_page_test.dart:82)），
  改动会红，需同批改成「无徽章」。
- 别误伤匿名已注册（②）与投票/竞选（③）——它们必须继续显示「当前身份」。

## 完成标准

- 纯访客：无「当前身份」徽章、无 `current-identity-visitor` key、标题行不留空。
- 匿名已注册：访客卡显示「当前身份」。
- 投票/竞选：对应卡显示「当前身份」。
- 链读失败：三卡均无徽章（回归）。
- `flutter analyze` 0 问题、`flutter test` 全绿。
