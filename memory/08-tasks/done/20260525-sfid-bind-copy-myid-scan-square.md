# SFID 绑定文案和 wuminapp 现场扫码框修正

## 任务需求

1. SFID 公民绑定弹窗只在签名二维码步骤将标题“扫描档案码”改为“扫码签名”，上一步扫描档案码不改。
2. 删除签名二维码步骤中的“第二步：用公民钱包扫码签名”提示。
3. 将“下一步：扫描签名结果”改为“扫描签名结果”。
4. 公民列表列名“SFID码”改为“身份ID”。
5. wuminapp 电子护照现场签名扫码页面改成真正的正方形相机扫码框。

## 建议模块

- SFID 前端公民绑定弹窗和公民列表。
- wuminapp 电子护照现场签名页。

## 影响范围

- `sfid/frontend/citizens/`
- `wuminapp/lib/my/myid/`
- `memory/05-modules/sfid/frontend/`
- `memory/05-modules/wuminapp/user/`

## 主要风险点

- 只能改签名二维码步骤标题，不能影响上一步扫描档案码标题。
- 公民列表列名改为“身份ID”后，不改变底层 `sfid_number` 字段。
- wuminapp 扫码框必须是固定正方形相机区域，不再保留原来的大矩形相机画面。

## 验收标准

- 扫描档案码步骤标题仍为“扫描档案码”。
- 签名二维码步骤标题为“扫码签名”，且不显示“第二步：用公民钱包扫码签名”。
- 签名二维码步骤按钮文案为“扫描签名结果”。
- 公民列表列名显示“身份ID”。
- wuminapp 现场签名扫码页显示固定正方形扫码框。

## 执行记录

- 已将 `BindModal.tsx` 的弹窗标题改为按步骤切换：扫描档案码步骤仍为“扫描档案码”，签名二维码步骤为“扫码签名”。
- 已删除签名二维码步骤里的“第二步：用公民钱包扫码签名”提示。
- 已将按钮“下一步：扫描签名结果”改为“扫描签名结果”。
- 已将 `CitizensView.tsx` 公民列表列名“SFID码”改为“身份ID”。
- 已将 `MyIdSignPage` 现场签名扫码区域改为固定 260×260 正方形相机框。
- 已同步 SFID 前端布局文档和 wuminapp 用户模块文档。

## 验证记录

- `npm run build` 通过。
- `dart format lib/my/myid/myid_sign_page.dart` 通过。
- `flutter analyze lib/my/myid/myid_sign_page.dart` 通过。

- 状态：done

## 完成信息

- 完成时间：2026-05-25 12:49:59
- 完成摘要：已按步骤限定修改 SFID 绑定弹窗文案、公民列表列名，并将 wuminapp 现场签名扫码页改为固定正方形相机框；构建和分析通过。
- 对照清单：memory/07-ai/pre-submit-checklist.md
- 对照总标准：memory/07-ai/definition-of-done.md
