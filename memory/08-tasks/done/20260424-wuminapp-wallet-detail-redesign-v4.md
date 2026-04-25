# wuminapp 钱包详情页改版 v4(QR 地址居中 / 删刷新按钮 + 下拉刷新整页)

创建日期:2026-04-24
完成日期:2026-04-24
所属模块:wuminapp(Flutter 热钱包客户端)
主责 Agent:Mobile Agent
前置任务:
- memory/08-tasks/done/20260424-wuminapp-wallet-detail-redesign-v3.md
状态:DONE

## 执行结果

- `wallet_qr_dialog.dart`:地址行从 `Row + Flexible + IconButton` 换成 `Stack(alignment: center) + Padding(horizontal: 32) + GestureDetector(Text) + Positioned(right: 0) IconButton`,地址在 QR 正下方居中,复制图标浮右
- `wallet_onchain_balance_card.dart`:删 `_buildRefreshButton`;State 公开为 `WalletOnchainBalanceCardState`,`_refresh` 公开为 `refresh`;padding 收紧到 `fromLTRB(16, 8, 16, 12)`,标题与金额行间距 `12 → 8`;第 1 行只剩标题,卡片高度明显降低
- `wallet_page.dart`:`body: ListView` 外包 `RefreshIndicator(onRefresh: _onPullRefresh)`;新增 `_balanceCardKey: GlobalKey<WalletOnchainBalanceCardState>`;`_onPullRefresh` 用 `Future.wait` 并发刷新链上余额 + 交易记录;`ListView` 加 `physics: AlwaysScrollableScrollPhysics`;清算行余额刷新预埋 TODO 注释
- `flutter analyze` 0 error / 0 warning
- `flutter test` 82/82 全通过
- wumin / citizenchain / sfid 零改动

## 遗留(非本任务)

- 清算行余额刷新本轮不接(动作卡"余额"列仍是 `0.00 元` 占位常量),`_onPullRefresh` 内已埋 TODO 注释,等清算行功能落地后追加刷新调用

## 任务需求

修复 v3 引入的偏差 + 用下拉刷新替代余额卡的刷新按钮:

1. QR 弹窗地址行回归 Stack 模式,地址在 QR 下方完全居中,复制图标浮在右侧不抢中心
2. 链上余额卡:
   - 删掉右上刷新按钮(整个 IconButton 体系)
   - `_refresh()` 方法对外暴露为公开 `refresh()`,供外层下拉刷新触发
   - 删掉刷新按钮后第一行只剩标题,卡片高度自然变低
   - padding 进一步收紧 + 行间距压小
3. `WalletDetailPage`:
   - ListView 外包 `RefreshIndicator`,下拉触发 `_onPullRefresh`
   - 下拉时同时刷新链上余额(通过 GlobalKey 调子卡的 refresh)+ 交易记录(复用 `_loadRecentRecords`)
   - 清算行余额刷新本轮不接(静态 0.00 元),回调里加 TODO 注释,等清算行功能落地再补

## 必须遵守

- 只改 wuminapp/,wumin / citizenchain / sfid 零改动
- 不搞兼容/保留/过渡
- 中文注释到位
- 钱包名编辑态/展示态颜色保持 v3 状态,不动
- 动作卡(`wallet_action_card.dart`)本轮不改,等清算行功能落地后再加 refresh hook

## 输出物

### PR-A QR 地址行 Stack 居中

文件:`wuminapp/lib/wallet/ui/cards/wallet_qr_dialog.dart`

定位 build() 内地址行(当前为 `Row + Flexible + IconButton(Icons.copy)`),改为:

```dart
// 地址居中显示在 QR 下方,复制图标浮在右侧不抢中心。
Stack(
  alignment: Alignment.center,
  children: [
    Padding(
      // 左右各留 32 给复制按钮和对称占位,确保地址视觉居中。
      padding: const EdgeInsets.symmetric(horizontal: 32),
      child: GestureDetector(
        onTap: _copyAddress,
        child: Text(
          widget.wallet.address,
          textAlign: TextAlign.center,
          style: TextStyle(
            fontSize: 11,
            color: Colors.grey[500],
            fontFamily: 'monospace',
          ),
        ),
      ),
    ),
    Positioned(
      right: 0,
      child: IconButton(
        icon: const Icon(Icons.copy, size: 14),
        color: Colors.grey[600],
        tooltip: '复制地址',
        padding: EdgeInsets.zero,
        constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
        onPressed: _copyAddress,
      ),
    ),
  ],
),
```

文件头注释更新:把 v3 写的"地址行 Row 居中"改为"地址 Stack 居中,复制图标 Positioned 浮右"。

### PR-B 链上余额卡删刷新按钮 + 高度收紧 + 暴露 refresh()

文件:`wuminapp/lib/wallet/ui/cards/wallet_onchain_balance_card.dart`

改动:

