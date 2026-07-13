# 官网:标签页图标/标题 + 产品页下载按钮

任务需求：
1. 浏览器标签页:favicon 从紫色闪电改为官网国旗图;标题从「公民区块链 | 中华民族联邦共和国公民储备委员会」改为「公民链｜中华联邦公民储备委员会」。
2. 产品页三卡右上角加醒目「下载」按钮(金色加大):公民/公民钱包卡弹 iOS(去 App Store 提示)+ Android(直下 GitHub 最新 APK);公民链卡弹 macOS/Windows/Linux-arm/Linux-amd(直下 GitHub 最新安装包)。

所属模块：citizenweb（官网前端）。节点/链/Worker 零改。

必须遵守：
- 下载走 GitHub releases/latest/download/<固定资产名>（固定名契约见输出物）
- iOS 纯提示文案（无 App Store 直链）
- 标题照抄用户字面（全角 ｜）
- 复用现有 gold/navy Tailwind token 与 GlowCard 结构

输出物：
- index.html:title + favicon(png) 改;删 public/favicon.svg;新增 public/favicon.png(sips -Z 128 由 flag-emblem.png 生成)
- components/DownloadButton.tsx(下拉 + 点击外部关闭 + 固定名直下 + iOS alert)
- pages/Ecosystem.tsx:三卡加 downloads 配置 + 卡头右上放按钮
- 文档更新(CITIZENWEB_TECHNICAL 3.4/3.5)

固定资产名契约（发版须挂同名到最新 release,否则直下 404）：
- citizenchain-macos-arm64.dmg / citizenchain-windows-x64.msi / citizenchain-linux-arm64.deb / citizenchain-linux-amd64.deb
- citizenapp-android.apk / citizenwallet-android.apk

验收标准：
- tsc -b + vite build 通过、eslint 通过
- 标签页标题/图标已换;三卡下载按钮金色加大、下拉选项与直下链接正确;iOS 弹提示

## 进度

- [x] favicon.png 生成 + index.html title/icon + 删闪电 svg
- [x] DownloadButton.tsx + Ecosystem 三卡接入
- [x] build/lint 通过;浏览器 DOM 验证(标题/favicon href/3 按钮金色 20px 粗体/各卡下拉选项与固定名链接/iOS 为按钮)
- [x] 文档更新(CITIZENWEB_TECHNICAL 3.4/3.5 + 修部署清单过时 favicon.svg)
- [ ] 【发布对齐】发版按固定资产名挂到 GitHub 最新 release(当前缺 Android/Linux-arm/钱包资产、deb/msi 带版本号需改名)
- [ ] 【部署】与公民宪法 tab 一起 wrangler/官网构建部署
