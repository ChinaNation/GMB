# wuminapp 我的钱包列表页改版(单列横向卡 + 余额千分位 + 拖拽排序)

创建日期:2026-04-24
完成日期:2026-04-24
所属模块:wuminapp(Flutter 热钱包客户端)
主责 Agent:Mobile Agent
参照文件:`/Users/rhett/GMB/wumin/lib/ui/home_page.dart`(钱包列表卡片样式)
状态:DONE

## 执行结果

- **数据层**:`WalletProfileEntity` 加 `sortOrder` 字段;`WalletProfile` 模型同步;`getWallets()` 改为 `sortBySortOrder().thenByWalletIndex()`;新增 `reorderWallets(List<int>)`;首次启动按原 `walletIndex` 升序填 `sortOrder`(SharedPreferences 幂等 flag 保护);build_runner 重新生成 Isar 代码 ✓
- **千分位**:`AmountFormat.formatThousands(double?, {decimals=2})` 复用现有 `_addThousandSeparator`,无单位,null/NaN/Infinity → "--"
- **卡片重写**:新建顶层 `WalletListTile`(`@visibleForTesting`)+ 私有 `_ScanButton`;46×46 图标 + 钱包名 + "当前"标签 + 千分位余额 + 扫码图标(仅热钱包) + 三点菜单(重命名/删除钱包 2 项)
- **列表布局**:`GridView` → `ReorderableListView.builder`,长按拖拽,`_onReorder` 先 setState 后 await `reorderWallets`
- **清理**:Dismissible 滑动删除、冷热小标签、活跃浅绿背景、末位橙色背景全删,无残留
- **测试**:23 个新增测试 + 原 82 个 = 105/105 全过
- `flutter analyze` 0 error / 0 warning
- wumin / citizenchain 零改动

## 偏离项

1. 千分位未引入 `intl` 包,复用 `AmountFormat._addThousandSeparator`(同算法)避免新增依赖
2. `WalletListTile` 改顶层 + `@visibleForTesting`(Dart `_` 私有 widget 跨文件 import 不到,测试需要直接构造)
3. 首次迁移 `sortOrder` 填 0..N-1 而非 walletIndex 原值(顺序语义等价,数值更紧凑)
4. 多了 `showActions` 字段:选择模式下隐藏右侧扫码 + 三点菜单,沿用原 `_isSelectionMode` 行为

## Follow-up(已写入任务卡)

- 通用扫码 universal 路由能力:`QrScanPage` 加 `QrScanMode.universal`,扫到任意 QR 按 `kind` 自动 dispatch(收款码→支付 / 登录码→登录 / 签名码→签名);现 `qr_router` 已识别 kind,但 `loginChallenge` 走 reject、`signRequest` 无 page 处理,需补全

## 任务需求

把 wuminapp"我的 → 我的钱包"列表页([wuminapp/lib/wallet/ui/wallet_page.dart:22-565](wuminapp/lib/wallet/ui/wallet_page.dart:22) `MyWalletPage`)的钱包卡片改成与 wumin 冷钱包一样的横向单列样式,并按 wuminapp 端需求做 4 项差异化:

1. 第 2 行从地址改成**链上余额(只数字 + 千分位)**
2. 扫码图标按冷热区分:**热钱包显示,冷钱包隐藏**;原"冷/热"小标签删掉
3. 三点菜单只保留 **重命名 / 删除钱包**(去掉 wumin 的"钱包详情")
4. **整张卡片可点击进钱包详情**

并把列表布局从 `GridView 2 列` 改成 `ReorderableListView 单列`,支持拖拽排序。

## 必须遵守

- 只改 wuminapp/。wumin / citizenchain / sfid 零改动
- 不搞兼容/保留/过渡(冷热标签 / Dismissible 滑动删除 / 末位钱包橙背景 / 活跃钱包浅背景全部删,不留)
- 中文注释到位
- 数据迁移要无感:已有钱包按现有 `walletIndex` 顺序填 `sortOrder`,不丢顺序
- 余额行**只数字 + 千分位 + 两位小数**,无单位、无 GMB 后缀
- 扫码按钮点击本轮调现有 `QrScanPage(mode: QrScanMode.transfer)`,通用 universal 路由能力作为 follow-up 单独任务

## 输出物(PR 拆分,可串行执行)

### PR-1 · 数据层:`sortOrder` 字段 + 拖拽持久化

文件:
- `wuminapp/lib/Isar/wallet_isar.dart`
- `wuminapp/lib/wallet/core/wallet_manager.dart`

改动:

