# 20260709 我的 tab 默认头像对齐用户主页 + 背景点击改为进主页

## 需求
1. 我的 tab（`lib/my/user/user.dart`）头像**未设置时**显示和用户主页一致的默认头像
   （`assets/avatars/default_N.svg`，N=账号 codeUnits 求和 %6+1，同账号稳定同图）；
   当前是 `Icon(Icons.person)` 小人图标。
2. 删除「点击头部背景 → 更换背景图」功能；改为「点击背景 → 进入用户主页」（与右侧箭头 `_openMyProfile` 一致）。

## 边界
- 只改 `lib/my/user/user.dart` 两处；不动用户主页 / 编辑资料 / 后端 / 背景图展示逻辑。
- 头像**已设置**时（本地 `avatarPath`）仍显示该图，只改未设置的兜底。
- 默认头像 seed = 默认钱包地址（= 用户主页 `ownerAccount`，保证同一账号两处默认头像一致），复用 ProfileHeaderCard._DefaultAvatar 的哈希算法。

## 改动
- `_SquareAvatar` 加 `seed`；未设图兜底 `Icon(person)` → `_DefaultAvatar`（SVG）；`_buildProfileCard` 传 `seed: _communicationAddress`。
- 背景 `GestureDetector.onTap`：`_pickBackgroundImage` → `_openMyProfile`。
- 删 `_pickBackgroundImage` + `_imagePicker` 字段 + `image_picker` import + 不再用的 `_inkGreen` 常量。

## 验证
- `flutter analyze` 干净；真机：未设头像时我的 tab 头像 = 用户主页默认头像（同一张）；点背景进主页、不再弹换背景。
