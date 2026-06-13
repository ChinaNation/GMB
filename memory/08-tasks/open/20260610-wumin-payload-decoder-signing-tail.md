# 任务卡：wumin 公民钱包扫码解码器签名扩展尾校验修复

## 背景(根因已查实)

桌面端发起国储会转账提案(`propose_transfer`, 19.0, org=0),wumin 扫码红色"无法独立验证交易内容,禁止签名"。

- QR 的 `payload_hex` 是**完整 SigningPayload**:call_data 后永远跟 ≥77 字节扩展尾
  `era(0x00) + Compact<nonce> + Compact<tip> + mode(0x00) + spec(4) + tx(4) + genesis(32) + birth(32) + None(0x00)`
  (节点端 `citizenchain/node/src/governance/signing.rs:628-677`;wuminapp 走 polkadart `signingPayload.encode` 同布局)。
- 提交 `84080b6a`(2026-06-07 重构多签账户体系)把多个解码分支改成"严格到尾/总长精确",对真实 payload 永远不成立 → decode 返回 null → `decodeFailed` → 红色拒签。
- 单测夹具只构造纯 call_data(无尾),所以 `flutter test` 全绿没拦住。

## 必红分支清单(修复目标)

| 分支 | pallet.call | 坏检查 |
|---|---|---|
| `_decodeProposeTransfer` | 19.0 | `offset+remarkLen != bytes.length` |
| `_decodeProposeSafetyFund` | 19.1 | 同上 |
| `_decodeProposeSweep` | 19.2 | `bytes.length != 50` |
| `_decodeProposeDestroy` | 14.0 | `bytes.length != 51` |
| `_decodeJointVote` | 23.0 | `bytes.length != 43` |
| `_decodeProposeCreateInstitution` | 17.5 | `offset != bytes.length` |
| `_decodeProposeCreatePersonal` | 7.0 | `offset+4+16 != bytes.length` |
| `_decodeProposeAdminSetChange` | 12.0 | `offset+4 != bytes.length` |
| `_decodeProposeKeyChange` | 16.0 | `bytes.length != 66` |

`internal_vote`(22.0)等只查最小长度的分支一直正常(容忍尾部),这解释了"投票能签、发提案必红"。

## 方案(只改 wumin,不动链端/节点端)

统一新增 `_hasValidSigningTail(bytes, callEnd)`:校验 call_data 在 callEnd 结束、其后是合法扩展尾(era 0x00 / nonce compact / tip compact / mode 0x00 / 固定 73B 且末字节 0x00)。**所有链上 extrinsic 分支**统一接上(含原本容忍尾部的分支,一并收紧);非链上 challenge 分支(管理员激活/解密/CPMS 删档/SFID 治理 JSON)保持原样;WASM 升级哈希直签例外路径不变。

测试:夹具加 `withSigningTail()` 拼真实尾部;每个链上分支补"无尾 → null"与"篡改尾 → null"用例。

## 验收

- [x] `flutter analyze` 0 issue
- [x] `flutter test` 全过 116/116(含新尾部用例:国储会绿色路径/大 nonce/裸 call_data 拒签/篡改尾拒签)
- [ ] 端到端:桌面端国储会转账提案 → wumin 扫码绿色 → 签名回传上链(user 真机验证)
- [ ] 回归扫码:安全基金/划转/决议销毁/联合投票/机构创建/个人多签创建/管理员变更/GRANDPA 换钥/内部投票

## 完工记录(2026-06-10,代码完工,待真机验证)

- `wumin/lib/signer/payload_decoder.dart`:新增 `_hasValidSigningTail(bytes, callEnd)`(era 0x00 → Compact nonce → Compact tip → mode 0x00 → 固定 73B,末字节 0x00,immortal 下 birth==genesis 逐字节比对);**全部链上 extrinsic 分支**统一接上(9 个必红分支修复 + 原容忍尾部的 transfer/internal_vote/finalize/proposalIdOnly/cast_referendum/resolution_issuance/清算行 7 类一并收紧);`_decodeCancelPassedProposal` 顺带修正"把尾部误读进 reason"。非链上 challenge 分支(激活/解密/CPMS 删档/SFID JSON)与 WASM 哈希直签例外不动。
- `wumin/test/signer/payload_decoder_test.dart`:新增 `signingTail()/withSigningTail()` 夹具,全部链上用例改带尾构造;step2d fixture(纯 call_data 真源)在 decode 前拼尾;新增 4 个回归用例锁死约定。
- `wumin/test/signer/offline_sign_service_test.dart`:3 个裸 call_data 夹具改带尾;顺带修掉 transfer 夹具里旧解码器容忍的垃圾尾字节 `0x91`。
- 链端/节点端 0 改动。
