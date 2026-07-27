# model B:全 //index 硬派生 + 无根存储 + 重新创世

状态:open(用户授权 2026-07-27;Step 1 本窗口进行中)
所属模块:Mobile(citizenwallet/citizenapp) + Chain(citizenchain 创世)
关联/推翻:`memory/08-tasks/open/20260726-citizenapp-citizenwallet-hd-wallet-derivation.md`(HD 卡「账户0=bare 护 9c3e」硬约束被本卡**推翻**);ADR-022 派生地基(待 Step 3 改档)

## 决策(用户拍板)

> ⚠️ **2026-07-27 存储对调(D1 再反转)**:下面两条 citizenwallet/citizenapp 存储归属**已对调**——现为 **citizenwallet(冷)存种子+助记词、citizenapp(热)无根(只存公私钥对)**。派生/金标/重新创世等其余不变。citizenwallet 侧已落地(`20260727-citizenwallet-store-seed-mnemonic-reversal.md`,213 tests);citizenapp 侧另一窗口按无根做。

- **全 `//index` 硬派生,无 bare 根**:账户0 = `//0`(0 基),每账户各自独立 child mini-secret,单账户私钥泄漏只伤自己。
- **~~citizenwallet = 无根存储~~ → 已对调为 citizenwallet(冷)存种子+助记词**(签名/私钥导出从存储种子现场派生;钱包详情第一卡助记词 reveal)。
- **~~citizenapp = 保留种子/助记词~~ → 已对调为 citizenapp(热)无根**(只存公私钥对,复用原给 citizenwallet 建的 no-root 设计);仍**单热钱包**。
- **Step 3**:citizenchain 创世**换掉全部创世机构管理员公钥 + 程伟公钥**(bare→`//0`)+ 重新创世。

## 三大步