1. **删 `_buildRefreshButton()` 整段**
2. **保留 `_isLoading` / `_hasError` / `_balance` 状态字段**(`_buildAmountSection` 还要用)
3. **`_refresh()` 改为公开**:重命名 `_refresh` → `refresh`,所有调用点同步改;`refresh()` 仍可被内部 `initState` 直接调,也可被外部 `GlobalKey<_WalletOnchainBalanceCardState>` 调
4. **`State` 类改为公开**:类名 `_WalletOnchainBalanceCardState` → `WalletOnchainBalanceCardState`(去掉下划线,让 `GlobalKey<WalletOnchainBalanceCardState>` 可以引用),`createState()` 同步改
5. **build 重构**:
   ```dart
   return Container(
     decoration: AppTheme.cardDecoration(radius: AppTheme.radiusLg),
     padding: const EdgeInsets.fromLTRB(16, 8, 16, 12),  // 顶 8 / 底 12,进一步收紧
     child: Column(
       crossAxisAlignment: CrossAxisAlignment.start,
       children: [
         const Text(
           '链上余额',
           style: TextStyle(
             fontSize: 14,
             fontWeight: FontWeight.w600,
             color: AppTheme.textSecondary,
           ),
         ),
         const SizedBox(height: 8),  // 12 → 8
         Row(
           crossAxisAlignment: CrossAxisAlignment.end,
           children: [
             Expanded(child: _buildAmountSection()),
             const SizedBox(width: 8),
             const Padding(
               padding: EdgeInsets.only(bottom: 4),
               child: Text(
                 'GMB',
                 style: TextStyle(
                   fontSize: 15,
                   fontWeight: FontWeight.w500,
                   color: AppTheme.textTertiary,
                 ),
               ),
             ),
           ],
         ),
       ],
     ),
   );
   ```
6. **`_buildAmountSection`** 不动(三态分支保留)
7. **文件头注释更新**:写明"刷新按钮已移除,刷新由外层 RefreshIndicator 下拉触发,通过 GlobalKey 调 refresh()。卡片高度进一步收紧"

### PR-C wallet_page.dart 加 RefreshIndicator + 整页下拉刷新

文件:`wuminapp/lib/wallet/ui/wallet_page.dart`

改动:

1. **新增 GlobalKey 字段**:
   ```dart
   // 中文注释:外层下拉刷新通过此 key 触发链上余额卡的 refresh()。
   final GlobalKey<WalletOnchainBalanceCardState> _balanceCardKey =
       GlobalKey<WalletOnchainBalanceCardState>();
   ```
2. **新增下拉刷新回调**:
   ```dart
   /// 整页下拉刷新:
   /// - 链上余额卡:通过 GlobalKey 调 refresh()
   /// - 交易记录:复用 _loadRecentRecords()
   /// - 清算行余额(动作卡"余额"列):本轮 0.00 元 写死占位,
   ///   待清算行功能落地后,在此处追加刷新调用。TODO(清算行)
   Future<void> _onPullRefresh() async {
     final futures = <Future<void>>[
       Future(() async {
         try {
           await _balanceCardKey.currentState?.refresh();
         } catch (_) {
           // 链上余额刷新失败由卡片内自处理,这里不阻塞其他刷新
         }
       }),
       _loadRecentRecords(),
     ];
     await Future.wait(futures);
   }
   ```
3. **build() 把 ListView 包进 RefreshIndicator**:
   ```dart
   body: RefreshIndicator(
     onRefresh: _onPullRefresh,
     child: ListView(
       padding: const EdgeInsets.all(16),
       physics: const AlwaysScrollableScrollPhysics(),  // 即使内容不满屏也能下拉
       children: [
         WalletIdentityCard(...),
         const SizedBox(height: 16),
         WalletActionCard(wallet: widget.wallet),
         const SizedBox(height: 16),
         WalletOnchainBalanceCard(
           key: _balanceCardKey,
           wallet: widget.wallet,
         ),
         const SizedBox(height: 24),
         _buildTransactionHistorySection(),
       ],
     ),
   ),
   ```
4. **import 补 `WalletOnchainBalanceCardState`**(从余额卡文件导出)

### 测试更新

- `wuminapp/test/wallet/ui/cards/wallet_onchain_balance_card_test.dart`:
  - 原"刷新按钮在第 1 行"的断言**删掉**(刷新按钮已移除)
  - 新增断言:第 1 行只有标题,无 IconButton(可用 `find.descendant(of: ..., matching: find.byType(IconButton))` 空查)
  - 原下拉刷新触发 refresh() 的逻辑可以加一条:通过 `GlobalKey` 调 `refresh()` 后状态变化(如 `_isLoading` 一过即过)
- `wallet_qr_dialog_test.dart`:
  - 不需要改断言,`Icons.copy` 仍然能 find;只是父结构从 Row 变 Stack 不影响 find 语义
- `wallet_identity_card_test.dart`:不动

## 验收

- `flutter analyze` 0 error / 0 warning
- `flutter test` 全部通过
- 人工验证:
  - 进钱包详情页,链上余额卡明显比 v3 矮
  - QR 弹窗里 QR 居中、地址也居中(在 QR 正下方),复制图标浮在地址右侧
  - 下拉钱包详情页可触发刷新动画,链上余额会重查、交易记录会重拉
  - 钱包名编辑态字体仍是黑色(v3 修复保留)
- `git diff --stat wumin/ citizenchain/ sfid/` 输出空

## Review 关注点

- `_WalletOnchainBalanceCardState` 改公开后,是否有其他地方私下引用了下划线版本
- `_balanceCardKey` 在 `dispose` 时不需要 dispose(GlobalKey 自管),但要确认 lifecycle 没坑
- RefreshIndicator 的 onRefresh 必须返回 `Future`,不能 `async; return;` 立刻完成,否则进度圈一闪而过(实际并发等待 `Future.wait` 自然解决)
- 不要误删 `_isLoading` 字段(`_buildAmountSection` 不用,但 `refresh()` 内仍 setState 用它做并发保护)
