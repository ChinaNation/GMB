# wuminapp 我的钱包列表卡片 v6(删"当前"标签 / 删扫码图标 / 钱包图标按冷热配色)

创建日期:2026-04-24
完成日期:2026-04-24
所属模块:wuminapp(Flutter 热钱包客户端)
主责 Agent:Mobile Agent
前置任务:memory/08-tasks/done/20260424-wuminapp-wallet-list-redesign.md(v5)
状态:DONE

## 执行结果

- `WalletListTile`:删 `isActive` 字段 + 构造参数 + "当前"标签 Container;钱包名 Row 简化为单 Text
- `_MyWalletPageState`:删 `_activeWalletIndex` 字段 + `_loadActiveWallet` 方法;`_walletService.setActiveWallet` 在 `_openWalletDetail` 的 selectForTrade 业务里仍保留(与列表标签无关)
- `WalletListTile`:删 `_ScanButton` widget 类 + `onScan` 字段 + 扫码区块;`_openScan` 方法删;`import qr_scan_page.dart` 删;v5 留的 `// TODO(扫码 universal): ...` 整段注释删
- `WalletListTile.build` 开头新增 `final isHot / iconBg / iconColor` 三行;左侧 46×46 Container 用 isHot 二选一(热=primary 墨绿,冷=info 蓝)
- `wallet_list_tile_test.dart`:删 active/扫码相关旧用例;新增 4 条断言("当前"零渲染 / 扫码按钮零渲染 / 热钱包图标 primaryDark / 冷钱包图标 info);保留钱包名/余额/三点菜单/InkWell 测试
- `flutter analyze` 0 error / 0 warning
- `flutter test` 107/107 全通过(单文件 wallet_list_tile 12/12)
- grep 零残留:`_ScanButton` / `_openScan` / `onScan:` / `isActive:` / `TODO(扫码 universal` / `QrScanPage` 在 wallet_page.dart 内全部 0 命中
- `AppTheme.primaryLight` 在 wallet_page.dart 内已无引用
- wuminapp 范围内无误改;**citizenchain 工作区有 17 个文件改动属于其他来源**(非本次 agent),已在用户处 flag

## Follow-up 取消

- v5 任务卡里提到的"扫码 universal 路由 follow-up" 作废:扫码图标已删,钱包卡片不再需要扫码入口;扫码支付/扫码加好友等已有专门入口,扫码登录/扫码签名属于冷钱包功能,wuminapp 钱包卡片不需要扫码功能

## 任务需求

3 项简化:

1. **删"当前"标签**:钱包列表卡片不再有 active 状态视觉(点钱包卡片直接进详情,不存在"选中当前"概念)
2. **删扫码图标 + 配套全套 scan 代码**:扫码支付/扫码加好友有专门入口,扫码登录/扫码签名属于冷钱包功能,wuminapp 钱包卡片不需要扫码入口
3. **钱包图标按冷热配色**:
   - 热钱包:`AppTheme.primary` 系(墨绿主色调)
   - 冷钱包:`AppTheme.info` 系(蓝色,离线签名设备调性)

## 必须遵守

- 只改 wuminapp/。wumin / citizenchain / sfid 零改动
- 不搞兼容/保留/过渡:扫码相关代码全删,**不留 TODO 不留注释占位**
- 中文注释到位
- v5 任务卡里提到的"扫码 universal 路由 follow-up"作废,**不再创建**该 follow-up 任务卡
- `AppTheme.info` 已存在(`/Users/rhett/GMB/wuminapp/lib/ui/app_theme.dart:44 = Color(0xFF3B82F6)`),直接用

## 输出物

### PR-A 删"当前"标签 + 清掉 isActive

文件:`/Users/rhett/GMB/wuminapp/lib/wallet/ui/wallet_page.dart`

- `WalletListTile` 删:
  - `final bool isActive` 字段
  - 构造参数 `required this.isActive`
  - build 内 `if (isActive) ...[ SizedBox(width: 8) + Container("当前") ]` 整段
  - 钱包名 Row 简化:只剩 `Flexible(Text(wallet.walletName))`,不需要 Row 包裹时直接降级
