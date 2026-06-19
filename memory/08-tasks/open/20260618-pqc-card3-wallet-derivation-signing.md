# PQC card3:钱包同源派生 + ML-DSA 签名 + QR/extrinsic

关联决策:`memory/04-decisions/ADR-022-unified-pqc-crypto.md`(§2 §4)
状态:open(依赖 card1 gmb-pqc、card2 链端 pqc_dispatch)

任务需求:
冷热钱包(wuminapp/wumin)同源派生三套密钥 + ML-DSA 签名,**地址锚点不变**:
1. **AccountSeedV1 HKDF 三分叉**:`wuminapp/lib/wallet/core/wallet_manager.dart:541-555` + `wumin/lib/wallet/wallet_manager.dart:398-417`,在 mini-secret 后接 HKDF 派生 sr25519(锚点,逐字节不变地址)/ ML-DSA-65(签名)/ ML-KEM-768(加密)。冷热**逐字一致**。
2. **Dart ML-DSA/ML-KEM 经 gmb-pqc FFI**(无纯 Dart 实现;wuminapp 已有 `rust/` + smoldot-pow FFI 先例)。
3. **签名出口换 ML-DSA**:`wuminapp wallet_manager.dart:616/639/671` + `wumin:346/382`。
4. **extrinsic 走 pqc_dispatch**:`wuminapp/lib/rpc/signed_extrinsic_builder.dart:103/186`(原 `SignatureType.sr25519`)改为构造 account-keys.pqc_dispatch general-tx 带 ML-DSA 签名(**不走 MultiSignature**)。
5. **QR 冷签协议 sig_alg 升级**:`wumin/lib/qr/bodies/sign_request_body.dart:40` + `sign_response_body.dart:36` + `qr_signer.dart:118`(buildResponse)+ `:134-151`(验签加 ML-DSA 分支)+ wuminapp 对称镜像,共 4+ 处硬编码 `sr25519` 同步改 `ml-dsa-65`(漏一处冷热口径裂)。
6. **QR 容量**:ML-DSA 签名 ~3.3KB vs `payload_hex` 32768 上限 + 二维码物理密度,出方案(分片 / 哈希+带外 / NFC)。

所属模块:Mobile(wuminapp 热钱包 + wumin 冷钱包)

输入文档:
- memory/04-decisions/ADR-022-unified-pqc-crypto.md
- memory/07-ai/unified-protocols.md(P-QR sig_alg / P-SIGN / P-TX-009)
- wuminapp / wumin 模块完成标准

必须遵守:
- 冷热同一派生规则,domain 标签 / 顺序逐字一致(否则地址漂移)。
- sig_alg 四处 + extrinsic 两处同步改,零遗漏。
- 不暴露多公钥概念给用户(仍一个账户一个地址一份余额)。
- 冷钱包保持离线签名能力。

输出物:
- 双端派生 / 签名 / QR / extrinsic 改造 + Rust FFI + 中文注释 + 测试(同源派生 fixture / QR 往返 / 签名验签)+ 文档

验收标准:
- 一套助记词冷热恢复同一 AccountId,sr25519 地址逐字节不变;ML-DSA 签名被链端 pqc_dispatch 验签通过。
- QR sig_alg=ml-dsa-65 冷热往返 OK;QR 容量方案落地、真机扫码验收。
- 残留 sr25519 硬编码按域收敛。
