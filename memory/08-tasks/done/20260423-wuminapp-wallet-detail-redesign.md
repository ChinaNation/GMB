# wuminapp 钱包详情页改版(我的-钱包-我的钱包-钱包详情)

创建日期:2026-04-23
完成日期:2026-04-23
所属模块:wuminapp(Flutter 热钱包客户端)
主责 Agent:Mobile Agent
状态:DONE

## 执行结果

- 新建 5 个文件(4 张卡片 + 1 个占位页) + 5 个 widget 测试
- 改动 3 个代码文件(`chain_rpc.dart` / `wallet_page.dart` / `onchain_trade_page.dart` 注释)+ 2 处受影响注释补正(`bind_clearing_bank_page.dart` / `clearing_bank_prefs.dart`)
- 删除 3 个旧文件(`clearing_payment_entry_page.dart` / `offchain_clearing_receive_page.dart` / `clearing_bank_list_page.dart`)
- `flutter analyze` 0 error / 0 warning
- `flutter test` 75 个测试全部通过
- grep 零残留 10/10 全通过
- wumin / citizenchain / sfid 零改动

## 新建文件

- `wuminapp/lib/wallet/ui/cards/wallet_action_card.dart` - 第 1 卡片(充值/提现占位,SnackBar"功能开发中")
- `wuminapp/lib/wallet/ui/cards/wallet_identity_card.dart` - 第 2 卡片(翠绿渐变身份卡,钱包名可改 + 短地址复制 + QR 入口)
- `wuminapp/lib/wallet/ui/cards/wallet_qr_dialog.dart` - QR 弹窗(`WUMIN_QR_V1 kind=user_contact`)
- `wuminapp/lib/wallet/ui/cards/wallet_onchain_balance_card.dart` - 第 3 卡片(链上 `free+reserved` 总余额)
- `wuminapp/lib/trade/offchain/clearing_bank_settings_page.dart` - 设置清算行占位页

## 删除文件(按"不搞兼容"铁律直接删)

- `wuminapp/lib/trade/offchain/clearing_payment_entry_page.dart`
- `wuminapp/lib/trade/offchain/offchain_clearing_receive_page.dart`
- `wuminapp/lib/trade/offchain/clearing_bank_list_page.dart`

## 关键改动点

- `chain_rpc.dart` 新增 `fetchTotalBalance()` + `_decodeTotalBalanceFromAccountData()`,解码 `free`(offset 16)+ `reserved`(offset 32),最新块
- `wallet_page.dart` `WalletDetailPage.build` 重组为 3 卡片 + 交易记录;菜单 `clearing_bank_v2` → `clearing_bank`,跳 `ClearingBankSettingsPage`;删除 `_qrKey` / `_isSavingQr` / `_saveQrToGallery` / `_formatAddressTwoLines` / `_HollowQrPainter` / `_openClearingPaymentEntry` / 原余额卡片 / 原编辑态
- 钱包名持久化仍走 `_saveWalletName()`,以 `onNameChanged` 回调传给身份卡

## 遗留(非本任务)

- `wuminapp/lib/user/user.dart` 里同名的 `_HollowQrPainter` 和 `_saveQrToGallery` 属于通讯录 QR 展示,独立实现,不在本次改版范围
- 充值/提现的实际业务逻辑、提现按钮下"清算行余额"小字、清算行搜索/绑定的真实交互,等清算行功能需求细化后开独立任务卡



## 任务需求

重做 wuminapp 钱包详情页 `WalletDetailPage`([wuminapp/lib/wallet/ui/wallet_page.dart:628-1272](wuminapp/lib/wallet/ui/wallet_page.dart:628)),从上到下改为 3 张卡片 + 交易记录:

