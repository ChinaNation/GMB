# PQC 抗量子迁移 Step 2:钱包(wuminapp + wumin)

- 状态:open(设计已定,代码未启动,依赖 Step 1 链端就绪)
- 创建日期:2026-06-07
- 关联决策:`memory/04-decisions/ADR-016-account-key-pqc-migration.md`
- 关联设计:`memory/05-modules/wuminapp/wallet/WALLET_TECHNICAL.md §13`、`memory/05-modules/wumin/COLD_WALLET_PQC_TECHNICAL.md`
- 上层关系:范围 A 第 2 步。**Step 1 = `20260607-pqc-step1-chain-runtime-node.md`(区块链)**,本卡 = Step 2(钱包)。
- 前置依赖:Step 1 的链端 `account-keys` pallet(`bind_pqc_key` / `pqc_dispatch`)、协议 `sig_alg` 枚举、`gmb-pqc` crate 已就绪。

## 0. 目标

让 `wuminapp`(热钱包)与 `wumin`(冷钱包)从同一助记词派生 ML-DSA-65,完成账户的 PQC 绑定与 PQC 签名交易,全程**不换助记词/账户/地址/余额**。前期账户仅 sr25519,后期自然切 PQC 唯一签名。

## 1. 共用底座
- 复用 Step 1 的 **`gmb-pqc`** crate(派生/签名/验签),冷热钱包经原生 C FFI 调用,**派生规则与 node 强制一致**(铁律)。
- `AccountRootSeedV1` = 现有 `CryptoScheme.miniSecretFromEntropy` 的 32B 输出(`wuminapp/lib/wallet/core/wallet_manager.dart:541-545`),语义升格、值不变 → sr25519 地址不变。

## 2. wuminapp(热钱包)
- 派生:`wallet_manager.dart:547-555` 旁加 `_deriveMlDsa65FromSeed`(走 `gmb-pqc` FFI);FFI 落点 `wuminapp/rust/src/ml_dsa.rs`,照现有 `#[no_mangle] unsafe extern "C"` 风格(`wuminapp/rust/src/lib.rs:52-133`)。
- 绑定:调 Step 1 链端 `AccountKeys.bind_pqc_key`(外层 sr25519 + 内层 ML-DSA-65 自签 challenge)。
- PQC 交易:`signed_extrinsic_builder.dart:103,186` 除 `SignatureType.sr25519` 外,新增 general-transaction 构造,调 `AccountKeys.pqc_dispatch`。
- 二维码:`qr/bodies/{sign_request,sign_response,login_receipt}_body.dart` 的 `sig_alg` 硬编码(`:37-40`/`:46-47`)放开为枚举 `sr25519 | ml-dsa-65`。
- UI:只展示一个账户/地址/余额;账户状态机(Sr25519Only→Bound→PqcOnly)呈现为"抗量子升级"开关,不改地址展示。
- 本地存储:`AccountRootSeedV1` 设备加密保存、设备解锁;不出本机。

## 3. wumin(冷钱包)
- 同源派生(共用 `gmb-pqc`),离线生成 ML-DSA-65 密钥;纯助记词冷用户重新导入即可派生 PQC 钥匙完成绑定。
- 离线签名:`bind_pqc_key` 双签(外层 sr25519 + 内层 ML-DSA-65)、`pqc_dispatch` 的 ML-DSA-65 签名,全部离线完成,不依赖在线服务。
- `wumin/lib/signer/` 解码/签名路径扩展 `sig_alg` 枚举;`payload_decoder` 增加 `bind_pqc_key` / `pqc_dispatch` 解码分支,展示验真字段与链上 call 字面一致。
- 二维码协议一致性通过 `qr-protocol-spec.md` + fixtures 强制对齐(冷热各自实现,不共享 Dart 代码)。

## 4. 协议
- 对齐 `unified-protocols.md` 已登记的 P-QR-002(`sig_alg` 枚举)、P-TX-008(`bind_pqc_key`)、P-TX-009(`pqc_dispatch`)、P-SIGN-001(general-tx)。
- 新增 QR fixtures:ML-DSA-65 的 `sign_request/sign_response/bind_pqc_key/pqc_dispatch` 样例,冷热两端对齐测试。

## 5. 必须遵守
- 冷热钱包**同一套派生规则**,不允许只改一侧(铁律)。
- Passkey 本轮不纳入。
- 改 QR 协议 / extrinsic 编码前先核对 `unified-protocols.md` 登记项。
- 不搞兼容双轨;account 状态机自然过渡。

## 6. 待定(实现期)
- 热钱包是否只保存加密后的 `AccountRootSeedV1`(任务卡安全原则倾向)而非长期明文助记词。
- ML-DSA-65 在移动端的 keygen/sign 性能(Rust FFI,需实测)。

## 验收标准
- 一套助记词在 wuminapp 与 wumin 恢复同一账户主体;sr25519 地址不变。
- 可完成 `bind_pqc_key` 绑定与 `pqc_dispatch` PQC 转账,UI 单账户单余额。
- 冷钱包离线完成 ML-DSA-65 签名;`AccountRootSeedV1` 不出本机。
- 冷热 fixtures 对齐;flutter 测试通过;残留 sr25519 硬编码清理;文档回写。
