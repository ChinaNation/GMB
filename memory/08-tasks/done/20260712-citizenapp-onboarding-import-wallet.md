# 任务卡：首启补齐导入钱包(抽取独立文件 + onboarding 次入口,二元 fail-closed)

状态：已完成(2026-07-12)

任务需求：
首启 CreateWalletOnboardingPage 原"无导入",新设备恢复钱包只能先建新钱包再进 app 导入。补齐:
首启除创建外提供"导入已有钱包"入口,复用 ImportWalletPage;导入二元成败——成功=导入+子钥注册
都成功才进 app,失败=弹窗提示并留在导入页保留助记词可重试。

所属模块：citizenapp / wallet

改动(纯客户端,worker 零改;importWallet 逻辑不动,已 fail-closed)：
1. 新增 lib/wallet/pages/import_wallet_page.dart：ImportWalletPage 从 wallet_page.dart 抽出(独立文件,
   带二元 fail-closed 文档);imports=dart:async/material/bip39_input/app_theme/wallet_manager/
   create_wallet_flow/chain_tx_monitor。
2. wallet_page.dart：删 ImportWalletPage 类、加 import 新文件、删孤儿 import bip39_input(仅它用);
   现有引用点(:405 app 内导入菜单)靠新 import 续用。
3. create_wallet_onboarding_page.dart：加"已有钱包？导入助记词"次按钮(同受设备锁屏门禁 canCreate),
   _openImport push ImportWalletPage → 返回 true 即 onCreated 放行;标题改"设置你的公民钱包";
   更新"无导入"文档注释;onCreated 文档改"创建或导入成功"。
4. wallet_gate_test.dart："无导入入口"用例改为"含创建与导入入口";补"点导入进 ImportWalletPage"用例;
   加 import_wallet_page import。

验收结果：
- dart analyze 改动文件(含测试)：No issues found(wallet_page 无孤儿 import 残留);dart format：0 changed。
- flutter test test/wallet/ 全量 +98 全过(含新增两条),无回归。
- 文档:回填 20260712-fail-closed 卡"待产品定"项;20260707-wallet-gate 卡"不提供导入"口径加反转注记;
  四层门禁记忆补记首启双入口。

设计取舍(已定)：只热钱包助记词导入(冷钱包过不了 WalletGate 不加);onCreated 回调名保留;
整页复用不内联;PopScope 仍禁退出门禁,导入页自带 AppBar 可返回 onboarding。
