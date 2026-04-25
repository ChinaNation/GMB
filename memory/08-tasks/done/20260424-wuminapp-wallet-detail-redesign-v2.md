# wuminapp 钱包详情页改版 v2(卡片顺序 + 动作卡三列 + 余额卡重排 + QR 复制/下载)

创建日期:2026-04-24
完成日期:2026-04-24
所属模块:wuminapp(Flutter 热钱包客户端)
主责 Agent:Mobile Agent
前置任务:memory/08-tasks/done/20260423-wuminapp-wallet-detail-redesign.md
状态:DONE

## 执行结果

- `wallet_page.dart`:卡片顺序改为身份卡 → 动作卡 → 链上余额卡 → 交易记录
- `wallet_action_card.dart`:重写为 3 列布局,`_ClickableAction`(充值/提现 + InkWell)+ `_StaticBalance`(余额无交互);小字 `0.00 元` 占位,非断空格 `\u00A0` 对齐
- `wallet_onchain_balance_card.dart`:刷新按钮右上(与标题同行),GMB 右下(与金额 baseline 对齐);加载/错误/正常三态分别在金额位处理
- `wallet_qr_dialog.dart`:抽出 `_WalletQrDialogContent` StatefulWidget,`GlobalKey _qrKey` + `RepaintBoundary` 包 QR;地址行加复制图标,底部按钮行加下载图标,走 `SaverGallery.saveImage`
- 测试:`wallet_action_card_test` 硬断言整卡仅 2 个 InkWell;`wallet_onchain_balance_card_test` 断言刷新和 GMB 位置;`wallet_qr_dialog_test` 断言复制/下载图标可见,下载走异常分支不崩
- `flutter analyze` 0 error / 0 warning
- `flutter test` 81/81 全通过
- wumin / citizenchain / sfid 零改动
- `pubspec.yaml` 未改(saver_gallery ^4.1.0 依赖本就存在)

## 偏离项

- 原计划 qr_dialog 下载测试用 `pumpAndSettle`,实际会被 loading 动画卡死;改用固定次数 `pump(Duration(ms:100)) × 10` 推进异步链,注释写明原因

## 遗留(非本任务)

- 动作卡"余额"列小字 `0.00 元` 为占位,等清算行功能落地后接真实数据(本地 prefs `clearing_bank_shenfen_id_<walletIndex>` + 链下账本查询)
- 充值 / 提现按钮实际业务链路仍等清算行功能落地

## 任务需求

延续上一轮 v1 改版,做 4 个小项优化:

1. **卡片顺序对调**:`WalletIdentityCard`(身份卡)从第 2 位挪到第 1 位,`WalletActionCard`(动作卡)从第 1 位挪到第 2 位。链上余额卡和交易记录位置不变
2. **动作卡扩 3 列**:从 "充值 + 提现" 2 列扩到 "充值 + 提现 + 余额" 3 列;"余额"为**静态展示**(不可点击),下方小字显示该钱包在清算行的余额(本轮占位 `0.00 元`,等清算行功能落地后对接)
3. **链上余额卡布局重排**:刷新按钮从右中移到**右上角**,"GMB" 从金额右侧移到**右下角**
4. **QR 弹窗补 2 个操作**:
   - 地址文本后加 `Icons.copy` 图标按钮,点击复制地址 + SnackBar
   - 关闭按钮右侧加 `Icons.download` 图标按钮,点击把二维码保存到相册

## 输入文档

- memory/08-tasks/done/20260423-wuminapp-wallet-detail-redesign.md(v1 完整技术方案)
- 相关代码文件:
  - wuminapp/lib/wallet/ui/wallet_page.dart
  - wuminapp/lib/wallet/ui/cards/wallet_action_card.dart
  - wuminapp/lib/wallet/ui/cards/wallet_identity_card.dart
  - wuminapp/lib/wallet/ui/cards/wallet_onchain_balance_card.dart
  - wuminapp/lib/wallet/ui/cards/wallet_qr_dialog.dart

## 必须遵守

