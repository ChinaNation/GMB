# PQC card3:钱包派生(sr25519 直接)+ ML-DSA 签名 + 冷钱包 FFI + 离线 metadata + QR 分片

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§2/§7/§8)
状态:open(依赖 card1 spike S2/S3 全绿)

任务需求(冷热钱包在位升级 PQC 签名,地址逐字节不变):
1. **同源派生(只两支,sr25519 不套 HKDF)**:`AccountSeedV1`=现有 miniSecretFromEntropy(`citizenapp wallet_manager.dart:544`/`citizenwallet:401`);sr25519 锚点沿用 `fromSeed` 直接派生;ML-DSA-65 = `HKDF-SHA512(AccountSeedV1,"GMB/account/ml-dsa-65/seed32/v1")`→ξ→keygen。**账户不派生 ML-KEM(决策3)**。冷热逐字一致,钉 golden vector(含 ξ)。
2. 🔴 **(B9)冷钱包 citizenwallet 新建 Rust FFI 子工程**(现纯 Dart 无 rust/):对标 citizenapp/rust 的 cdylib/staticlib + Android/iOS target + cbindgen,把 gmb-pqc 编进冷热两端;FFI 暴露 `ml_dsa65_public_from_seed`/`ml_dsa65_sign`。
3. 🔴 **(B2)General Transaction 编码**:polkadart 0.7.1 只编 legacy 0x84,**不能编 v5 General Transaction** → 按 card1 S2 结论(fork/patch polkadart 或自写 v5 SCALE 编码器:v5 preamble + extension_version + 嵌套 `(GmbPqcAuth,AuthorizeCall)` extra);PQC proof 入 extra,**不走 MultiSignature、不走 pqc_dispatch pallet call**(`signed_extrinsic_builder.dart:103/186`)。
4. 🔴 **(B10)冷钱包离线 metadata 策略**:"按 metadata 重建 `following_extensions_hash`" 整体留在线热钱包;QR 携带重建所需最小要素(extension_version + 各后续扩展显式值 nonce/tip/era/spec/tx_version/genesis + 预算 hash),**冷钱包用 gmb-pqc 本地 SCALE 重算并比对**,且**自己从助记词派生 ML-DSA 公钥核对 `pqc_pubkey_hash`**(不盲信 QR 公钥);**严禁退化成 wasm 式纯哈希盲签**,保持两色识别·decodeFailed 即拒。
5. 🔴 **(B11)QR 分片**:envelope 加 `chunk_index/chunk_total/total_hash`;渲染端多帧轮播(单帧字节按 ECC=M version≤40 反推留裕量);扫描端分片聚合状态机(按 id 归并/去重补帧/缺帧提示/校验 total_hash 再解析);放开 32768 上限;最坏 bootstrap(sr25519 64B+ML-DSA 3309B+公钥 1952B+call,hex≈10KB+)真机实测。
6. **查 `PqcPolicy` 分流**:Sr25519Only→sr25519 extrinsic;PqcPrepared/PqcPrimary→PQC General Transaction;未绑定→首笔 bootstrap(同一确认出 sr25519 bootstrap 签名 + ML-DSA 交易签名 + 携带 ML-DSA 公钥)。
7. **QR body 四处(冷热 request/response)放开** `sig_alg(sr25519|ml-dsa-65)`+`auth_mode`+`key_version`+`chunk_*`,Phase A 仍只收 sr25519;🔴 **进签名的 hash 一律 gmb-pqc blake2_256,禁复用 `qr_signer` 的 sha256**(`qr_signer.dart:180`)。
8. **(H15)登录签名响应 sign_response** 也是用户钱包签名面,纳入本卡按 sig_alg 升级。
9. UI 不展示 PQC 公钥/绑定过程/换账户;bootstrap 首笔冷钱包确认页仍展示业务 call 中文摘要(校验不缺位,见 4);同助记词恢复同地址+同 ML-DSA 密钥。

所属模块:Mobile(citizenapp 热钱包 + citizenwallet 冷钱包)

输入文档:ADR-022 / unified-protocols(QR + pqc 协议)/ citizenapp/citizenwallet 完成标准

必须遵守:`AccountSeedV1` 不变;**sr25519 分支绝不套 HKDF**;冷热派生/QR 字段逐字一致;冷钱包保持离线(含 bootstrap 离线扫码)且不盲签;UI 不暴露多算法概念。

输出物:双端派生(sr25519 直接+ML-DSA HKDF)+ 冷钱包 FFI 子工程 + v5 General Transaction 编码 + 离线 metadata 重算 + QR 分片(冷热)+ sign_response 升级 + 中文注释 + 测试(golden vector 含ξ / bootstrap 往返 / QR 分片真机)+ 文档。

验收标准:
- 同助记词冷热恢复同一 AccountId,sr25519 地址逐字节不变;Dart 重建 following_extensions_hash 与链端逐字节一致(card1 golden vector)。
- Dart 产出的 v5 General Transaction 被链端 GmbPqcAuth 接受;未绑定首次 bootstrap 扫码(含分片)成功;后续 ML-DSA 成功。
- 冷钱包离线本地重算 + 派生核对 pqc_pubkey_hash 通过,不盲签;QR 分片真机最坏 bootstrap 可扫;残留 sr25519/sha256/pqc_dispatch 命名清零。
