# 任务卡：CitizenApp / CitizenWallet 系统弹窗中文化(甲档)

状态：进行中(2026-08-06)

## 任务需求

用户逐字要求:**两端(公民 / 公民钱包)的 iOS 与 Android 全部统一为默认中文;
用户手机是英文的则显示英文,其它语言一律回落中文。**

触发场景:iPhone 上打开公民 App 弹出 `"公民" would like to send you notifications`。

范围为**甲档**(系统弹窗层),乙档(App 界面 4300+ 条文案全量 i18n)另立卡
`20260806-citizenapp-citizenwallet-full-i18n.md`,本次不做。

## 诊断结论(2026-08-06 实测)

英文弹窗的根因:**两端 iOS 只声明了英文本地化**。

```text
CFBundleDevelopmentRegion = $(DEVELOPMENT_LANGUAGE) → pbxproj developmentRegion = en
knownRegions = en, Base                （无 zh-Hans）
.lproj = 只有 Base.lproj               （无 zh-Hans.lproj / en.lproj）
CFBundleLocalizations = 未设置
```

装机产物实测 `CFBundleDevelopmentRegion = en`。iOS 据此认定 App 只支持英文,
于是**所有系统生成的弹窗模板句式走英文**(`would like to send you notifications`、
`Allow` / `Don't Allow` 等)。其中「公民」是 `CFBundleDisplayName`,本来就是中文。

三类文字必须分清:

| 类别 | 改前 | 处理 |
|---|---|---|
| 系统模板句式(通知/相机/相册授权框框架文字) | 英文 | 声明 zh-Hans 本地化后由系统自动中文 |
| 权限用途说明 `NS*UsageDescription` | **本来就是中文** | 搬进 `.lproj/InfoPlist.strings` 并补英文对照 |
| 生物识别对话框 | `localizedReason` 中文;Android 侧标题/按钮为插件英文默认串 | 补 `AndroidAuthMessages` / `IOSAuthMessages` |

**平台事实(必须登记)**:Android 运行时权限弹窗(相机/通知等)的正文由系统渲染、
跟随手机系统语言,App 无权干预 —— 这部分本来就跟随手机语言,不需要也无法改。
App 能控的是弹窗里显示的 App 名、以及生物识别对话框文案。

## 实施范围

iOS(两端各一份):
- `developmentRegion`: `en` → `zh-Hans`(这一条就是「其它语言回落中文」的开关)
- `knownRegions` 补 `zh-Hans`;`Info.plist` 显式设 `CFBundleLocalizations = [zh-Hans, en]`
- 新建 `zh-Hans.lproj/InfoPlist.strings`(中文)与 `en.lproj/InfoPlist.strings`(英文)

Android(两端各一份):
- `values/strings.xml` 保持中文 = 默认(Android 天然回落无后缀目录 ⇒ 非英文语言全中文)
- 新增 `values-en/strings.xml` 英文对照

生物识别(两端共 5 个调用点):
- `citizenapp`: `main.dart`、`my/user/user.dart`
- `citizenwallet`: `main.dart`、`ui/settings_page.dart`、`wallet/wallet_manager.dart`
- 按当前 locale 选中/英文,补 `AndroidAuthMessages` / `IOSAuthMessages`

## 验收

- [x] **iOS 装机产物实测**(公民):`Runner.app` 内同时存在 `zh-Hans.lproj` / `en.lproj`;
      `CFBundleDevelopmentRegion = zh-Hans`;`CFBundleLocalizations = ["zh-Hans","en"]`;
      两套 `InfoPlist.strings` 解出中/英各自内容正确
- [x] **Android APK 实测**(公民钱包):`aapt2 dump resources` 显示
      `string/app_name` 同时有 `()` = 「公民钱包」与 `(en)` = 「Citizen Wallet」
- [x] 生物识别文案单源落地,5 个调用点全部改造完毕
- [x] 测试:两端各 5 例全绿(中文→中文、en/en_US/en_GB/en_AU→英文、
      ja/fr/de/ko/es→**全部回落中文**、messages 双平台覆盖、
      冷端钉死不得出现 iOS 回退按钮)
- [x] analyze 两端零问题
- [x] 键集一致性自检:两端 zh/en `.lproj` 键集完全相同;
      `Info.plist` 的 `NS*UsageDescription` 无漏译
- [x] 文档更新(`WALLET_TECHNICAL.md`「系统弹窗本地化」)、注释完善、残留清理
      (`android:label` 已无写死中文)
- [ ] **真机人工验收**:中文手机看中文弹窗;系统语言临时切英文看英文弹窗(两端两 App)

## 实施记录(与原计划的偏差)

1. **`.lproj` 必须登记进 Xcode 工程** —— pbxproj 不做通配。已补
   `PBXFileReference` ×2 + `PBXVariantGroup` + `PBXBuildFile` + Resources 构建阶段引用,
   `plutil -lint` 通过。这是"文件写了但弹窗还是英文"的唯一隐性坑。
2. **`local_auth` 不再导出消息类**,需把 `local_auth_android` / `local_auth_darwin`
   提为直接依赖(版本跟随传递依赖,不升级)。`AuthMessages` 基类由
   `local_auth_android` 一并导出,无需再引 `local_auth_platform_interface`。
3. **两端 local_auth 版本不同**:热端 2.3.0(Android 10 个字段)、冷端 3.0.2
   (Android 仅 3 个字段),各自适配,不强行对齐版本。
4. **测试逼出可测性缺陷**:生产直读 `PlatformDispatcher.instance` 时
   `TestPlatformDispatcher.localeTestValue` 覆写的是测试包装器、影响不到真单例 ——
   这段逻辑原本完全不可测。已加 `debugLocale` 注入接缝(照仓库 `debugXxx` 惯例)。
5. **`aps-environment` entitlement 仍缺**(装机时暂摘):通知授权弹窗即使允许也拿不到
   token。与本卡的中文化是两件事,待用户决定是否恢复。
