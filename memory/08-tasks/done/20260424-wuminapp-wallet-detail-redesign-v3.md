# wuminapp 钱包详情页改版 v3(顶部 padding 收紧 / QR 弹窗微调 / 编辑态字体修复)

创建日期:2026-04-24
完成日期:2026-04-24
所属模块:wuminapp(Flutter 热钱包客户端)
主责 Agent:Mobile Agent
前置任务:
- memory/08-tasks/done/20260423-wuminapp-wallet-detail-redesign.md
- memory/08-tasks/done/20260424-wuminapp-wallet-detail-redesign-v2.md
状态:DONE

## 执行结果

- `wallet_onchain_balance_card.dart`:卡片 padding `vertical 20` → `fromLTRB(16, 8, 16, 16)`,顶部上移 12px
- `wallet_qr_dialog.dart`:删副标题"扫码转账或绑定";复制图标 `size 18 → 14`,constraints `32 → 24`;下载从 `IconButton` 改 `Expanded(TextButton('下载'))`,与"关闭"对称等宽,loading 时按钮内显示进度圈
- `wallet_identity_card.dart`:编辑态 TextField 字体/光标/下划线改黑色系(`Colors.black87` + `Colors.black54`),展示态白色保持
- `wallet_qr_dialog_test.dart`:`find.byIcon(Icons.download)` 断言改为 `find.widgetWithText(TextButton, '下载')`
- `flutter analyze` 0 error / 0 warning
- `flutter test` 81/81 全通过
- wumin / citizenchain / sfid 零改动

## 任务需求

3 个 UI 微调:

1. 链上余额卡:`链上余额` 标题 + 刷新按钮距离卡片顶部 padding 收紧,整体上移
2. QR 弹窗:
   - 复制图标缩小
   - 删掉副标题 "扫码转账或绑定"
   - 下载图标改成 "下载" 文字按钮,与 "关闭" 对称等宽
3. 修改钱包名时编辑态字体改黑色(当前白底+白字看不见)

## 必须遵守

- 只改 wuminapp/,不改 wumin / citizenchain / sfid
- 不搞兼容/保留/过渡
- 中文注释到位
- 展示态钱包名保持白色不变,只改编辑态字体颜色
- 链上余额卡只调 padding,不动其他逻辑

## 输出物

### PR-A 链上余额卡顶部 padding

文件:`wuminapp/lib/wallet/ui/cards/wallet_onchain_balance_card.dart`

改一行:
```dart
// 旧:
padding: const EdgeInsets.symmetric(vertical: 20, horizontal: 16),
// 新:
padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
```

文件头注释补一行说明"顶部 padding 收紧到 8,标题和刷新按钮贴近卡片顶部"。

### PR-B QR 弹窗复制图标缩小 + 删副标题 + 下载按钮对称

文件:`wuminapp/lib/wallet/ui/cards/wallet_qr_dialog.dart`

改动 1 · 删副标题:删除 `Text('扫码转账或绑定', ...)` 那行 + 它紧邻的 `SizedBox(height: 4)`(名称下面的小间距)。原 16px 间距(QR 上方)保留。

改动 2 · 复制图标缩小:
```dart
IconButton(
  icon: const Icon(Icons.copy, size: 14),  // 18 → 14
  color: Colors.grey[600],
  tooltip: '复制地址',
  padding: EdgeInsets.zero,
  constraints: const BoxConstraints(minWidth: 24, minHeight: 24),  // 32 → 24
  onPressed: _copyAddress,
),
```

改动 3 · 下载按钮改成对称的文字按钮。原底部 Row 重写:
```dart
Row(
  children: [
    Expanded(
      child: TextButton(
        onPressed: () => Navigator.of(context).pop(),
        child: const Text('关闭'),
      ),
    ),
    const SizedBox(width: 8),
    Expanded(
      child: TextButton(
        onPressed: _isSaving ? null : _saveQrToGallery,
        child: _isSaving
            ? const SizedBox(
                width: 18, height: 18,
                child: CircularProgressIndicator(strokeWidth: 2),
              )
            : const Text('下载'),
      ),
    ),
  ],
),
```

文件头注释更新:"删掉副标题 / 复制图标缩小 / 下载与关闭对称等宽两个 TextButton"。

### PR-C 钱包名编辑态字体改黑色

文件:`wuminapp/lib/wallet/ui/cards/wallet_identity_card.dart`

`_isEditingName == true` 分支的 TextField:
```dart
TextField(
  controller: _nameController,
  autofocus: true,
  style: const TextStyle(
    fontSize: 18,
    fontWeight: FontWeight.w700,
    color: Colors.black87,  // 原 Colors.white
  ),
  cursorColor: Colors.black87,  // 原 Colors.white
  decoration: const InputDecoration(
    isDense: true,
    contentPadding: EdgeInsets.symmetric(vertical: 4),
    enabledBorder: UnderlineInputBorder(
      borderSide: BorderSide(color: Colors.black54),  // 原 white.withAlpha(180)
    ),
    focusedBorder: UnderlineInputBorder(
      borderSide: BorderSide(color: Colors.black54),  // 原 white.withAlpha(180)
    ),
  ),
  textInputAction: TextInputAction.done,
  onSubmitted: _submitName,
  onTapOutside: (_) {
    _submitName(_nameController.text);
  },
)
```

展示态 `Text(_walletName, ...)` 保持 `Colors.white` 不动。

文件头注释补一行:"编辑态 TextField 字体/光标/下划线改黑色,避开 Material TextField 默认白底导致白字看不见的问题"。

### 测试更新

- `wallet_qr_dialog_test.dart`:
  - 删除"扫码转账或绑定"文本断言(若有)
  - 把原 `find.byIcon(Icons.download)` 的断言改成 `find.widgetWithText(TextButton, '下载')`,断言下载文字按钮存在
  - 复制图标的 size 14 不必硬断言,只需保留 `Icons.copy` 可见的断言
- `wallet_identity_card_test.dart`:可加一条"编辑态 TextField 的 style.color == Colors.black87"的断言(可选)
- `wallet_onchain_balance_card_test.dart`:无需改动,仍验证标题 + 刷新按钮 + GMB 位置

## 验收

- `flutter analyze` 0 error / 0 warning
- `flutter test` 全部通过
- `git diff` 范围只覆盖 wuminapp/,wumin / citizenchain / sfid 零改动

## Review 关注点

- 展示态钱包名是否仍是白色(只改编辑态)
- QR 弹窗副标题彻底删除,无残留 Text 或 SizedBox
- 下载按钮 loading 态显示进度圈,disable 正确
- 链上余额卡只改 padding,其他不动
