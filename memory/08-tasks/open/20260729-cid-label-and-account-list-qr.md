# 身份 CID 号文案统一 + 公民钱包账户列表钱包码入口

状态：open（2026-07-29 两项均已完成并验证；留一条待你拍板的文案一致性项）

## 实现与验证（2026-07-29）

### 第 1 项：身份 CID 号文案

`governance/InstitutionDetailPage.tsx:175` 单点改为「机构类型 /身份CID号」，`NrcSection`
（国家储委会）/ `PrcSection`（省储委会）/ `PrbSection`（省储行）三个 tab 同时生效。
`node/frontend` `tsc -b` 通过；全仓 `身份ID` 仅剩清算行 tab 一处（见下方待拍板项）。

### 第 2 项：账户列表钱包码入口

- 新增 `citizenwallet/lib/ui/widgets/wallet_qr_dialog.dart`：`buildWalletQr` +
  `showWalletQrDialog` 从账户详情页抽出，成为钱包码构造与展示的单源。
- `account_detail_page.dart`：删本地弹窗副本，改调共享函数（原 ~70 行弹窗代码收敛为 6 行）。
- `wallet_detail_page.dart`：账户行右箭头左侧加 `Icons.qr_code_rounded` 独立热区
  （`IconButton` + `Semantics`，`visualDensity: compact`），点它弹钱包码、不触发整行 onTap。
- `test/ui/account_detail_page_test.dart`：`buildWalletQr` import 随迁。
- `test/ui/wallet_detail_page_test.dart`：新增测试，断言入口存在、与右箭头并存、
  弹出的是钱包码文案、**未跳进账户详情**、无任何时效文案、关闭后消失。

| 项 | 结果 |
|---|---|
| `flutter analyze`（citizenwallet） | 0 问题 |
| `flutter test`（citizenwallet） | **249 passed**（原 248 + 新 1） |
| `tsc -b`（node/frontend） | 通过 |

## 待你拍板

清算行 tab 的 `transaction/offchain/institution/institution-detail.tsx:135` 仍是
「机构身份ID」。你只指定了国家储委会/省储委会/省储行三个 tab，按 `no-scope-expansion` 未动。
要不要一并统一成「身份CID号」，你定。

## 任务需求

1. 区块链软件节点前端「国家储委会 / 省储委会 / 省储行」3 个 tab 中所有机构的
   `机构类型 /身份ID` 改成 `机构类型 /身份CID号`。
2. 公民钱包「钱包详情」页账户列表中，每个账户的右箭头左侧增加一个小二维码图标，
   点击显示该账户的钱包码——即把账户详情里账户名称右侧的那个二维码，在账户列表也开一个入口，
   用户不进账户详情就能打开。

## 已确认边界

- 第 1 项：三个 tab 共用同一组件 `citizenchain/node/frontend/governance/InstitutionDetailPage.tsx`
  （被 `NrcSection` / `PrcSection` / `PrbSection` 引用），文案是单源，改一处即覆盖三个 tab。
- 第 1 项**不动**清算行 tab 的 `transaction/offchain/institution/institution-detail.tsx:135`
  「机构身份ID」——用户只指定了那三个 tab，按 `no-scope-expansion` 不附加。该处文案与新口径不
  一致，作为已知项记录，待用户决定是否一并统一。
- 第 2 项必须**复用**账户详情已有的钱包码弹窗，不得复制第二份（`no-remnants` / 单源）。
  因此把 `buildWalletQr` 与弹窗一起抽到 `citizenwallet/lib/ui/widgets/wallet_qr_dialog.dart`，
  账户详情与账户列表都调它。
- 图标用 `Icons.qr_code_rounded`，与账户详情页现有的显示型二维码入口一致。
  死规则 `scan-icon-must-be-scan-line-svg` 约束的是**扫码**入口（只能用
  `assets/icons/scan-line.svg`），本项是**展示**二维码，不适用。
- 点击二维码图标不得触发行的 `onTap`（进入账户详情），两个热区必须互斥。
- 不改钱包码载荷、不改协议。

## 预计修改目录

- `citizenchain/node/frontend/governance/InstitutionDetailPage.tsx`：文案。
- `citizenwallet/lib/ui/widgets/wallet_qr_dialog.dart`：新增，承载 `buildWalletQr` + 弹窗。
- `citizenwallet/lib/ui/account_detail_page.dart`：改为 import 共享弹窗，删本地副本。
- `citizenwallet/lib/ui/wallet_detail_page.dart`：账户行加二维码图标入口。
- `citizenwallet/test/ui/account_detail_page_test.dart`：import 随迁。
- `citizenwallet/test/ui/wallet_detail_page_test.dart`：补账户列表二维码入口测试。

## 主要风险

- `buildWalletQr` 迁移会破坏现有测试的 import，必须同批改。
- 账户行内嵌可点击图标容易与整行 `InkWell` 抢手势，需确认点图标不进详情页。

## 完成标准

- 三个 tab 均显示「机构类型 /身份CID号」。
- 账户列表每行右箭头左侧有二维码图标，点击弹出该账户钱包码（内容与账户详情里的一致）。
- 点图标不进入账户详情；点行其余区域仍进入账户详情。
- `flutter analyze` 0 问题、`flutter test` 全绿、node 前端 `tsc -b` 通过。
