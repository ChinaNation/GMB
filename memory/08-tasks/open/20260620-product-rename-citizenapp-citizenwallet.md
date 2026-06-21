# 产品英文名彻底统一：CitizenApp / CitizenWallet

- 状态：进行中（2026-06-20 立卡）
- 类型：跨产品破坏式改名（无兼容、零残留、开发期硬切）
- 主入口：当前主聊天（任务调度器）
- 涉及 Agent：Blockchain / SFID / CPMS / Mobile（全产品）

## 1. 背景与目标

把两个客户端产品的英文名、模块 id、目录、包名、bundle、协议常量、CI、文档彻底统一，**零旧名残留**：

- 公民（在线/热钱包）：`wuminapp` → **`CitizenApp`**（模块 id / 目录 `citizenapp`）
- 公民钱包（离线/冷钱包）：`wumin` → **`CitizenWallet`**（模块 id / 目录 `citizenwallet`）
- 扫码协议常量：`WUMIN_QR_V1` → **`CITIZEN_QR_V1`**

中文名不变（公民 / 公民钱包）。用户明确：无用户、开发期、不搞兼容、一次性彻底改。

## 2. 命名映射总表（钉死）

| 维度 | 旧 | 新 |
|---|---|---|
| 产品英文名 | wuminapp / wumin | CitizenApp / CitizenWallet |
| 顶层目录 | `wuminapp/` `wumin/` | `citizenapp/` `citizenwallet/` |
| Dart 包名 | `wuminapp_mobile` `wumin` | `citizenapp` `citizenwallet` |
| applicationId / bundle / namespace | `org.chinanation.citizen`（热）/ `org.citizenwallet`（冷） | `org.citizenapp` / `org.citizenwallet` |
| Kotlin 包路径 | `com/wuminapp/wumin/` | `org/citizenwallet/`、`org/citizenapp/` |
| MethodChannel / EventChannel | `org.citizenwallet/security(_events)` | `org.citizenwallet/security(_events)` |
| QR 协议常量 + 字符串 | `WUMIN_QR_V1` | `CITIZEN_QR_V1` |
| QR 解析文件 | `wuminQr.ts`（×3） | `citizenQr.ts` |
| 共享签名组件 | `WuminSignatureModal` `WuminSignaturePanel` | `CitizenSignatureModal` `CitizenSignaturePanel` |
| CI workflow | `wuminapp-ci.yml` `wumin-ci.yml` | `citizenapp-ci.yml` `citizenwallet-ci.yml` |
| CI secret | `WUMINAPP_*` `WUMIN_*` | `CITIZENAPP_*` `CITIZENWALLET_*` |
| OTA manifest | `wuminapp-android-update.json` | `citizenapp-android-update.json` |
| Isar 实例名 | `wuminapp_wallet` `wumin_wallet` | `citizenapp` `citizenwallet` |
| polkadot-sdk fork 分支 | `ss58-2027-fix-wuminapp-grandpa` | `ss58-2027-fix-citizenapp-grandpa` |

**原则**：产品 token = 恰好 `citizenapp` / `citizenwallet`，剥离一切遗留 token（`chinanation`、`_mobile`、`_wallet`、旧 umbrella `wuminapp`）；仅保留真正功能后缀（`-ci`、`-android-update`、`/security`、`RELEASE_KEYSTORE`）。

## 3. 分阶段执行（每阶段过编译/analyze/test 门）

