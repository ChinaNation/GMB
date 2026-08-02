# citizenapp+citizenwallet 钱包改 substrate 官方 HD 派生(单主助记词 //index 多账户)

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(本卡**修订**其派生地基,见 Phase 0)
状态:open(需求分析 + 用户拍板已完成 2026-07-26;待 Phase 0 门禁通过后进 Phase 1 代码)
所属模块:Mobile(citizenapp 热钱包 + citizenwallet 冷钱包)

> ⚠️ **2026-07-27 被 model B 推翻**:本卡「账户0 = bare 逐字节不变 / 护创世 9c3e / HD 纯增量 / ADR-022 三不变量」的硬约束已**作废**。新决策 = 全 `//index` **无 bare 根**(账户0 = `//0`)+ citizenwallet **无根存储**(只存每账户 child mini-secret,不存种子/助记词),账户地址全变 → **需重新创世**(Step 3 换全部创世机构管理员 + 程伟公钥)。见 `20260727-citizenwallet-modelb-index-derivation.md`。下文 Phase 0 / 创世护栏内容仅存档,**勿据以实现**。

## 背景 / 现状(核验锚点)

当前「我的-钱包-我的钱包」是**扁平 1:1:1**:一套助记词 = 一个种子(miniSecret 32B) = **一个** sr25519 私钥(`fromSeed`,无 junction) = 一个公钥 = 一个 2027 号 SS58。多地址靠**多套独立助记词**,不是同种子分叉。

- 派生核心:`citizenapp/lib/wallet/core/wallet_manager.dart:551 _deriveSr25519FromSeed`(`Keyring.sr25519.fromSeed(seed)`,无路径)
- 种子:`:544 _mnemonicToMiniSecret`(`CryptoScheme.miniSecretFromEntropy`)
- 「添加钱包」每次新生成助记词:`:294 bip39.generateMnemonic`
- accountId = sr25519 公钥原字节 `0x`+64hex(`:1026`);ss58 = encode(公钥, 2027)(`:93 _ss58Format`)
- 全仓 grep `derivePath/hardDerive/softDerive/fromUri/accountIndex` 零命中 → 确认无 HD

**依赖库可行性(已核验):** `polkadart_keyring 0.7.0` 的 `Keyring.sr25519.fromUri('<助记词>//<index>')` 走 `SecretUri` + `ExtendedKey.deriveKeyHard`(`.../polkadart_keyring-0.7.0/lib/src/sr25519.dart:66-95`),即 substrate 官方硬 junction 派生。无需换库、无需改链。

## 目标模型

一套助记词 → 一个种子 → **多个私钥(`//index` 硬派生)** → 各自一个公钥 → 各自一个 2027 号 SS58。一句助记词恢复全部账户。

## 用户拍板(2026-07-26)

1. **全量改**:citizenapp 热钱包 + citizenwallet 冷钱包一起上 HD,并**同步返工 PQC 线**(ML-DSA 每账户随 junction 重派、金标重钉、QR/FFI 连带)。
2. **账户模型 = 单主助记词 + `//index` 多账户**:全 App 一套主助记词;「添加钱包/账户」= 加下一 index,**不再生成新助记词**;默认用户 = 最靠前热账户(沿用现规则,身份逻辑相应改)。
3. **推翻 ADR-022 三不变量**:「sr25519 `fromSeed` 直出 / `account_seed` 不变 / sr25519 地址逐字节不变」被本卡取代。开发期零用户(memory `in-development-zero-users`)→ **无迁移、直接重派生**。

## 路径约定(Phase 0 钉死)

- 🔴 **账户 0(主账户)= 现 `fromSeed(miniSecret)` 直出,逐字节不变**;HD 账户**只作增量**:账户 N≥1 = `<助记词>//<N>` 硬派生。这与 polkadot.js 默认行为一致(bare 助记词=根账户,`//N`=派生账户),且是**护住创世注资地址的硬约束**(见下)。
- ss58 prefix **2027** 不变;accountId 仍 = 派生公钥原字节 `0x`+64hex。
- 不加 BIP44 式 `//44//354` 前缀(substrate 原生 sr25519 junction,非 SLIP-0010)。

