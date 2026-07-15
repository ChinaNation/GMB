# 20260709 我的 tab 默认头像对齐用户主页 + 背景点击改为进主页

> 2026-07-15 资源规则已由 `20260715-citizenapp-profile-fallback-unification` 取代：“我的”页、唯一用户主页及其他入口统一使用 `assets/profile_defaults/` 的本地照片。以下记录已同步为当前实现口径。

## 需求
1. 我的 tab（`lib/my/user/user.dart`）头像**未设置时**显示和用户主页一致的本地默认照片
   （`assets/profile_defaults/`，由 `ProfilePresentation` 按账户稳定选取）。
2. 删除「点击头部背景 → 更换背景图」功能；改为「点击背景 → 进入用户主页」（与右侧箭头 `_openMyProfile` 一致）。

## 边界
- 只改 `lib/my/user/user.dart` 两处；不动用户主页 / 编辑资料 / 后端 / 背景图展示逻辑。
- 头像**已设置**时（本地 `avatarPath`）仍显示该图，只改未设置的兜底。
- 默认资料 seed = 默认钱包地址（= 用户主页 `ownerAccount`），统一调用 `ProfilePresentation`，保证同一账户跨入口一致。

## 改动
- `_SquareAvatar` 接收 `seed`；未设图时从 `profile_defaults` 选择照片；`_buildProfileCard` 传 `seed: _communicationAddress`。
- 背景 `GestureDetector.onTap`：`_pickBackgroundImage` → `_openMyProfile`。
- 删 `_pickBackgroundImage` + `_imagePicker` 字段 + `image_picker` import + 不再用的 `_inkGreen` 常量。

## 验证
- `flutter analyze` 干净；真机：未设头像时我的 tab 头像 = 用户主页默认头像（同一张）；点背景进主页、不再弹换背景。