- `_MyWalletPageState`:
  - 删 `_activeWalletIndex` 字段(若仅用于"当前"标签判断;若还在"切换 active 钱包"逻辑里用,只删传参不删字段并加注释说明)
  - `WalletListTile(...)` 调用处删 `isActive: ...` 传参
- import 清理:`AppTheme.primaryLight` 若不再使用就移除(注意此颜色可能在文件其它地方还用,grep 确认)

### PR-B 扫码功能彻底删除

文件:`/Users/rhett/GMB/wuminapp/lib/wallet/ui/wallet_page.dart`

- 删 `_ScanButton` widget 类整段
- `WalletListTile` 删:
  - `final VoidCallback onScan` 字段
  - 构造参数 `required this.onScan`
  - build 内 `if (showActions && wallet.isHotWallet) ...[ SizedBox + _ScanButton ]` 整段
- `_MyWalletPageState`:
  - 删 `_openScan(WalletProfile)` 方法
  - `WalletListTile(...)` 调用处删 `onScan: () => _openScan(wallet)` 传参
- 删 `import 'package:wuminapp_mobile/qr/pages/qr_scan_page.dart'`(若 wallet_page.dart 不再用)
- 删 v5 留的 `// TODO(扫码 universal): ...` 整段注释(不论几行)

### PR-C 钱包图标按冷热配色

文件:`/Users/rhett/GMB/wuminapp/lib/wallet/ui/wallet_page.dart`

`WalletListTile` 左侧 46×46 钱包图标 Container 改造:

```dart
// 中文注释:钱包图标按冷热区分配色 —— 热=墨绿主色,冷=蓝(离线签名设备调性)。
final isHot = wallet.isHotWallet;
final iconBg = isHot ? AppTheme.primary.withAlpha(20) : AppTheme.info.withAlpha(20);
final iconColor = isHot ? AppTheme.primaryDark : AppTheme.info;

Container(
  width: 46,
  height: 46,
  decoration: BoxDecoration(
    color: iconBg,
    borderRadius: BorderRadius.circular(AppTheme.radiusSm),
  ),
  child: Icon(Icons.account_balance_wallet_rounded, color: iconColor, size: 24),
)
```

### PR-D 测试更新

文件:`/Users/rhett/GMB/wuminapp/test/wallet/ui/wallet_list_tile_test.dart`

- 删:"active 时显示'当前'标签"断言
- 改:增加新断言"任何 wallet 都不渲染 '当前' 文本(`find.text('当前')` → `findsNothing`)"
- 删:"热钱包显示扫码图标"和"冷钱包不显示扫码图标"两条断言
- 改:增加新断言"任何 wallet 都不渲染扫码按钮(`find.byIcon(Icons.qr_code_scanner)` → `findsNothing`)"
- 新增:"热钱包图标 Icon 颜色为 AppTheme.primaryDark"
- 新增:"冷钱包图标 Icon 颜色为 AppTheme.info"
- 删 `WalletListTile(... isActive: true / false ...)` 测试构造参数;删 `onScan: ...`

构造 WalletListTile 时:
```dart
WalletListTile(
  wallet: ...,
  onTap: ...,
  onRename: ...,
  onDelete: ...,
  // 不再有 isActive / onScan
)
```

## 验收

- `flutter analyze` 0 error / 0 warning
- `flutter test` 全部通过
- 人工验证:
  - 列表里所有钱包都没有"当前"小标签
  - 列表里所有钱包都没有扫码图标
  - 热钱包左侧图标:墨绿色调
  - 冷钱包左侧图标:蓝色色调
  - 点击卡片能进钱包详情(行为不变)
  - 拖拽排序仍可用(行为不变)
  - 三点菜单仍是 重命名/删除钱包(行为不变)
- `git diff --stat wumin/ citizenchain/ sfid/` 输出空

## Review 关注点

- 是否真的彻底删干净扫码代码(_ScanButton/_openScan/onScan 字段/import/TODO 注释 一条不剩)
- "当前"标签和 isActive 字段是否有遗漏(grep `isActive` 应在 `WalletListTile` 内零结果)
- 钱包图标颜色判断是否真按 `wallet.isHotWallet` 而不是其他属性
- AppTheme.primaryLight import 是否要清理