- **Step 1 citizenwallet(本窗口)**
  - **1.1 无根密钥模型 + 派生核心 + 新金标 ✅(2026-07-27)**:`wallet_manager.dart` 彻底重建(全 `//index`、child mini-secret 提取=`SecretUri` junction cc + `sr25519.hardDeriveMiniSecretKey`、每账户密钥存/读/删、签名 `fromSeed(child)` 重建、加账户带助记词+归属校验、删账户/钱包清各账户密钥、删 `getMasterMnemonic` 及全部 master 种子/助记词持久化);`wallet_secure_keys.dart`→`accountMiniSecretV1(accountId)`;`wallet_detail_page.dart` 删助记词查看区(身份卡只留图标+名称)+ 加账户弹助记词框;`wallet_isar.dart` 注释更新;新金标 `derivation_golden_test.dart`(//0 新值 + 不变式 `fromSeed(child)==//index`);`wallet_manager_test`/`wallet_secure_keys_test`/`wallet_model_test` 重写。**`dart analyze` 0 + `flutter test` 209 passed;残留复扫 0。** 金标向量见下。
  - **1.2 账户私钥展示(req 3)✅(2026-07-27)**:`account_detail_page.dart` SS58 下方加「私钥」区(默认隐藏"点击查看私钥"→确认→`getAccountPrivateKey` 生物识别→展开 `0x<64hex>` child mini-secret,纯 Text 不可复制,`ScreenshotGuard` 防截屏/录屏即隐藏);删"私钥统一回助记词" banner + "不展示私钥"类注释;账户0 不再特殊(model B 均为隔离叶子);`account_detail_page_test` 反转 C-1(私钥区在场但默认隐藏)。**analyze 0 + test 209 passed;残留 0。**
  - **1.3 收尾**:supersede ADR-022/HD 卡 + 记忆;全仓 citizenwallet 文档;终验;Step 2 交接契约。
- **Step 2 citizenapp(新窗口)**:镜像 `//index` + 复用金标;**无根**(见上 §决策 存储对调)。**已扩展为全量跨模块程序(2026-07-27)**:citizenapp 无根多账户 + CID 身份层(自助占号/换绑)+ 注册局改造 + 链侧新 extrinsic + 重新创世,见 `20260727-citizenapp-cid-identity-rootless-wallet.md`。PQC 随 HD 卡 Phase 3。
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
> ⚠️ **已由 `20260727-citizenapp-cid-identity-rootless-wallet.md` 接管并扩展为全量跨模块程序(2026-07-27)**;下列「保留种子/默认用户」项作废,以新卡为准。
- **逐字节复用本卡金标表**(//0//1//2 accountId/ss58/child)——citizenapp 派生必须与 citizenwallet 完全一致;起步先跑派生金标 spike 对齐再动代码。
- 派生核心改全 `//index`(账户0=`//0`,删 bare 分支);**citizenapp 无根**(只存账户公私钥对;与 §决策 对调一致,citizenwallet 反而存种子/助记词)。
- **单钱包多账户**;**删默认用户**,身份主键切 **CID**(详见新卡)。
- 核心文件:`citizenapp/lib/wallet/core/wallet_manager.dart`(`_deriveSr25519FromSeed`/`_keyPairFromSeedHex`/`_selfHealSeedFromMnemonic`/`_registerDeviceSubkey` 全按 //index)。
- PQC 线(ML-DSA 每账户)随 PQC card3,勿双写。

## Step 3 起步清单(仓库其余,新窗口)
- citizenchain 创世换**全部创世机构管理员公钥 + 程伟公钥**(bare→`//0`)。**D3 已定(用户 2026-07-27):到时用户直接提供新 `//0` 公钥(程伟 + 全部创世管理员),Step 3 只把提供的公钥写入创世常量,无需其助记词、无需我方派生。**
- 机构注册表重生成([[registry-regen-after-genesis]]);ADR-022 §2 正式改档 + 白皮书 + 全仓文档;全仓 grep bare/fromSeed 残留;重新创世部署 + 冷热链逐字节一致性验收。

## Step 3 执行台账(2026-07-27 逐文件核实 + 用户拍板)

> ⚠️ **审计教训**:第一轮只扫顶层 `pub const …ADMINS` → 漏掉 `ChinaCb`/`ChinaCh` 结构体**内嵌 `admins:` 字段**(NRC/PRC/PRB 管理员),把范围误报成 231。用户当场质疑后重扫,真实规模 **≈1024**。判残/审计务必扫结构体内嵌字段,勿只扫顶层 const([[dead-code-scan-three-blind-spots]])。

**A 类|真 sr25519 密钥对公钥(model B 影响,必换)= ≈1024:**

| 项 | 位置 | 数量 |
|---|---|---|
| 程伟 `9c3e…1068` | `citizenchain.rs:77/85/90/95` + `china_zf.rs:28`(FSC_GENESIS_ADMIN_ACCOUNT) | 1(5 处 hex) |
| NRC + 43省 PRC 管理员 | `china_cb.rs` 内嵌 `admins:`(44 机构) | 406 |
| 43省 PRB 管理员 | `china_ch.rs` 内嵌 `admins:`(43 机构) | 387 |
| 注册局 FRG | `china_zf.rs:61` `FEDERAL_REGISTRY_ADMINS`(43省×5) | 215 |
| 司法院 NJD | `china_sf.rs:17` `NATIONAL_JUDICIAL_YUAN_ADMINS`(带 admin_role) | 15 |

**B 类|`grandpa_key`(ed25519 共识权威,`china_cb.rs` 44 个 = NRC+43 PRC):用户拍板【不换,保原值】。** 共识密钥与 model B(sr25519 钱包派生)不同源;[[genesis-single-grandpa-authority-accepted]] 创世权威集只 1 把 NRC。

**C 类|blake2b CID 派生地址(无私钥,model B 碰不到,不换)——已哈希实算逐字节证明:** 全机构 `main_account`/`fee_account`/`stake_account`/`SAFETY_FUND_ACCOUNT`/`NRC_HE_ACCOUNT` + 基金会 main/fee。公式 `blake2_256(GMB‖op_tag‖ss58_le(2027)‖cid_number)`(`account_derive.rs`;op_tag `OP_MAIN=0x01`/`OP_FEE=0x02`…)。`rederive_accounts.py` 只重派这类,域/tag 不变即不动。

**用户拍板交付(2026-07-27):**
- 形态 = **用户给完整新数组**(用户自有工具按 model B //0 重派,我只写入+验收,不接触助记词/不我方派生)。
- A 类交付 = **全覆盖、按各文件现有顺序整组吐新公钥**(cb/ch/zf/sf 全部 admins + 程伟 //0)。**顺序即语义**:FRG 位置编码省组、NJD 位置编码 admin_role、cb/ch admins 位置对应机构行——严禁错位。
- 只换 account_id/公钥;CID、姓名、省份/角色映射全不动。

**S3.1 波及的测试/播种引用(替换后须同步核):** `genesis/src/institution/seeder.rs`(insert_fixed_admins/FSC/CITIZENCHAIN_GENESIS_*)、`runtime/src/configs.rs`、`runtime/src/genesis.rs`、`runtime/src/tests/cases.rs`、`votingengine/internal-vote/src/tests/mod.rs`。

## S3.1 已落地(2026-07-27):自生成密钥 + 全量落位

**生成器 = `citizenwallet/tool/derive_admin_pubkeys.dart`**(dart run,复用 polkadart_keyring 与 App/金标同源):自生成 24 词助记词 → model B `//index` 派生 → 输出 7 对 txt(公钥/完整)+ 就地补丁 5 个创世常量文件。护栏:金标自检(pub+child)、每 key `fromSeed(child)==fromUri` 双路一致、全局公钥去重、落位后重解析计数校验、只改 hex 值(骨架剔 hex! 逐字节一致)。`--dry-run` 预览到 `<out>/rust_preview/`。

**用户拍板补充(2026-07-27):**
- **程伟(基金会+FSC)= 一套助记词、一个 CID、一个账户**(不拆岗!),只把旧 `9c3e…1068` 替换成新 //0;`citizenchain.rs` 4 处 + `china_zf.rs` FSC 1 处;CID/姓名/结构/[[foundation-genesis-identity-frozen-guard]] 冻结守卫**全不动**。
- **发币账户 = 仅出密钥文件**(CitizenConsole 充值发币链下签名钥,不写 citizenchain 创世)。
- 省储会/储行/注册局 = 每省 1 套助记词;NRC/NJD = 各 1 套助记词多账户。

**已写入仓库(git diff 1028 改 / 1028,骨架逐字节一致,旧 9c3e 清零):**
| 机构 | 助记词 | 账户 | 落点 |
|---|---|---|---|
| 省储会 PRC | 43 | 43×9=387 | china_cb.rs admins[1..43] |
| 省储行 PRB | 43 | 387 | china_ch.rs admins[0..42] |
| 联邦注册局 FRG | 43 | 215 | china_zf.rs FEDERAL_REGISTRY_ADMINS |
| 国储会 NRC | 1 | 19 | china_cb.rs admins[0] |
| 司法院 NJD | 1 | 15 | china_sf.rs NATIONAL_JUDICIAL_YUAN_ADMINS |
| 程伟(基金会+FSC) | 1 | 1 | citizenchain.rs×4 + china_zf.rs FSC×1(替换 9c3e) |
| 发币账户 | 1 | 1 | **仅密钥文件,不入库** |

**机密总账**(唯一恢复凭证,离线备份、勿进 git):用户本地 `~/genesis_out/*_完整.txt`(14 文件)。

**S3.1 完成校验:** `cargo check -p primitives` **exit 0**(1028 值编译通过);seeder/configs/cases.rs/internal-vote tests **全按常量名引用**、无硬编码 admin 值;旧 admin 值(旧 NRC 首值/9c3e)全仓 0 残留。

## S3.2 已完成(2026-07-27):model B 派生正式改档

- **ADR-022 §2 改写**:行 34 删「派生路径不改/地址逐字节不变」;2026-07-26 model-A 修订块 → 替换为 model B(账户0=//0、无 bare 根、每账户 child mini-secret 泄漏隔离、三不变量作废、9c3e→程伟新 //0 已重新创世);派生规则码块 `fromSeed(AccountSeedV1_N)`(=助记词//N child,账户0=//0)。`AccountSeedV1` 全文重定义为「每账户各自 32B child」。§12 行189「地址逐字节不变」= **PQC 在位升级验收**语义(创世后 ML-DSA 切换不改址),保留。
- **`CITIZENWALLET_PQC_TECHNICAL.md` 行9** → model B //index(账户=child mini-secret)。
- **不动(已核正交)**:白皮书(PQC 在位升级语义有效)、`unified-protocols.md`(P-256 子钥编码)、ADR-026(签名消息域)。
- **记忆**:`wallet-hd-derivation-supersede-adr022` 已 model-B(不动);MEMORY.md 索引一致。
- **全仓文档 model-A 死口径复扫 = 0 残留**。
- **交界处(留 Step 2)**:citizenapp `WALLET_TECHNICAL.md` 行 114/369/394 `fromSeed(miniSecret/AccountSeedV1)` = citizenapp 无根改造窗口(`20260727-citizenapp-cid-identity-rootless-wallet.md`)一并改,本步不碰(避免撞车)。

## S3.3 + S3.4 已完成(2026-07-27):无静态产物残留(除冻结 chainspec)

- **S3.3 数据包**:`citizenapp/tools/generate_public_institution_bundle.mjs` 从**链上**解码机构身份(cid+account_name,**无 admins 字段**),且 main_account/CID 未变 → 数据包对 admin 变更 no-diff;onchina 等消费方链上实时读。**无需重生成**(数据包重烤并入 S3.5 读新链)。
- **S3.4 残留扫**:链端**无** App 式 sr25519 bare/fromSeed 派生(链只存 admin pubkey);新程伟值在 json/ts/dart 静态产物命中 **0**。**唯一嵌旧 admin 值的静态产物 = 冻结 chainspec** `citizenchain/node/chainspecs/citizenchain.plain.json`(2.4MB,`include_bytes!` 进 node `chain_spec.rs:20`)→ 必在 S3.5 重烤。旧 9c3e 仅存于生成器 `derive_admin_pubkeys.dart:60` 替换锚点(已用完,无害)。

## S3.5 重新创世(待执行,操作性重活)

**唯一需重生成的产物 = 冻结 chainspec**(现嵌旧 admin)。烤制机制 `citizenchain/scripts/bake-chainspec.sh`:**正式创世须先 GitHub WASM CI 成功**,再 `--finalize --wasm <编译产物> --wasm-ci-run-id <RUN> --wasm-ci-head-sha <SHA>`,同步 5 目标(node chainspec + citizenapp assets/chainspec/light_sync_state/public_institutions + wrangler)。之后 clean-run/部署中枢([[regenesis-deploy]])。

**前置未做:** 全 runtime `cargo build/test`(仅 primitives crate 已 check)+ genesis seeder 测试须先全绿。

## 程伟 CID 改新规则(2026-07-27,用户拍板)

- 旧 `GZ000-CTZN6-198805200-2026`(旧地域化 GZ 省码)→ 新 **`CN220-CTZN2-198805200-2026`**。
- **新规则**:公民(CTZN/NATP/SMTP)CID **去地域化**,R5 = `CN` 国家码 + 号段(单源 `primitives/cid/generator.rs`,金标 `citizen_cid_number_golden` 钉死;dev //0 → `CN951-CTZN1-539598435-2026`)。
- **⚠️ 手工值非生成器直出**:R5 `CN220` 是程伟新 //0 账户 `0cb1d05c…` 经生成器算得的号段;但 N9 用户指定用**生日 `198805200`**(非生成器哈希出的 `654350186`,为可读性)。校验位 M1 是 N9 的函数 → N9=198805200 时**唯一合法 M1=`2`**(用户初写的 `8` 是旧 N9 的校验位,会被链拒)。**后续切勿用生成器"重生成"程伟 CID**(会得到 `CN220-CTZN2-654350186-...` 不同 N9);此值手工钉死。
- 全仓 22 处统一替换(21 文件:2 创世常量 citizenchain.rs:22/china_zf.rs:30 + 链/onchina/node/App 测试夹具 + 文档),旧 CID 残留 0,同长度不影响编译。parse 校验(格式+校验位+CTZN 盈利族)通过;S3.5 全 runtime 构建为最终 genesis-parse 闸。

## 本窗口自审 + 修复(2026-07-27)

审计发现 9 项,已处置:
- **P0(我的失误,已修)**:全仓 CID sed 误改历史档案卡(`done/20260717` + 已完成 `20260722`)→ 已还原 GZ000(历史值不可篡改,[[read-audit-recipe-first]] 铁律4)。
- **P1(已修)**:单次性生成器工具用后已删(`citizenwallet/tool/derive_admin_pubkeys.dart` + tool/ 目录);账户已生成、密钥已备份、创世后管理员走链上更换,工具无复用价值(重跑会与 ~/genesis_out 备份解耦)。
- **P5(已修)**:`GENESIS_TECHNICAL.md:33` 陈旧法定代表人账户 → 程伟新 //0 `0cb1d05c…`。
- **P6(已修)**:ADR-022 §2 KDF 定义 `IKM=AccountSeedV1`→`AccountSeedV1_N`(每账户 child)。
- **P7(已修)**:citizenapp `WALLET_TECHNICAL.md` 加 model-B supersede 横幅 + 派生核心 3 处改 //index(存储/多账户段落仍留 Step 2)。
- **P2/P3/P4/P8(用户确认无需改)**:密钥已备份+链上换届;只预检不动代码;创世固定;发币账户后期自配。
- **已验证无恙**:admin 落位 grandpa(44)/C类/基金会 blake2b 全未误伤;程伟 CID 合法自洽无派生/字节断言破;primitives cargo 0 + CID 44/0;citizenwallet flutter 0;citizenapp 改动 4 文件 dart analyze 0;全链 cargo check(预检)进行中。

**待办:** S3.5(WASM CI→bake --finalize→重烤 chainspec+assets→clean-run/部署)/S3.6(冷热逐字节验收)。S3.5 涉 CI + 对外部署,须用户显式确认 + 可能需其环境/节点权限。