- [x] **P0 命名权威源**（2026-06-20 完成）：AGENTS.md:105 / agent-rules.md:75 产品命名硬规则补英文名 CitizenApp/CitizenWallet + 废弃旧名；unified-protocols.md P-QR 协议名随 P1 sed 改 CITIZEN_QR。unified-naming.md 目录表行留 P2 随 git mv 同步（保持登记=文件系统一致）
- [x] **P1 QR 协议跨端**（2026-06-20 完成）：`WUMIN_QR`→`CITIZEN_QR`（含 V1/V2 + 字符串 + const + 注释 + 6 夹具 + spec + 测试 fn `wumin_qr`→`citizen_qr`）；`wuminQr.ts`×3 → `citizenQr.ts` + 全 import；共享组件 `WuminSignature*`→`CitizenSignature*`（sfid 8 文件 + 2 组件文件 git mv）。验收：活跃树 `WUMIN_QR/wuminQr/WuminSignature` 零残留 + sfid/cpms 前端 `tsc --noEmit` exit=0。待补：cargo（链/cpms/sfid 后端）+ flutter（两端）真实编译验收
- [x] **P2 目录/文件 git mv**（2026-06-20 完成）：顶层 `wumin/`→`citizenwallet/`、`wuminapp/`→`citizenapp/`、Kotlin 包目录、CI workflow、生成器、helper 脚本；**+ cargo check 抓到并补全的遗漏**：`sfid/backend/wuminapp/`→`citizenapp/`（断了 `mod citizenapp;`）、memory 目录 `01-architecture/wuminapp`·`05-modules/wuminapp`·`05-modules/wumin`、各 `*WUMINAPP*.md`/`*WUMIN*.md`/`ADR-018·020`/`module-checklists·dod·templates/wuminapp.md`/`wuminapp-vs-wumin.md`、24 张 open 活跃任务卡文件名（done 历史卡文件名保留）。教训：git mv 只移顶层目录会漏掉同名子模块/文档，必须 `rg --files | rg wumin` 全量补
- [x] **P3 Dart 包名 / import**（2026-06-20 完成）：pubspec `name: citizenwallet` / `citizenapp`，208+20 文件 `package:` import 全改。验收：**citizenwallet flutter analyze 0 + 全测试 All passed**；**citizenapp flutter analyze 0 + qr/signer/wallet/governance/rpc 152 测试 All passed**
- [x] **P4 原生层**（2026-06-20 完成）：bundle/namespace/applicationId `org.citizenwallet` / `org.citizenapp`，MethodChannel/EventChannel 三端锁步，Isar 实例名 `citizenwallet`/`citizenapp`，Kotlin 包路径，iOS pbxproj/AppDelegate；已修 pbxproj 残留 bug `[sdk=iphoneos*]=wuminapp`→`org.citizenapp`
- [x] **P5 CI / OTA / fork**（2026-06-20 完成）：✅ workflow 改名 `citizenapp-ci.yml`/`citizenwallet-ci.yml`、workflow 内 secret 名 `CITIZENAPP_*`/`CITIZENWALLET_*`、OTA manifest/tag/UA/`"app"` key、guardrail 路径。✅ **fork 分支**：gh api 在 ChinaNation/polkadot-sdk 建 `ss58-2027-fix-citizenapp-grandpa`→154590c2（同 commit，旧分支暂留）；citizenchain/Cargo.toml 66 行改新分支；citizenchain/Cargo.lock(176)+sfid/backend/Cargo.lock(70) 重生，**0 旧分支残留**；**citizenchain `cargo check` 通过(57s) + sfid 后端通过(22s)**。✅ **GitHub Secrets**：冷钱包原 4 个 `WUMIN_*` 已删、4 个 `CITIZENWALLET_*` 已建（GitHub secret 只写不可读，原 keystore .jks 本机已丢→开发期 re-key：JBR keytool 生成新 `~/keys/citizenwallet-release.jks` alias=upload，base64 入 secret，key.properties 已指向新库；3 个密码/别名沿用原值）。热钱包 `WUMINAPP_*` 本就不存在（citizenapp 发布签名从未配置）
- [x] **P6 白皮书 / 官网 / 残余注释**（2026-06-20 完成）：docs 白皮书 + website Ecosystem 展示名、`Citizen Wallet`→`CitizenWallet`、`Wumin`→`CitizenWallet`、cpms 后端函数名 `verify_wumin_login_signature`/`bind_admins_from_wumin`、生成器 JS 变量、**runtime 注释**（wuminapp→citizenapp / wumin→citizenwallet，仅注释，二次确认已授权）

## 完成核验（2026-06-20）

全树 `wuminapp`/`wumin` 命中仅剩 3 类**故意保留**项，其余**100% 零残留**（Cargo fork 已解决）：
1. 命名废弃规则：AGENTS.md / agent-rules.md 各 1 行（指名旧词=规则本身，必须保留）
2. smoldot vendored fork：6 行（fork-vendor-baseline 豁免）
3. done 历史卡：230（保留历史，文件名+内容均不动）

验收门（全过）：citizenwallet/citizenapp `flutter analyze` 0 + 测试全过；sfid/cpms 前端 `tsc --noEmit` 0；**cpms / sfid / citizenchain 三个 Rust 后端 `cargo check` 全部通过**；fork 分支改名后两个 Cargo.lock 0 旧分支残留。

## 4. 远端处理（全部完成）

1. ✅ fork 分支：新 `ss58-2027-fix-citizenapp-grandpa` 已建，**旧 `ss58-2027-fix-wuminapp-grandpa` 已删**
2. ✅ GitHub Secrets 冷钱包：`WUMIN_*`(4)→`CITIZENWALLET_*`(4)（删旧建新，开发期 re-key，新 keystore `~/keys/citizenwallet-release.jks`）
3. ✅ GitHub Secrets 热钱包：补齐 citizenapp 发布签名（此前从未配置）——新 keystore `~/keys/citizenapp-release.jks` alias=upload + `citizenapp/android/key.properties`(gitignored) + 5 个 `CITIZENAPP_*`(BASE64/SHA256/STORE_PASSWORD/KEY_ALIAS/KEY_PASSWORD) 全设；CI 引用名一一匹配，keystore 校验通过

## 5. runtime 二次确认（AGENTS.md 硬规则）

`citizenchain/runtime/` 内 wumin/wuminapp 注释引用的修改已获用户明确二次确认（"两个都同意，开始"）。仅注释/文案，无逻辑变更。

## 6. 验收门（零残留）

```
rg -i 'wuminapp|wumin|WUMIN_QR_V1' --hidden -g '!.git' -g '!memory/08-tasks/done/**'
```
活跃树应为零命中（smoldot-pow vendored fork 内部按 fork-vendor-baseline 豁免；done 历史卡保留）。
各端：cargo test（链/cpms/sfid）+ flutter analyze/test（两 App）+ tsc（前端）全过。

## 7. 受影响目录清单（附注）

- `wuminapp/`→`citizenapp/`：包名/bundle/原生/Isar/OTA/全量代码文案（代码+残留）
- `wumin/`→`citizenwallet/`：包名/bundle/MethodChannel/原生/全量代码（代码+残留）
- `citizenchain/node/`、`cpms/`、`sfid/`、`website/`：CITIZEN_QR_V1 + citizenQr + 组件 + 文案（代码）
- `citizenchain/runtime/`：仅注释引用（已二次确认）
- `citizenchain/Cargo.toml`+`Cargo.lock`：fork 分支名（配置）
- `.github/`、`scripts/`、`tools/`：workflow/secret/路径/生成器（配置+工具）
- `docs/`、`memory/`：白皮书、命名权威源、模块文档、ADR、open 任务卡（文档+残留）
