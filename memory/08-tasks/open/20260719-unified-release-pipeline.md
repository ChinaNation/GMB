# 任务卡:统一正式发布流水线(控制台 Release=取最新成功 CI 签名产物 → 滚动 Release → 官网直下)

> 状态：open / paused（2026-08-02；流水线代码已落地，CitizenApp、CitizenWallet 和
> CitizenChain 正式 Releases 按用户要求暂停；CitizenWeb 已独立完成生产部署。）

## ⚠️ 设计修订(2026-07-19 二次拍板,推翻 promote 模型)
用户后续确认:**Release 重新编译**(非 promote 搬运)、**版本方案**、runtime 保持 spec_version。新方案覆盖下方旧"方案 A promote"描述。
- **版本(源码为准,起步 v1.0.0)**:运行 CI → 补丁 +0.0.1;正式 Release → 次版本 +0.1(**补丁归零**)。App/钱包存 `pubspec.yaml` `version: X.Y.Z+CODE`(CODE 每次 +1,供 Android 更新检测);链存 `tauri.conf.json`+`Cargo.toml`;runtime 不套,仍用链上 spec_version。
- **运行 CI**:控制台 bump 补丁 → 提交(软件名)+推送 → dispatch CI(**校验构建**,不发布)。
- **正式 Release**:控制台 bump 次版本 → 提交+推送 → dispatch CI **release 模式**(**重编译+签名+发布到滚动 Release**)。
- **滚动 Release**:唯一固定 tag `release`,始终 Latest,各产品 `gh release upload --clobber` 只覆盖自己的资产;官网/应用内更新走 `releases/latest/download/<资产名>`。
- 提交名:CitizenApp / CitizenWallet / CitizenChain / CitizenChain WASM(软件名,不用 chore()。
- **as-built**：`common.sh` 已实现 `bump_pubspec_version` / `bump_chain_version`；三个产品
  动作的 `ci` 与 `release` 均先更新源码版本、提交并推送，再 dispatch 对应 workflow。
  `ci` 模式只做验证构建，`release` 模式在 GitHub 重新编译、签名并发布到固定 `release` tag。
- **待执行**：三个正式 Release 的真实生产验收；macOS/Windows/iOS 平台签名材料和流程按
  用户后续授权补齐。暂停期间不得自行触发 Release workflow。

## 当前唯一设计

1. 正式 Release 必须通过 GitHub workflow 重新编译和签名，不从本地包发布，也不把普通
   CI artifact 直接提升为正式版。
2. 一个仓库维护一个固定 `release` tag；各产品 workflow 只覆盖自己的资产，官网和应用内
   更新统一使用 `releases/latest/download/<资产名>`。
3. iOS 走 App Store，不把 `.ipa` 作为官网可下载资产；Apple 签名材料后续单独配置。
4. runtime 不套用桌面端语义化版本；runtime 继续使用链上 `spec_version` 和独立 WASM 流程。

## 统一资产名(可下载 = 6 个;iOS 走商店不下载)
- 公民链AMD.deb、公民链ARM.deb、公民链.exe、公民链.dmg(macOS 单一)、公民.apk、公民钱包.apk。
- iOS:公民(App Store)、公民钱包(App Store)——不进 Release。
- 更新清单:citizenapp-android-update.json(App 红点)、citizenchain-latest.json(链 updater),同挂滚动 Release。

## 签名现状 + 需补
- Android(公民.apk/公民钱包.apk):✅ 已有 `GMB_APP_KEY`(两端共用)。
- 链 updater:✅ 已有 `GMB_TOP_KEY`/`GMB_TOP_PUBKEY`。
- macOS(.dmg):❌ 缺 Developer ID 签名 + 公证 → 补 `GMB_MAC_KEY` + `GMB_MAC_NOTARY`。
- Windows(.exe):❌ 缺 Authenticode → 补 `GMB_WIN_KEY`。
- Linux(.deb):无需系统签名。
- iOS:App Store 签名材料后配(暂不加配置行)。

## 落地进度(2026-07-19,代码完成,待提交推送 + 验证性 CI)
- ✅ 阶段1 控制台配置列表(**密钥种类全齐,值后配**;需重启控制台生效):
  - CitizenApp/CitizenWallet:`GMB_APP_KEY`(安卓,已有值)+ `GMB_IOS_KEY`(iOS App Store 签名,新增,值后配)。
  - CitizenChain:`GMB_SSH_KEY`/`GMB_TOP_KEY`/`GMB_TOP_PUBKEY`(已有)+ `GMB_MAC_KEY`/`GMB_MAC_NOTARY`(macOS 签名+公证)+ `GMB_WIN_KEY`(Windows 签名)。
- ✅ 阶段2 命名统一:链 4 产物→公民链AMD.deb/公民链ARM.deb/公民链.exe/公民链.dmg;App/钱包已是 公民.apk/公民钱包.apk。
- ✅ 阶段2 CI 每次产签名正式包:三 workflow GMB_RELEASE_MODE 恒 true(删 debug 分支),Android 用 GMB_APP_KEY、链 updater 用 GMB_TOP_KEY 签名并 upload-artifact(retention 30)。macOS/Windows OS 级签名待 certs(护栏后加)。
- ✅ 阶段2/3 去 CI 内建 Release:链 publish job 改为「生成 citizenchain-latest.json」清单 artifact(URL 指向 releases/latest/download);App 删 gh release 步骤;钱包本无。
- ✅ 阶段3 滚动 Release：三个 workflow 的 `release` 模式重新编译并发布到固定 `release` tag；
  控制台按钮负责版本更新、提交、推送和 dispatch，不在本机上传正式包。
- ✅ 阶段4 官网:citizenweb Ecosystem.tsx 6 资产名→中文正式名;DownloadButton 加 encodeURIComponent;iOS 保持 store。
- ⏸ 待：CitizenApp、CitizenWallet、CitizenChain 三个正式 Releases 的生产触发与验收；
  macOS/Windows 系统签名和 iOS App Store CI。CitizenWeb 已经由 CitizenConsole 完成生产部署。
- ⚠️ 生产 CI 无法本地实跑验证;首次推送即为验证运行。

## 分阶段
1. **控制台配置列表补签名项**(本阶段·本地安全):CitizenChain 模块 `secrets.github` 加 `GMB_MAC_KEY`/`GMB_MAC_NOTARY`/`GMB_WIN_KEY` + secretComments 说明。改 server.mjs 需**重启控制台**。(iOS 行待 App Store CI 流程设计时再加。)
2. **CI 产签名正式包 + 统一命名**(生产 CI,待确认):三 workflow 的构建改为每次产出**签名正式包**并上传 run artifact(retention);产物名统一到上面 6 名;链补 macOS 公证 + Windows 签名步骤(用新 secret)。
3. **滚动 Release 发布流 + 控制台 Release 按钮**：控制台更新版本并 dispatch `release`
   workflow；GitHub 重新编译、签名并用 `gh release upload <固定tag> --clobber` 更新本产品资产。
4. **官网对齐**:citizenweb 的资产名改成上面 6 个中文正式名;iOS 保持 store 跳转;确认 `releases/latest/download` 命中滚动 Release。
5. **iOS App Store(后)**:补 iOS 构建 + App Store Connect 上传 + 证书配置行。

## 验收
控制台点某产品「正式 Release」→ 更新该产品源码版本并提交、推送 → dispatch 该产品
`release` workflow → GitHub 重新编译和签名 → 覆盖唯一滚动 Release 中本产品资产 → 官网
`releases/latest/download/<名>` 和应用内更新命中同一版本；三产品资产互不覆盖。