1. **第 1 卡片 · 充值/提现双按钮(纯 UI 占位)**
   - 左:充值按钮,图标 `Icons.arrow_circle_down_outlined` + 文字"充值",点击弹 SnackBar "功能开发中"
   - 右:提现按钮,图标 `Icons.arrow_circle_up_outlined` + 文字"提现",点击弹 SnackBar "功能开发中"
   - **本轮不显示清算行余额小字,不接任何充值/提现链路**,等清算行功能落地后另开任务卡
   - 备注:后续完善时,充值 = 钱包链上余额 → 该钱包绑定的清算行;提现 = 清算行 → 该钱包链上余额

2. **第 2 卡片 · 钱包身份卡(参照 wumin 冷钱包第 1 卡片样式)**
   - 参照:[wumin/lib/ui/wallet_detail_page.dart:277-342](wumin/lib/ui/wallet_detail_page.dart:277)
   - 布局:
     - 左:钱包图标(48×48 圆形,半透明白底)
     - 中上:钱包名称(点击可修改,复用现有 `_saveWalletName()` 逻辑)
     - 中下:钱包地址(缩短显示 + 长按/点击复制)
     - 右:QR 小图标(36×36),点击弹出大二维码
   - 大二维码内容 = `WUMIN_QR_V1 kind=user_contact { address, name }`(维持现状不改)
   - 三种扫码场景由扫码方自行处理,不生成多份 QR:
     - 通讯录扫 → 取 name+address
     - 扫码支付扫 → 取 address 当收款方,走链下支付
     - 地址栏扫 → 取 address 回填

3. **第 3 卡片 · 钱包真实链上余额**
   - RPC 查**最新块**(不传 block_hash,`state_getStorage` 默认取最新)
   - 字段 = `free + reserved`(真实总余额,和 polkadot.js apps 对齐)
   - 不用 finalized,区块是否最终化不影响钱包真实余额

4. **交易记录区块**
   - 保留现有实现([wallet_page.dart:1272 上下](wuminapp/lib/wallet/ui/wallet_page.dart:1272)),不改

5. **右上角 3 点菜单改造 · 清算行**
   - 菜单项 `clearing_bank_v2` → 改 key 为 `clearing_bank`、文案"清算行"
   - 点击打开新建的**设置清算行**页:顶部搜索框 + 空列表占位 + "暂无结果"提示,**不放假数据,不接 API**,等后续清算行需求细化
   - 按"不搞兼容"铁律,彻底删除:
     - `ClearingPaymentEntryPage` 整页及所有路由引用
     - `OffchainClearingReceivePage`(收款码页)—— 收款能力已由第 2 卡片二维码承担
     - `ClearingPaymentEntryPage` 里"绑定清算行"等子入口,按需搬到"设置清算行"页或删除

## 输入文档

- memory/00-vision/project-goal.md
- memory/01-architecture/repo-map.md
- memory/05-modules/wuminapp-vs-wumin.md
- memory/07-ai/chat-protocol.md
- memory/07-ai/agent-rules.md
- memory/04-decisions/ADR-006-扫码支付-step1-同行MVP.md
- memory/05-architecture/qr-protocol-spec.md
- 参照源文件:
  - wuminapp/lib/wallet/ui/wallet_page.dart(目标页)
  - wumin/lib/ui/wallet_detail_page.dart:277-342(第 1 卡片参照样式)
  - wuminapp/lib/wallet/core/wallet_manager.dart(WalletProfile 模型)
  - wuminapp/lib/qr/qr_router.dart(QR 路由)
  - wuminapp/lib/trade/offchain/clearing_payment_entry_page.dart(待删)
  - wuminapp/lib/trade/offchain/offchain_clearing_receive_page.dart(待删)

## 必须遵守

