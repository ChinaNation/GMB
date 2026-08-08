# 任务卡：CID finalized 后全页面就地刷新与 SS58 展示收口

状态：进行中（2026-08-07）

## 任务需求

1. CID 注册只有在链上 finalized 身份闭环成立后才算成功。
2. 注册成功后，广场、聊天、我的、通讯录、创作者、会员和身份页必须在同一 App
   运行会话内就地更新，不允许继续显示“尚未注册”、再次弹注册窗口或要求重启。
3. `dropped`、`retracted`、`finalityTimeout` 和订阅中断不是确定失败；必须继续按
   finalized 链状态核验注册结果。
4. 用户主页、通讯录卡片和通讯录搜索暂时移除 SS58 地址展示与搜索入口；资料模型、
   内部 `account_id` / `ss58_address`、转账核验和用户码继续保留。
5. 用户主页与通讯录卡片的 `CID：xxxx` 统一改为 `公民号：xxxx`。
6. 完成后更新文档、完善中文注释、完善测试并清理旧逻辑、旧注释和旧 UI 文案残留。

## 所属模块

- CitizenApp：RPC 交易状态、CID 身份、常驻页面、用户主页、通讯录。
- Flutter 共享层同时覆盖 iOS 与 Android，不修改平台原生层。

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-required-reading.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/workflow.md`
- `memory/07-ai/definition-of-done.md`
- `memory/07-ai/pre-submit-checklist.md`
- `memory/07-ai/module-checklists/citizenapp.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`
- `memory/08-tasks/open/20260807-citizenapp-tx-confirm-resubscribe.md`

## 必须遵守

- finalized 区块高度本身不等于注册成功；必须核验 CID 与 AccountId 双向绑定闭环。
- 不把 `dropped` 等非确定状态包装成确定失败。
- 不修改 CitizenChain runtime、交易载荷、签名、nonce、QR_V1、Cloudflare 或 Isar schema。
- 不删除资料模型与业务内部的 `account_id` / `ss58_address`。
- 不在页面进入、切换 Tab 或注册完成时生成设备密钥或登记 P-256 设备子钥。
- 只在 `main` 分支和主检出工作，不创建分支或 worktree，不触碰 GitHub 远端。

## 预计修改目录

- `citizenapp/lib/rpc/`：修正交易池非确定状态与 finalized 结果收敛；代码、注释、残留清理。
- `citizenapp/lib/my/myid/`：统一注册成功后的缓存失效和身份变化广播；代码、注释、残留清理。
- `citizenapp/lib/my/user/`：通讯录身份刷新、SS58 展示/搜索移除及公民号文案；代码与 UI 残留清理。
- `citizenapp/lib/8964/`：广场身份刷新与用户主页展示收口；代码、注释、布局和文案清理。
- `citizenapp/lib/chat/`、`citizenapp/lib/my/creator/`、`citizenapp/lib/my/membership/`：复核并补齐身份广播后的就地刷新；代码与注释。
- `citizenapp/test/`：完善现有 RPC、注册、主页、通讯录和常驻页面测试；不新增测试文件。
- `memory/01-architecture/citizenapp/`、`memory/05-modules/citizenapp/`：更新 finalized 成功和页面刷新契约；文档与旧口径清理。

## 输出物

- finalized 注册结果收敛代码。
- 全页面身份就地刷新代码。
- 用户主页与通讯录展示调整。
- 关键逻辑中文注释。
- 单元测试、Widget 测试和回归测试。
- CitizenApp 技术文档更新与残留清理。

## 验收标准

- finalized CID↔AccountId 闭环成立后，所有已挂载页面无需重启立即更新。
- `dropped` 后交易实际 finalized 成功时最终显示注册成功，不显示确定失败。
- 明确 `invalid`、`usurped` 或 finalized Dispatch 失败仍正确拒绝。
- 用户主页、通讯录卡片和通讯录搜索不再展示或搜索 SS58；用户码与内部模型保持不变。
- 用户主页和通讯录卡片统一显示“公民号”。
- Flutter analyze、受影响测试和全量测试通过。
- Android 与 iOS 真实页面和真实 finalized 注册流程完成验收；无法在当前环境完成的外部步骤必须如实记录，任务卡不得虚报完成。
- 文档、中文注释和残留清理完成。

## 执行记录

- 2026-08-07：用户确认完整技术方案并明确授权执行；确认 SS58 仅从指定页面和搜索中
  暂时移除，资料模型与内部账户字段继续保留。
- 2026-08-07：实现交易池非确定状态与 CID finalized 目标绑定对账；注册闭环成立后由
  `MyIdService` 统一失效身份缓存并递增身份 revision。身份缓存增加版本/代际隔离，旧的
  “未注册”链读不能被新版本复用或回写当前缓存。
- 2026-08-07：我的、广场、聊天、身份页沿用或补齐 `account_id + cid_number` 比对；
  通讯录、创作者、会员补齐全局身份 revision 监听，通讯录和创作者增加加载代际保护。
- 2026-08-07：用户主页与通讯录卡片移除 SS58 展示，统一显示“公民号”；通讯录搜索
  删除 `account_id / ss58_address` 匹配，内部模型、转账和用户码未删除。
- 2026-08-07：`flutter analyze` 通过（0 问题）；CitizenApp 全量 `flutter test` 第二轮
  通过（1129 项通过、48 项按宿主原生库条件跳过、0 失败）。定向测试覆盖 `dropped`
  非确定语义、finalized 后续块目标命中、缓存失效竞态、服务层一次失效/一次广播、
  同账户注册后通讯录原地退出引导、主页/通讯录 SS58 零展示及搜索零匹配。
- 2026-08-07：Pixel 8a Android 16 完成 debug 构建、安装和进程启动；iPhone iOS 27.0
  完成 Xcode debug 构建、安装，设备进程列表确认 Runner 运行。iOS 调试 VM Service
  60 秒内未发现，因此未把调试连接或页面交互记为通过。

## 待真实交互验收

- 真实“未注册 → 提交 CID → 链上 finalized → 各已挂载页面原地更新”需要未注册热钱包、
  设备生物识别和链上费用，会产生真实链交易。本次未擅自创建钱包、签名或发送交易；因此
  任务保持“进行中”，待用户在设备上授权该真实交易并逐页确认后才能改为完成。
