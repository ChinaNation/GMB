# model B:全 //index 硬派生 + 无根存储 + 重新创世

状态:open(用户授权 2026-07-27;Step 1 本窗口进行中)
所属模块:Mobile(citizenwallet/citizenapp) + Chain(citizenchain 创世)
关联/推翻:`memory/08-tasks/open/20260726-citizenapp-citizenwallet-hd-wallet-derivation.md`(HD 卡「账户0=bare 护 9c3e」硬约束被本卡**推翻**);ADR-022 派生地基(待 Step 3 改档)

## 决策(用户拍板)

- **全 `//index` 硬派生,无 bare 根**:账户0 = `//0`(0 基),每账户各自独立 child mini-secret,单账户私钥泄漏只伤自己。
- **citizenwallet = 无根存储**:设备只存每账户 child mini-secret(32B AES-GCM 密文,按 accountId),**不存种子/助记词**;签名读该账户密钥现场重建;加账户需临时输本钱包助记词;助记词只在创建时一次性显示;备份唯一凭证=用户自抄。
- **citizenapp(Step 2)**:热钱包**保留**种子/助记词存储;**改为只能创建一个热钱包**。
- **Step 3**:citizenchain 创世**换掉全部创世机构管理员公钥 + 程伟公钥**(bare→`//0`)+ 重新创世。

## 三大步

- **Step 1 citizenwallet(本窗口)**
  - **1.1 无根密钥模型 + 派生核心 + 新金标 ✅(2026-07-27)**:`wallet_manager.dart` 彻底重建(全 `//index`、child mini-secret 提取=`SecretUri` junction cc + `sr25519.hardDeriveMiniSecretKey`、每账户密钥存/读/删、签名 `fromSeed(child)` 重建、加账户带助记词+归属校验、删账户/钱包清各账户密钥、删 `getMasterMnemonic` 及全部 master 种子/助记词持久化);`wallet_secure_keys.dart`→`accountMiniSecretV1(accountId)`;`wallet_detail_page.dart` 删助记词查看区(身份卡只留图标+名称)+ 加账户弹助记词框;`wallet_isar.dart` 注释更新;新金标 `derivation_golden_test.dart`(//0 新值 + 不变式 `fromSeed(child)==//index`);`wallet_manager_test`/`wallet_secure_keys_test`/`wallet_model_test` 重写。**`dart analyze` 0 + `flutter test` 209 passed;残留复扫 0。** 金标向量见下。
  - **1.2 账户私钥展示(req 3)✅(2026-07-27)**:`account_detail_page.dart` SS58 下方加「私钥」区(默认隐藏"点击查看私钥"→确认→`getAccountPrivateKey` 生物识别→展开 `0x<64hex>` child mini-secret,纯 Text 不可复制,`ScreenshotGuard` 防截屏/录屏即隐藏);删"私钥统一回助记词" banner + "不展示私钥"类注释;账户0 不再特殊(model B 均为隔离叶子);`account_detail_page_test` 反转 C-1(私钥区在场但默认隐藏)。**analyze 0 + test 209 passed;残留 0。**
  - **1.3 收尾**:supersede ADR-022/HD 卡 + 记忆;全仓 citizenwallet 文档;终验;Step 2 交接契约。
- **Step 2 citizenapp(新窗口)**:镜像 `//index` + 复用金标;热钱包存种子/助记词;单热钱包;身份/默认用户逻辑;PQC 线(随 HD 卡 Phase 3)。
- **Step 3(新窗口)**:citizenchain 创世换全部管理员+程伟公钥 + 重新创世;机构注册表重生成;ADR-022/白皮书/文档;全仓残留扫;部署 + 冷热链一致性验收。

## 派生金标(冷热共享单源,dev 助记词 `bottom drive obey ... walk`,ss58=2027)

| index | accountId | ss58 | child mini-secret |
|---|---|---|---|
| //0 | `0x2afba9278e30ccf6a6ceb3a8b6e336b70068f045c666f2e7f4f9cc5f47db8972` | `w5CZACAABUbK4jspzPB5be9trhtSgRCRZFafGe7kvFPvxq8M2` | `0x914dded06277afbe5b0e8a30bce539ec8a9552a784d08e530dc7c2915c478393` |
| //1 | `0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48`(不变) | `w5FhUDLW4BxsE1QXK4sNjPZ8rqSnK2QeVpUfXzqczpWdxChxV` | `0x4433c3ada0cf37c3050d5435321872f4f84ef53d8b5f1f1560689d500b882245` |
| //2 | `0x46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a`(不变) | `w5DBpRvbgkersZohanGQiXa4qQLS1n7VQaSFwBaq4irJmgDn5` | `0x5418179cea7224f2d9d2ab437773c2fdb266e52ef7fa52c0d9c15c6ca6068748` |

不变式:`fromSeed(childMiniSecret) == <助记词>//index` 逐字节(金标测试钉死)。

## 关键实现要点
- child mini-secret 提取必须走 `sr25519` 底层(keyring `KeyPair` 不暴露私钥;`SecretKey.encode()` 丢 nonce 不可往返)——`SecretUri.fromStr('//N').junctions[i].junctionId[0..32]` 作 cc,`rootSecret.hardDeriveMiniSecretKey([], cc)` 取子 `MiniSecretKey.encode()`。
- 签名 `Keyring.sr25519.fromSeed(childMiniSecret)` 复现账户(== `//N`)。

## Step 1 完成(2026-07-27)
1.1 + 1.2 + 1.3 全部落地;citizenwallet `dart analyze` 0 + `flutter test` 209 passed;残留复扫 0;文档/记忆已更新(HD 卡加推翻横幅、`wallet-hd-derivation-supersede-adr022` 记忆升级 model B、MEMORY.md 索引同步)。改动只在主检出 `citizenwallet/` + `memory/`。

## Step 2 起步清单(citizenapp,新窗口)
- **逐字节复用本卡金标表**(//0//1//2 accountId/ss58/child)——citizenapp 派生必须与 citizenwallet 完全一致;起步先跑派生金标 spike 对齐再动代码。
- 派生核心改全 `//index`(账户0=`//0`,删 bare 分支);热钱包**保留**种子/助记词存储(与 citizenwallet 无根不同)。
- **改为只能创建一个热钱包**;默认用户 = 最靠前 `//0` 账户,身份逻辑相应改。
- 核心文件:`citizenapp/lib/wallet/core/wallet_manager.dart`(`_deriveSr25519FromSeed`/`_keyPairFromSeedHex`/`_selfHealSeedFromMnemonic`/`_registerDeviceSubkey` 全按 //index)。
- PQC 线(ML-DSA 每账户)随 PQC card3,勿双写。

## Step 3 起步清单(仓库其余,新窗口)
- citizenchain 创世换**全部创世机构管理员公钥 + 程伟公钥**(bare→`//0`)→ 需各自助记词跑新派生。**D3 未决**:谁提供助记词 / 是否给创世身份改用已知 dev 助记词的 `//0`。
- 机构注册表重生成([[registry-regen-after-genesis]]);ADR-022 §2 正式改档 + 白皮书 + 全仓文档;全仓 grep bare/fromSeed 残留;重新创世部署 + 冷热链逐字节一致性验收。