- 只改 wuminapp/,不改 wumin / citizenchain / sfid
- 不搞兼容/保留/过渡,要删的代码直接删
- 中文注释到位
- "余额"按钮**不可点击**,不要加 InkWell/GestureDetector/点击回调;只是视觉占位
- 动作卡 3 列等宽,底部对齐(充值/提现下方留等高占位,和"余额"下方小字对齐)
- QR 下载依赖 `saver_gallery` 包,若 pubspec 里已被清理需补回;Info.plist 权限声明若已有则保留,本轮不加
- 链上余额口径保持不变(`free + reserved` 最新块)
- QR payload 保持不变(`WUMIN_QR_V1 kind=user_contact`)

## 输出物

### PR-A 卡片顺序对调

- 改 `wallet_page.dart` 的 `build()` 方法,`body: ListView` 内三个卡片从
  ```
  WalletActionCard → SizedBox(16) → WalletIdentityCard → SizedBox(16) → WalletOnchainBalanceCard
  ```
  改为
  ```
  WalletIdentityCard → SizedBox(16) → WalletActionCard → SizedBox(16) → WalletOnchainBalanceCard
  ```
- 不改卡片内部实现

### PR-B 动作卡扩 3 列 + 清算行余额占位

- `wallet_action_card.dart` 重构:
  - 布局用 `Row` + 3 个 `Expanded`,三列等宽
  - 每列结构:
    ```
    Column(crossAxisAlignment: center, [
      Container(56×56 圆形,primary.withAlpha(15)) + Icon,
      SizedBox(8),
      Text(标签, 14/w600/primaryDark),
      SizedBox(4),
      Text(小字, 12/textTertiary),  // 前两列显示空占位 '' 或 SizedBox(height:16) 保对齐,第 3 列显示 '0.00 元'
    ])
    ```
  - 三列定义:
    | 列 | 图标 | 标签 | 小字 | 交互 |
    |----|------|------|------|------|
    | 1 | `Icons.arrow_circle_down_outlined` | 充值 | 占位(空白等高) | `InkWell` 点击 SnackBar "功能开发中" |
    | 2 | `Icons.arrow_circle_up_outlined` | 提现 | 占位(空白等高) | `InkWell` 点击 SnackBar "功能开发中" |
    | 3 | `Icons.account_balance_wallet_outlined` | 余额 | `0.00 元` | **无交互**,纯展示 |
  - "充值"/"提现" 的 `InkWell` 只包圆圈区域或整列都可,要求 ripple 不超出圆圈边界;推荐 `Material(type: transparency) + InkWell(borderRadius: 28, child: ...)` 只作用圆圈
  - 整卡 `padding` 从 `vertical: 20, horizontal: 16` 按视觉需要可调(保持不臃肿)
  - 文件头注释更新:说明第 3 列"余额"为静态展示,等清算行功能落地后接真实数据

### PR-C 链上余额卡重排

- `wallet_onchain_balance_card.dart`:
  - 原 `Column` 两行结构改为:
    ```
    Row: [Text('链上余额', 14/w600/textSecondary), Spacer, 刷新按钮(IconButton 或 CircularProgressIndicator)]
    SizedBox(12)
    Row(crossAxisAlignment: baseline, textBaseline: alphabetic): [
      金额(32/w700/primaryDark) + Text('元', 22/w700/primaryDark),
      Spacer,
      Text('GMB', 15/w500/textTertiary),
    ]
    ```
  - 错误态"查询失败,点击刷新"可直接放在第二行(替代金额位置),或作为第 3 行小字,保持右上刷新/右下 GMB 不变
  - 加载态占位"— 元"保留,右下 GMB 仍显示
  - `_buildAmountRow` 适配新布局,拆成 `_buildTopRow`(标题+刷新)和 `_buildBottomRow`(金额+GMB)或其他清晰结构
  - 文件头注释补一行说明"刷新右上 / GMB 右下"的新布局

### PR-D QR 弹窗补复制/下载

