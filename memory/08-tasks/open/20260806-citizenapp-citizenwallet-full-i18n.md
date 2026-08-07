# 任务卡：CitizenApp / CitizenWallet 界面文案全量国际化(乙档,后期实现)

状态：待排期(2026-08-06 立卡;用户明确「后期再实现」,本卡不阻塞任何在办任务)

## 背景

2026-08-06 用户要求两端系统弹窗统一「默认中文、英文手机显示英文、其它语言回落中文」。
当时把范围拆成两档,**甲档(系统弹窗 + App 名 + 权限说明 + 生物识别对话框)已完成**
(见 `20260806-citizenapp-citizenwallet-system-dialog-i18n.md`);本卡是**乙档**:
App 自身界面文案的全量国际化。

## 任务需求

英文手机上**整个 App 界面**显示英文,其它语言一律回落中文;中文仍是默认语言。

## 现状(2026-08-06 实测)

- 两端**零 i18n 基建**:`flutter_localizations` / `intl` / ARB / `l10n.yaml` /
  `supportedLocales` / `localizationsDelegates` **全部不存在**。
- Dart 界面中文字面量规模:
  - `citizenapp`：约 **3563 条**,分布在 **344** 个 `.dart` 文件
  - `citizenwallet`：约 **753 条**,分布在 **45** 个 `.dart` 文件
- 甲档已建立的本地化骨架可直接复用:iOS `developmentRegion=zh-Hans` +
  `zh-Hans.lproj`/`en.lproj`,Android `values/`(中文默认)+ `values-en/`。

## 实施要点(排期时再细化)

1. 建 Flutter i18n 骨架:`flutter_localizations` + `intl` + `l10n.yaml` + ARB
   (`app_zh.arb` 为真源,`app_en.arb` 为英文);`MaterialApp` 配
   `localizationsDelegates` / `supportedLocales: [zh-Hans, en]`;
   `localeResolutionCallback` 兜底:未命中语言一律回落 `zh-Hans`。
2. 逐模块抽取字面量到 ARB(按 `lib/` 模块分批,避免一次性巨型 diff),
   同步替换调用点为 `AppLocalizations.of(context)!.xxx`。
3. **英文措辞必须人工校对**,禁止机翻直接入库(金融/治理/链上术语一旦译错会误导用户)。
   术语表须与白皮书、宪法英文版一致(`docs/` 下已有英文资产可对齐)。
4. 非 UI 文案不纳入:日志、异常内部串、`AppLog` 诊断文本保持中文(不面向终端用户)。
5. 测试:每批抽取后跑对应 widget 测试;补一组 locale 切换测试
   (中文 locale → 中文、英文 locale → 英文、法文 locale → 中文回落)。

## 风险与边界

- 巨型改动,必须分批;单批控制在可 review 的规模。
- 与「iOS/Android 两端必须一致」铁律相容:Dart 共享代码改动天然双端生效。
- 排期前不得部分开工(半本地化界面比全中文更糟:同屏中英混排)。

## 关联

- 甲档任务卡：`20260806-citizenapp-citizenwallet-system-dialog-i18n.md`
- 铁律：`memory/07-ai/agent-rules.md`「iOS 与 Android 两端必须同步一致」