- 不可突破模块边界:**只改 wuminapp,不同步改 wumin 冷钱包**
- 不可擅自改 citizenchain / sfid-backend,仅消费其现有 RPC 和 API
- QR 协议维持现状(`WUMIN_QR_V1 kind=user_contact`),不新增 QR 类型
- 钱包名点击修改复用 `_saveWalletName()`,不重写存储
- 链上余额严格取 `free + reserved`,**不准只取 free**(踩过的坑,不得重复)
- "不搞兼容/保留/过渡"铁律:删除的页面和路由不得留注释兜底或灰化保留
- 中文注释到位,移除残留代码(原余额卡片、原 QR 卡片、原菜单回调、原清算行收款码页路由)
- 第 1 卡片两个按钮本轮**不接任何业务逻辑**,只弹 SnackBar

## 输出物

- **PR-1:第 2 卡片身份卡**(Mobile Agent)
  - 新建 Widget:`WalletIdentityCard`
  - 左图标、中名称(可改)、中地址(可复制)、右 QR 小图标→弹窗
  - 移除原居中大 QR 卡片
  - 钱包名点击修改复用现有逻辑

- **PR-2:第 1 卡片充值/提现占位**(Mobile Agent)
  - 新建 Widget:`WalletActionCard`
  - 两个 IconButton + 文字,点击弹 SnackBar "功能开发中"
  - 移除原"钱包余额"卡片
  - 图标:充值 `Icons.arrow_circle_down_outlined` / 提现 `Icons.arrow_circle_up_outlined`

- **PR-3:第 3 卡片链上余额**(Mobile Agent)
  - 新建 Widget:`WalletOnchainBalanceCard`
  - RPC `state.getStorage(System::Account, address)` 最新块
  - 解码 `AccountData`,返回 `free + reserved`
  - 失败态:显示"-- GMB"并附本地 `WalletProfile.balance` 作为兜底(可选)
  - 卡片下方衔接现有交易记录区块

- **PR-4:右上角菜单 + 设置清算行占位页**(Mobile Agent)
  - 菜单项改名"清算行",key 改 `clearing_bank`
  - 新建 `ClearingBankSettingsPage`:AppBar + 顶部 `TextField` 搜索框 + `ListView` 空列表 + 空态"暂无结果"
  - **删除** `ClearingPaymentEntryPage`、`OffchainClearingReceivePage` 及所有 import / 路由 / 菜单引用
  - 检查是否还有其他地方引用这两个页面,一并清理

- 中文注释到位
- Flutter 单元测试/widget 测试覆盖 3 张卡片的渲染
- 残留清理:旧余额卡片、旧 QR 卡片、旧菜单回调、旧路由、旧页面文件

## 验收标准

- 4 个 PR 全部合并,钱包详情页从上到下:充值/提现卡 → 身份卡 → 链上余额卡 → 交易记录
- 第 1 卡片两个按钮点击弹 SnackBar "功能开发中"
- 第 2 卡片钱包名可点击修改并持久化;右侧 QR 小图标点击弹大码;二维码内容仍是 `WUMIN_QR_V1 kind=user_contact`
- 第 3 卡片显示链上真实余额(`free + reserved`,最新块),和 polkadot.js apps 余额对齐
- 右上角菜单显示"清算行",点击打开设置清算行页(搜索框 + 空列表)
- 仓库内不再存在 `ClearingPaymentEntryPage`、`OffchainClearingReceivePage` 的任何引用(grep 零残留)
- `flutter analyze` 0 error / 0 warning
- `flutter test` 全部通过
- wumin 冷钱包代码**零改动**
- 人工跑一遍:创建钱包 → 进详情页 → 改名 → 弹二维码 → 点充值/提现 → 进清算行设置页,全部行为符合本任务卡

## Review 关注点

- 是否有残留的 `clearing_bank_v2` / `ClearingPaymentEntryPage` / `OffchainClearingReceivePage` 引用
- 链上余额是否真的 `free + reserved`,而不是只取 `free`
- 钱包名修改是否走原 `_saveWalletName()` 逻辑,没重写
- 二维码 payload 是否原样保留,未引入新 QR 类型
- 是否误改 wumin 冷钱包