## 🔴 创世已注资 sr25519 个人钱包护栏(核验实锤 2026-07-26)

创世六笔注资中 **5 笔是机构/协议账户**(blake2b `account_derive` op_tag:国储会安全基金/联邦注册局/司法院/技术发展基金会/联邦公民安全基金)——本卡**不动 blake2b 派生,零影响**。

**但 1 笔是个人 sr25519 钱包**:公民程伟钱包 `9c3e18f575c59236832054469ef0e69f16a1fe6c50b2b580fc7c71853ab71068`(200 亿元),固定 hex 写死在 `citizenchain/runtime/primitives/cid/china/citizenchain.rs:77`(`CITIZENCHAIN_GENESIS_ADMINS[0].account_id`)。链上余额不受 App 改动影响,但 **App 侧派生若改动主账户,会导致同一助记词再也派生不出这个地址 → App 失去对已注资地址的控制**。

→ 硬约束:**主账户派生绝不改**,HD 纯增量。Phase 0 须确认 `9c3e…1068` 确由现 `fromSeed` 方案生成(用户侧助记词),否则单独处理。

## 阶段拆分

### Phase 0 —— 门禁 / 设计(**不改生产钱包代码**)✅ 完成 2026-07-26
- [x] **派生金标向量**(citizenwallet `test/wallet/derivation_golden_test.dart`,固定 dev 助记词,ss58=2027):
  - 账户0(根/bare):`0x46ebddef8cd9bb167dc30878d7113b7e168e6f0646beffd77d69d39bad76b47a` / `w5DBnqoUytARopdnyWhmBq7ZPr74cJJewugoafJJynKLrirdE`
  - 账户1(//1):`0xb606fc73f57f03cdb4c932d475ab426043e429cecc2ffff0d2672b0df8398c48` / `w5FhUDLW4BxsE1QXK4sNjPZ8rqSnK2QeVpUfXzqczpWdxChxV`
  - 账户2(//2):`0x46f136b564e1fad55031404dd84e5cd3fa76bfe7cc7599b39d38fd06663bbc0a` / `w5DBpRvbgkersZohanGQiXa4qQLS1n7VQaSFwBaq4irJmgDn5`
- [x] **junction 标准正确**:`//Alice` 公钥 == 权威 Alice `0xd435…a27d`。
- [x] **base 一致**:`fromSeed(miniSecret)` == `fromUri(bare)` → `seedFromEntropy==miniSecretFromEntropy`,账户0=bare 逐字节等于现状,//N base 不漂。**「账户0」定义 = bare(不走 //0)。**
- [x] **seed-only 等价**:`0x<miniSecret>//N == <助记词>//N` → 冷签派生 N 只需 master mini-secret,不解密助记词。
- [x] **创世护栏**:账户0 派生不变已证 → 任何历史 bare 地址(含 `9c3e…1068`)结构性保留;无需(也无法,缺其助记词)反推 `9c3e` 来源。
- [x] **ADR-022 §2 修订块** + **PQC card3 取代指针**已落档(ML-DSA 改每账户 `account_seed_N`→HKDF)。

### Phase 1 —— citizenapp 热钱包
- [ ] 派生核心:`_deriveSr25519FromSeed`→按 index junction 派生;签名 `_keyPairFromSeedHex:727`、自愈 `_selfHealSeedFromMnemonic:707`、设备子钥 `_registerDeviceSubkey:769` 全按 index 派生。
- [ ] **存储模型重构**:主助记词/种子**存一次**(去掉 per-walletIndex seed 复制);`WalletProfileEntity` 加 `accountIndex` + master 关联;`SecureSeedStore` 键法、`hasSeed`、门禁 `isUsableHotWallet:245` 语义随之改。
- [ ] 创建/导入/加账户:新助记词=新身份;同助记词加账户=下一 index;默认用户身份逻辑 `getDefaultWallet:219`。
- [ ] 联系人 HKDF(salt=accountId 每账户仍不同,复核不串号)、UI(我的/钱包详情/备份)、`account_derivation_golden_test` 真跑、widget/单测全绿、`dart analyze` 0。

### Phase 2 —— citizenwallet 冷钱包
- [ ] 镜像同一 `//index` 派生;FFI 归并进 PQC card3 的 Rust 子工程;QR 字段;同助记词冷热恢复同址逐字节一致。

### Phase 3 —— PQC 线返工(随 PQC card3)
- [ ] ML-DSA-65 每账户重派 + golden(含 ξ)+ v5 General Transaction / QR 分片 / 离线 metadata 连带;本卡与 PQC card3 交接点写清,避免双写。

## 必须遵守
- 冷热派生逐字节一致(金标钉死);ss58 prefix 2027、accountId=公钥原字节格式不变。
- 开发期零用户 → **无 migration/兼容**(memory `no-compatibility` / `chain-dev-never-ask-migration`);非显式**不改 citizenchain**(`no-chain-changes`)。
- `citizenapp/lib/citizen/shared/account_derivation.dart` 的 blake2b **机构/个人多签**派生是**另一条链**,本卡不动(勿混淆)。

## 验收
- 同一助记词 `//0//1//2` 冷热双端派生同址、逐字节金标一致。
- 「添加账户」不产生新助记词;一句主助记词可恢复全部账户。
- citizenapp `dart analyze` 0 + 相关 golden/widget/单测全绿;citizenwallet 同。
- ADR-022 修订落档;PQC card3 取代指针就位。

---

## 执行顺序(用户 2026-07-26 定)

**Step 1 = citizenwallet(公民钱包/冷签)先做,做完验收后再 Step 2 = citizenapp(公民 App/热钱包)。** Phase 0 门禁(派生金标 + ADR-022 修订)是两步共同前置。

## Step 1 技术方案(citizenwallet) — 更改目录 + 注释要点

工作根:`/Users/rhett/GMB/citizenwallet/`。现状:与 citizenapp 镜像,`fromSeed` 直出无 junction,seedHex/助记词按 walletIndex 存,离线签名按 walletIndex 走 `signWithWallet`;是多钱包 + 分组的冷签设备(ss58=2027,`chain_constants.dart:9`)。

### 派生核心(密码学,Phase 0 金标钉死)
- 账户 0 = `Keyring.sr25519.fromSeed(masterMiniSecret32)`,**逐字节不变**(护 `9c3e…1068`)。
- 账户 N≥1 = 从**同一 master mini-secret** 走硬 junction 派生(镜像 `polkadart_keyring` `Sr25519KeyPair.deriveHardSoft`:`ExtendedKey.deriveKeyHard(root, [], junctionId(N))`),**签名只需 seed 不需助记词**,且 base 与账户 0、与 polkadot.js `mnemonic//N` 一致。
- Phase 0 必确认 `seedFromEntropy == miniSecretFromEntropy`(否则 //N base 会漂)+ 钉 `//N` junctionId 精确字节 + 交叉核验 polkadot.js 向量。

### 数据模型(用户 D2 定:两级结构 + 三级导航)
- **钱包(wallet)= 一套助记词 = 一个种子 = 一个 master**;设备持多个钱包(D1 保留多助记词)。
- **钱包 → N 个账户**(0,1,2…);账户 0=根派生(护创世地址),账户 N≥1=`//N`;每账户=一对公私钥=一个 ss58。
- **三级导航**:Lv1 钱包列表(只显示钱包名+分组)→ Lv2 钱包详情(助记词备份 + 账户列表 +「添加账户」)→ Lv3 账户详情(私钥/公钥/ss58/路径)。
- **助记词=钱包级**;**私钥/公钥/ss58=账户级**。私钥展示:账户0=32B mini-secret;账户N=派生密钥标量(只读;恢复靠钱包助记词+路径)。

### 逐文件改动
| 文件 | 改动 | 注释要点 |
|---|---|---|
| `lib/wallet/wallet_manager.dart` | 钱包/账户两级 API:`createWallet`/`importWallet` 建**钱包(master)+账户0**;新增 `addAccount(masterId)` 派生下一 index;`deriveAccount({masterSeed, accountIndex})`(0=根,N≥1=//N);签名改**按账户**(accountId→(masterId,accountIndex)→读 master seed→派生→签);`deleteAccount` / `deleteWallet`(删钱包连带全部账户+master seed/助记词) | 派生单源·账户0=根护创世地址·//N 增量·seed 不出类 |
| `lib/isar/wallet_isar.dart` | **拆两实体**:`WalletEntity`(master:walletIndex/walletName/masterId/groupNames/sortOrder/source)+ `AccountEntity`(masterId 外键/accountIndex/accountId unique/ss58Address unique/sortOrder);旧 `WalletProfileEntity` 退役;开发期零用户→旧库删重建不 migration | 钱包=master 一套助记词;账户=其下 //index 派生 |
| `lib/isar/wallet_isar.g.dart` | build_runner 重生成 | 生成物勿手改 |
| `lib/wallet/wallet_secure_keys.dart` | seed/助记词键改按 masterId:`masterSeedHexV1(masterId)`/`masterMnemonicV1(masterId)`,一钱包存一次 | 密钥按钱包(主种子)存 |
| `lib/wallet/mnemonic_cipher.dart` | 不改(AES-256-GCM AEK 保留) | — |
| `lib/signer/offline_sign_service.dart` | 签名解析改**按 accountId 定位目标账户**(QR `signerPublicKeyHex`→AccountEntity→(masterId,accountIndex)→派生→签);两色识别不变 | 冷签按账户非按钱包 |
| `lib/login/login_qr_handler.dart`、`lib/ui/login_sign_page.dart` | 登录签名同样按账户定位 | 同上 |
| `lib/ui/create_wallet_page.dart` / `import_wallet_page.dart` | 语义=新建/导入**钱包**(master+账户0) | — |
| `lib/ui/home_page.dart` | **Lv1**:钱包列表只显示钱包名(+分组);「添加」=新建/导入钱包 | 列表=钱包级 |
| `lib/ui/wallet_detail_page.dart` | 改 **Lv2 钱包详情**:钱包名+助记词(查看/备份)+账户列表(0,1,2…)+「添加账户」;点账户进 Lv3 | 助记词=钱包级;列账户 |
| `lib/ui/account_detail_page.dart`(**新**) | **Lv3 账户详情**:该账户私钥、公钥(accountId)、ss58、派生路径 | 账户级密钥展示 |
| `lib/ui/group_management_page.dart` | 分组作用于**钱包**(Lv1);机制不改 | 分组=钱包级 |
| `test/wallet/derivation_golden_test.dart`(**新**) | 钉固定助记词 根/`//1`/`//2` → pubkey/accountId/ss58 金标 | 冷热共享单源 |
| 现有 `test/wallet/*`、`test/signer/offline_sign_service_test.dart` | 按两实体 + 账户签名改 | — |

### 已定(2026-07-26 用户确认)
- **D1 = 保留多助记词**:一钱包=一套助记词;设备持多个独立钱包。
- **D2 = 三级导航**(见上「数据模型」)。

### Step 1 进度
- ✅ **S1.1 派生+存储核心**(2026-07-26):`wallet_isar.dart` 拆 `WalletEntity`+`AccountEntity`(g.dart 重生成);`wallet_secure_keys.dart` 按 masterId;`wallet_manager.dart` 两级模型 + `deriveAccount`(0=fromSeed,N≥1=`fromUri('0x<seed>//N')` seed-only)+ 按 accountId 签名 + addAccount/deleteWallet/deleteAccount + 生物识别测试注入口 `debugAuthGate`;`WalletProfile/signWithWallet` 退役。测试:`wallet_manager_test`(建/加/签/删/查重)+ `wallet_model_test` + `wallet_secure_keys_test` 更新;全 wallet 测试 29 passed;新代码 `dart analyze` 零问题。**未迁移调用方 36 报错(offline_sign/login/UI)= 预期,归 S1.2/S1.3**;全项目 build 绿在 S1.3 收尾恢复。
- ✅ **S1.2 签名服务层按账户**(2026-07-26):`offline_sign_service` 入参 walletIndex→accountId、删 cold 分支、`getAccountByAccountId`+`signForAccount`;错误码 walletNotFound/walletMismatch→accountNotFound/accountMismatch(删 coldWalletUnsupported)。单测重写(fake 覆写 getAccountByAccountId/signForAccount)10 passed;`login_qr_handler` 无需改(已按 accountId)。
- ✅ **S1.3 UI 三级 + 全局扫码**(2026-07-26):Lv1 `home_page`(只列钱包名,顶部全局扫码入口)→ Lv2 `wallet_detail_page`(助记词备份+账户列表+添加账户+分组)→ Lv3 `account_detail_page`(**新建**:私钥[种子URI]/公钥/ss58/路径/收款QR/删账户)。**全局扫码**:`scan_page` 去 wallet 参数,按 QR `signerPublicKey` 在全设备账户定位(`getAccountByAccountId`),无此账户则拒;`offline_sign_page`/`login_sign_page` 改收 `Account`+walletName;`create/import/group` 迁移新模型。补 `getAccountSecretUri`(账户0=`0x<seed>`,N=`0x<seed>//N`)。
- ✅ **S1.4 收尾**(2026-07-26):删死代码(getWallet/getWalletByIndex/getActiveWalletIndex/setActiveWallet/getMasterSeedHex + 整个 active-wallet/`WalletSettingsEntity` 概念——全局扫码后无消费者);旧符号残留零(WalletProfile/signWithWallet/walletProfileEntitys/…);**全项目 `dart analyze` 0 + `flutter test` 201 passed**。

- ✅ **S1 复查 + S1-FIX**(2026-07-26,两独立评审 security+flutter,逐条回码核验):
  - **C-1(CRITICAL,已修)**:账户级"私钥"曾返回 `0x<masterSeed>//N` 泄露整钱包 → **账户详情页删除私钥区**(只留公钥/ss58/路径+收款QR),删 `getAccountSecretUri`;备份统一回 Lv2 助记词(用户拍板 Option A)。
  - **H-1/M-1(已修)**:`deleteWallet`/`deleteAccount`/`createWallet`/`importWallet` 全加 `_authGate`(app-lock 默认关,门禁是唯一兜底)。
  - **H-2(已消解)**:随 C-1 私钥展示路径移除。
  - **删除正确性(已修)**:`deleteWallet` 反转顺序(先删 Isar 行再清密钥,消僵尸钱包);`deleteAccount` 加 account0 锚点守卫 + 删除计数并入同一 writeTxn(消竞态)。
  - **拖拽重排(已修)**:`_onReorderWallet` 加 try/catch + 快照回滚 + SnackBar。
  - **残留清理(已修)**:删 `AccountEntity.sortOrder` 死字段;scan_page 单次解析(去第二次 jsonDecode);删 offline_sign_page 内置扫码死路径(改 raw 必填 + 解析失败/返回);wallet_detail 加"重导只恢复账户0"提示。
  - **M-4(不可行,已注)**:库 `KeyPair.lock()` 该版本 `fromEd25519Bytes(空)` 抛错,不可用清私钥,注释说明。
  - **测试补齐**:deleteAccount(非末位/级联/account0守卫)、getAccountByAccountId/getWalletByMasterId、删中间账户后 addAccount=max+1、rename/reorder、account_detail widget 钉死"不展私钥"。
  - **未纳入(既有基线,非 HD 引入)**:分组名逗号污染、`_deleteGroup` 死码、onDetect 判空、seed_hex 明文 vs 助记词 AES-GCM、`AndroidOptions(encryptedSharedPreferences)`、offline_sign 绿banner矛盾、walletIndex/masterId 寻址混用。**→ 七项已于 2026-07-26 由独立任务卡 `20260726-citizenwallet-baseline-fixes.md`(done)全部修复;`dart analyze` 0 + `flutter test` 215 passed。**
  - **终验**:`dart analyze` 0 + `flutter test` **209 passed**;S1-FIX 残留复扫全 0。

**Step 1(citizenwallet)完成并通过复查。** 派生金标 = 冷热共享单源(`test/wallet/derivation_golden_test.dart`),Step 2 citizenapp 逐字节复用。

### Step 1 → Step 2 交接
派生金标向量是**冷热共享单源**:Step 1 产出的 根/`//N` 向量,Step 2 citizenapp 必须逐字节复用(同助记词冷热同址)。派生规格写成两端共享注释/文档,Step 2 镜像实现。
