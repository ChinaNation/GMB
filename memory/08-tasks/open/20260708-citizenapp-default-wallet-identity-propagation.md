# CitizenApp 默认钱包切换全端身份传播

## 任务需求

- 钱包账户地址 = 用户唯一主键；钱包名 = 可变昵称。在「我的钱包」切换默认用户（拖拽置顶另一个热钱包）= 切换整个人：聊天、广场（发动态/点赞/评论）、会员、我的主页必须全部立即以新地址为准（用户 2026-07-08 明确拍板）。
- 病根（已诊断确证）：切换只写 Isar `sortOrder`，全 App 零广播；三个常驻页只在 initState 读一次默认钱包——「我的」tab（user.dart:66-73，且钱包入口 323-327 返回不回刷）、广场首页（square_home_page.dart:51 缓存身份 Future）、IM 会话列表（im_tab_page.dart:73-80 仅 App resume 重取）。形成「UI 显示旧身份、动作以新身份执行」脑裂。

## 所属模块

- citizenapp（Mobile Agent）；链端、Worker 零改动。

## 技术方案

1. `wallet/core/wallet_manager.dart`：新增 `static final ValueNotifier<int> walletsRevision`（钱包身份数据版本：列表增删/顺序/名称；余额刷新不计入，避免高频抖动）。在 `reorderWallets`、`createWallet`、`importWallet`、`importColdWallet`、`deleteWallet`、`clearWallet`、`updateWalletDisplay` 成功后自增。
2. `my/user/user.dart` ProfilePage：initState 挂 `walletsRevision` 监听 → `_loadState()`；dispose 移除。昵称/地址/认证勾/我的主页入参随之实时对齐。
3. `8964/pages/square_home_page.dart`：同样挂监听 → 重建 `_identityFuture`（作者点击 isSelf 判定、发布身份卡即时对齐）。
4. `im/im_tab_page.dart`：同样挂监听 → `_reload(syncFirst: true)`（会话列表 owner 即时切换）。
5. 会员页/我的主页/发布页均为 push 现取（SquareSessionProvider.ensureSession 每次实时解析默认钱包），无需改动。

## 必须遵守

- 不可突破模块边界；不做双保险残枝（统一走 revision 监听单机制）
- 切默认钱包需授权的既有逻辑（wallet_page._onReorder + verifyWalletAccess）不动

## 输出物

- 代码 + 中文注释
- 测试：wallet_manager reorder 自增 revision；square_home_page 收到 revision 变更后身份重载
- 文档回写本卡

## 验收标准

- `dart analyze` 干净；`flutter test test/wallet test/8964 test/im` 全绿
- 行为：拖拽切默认 → 返回「我的」tab 昵称/地址已变，我的主页打开即新用户；广场 isSelf 正确；IM 列表 owner 切换
- Review 问题已处理

## 完成记录（2026-07-08）

- 机制落地：`WalletManager.walletsRevision`（static ValueNotifier<int>）在 reorder/create/import/importCold/delete/clear/updateDisplay 后自增；delete/clear 的 bump 放在 Isar 事务提交后、安全存储清理前（清理可抛错，不能吞掉广播）。
- 接线四处常驻页：我的 tab（user.dart）、广场首页（square_home_page.dart）、IM 列表（im_tab_page.dart）、交易页 OnchainPaymentPanel（onchain_payment_page.dart，评审发现的第四个僵死点，纯本地重读）。
- 对抗式评审（3 lens × 独立验证，10 agent）确认 6 项全部修复：
  1. [HIGH] user.dart `_loadState` stale-wins 竞态 → `_loadGeneration` 世代守卫；
  2. [HIGH/MED] im_tab_page `_reload` 并发覆写 + realtime 误杀 → `_reloadGeneration` 世代守卫（含 _loading/finally 踩踏）；
  3. [MED] delete/clear bump 被存储异常跳过 → bump 前移；
  4. [MED] 交易页未接广播（删钱包后付款方停留死钱包）→ 接线；
  5. [MED] 广播粒度过粗（重命名冷钱包触发 2 次链查询 + IM 全量同步 + 整页 spinner）→ 三页监听器先做纯 Isar 廉价比对（地址+昵称未变即跳过）；
  6. 被驳回 1 项（ChainReadCache in-flight 合并使该场景不可达）。
- 测试：新增 reorder/rename 自增 revision 两用例 + 广场身份切换重载用例；顺手修一处既有过时断言（发布按钮已被其他会话在途改动改名「签名发布」）。`flutter test test/wallet test/8964 test/im test/transaction` 190 过 4 skip；`dart analyze` 我方文件零问题。
- 待用户真机验收：拖拽切默认 → 我的/广场/IM/交易四页即时跟随；会员页进入即新身份。
- 未提交 git。推特式主页布局改造（资料区移出头图等）是另一决策，未包含在本卡。
