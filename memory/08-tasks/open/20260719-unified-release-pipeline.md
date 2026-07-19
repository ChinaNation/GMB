# 任务卡:统一正式发布流水线(控制台 Release=取最新成功 CI 签名产物 → 滚动 Release → 官网直下)

> 状态:设计已确认(用户拍板 3 点),分阶段落地。先做控制台侧(用户:先把公民控制台功能完善)。iOS 走 App Store、证书值后配。

## ⚠️ 设计修订(2026-07-19 二次拍板,推翻 promote 模型)
用户后续确认:**Release 重新编译**(非 promote 搬运)、**版本方案**、runtime 保持 spec_version。新方案覆盖下方旧"方案 A promote"描述。
- **版本(源码为准,起步 v1.0.0)**:运行 CI → 补丁 +0.0.1;正式 Release → 次版本 +0.1(**补丁归零**)。App/钱包存 `pubspec.yaml` `version: X.Y.Z+CODE`(CODE 每次 +1,供 Android 更新检测);链存 `tauri.conf.json`+`Cargo.toml`;runtime 不套,仍用链上 spec_version。
- **运行 CI**:控制台 bump 补丁 → 提交(软件名)+推送 → dispatch CI(**校验构建**,不发布)。
- **正式 Release**:控制台 bump 次版本 → 提交+推送 → dispatch CI **release 模式**(**重编译+签名+发布到滚动 Release**)。
- **滚动 Release**:唯一固定 tag `release`,始终 Latest,各产品 `gh release upload --clobber` 只覆盖自己的资产;官网/应用内更新走 `releases/latest/download/<资产名>`。
- 提交名:CitizenApp / CitizenWallet / CitizenChain / CitizenChain WASM(软件名,不用 chore()。
- **as-built**:common.sh 加 `bump_pubspec_version`/`bump_chain_version`(已验证 patch/minor 正确);三 `*.sh` ci+release 都 bump+commit+dispatch;三 workflow 恢复 ci=校验/release=签名+发布并改 tag→滚动`release`(--latest --clobber),链版本改读源码(去 run_number),钱包补发布步骤+contents:write。mac/win OS 签名仍待 certs(release 构建 Android/updater 已签)。
- **待**:提交推送(3 workflow + 5 控制台 + 2 官网 + 任务卡)后首跑验证;重启控制台(配置行);certs 到手补 mac/win/iOS 签名。

## 确认的设计(用户拍板)
1. Release = **取 GitHub 上最新且 CI 成功的签名产物**发布(不重编译);签名材料写入公民控制台对应模块的配置列表,逐个配置。
2. 一个仓库三产品(ChinaNation/GMB)——**方案 A**:维护**一个"滚动正式版" Release(固定 tag,始终 Latest)**,每个产品 Release 按钮只 `gh release upload --clobber` 覆盖自己的资产;官网/updater 走 `releases/latest/download/<名>`。
3. **iOS 走 App Store**(`.ipa` 不作可下载资产,官网 iOS=商店跳转),Apple 证书**后面再配置**。
4. 正式包必须签名;Release 用的是 CI 最新成功版本的产物。

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
- ✅ 阶段3 滚动 Release + 控制台按钮改写:common.sh 加 promote_latest_ci_release(取最新成功 run→gh run download→gh release upload release --clobber,固定 tag `release` 始终 Latest);citizenapp/citizenwallet/citizenchain.sh 的 release 分支改为 promote(不再 dispatch 重编译)。
- ✅ 阶段4 官网:citizenweb Ecosystem.tsx 6 资产名→中文正式名;DownloadButton 加 encodeURIComponent;iOS 保持 store。
- ⏳ 待:①提交推送(触发 3 个验证性 CI run)②重启控制台(配置行)③certs 到手后补 mac/win OS 签名步骤 + iOS App Store CI(阶段5)④citizenweb 经控制台部署。
- ⚠️ 生产 CI 无法本地实跑验证;首次推送即为验证运行。

## 分阶段
1. **控制台配置列表补签名项**(本阶段·本地安全):CitizenChain 模块 `secrets.github` 加 `GMB_MAC_KEY`/`GMB_MAC_NOTARY`/`GMB_WIN_KEY` + secretComments 说明。改 server.mjs 需**重启控制台**。(iOS 行待 App Store CI 流程设计时再加。)
2. **CI 产签名正式包 + 统一命名**(生产 CI,待确认):三 workflow 的构建改为每次产出**签名正式包**并上传 run artifact(retention);产物名统一到上面 6 名;链补 macOS 公证 + Windows 签名步骤(用新 secret)。
3. **滚动 Release 发布流 + 控制台 Release 按钮改写**:新 release 逻辑=找该产品**最新成功 run** → 下载其签名产物 → `gh release upload <固定tag> --clobber` 覆盖到滚动 Release(+ 更新清单);替换现在的"dispatch CI 重编译"。三个 `actions/*.sh` release 分支改写。
4. **官网对齐**:citizenweb 的资产名改成上面 6 个中文正式名;iOS 保持 store 跳转;确认 `releases/latest/download` 命中滚动 Release。
5. **iOS App Store(后)**:补 iOS 构建 + App Store Connect 上传 + 证书配置行。

## 验收
控制台点某产品「正式 Release」→ 取其最新成功 CI 的签名产物 → 覆盖到唯一滚动 Release(Latest)→ 官网 `releases/latest/download/<名>` 直下最新正式版;App/链设置页红点更新命中同一 Release;三产品同仓库互不覆盖。
