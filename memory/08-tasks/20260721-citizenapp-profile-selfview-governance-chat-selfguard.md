# CitizenApp 用户主页：私信 self 守卫 + 自看动作按钮治理（关注/通知/订阅之外）

任务需求：用户确认的技术方案落地。关注/通知/订阅三项大功能的需求设计已分流独立会话；本窗口只做
「私信完善」+「自看时四个动作按钮的显示治理」。
所属模块：citizenapp / 8964 profile + chat

## 方案 A · 私信 self 守卫（防线收口）

- `lib/chat/open_direct_chat.dart` `openDirectChat`：解析 `sender` 后，若 `peerAddress.trim() == sender`
  → 弹「不能和自己发起聊天」并 return，不进聊天页。覆盖所有私信入口（广场主页/通讯录）。

## 方案 B · 自看按钮治理（按账户身份，非 isSelf 标志）

三态：
- 真·自看（我的-背景图，`isSelf:true`）：隐藏四按钮（现状不变）。
- 预览他人视角看自己（广场头像，`isSelf:false` 且账户=自己）：**显示但置灰不可点**（用户取向）。
- 真·他人：正常可点（现状不变）。

落点：
- `user_profile_page.dart`：加可注入 seam `viewerAccountLoader`（默认取 `WalletManager().getDefaultWallet().address`，
  测试可注入）；`_resolveOwnAccount()`（isSelf 时跳过）算 `_isOwnAccount`；给 `ProfileActionIcons` 与
  `CreatorSubscribeButton` 传 `enabled: !_isOwnAccount`。
- `profile_action_icons.dart`：加 `enabled`；false 时各 `_CircleIcon.onTap=null` + 灰色（`textTertiary`）。保留 isSelf 隐藏。
- `creator_subscribe_button.dart`：加 `enabled`；false 时按钮 `onPressed=null`（Material 自动置灰）。

## 必须遵守 / 边界

- 不碰关注流聚合、发帖通知推送、创作者订阅门禁的内部实现（独立会话）；铃铛「订阅动态→通知」改名也归通知设计。
- `WalletManager().getDefaultWallet()` 已在 `_loadWalletName`（try/catch）用且测试通过，判定沿用 try/catch，失败按非本人。
- 判定异步：按钮先按 enabled 渲染，isOwnAccount 解析后再置灰（预览态极短暂闪一下，可接受）。

## 输出物 / 验收

- 代码 + 中文注释；`flutter analyze` 0 问题。
- 测试：预览自己（注入 viewerAccountLoader=owner）点私信不触发 onOpenDirectChat spy（禁用）；真他人点私信正常触发（回归）；`flutter analyze` 与 profile/chat 相关测试通过。
- openDirectChat self 守卫因需真 WalletManager 难单测，作为防线收口，靠治理禁用为主。

## 执行结果（2026-07-21）

- **私信 self 守卫**：`open_direct_chat.dart` 解析 sender 后加 `peerAddress==sender → 弹「不能和自己发起聊天」return`。
- **自看治理（置灰）**：
  - `user_profile_page.dart`：加注入 seam `viewerAccountLoader`（默认 `WalletManager().getDefaultWallet().address`）；`_resolveOwnAccount()`（isSelf 跳过、try/catch）算 `_isOwnAccount`；`ProfileActionIcons` 与 `CreatorSubscribeButton` 传 `enabled: !_isOwnAccount`。
  - `profile_action_icons.dart`：加 `enabled`；false → 各 `_CircleIcon.onTap=null`；`_CircleIcon` 按 `onTap==null` 转灰（`textTertiary` + border）。保留 isSelf 隐藏。
  - `creator_subscribe_button.dart`：加 `enabled`；`actionable = enabled && !_busy`，三个按钮 `onPressed` 据此，false → Material 自动置灰。
- **三态**：我的自看=隐藏（不变）；广场头像看自己=显示但置灰不可点；真他人=可点（不变）。
- **测试**：新增「他人视角看自己→私信置灰不触发 onOpenDirectChat spy」；原「他人私信打开聊天」不传 loader→无钱包→isOwnAccount=false→仍可点（回归）。`flutter analyze` 0 问题；profile 全量 + 广场 + 我的 54 测试全过。
- **边界**：未碰关注流/发帖通知/订阅门禁内部实现（独立会话）；铃铛「订阅动态→通知」改名归通知设计。