- `WalletProfileEntity` 新增字段 `int sortOrder = 0`(Isar 字段)
- `WalletProfile` 模型同步加 `int sortOrder` 字段(默认 0)
- `WalletManager.getWallets()`:从 `sortBy(walletIndex)` 改成 `sortBy(sortOrder).thenBy(walletIndex)`(sortOrder 相同时回退 walletIndex 兜底,保证稳定)
- 新增方法 `Future<void> reorderWallets(List<int> walletIndexes)`:按传入顺序给每个钱包写入新的 `sortOrder = listIndex`,在一次 Isar 事务里完成
- 启动 / `loadWallets` 时检测:如果发现所有钱包 `sortOrder` 都是 0(初始值)且钱包数量 > 1,**一次性按 `walletIndex` 升序填 sortOrder**(`sortOrder = walletIndex`),保证旧用户不丢顺序;只做一次,后续即使所有 sortOrder 巧合都是 0 也不再触发
  - 实现思路:加一个 `SharedPreferences` flag `wallet_sort_order_initialized = true`,初始化后置位
- Isar schema 升级若导致 schema 变化,跑通 `pubspec.yaml` 现有 build_runner / isar_generator,确保生成代码同步更新

### PR-2 · 千分位格式化工具

文件:`wuminapp/lib/utils/amount_format.dart`(若不存在按相邻路径 `lib/wallet/...` / `lib/format/...` 找)

新增方法:
```dart
/// 千分位格式化,只输出数字字符串(不带单位/符号)。
/// 示例:1234567.89 → "1,234,567.89";0.0 → "0.00";null → "--"
static String formatThousands(double? value, {int decimals = 2}) { ... }
```

- 实现可用 `NumberFormat('#,##0.00', 'en_US')`(`intl` 包,wuminapp 已用过的话直接复用)
- null / NaN / Infinity 返回 "--"
- 写单元测试覆盖:0、负数、大数(亿级)、小数、null 各 1 条

### PR-3 · 钱包卡片重写

文件:`wuminapp/lib/wallet/ui/wallet_page.dart`(`MyWalletPage` 部分,line 22-565)

新建私有 widget `_WalletListTile`(或叫 `WalletListItemCard`),布局照抄 [wumin/lib/ui/home_page.dart](wumin/lib/ui/home_page.dart) 钱包卡片(参照 line 631-649 "当前"标签实现):

```
┌──────────────────────────────────────────────────┐
│ [图标 46×46]  钱包名 + (当前标签)    [扫码][⋮]    │
│              链上余额数字(千分位,grey,13pt)       │
└──────────────────────────────────────────────────┘
```

具体规则:

- **图标**:46×46,圆角 `AppTheme.radiusSm`,沿用现有 `wallet.walletIcon` 数据
- **钱包名**:18pt / w600 / 主色文字
- **"当前"小标签**(仅 active 钱包):
  ```dart
  Container(
    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
    decoration: BoxDecoration(
      color: AppTheme.primary.withAlpha(30),
      borderRadius: BorderRadius.circular(4),
    ),
    child: const Text('当前',
      style: TextStyle(fontSize: 10, color: AppTheme.primaryLight, fontWeight: FontWeight.w600)),
  )
  ```
  在钱包名右侧 `SizedBox(width: 8)` 后追加
- **第 2 行余额**:
  ```dart
  Text(
    AmountFormat.formatThousands(wallet.balance),
    style: TextStyle(fontSize: 13, color: AppTheme.textSecondary),
  )
  ```
- **扫码按钮**(只在 `wallet.isHotWallet` 时显示):
  - 38×38 灰色方块容器,`AppTheme.cardDecoration` 或 `Colors.grey.withAlpha(30)` 背景
  - 图标 `Icons.qr_code_scanner`(或 wumin 用的 svg `assets/icons/scan-line.svg` 若 wuminapp 也有)
  - 点击:`Navigator.push(QrScanPage(mode: QrScanMode.transfer))`,加注释 "TODO(扫码 universal):待 universal 路由 follow-up 任务卡落地后改 mode"
- **三点菜单**(`PopupMenuButton<String>`):
  - 项:`重命名` / `删除钱包`(2 项,无"钱包详情")
  - `重命名`:沿用现有重命名逻辑(若现在是 Dismissible 流程,改成 AlertDialog with TextField)
  - `删除钱包`:沿用现有删除逻辑,把原 Dismissible 拆出来的确认对话框搬过来
