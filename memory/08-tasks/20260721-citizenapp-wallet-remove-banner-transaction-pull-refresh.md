# CitizenApp 钱包页删连接状态栏 + 交易页加下拉刷新

任务需求：我的钱包页顶部区块链连接状态栏删除（与交易页那条重复）；交易页加下拉刷新，
刷新范围＝余额 + 本地交易记录 + 轻节点连接重探。
所属模块：citizenapp / wallet + transaction

## 定稿（用户确认）

1. 删钱包页顶部 `ChainProgressBanner`；连接状态由交易 tab 那条统一承担。
2. 交易页（`OnchainPaymentPanel`）加下拉刷新：余额 + 本地交易记录状态 + 轻节点连接重探。
3. 补测试。

## 落点

- `lib/wallet/pages/wallet_page.dart`：删 L611-615 `if(!_isSelectionMode) Padding(…ChainProgressBanner(busy:_balanceRefreshing))`；
  删无用 import `chain_progress_banner.dart`（钱包页仅此一处用）。`_balanceRefreshing` **保留**（L202 重入守卫仍读，非死码）；
  其下 `RefreshIndicator(_refreshBalancesFromChain)` 不动。
- `lib/transaction/onchain-transaction/onchain_payment_page.dart`：
  - 新增状态 `bool _refreshing = false;`。
  - 新增 `_onPullRefresh()`：`setState(_refreshing=true)` → `await _reloadWalletAndLocalRecords()`（现成，重载余额+本地记录）→ finally `setState(_refreshing=false)`。
  - `Expanded>ListView`（L807-808）套 `RefreshIndicator(onRefresh:_onPullRefresh, child: 原 ListView)`（ListView 已 AlwaysScrollableScrollPhysics）。
  - 该 `ChainProgressBanner`（L812）补 `busy: _refreshing`——busy false→true 触发 banner 内部 `_loadProgress()` 即时重探连接（`chain_progress_banner.dart:59`），复用现有机制。

## 边界 / 依据

- banner 本就自轮询连接（`_pollTimer`）；下拉只是额外触发一次即时重探；测试有 `_isFlutterTest` 跳网络。
- `_reloadWalletAndLocalRecords`/`_reloadWallet`(用注入 `currentWalletLoader`)/`_loadLocalRecords`(用注入 `localRecordsLoader`) 现成；不动链/后端。

## 输出物 / 验收

- `flutter analyze` 0 问题。
- 测试：交易页下拉触发 `currentWalletLoader`+`localRecordsLoader` 再次被调（重载）+ `RefreshIndicator` 存在 + banner 仍在；
  钱包页删 banner 后不再有 `ChainProgressBanner`（若原测试断言其存在则同步）。
- 现有 `test/ui/transaction_tab_page_test.dart:88` 断言交易页有 banner——保持（banner 未删）。

## 执行结果（2026-07-21）

- **钱包页**：`wallet_page.dart` 删顶部 `ChainProgressBanner(busy:_balanceRefreshing)` 块 + 无用 import `chain_progress_banner.dart`；`_balanceRefreshing`（重入守卫）与列表 `RefreshIndicator(_refreshBalancesFromChain)` 保留。
- **交易页**：`onchain_payment_page.dart` 加 `_refreshing` 状态 + `_onPullRefresh()`（setState true → `_reloadWalletAndLocalRecords()` 重载余额+本地记录 → finally false）；`Expanded>ListView` 套 `RefreshIndicator(onRefresh:_onPullRefresh)`；顶部 `ChainProgressBanner` 补 `busy:_refreshing`（false→true 触发内部 `_loadProgress` 即时重探轻节点连接）。
- **测试**：`transaction_tab_page_test.dart` 新增「下拉刷新重载余额+本地记录且保留连接状态栏」——注入计数 `currentWalletLoader`/`localRecordsLoader`，fling 触发后二者再次被调，且 `RefreshIndicator`/`ChainProgressBanner` 均在。
- **验证**：`flutter analyze` 两文件 + 测试 0 问题；交易页 4/4、钱包 101 全过。
- **边界**：链/后端未动；banner 自轮询机制未改，仅复用其 busy 触发即时重探。