- `wallet_qr_dialog.dart`:
  - 因为下载需要 `RepaintBoundary` + `GlobalKey` + StatefulWidget 生命周期,把 `WalletQrDialog.show` 内部 `Dialog` 内容抽成一个私有 `_WalletQrDialogContent extends StatefulWidget`,维护 `_qrKey = GlobalKey()` 和 `_isSaving = false`
  - 地址行改为:
    ```
    Row(mainAxisAlignment: center, [
      Flexible(child: Text(address, 11/grey/monospace, textAlign: center)),
      SizedBox(8),
      IconButton(icon: Icon(Icons.copy, size 18), onPressed: _copyAddress, tooltip: '复制地址'),
    ])
    ```
    点击复制 → Clipboard + SnackBar "钱包地址已复制"
  - QR 本身外包 `RepaintBoundary(key: _qrKey, child: Container(...QrImageView...))`
  - 底部按钮行改为:
    ```
    Row([
      Expanded(child: TextButton('关闭', onPressed: pop)),
      SizedBox(8),
      IconButton(
        icon: _isSaving ? CircularProgressIndicator(size 18, strokeWidth 2) : Icon(Icons.download),
        onPressed: _isSaving ? null : _saveQr,
        tooltip: '保存二维码到相册',
      ),
    ])
    ```
  - `_saveQr` 实现:
    ```
    boundary = _qrKey.currentContext.findRenderObject() as RenderRepaintBoundary
    image = await boundary.toImage(pixelRatio: 3.0)
    byteData = await image.toByteData(format: ImageByteFormat.png)
    bytes = byteData.buffer.asUint8List()
    fileName = 'wallet_qr_${DateTime.now().millisecondsSinceEpoch}.png'
    result = await SaverGallery.saveImage(Uint8List.fromList(bytes), fileName: fileName, skipIfExists: false)
    SnackBar(result.isSuccess ? '二维码已保存到相册' : '保存失败,请检查相册权限')
    ```
    异常统一 SnackBar "保存失败:$e"
  - 依赖:`pubspec.yaml` 查 `saver_gallery`,不存在则补回;import 路径对齐原 wallet_page.dart 删除前的用法
  - 文件头注释更新:说明新增的复制/下载交互

### 测试更新

- `wallet_action_card_test.dart`:断言渲染"充值/提现/余额" 3 个 label + 3 个指定图标;充值/提现点击弹 SnackBar "功能开发中";"余额"列**没有可点击区域**(搜 `InkWell` 或 `GestureDetector` 在第 3 列内为空);第 3 列小字显示 `0.00 元`
- `wallet_onchain_balance_card_test.dart`:断言刷新按钮在 `Column` 的第 1 行(和标题同行),"GMB" 在第 2 行末端
- `wallet_qr_dialog_test.dart`:渲染地址后的复制图标和关闭旁的下载图标;点击复制触发 Clipboard(可用 `TestDefaultBinaryMessengerBinding` mock);下载按钮在 loading 态 disable

### 依赖

- 如 `pubspec.yaml` 丢了 `saver_gallery`,补回(按原版本号或最新稳定版),跑 `flutter pub get`

## 验收标准

- `flutter analyze` 0 error / 0 warning
- `flutter test` 全部通过
- grep 零残留:
  ```
  grep "WalletActionCard" wuminapp/lib/wallet/ui/wallet_page.dart  → 应该在 WalletIdentityCard 之后出现
  ```
- 人工跑一遍:
  - 进钱包详情页 → 顺序是身份卡 / 动作卡 / 链上余额卡 / 交易记录
  - 动作卡 3 个图标分别充值 / 提现 / 余额;点充值/提现弹"功能开发中";点"余额"无反应
  - "余额"下方小字 `0.00 元`
  - 链上余额卡:右上刷新、右下 GMB
  - 身份卡点 QR 图标弹大码 → 地址后有复制图标(点击复制 + SnackBar);关闭按钮右侧有下载图标(点击下载到相册 + SnackBar)
- wumin / citizenchain / sfid 代码零改动

## Review 关注点

- "余额"列是否真的无交互(误加 InkWell/GestureDetector 扣分)
- 动作卡 3 列底部是否对齐
- 下载逻辑是否正确嵌入 RepaintBoundary
- `pubspec.yaml` 如改过,有没有相应跑 `flutter pub get`
- 不要误改 v1 改版已完成的业务文件