- **整卡 InkWell**:`onTap: () => _openWalletDetail(wallet)`
- **背景**:统一 `AppTheme.cardDecoration(radius: AppTheme.radiusMd)`,**不再区分活跃/末位**(active 用"当前"标签标识,删除浅色背景 + 末位橙色背景)

### PR-4 · 列表布局 GridView → ReorderableListView

`MyWalletPage.build()` 内:

- `GridView.builder(2 列, 1.8 比)` 替换为:
  ```dart
  ReorderableListView.builder(
    padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
    itemCount: _wallets.length,
    onReorder: _onReorder,
    itemBuilder: (ctx, idx) {
      final wallet = _wallets[idx];
      return Padding(
        key: ValueKey(wallet.walletIndex),
        padding: const EdgeInsets.only(bottom: 8),
        child: _WalletListTile(wallet: wallet, ...),
      );
    },
  )
  ```
- `_onReorder(int oldIdx, int newIdx)`:
  ```dart
  Future<void> _onReorder(int oldIdx, int newIdx) async {
    if (newIdx > oldIdx) newIdx -= 1;
    setState(() {
      final w = _wallets.removeAt(oldIdx);
      _wallets.insert(newIdx, w);
    });
    await _walletManager.reorderWallets(
      _wallets.map((w) => w.walletIndex).toList(),
    );
  }
  ```

### PR-5 · 清理残留

- 删除:`Dismissible` 滑动删除整段(被菜单"删除钱包"承担)
- 删除:冷热标签 widget(`isColdWallet` 文字"冷"/`isHotWallet` 文字"热")相关代码
- 删除:活跃钱包 `primary.withAlpha(15)` 背景判断
- 删除:末位钱包 `warning.withAlpha(15)` 橙色背景判断
- import 清理(被删的 widget 类引用)
- 加注释解释:"v5 改版:卡片样式抄 wumin home_page,active 用'当前'标签替代背景色,扫码按钮按冷热区分显示,菜单 2 项(重命名/删除)"

### PR-6 · 测试

- `wuminapp/test/wallet/core/wallet_manager_test.dart`(若已存在则补,否则新建):
  - `reorderWallets` 写入 sortOrder 后再 `getWallets` 顺序正确
  - `sortOrder` 字段相同时按 walletIndex 兜底排序稳定
- `wuminapp/test/utils/amount_format_test.dart`(若不存在新建):
  - `formatThousands(1234567.89)` == "1,234,567.89"
  - `formatThousands(0.0)` == "0.00"
  - `formatThousands(null)` == "--"
- Widget 测试 `wuminapp/test/wallet/ui/wallet_list_tile_test.dart`(新建):
  - 渲染钱包名 + "当前"标签(active 时)
  - 余额行显示千分位数字
  - 热钱包显示扫码图标,冷钱包不显示
  - 三点菜单只有 2 项("重命名"/"删除钱包")
  - 卡片 InkWell 点击触发回调

## 验收

- `flutter analyze` 0 error / 0 warning
- `flutter test` 全部通过
- 人工跑一遍:
  - "我的 → 我的钱包" 列表页:每行 1 个卡片,样式同 wumin 冷钱包
  - 余额行显示千分位数字(无 GMB 单位)
  - 热钱包卡片右侧有扫码图标,冷钱包没有
  - 三点菜单只有"重命名"和"删除钱包"
  - 点击卡片进钱包详情页
  - 长按拖拽可重排,重启 App 后顺序保持
- wumin / citizenchain / sfid 代码零改动
- 升级时已有钱包顺序与之前一致(按原 walletIndex 顺序)

## Follow-up(本任务不做,单独任务卡)

- 通用扫码 universal 路由能力增强:`QrScanPage` 加 `QrScanMode.universal`,扫码后按 `kind` 自动 dispatch(收款码 → 支付 / 登录码 → 登录 / 签名码 → 签名);现 `qr_router` 已识别 `kind`,但 `loginChallenge` 走 reject 逻辑、`signRequest` 无 page 处理,要补全
- 创建任务卡:`memory/08-tasks/open/20260424-wuminapp-qr-scan-universal-route.md`(本任务结束后顺带建)

## Review 关注点

- `sortOrder` 字段迁移逻辑是否一次性、幂等
- 是否真的删干净了冷热标签 / 末位橙色背景 / 活跃浅色背景
- 三点菜单是否只 2 项("钱包详情"误留扣分)
- 余额千分位格式化在 null / 0 / 大数 / 负数 边界
- ReorderableListView 拖拽后 setState 顺序与持久化顺序一致
- 扫码按钮是否真的按 isHotWallet 区分显示
