# 任务卡：CitizenWallet 补齐占号/吊销 CID 冷签解码(占号扫码转绿)

## 任务需求

来源:用户报「链上中国注册局新增公民,注册局占号签名用公民钱包扫码后红色禁止签名」。

只读诊断根因:注册局「占号」在链上是 `CitizenIdentity(10).occupy_cid(call_index=6)` 标准 extrinsic,onchina 后端已正确构造并二维码化(`occupy.rs:191-208 encode_occupy_cid_call`、QR 动作码 `b.a = chain_action_code(10,6) = 0x0A06`);但 CitizenWallet 的 `PayloadDecoder` 的 `CitizenIdentity(10)` 分支**只登记了 call 0/1**(register_voting_identity / upgrade_to_candidate_identity),没有 call 6。解码返回 `null` → `OfflineSignService` 严格两色模式判 `decodeFailed` → 红色「无法独立验证交易内容,禁止签名」。这是「未知交易=禁止盲签」分支,非布局/校验失败。

同源缺登记:吊销 `revoke_cid(call_index=8)`(`0x0A08`,`occupy.rs:210-217`)冷钱包同样缺分支,吊销扫码也会红。一并纳入。

## 所属模块

CitizenWallet(公民钱包冷签)。**只改 citizenwallet**:onchina 后端、citizenchain 链端均正确无需改;citizenapp 是精简版 `QrProtocol`(故意不带链交易常量,不签占号)不涉及。

## 输入文档

- 生成侧权威布局:`citizenchain/onchina/src/domains/citizens/occupy.rs:191-217`
- 链端定义:`citizenchain/runtime/misc/citizen-identity/src/lib.rs`(occupy_cid call_index=6:840、revoke_cid call_index=8:915)
- 判红逻辑:`citizenwallet/lib/signer/offline_sign_service.dart:57-96`
- [[project_cid_occupy_registry_2026_07_02]]、[[project_citizen_identity_strict_auth_vote_gates]]、[[project_unified_signing_protocol_adr026]]、[[project_qr_signing_two_color]]

## 范围(冷钱包三处登记 + 解码 + 测试)

occupy_cid(10.6)/revoke_cid(10.8)两条 call,冷钱包三处补齐:
1. `lib/signer/pallet_registry.dart`:CitizenIdentity 段补 `occupyCidCall = 6` / `revokeCidCall = 8` 常量。
2. `lib/signer/payload_decoder.dart`:`decode()` 的 CitizenIdentity(10) 分支加 call 6/8 路由 + 新增 `_decodeOccupyCid` / `_decodeRevokeCid`。逐字节对齐 onchina 布局:
   - occupy_cid:`[10][6] registrar[32] · cid_number:Vec · commitment[32] · residence_province_code:Vec · residence_city_code:Vec`(尾部 `_hasCallDataEnd`)。
   - revoke_cid:`[10][8] registrar[32] · cid_number:Vec`。
   - reviewFields 只用已登记 key(`registrar_account`/`cid_number`/`residence`),`field_labels.dart` 无需改;`commitment`/区码入机器 `fields`。
3. `lib/qr/qr_protocols.dart`:补 `occupyCid = 0x0a06` / `revokeCid = 0x0a08` 常量 + `fromDecodedAction` 加 `'occupy_cid'` / `'revoke_cid'` 条目。

测试:
- `test/signer/payload_decoder_test.dart`:占号/吊销 decode 用例(裸 call_data + 带签名尾各一)。
- `test/signer/pallet_registry_test.dart`:补 occupyCidCall=6 / revokeCidCall=8 断言。

## 铁律

- 逐字节对齐 onchina `encode_occupy_cid_call` / `encode_revoke_cid_call`([[project_unified_signing_protocol_adr026]])。
- 客户端按 pallet+call 路由解码,不撞名。
- 只在主检出 `/Users/rhett/GMB` 操作,不碰 worktree([[feedback_user_evaluates_in_main_checkout]]);改动留工作区不提交供 review。
- 不做兼容/迁移,链开发期直接改([[feedback_chain_dev_never_ask_migration]]);不附加范围外子任务([[feedback_no_scope_expansion]])。

## 验收标准

- `flutter test test/signer/payload_decoder_test.dart` GREEN(占号裸/带尾、吊销均绿签,action=`occupy_cid`/`revoke_cid`)。
- `flutter test test/signer/pallet_registry_test.dart` GREEN。
- `flutter analyze` 零新增。
- 手工核对:占号 QR(b.a=0x0A06)扫码从红转绿,吊销(0x0A08)同。
- 残留清零,文档/记忆回写。

## 进度(2026-07-15 已完成)

CitizenWallet 三处 + 测试全部落地,只改 citizenwallet:

- `pallet_registry.dart`:CitizenIdentity 段补 `occupyCidCall=6` / `revokeCidCall=8`。
- `payload_decoder.dart`:`decode()` CitizenIdentity(10) 分支加 call 6/8 路由;新增 `_decodeOccupyCid`(registrar[32]·cid:Vec·commitment[32]·province:Vec·city:Vec)/ `_decodeRevokeCid`(registrar[32]·cid:Vec),逐字节对齐 onchina `encode_occupy_cid_call`/`encode_revoke_cid_call`;尾部 `_hasCallDataEnd`(裸 call_data + 带签名尾都接);reviewFields 只用已登记 key(registrar_account/cid_number/residence),commitment/区码入机器 fields,`field_labels.dart` 未改。
- `qr_protocols.dart`:补 `occupyCid=0x0a06` / `revokeCid=0x0a08` 常量 + `fromDecodedAction` 加 `'occupy_cid'`/`'revoke_cid'` 条目。
- 测试:`payload_decoder_test.dart` 4 例(占号/吊销 × 裸/带尾)、`pallet_registry_test.dart` 1 例(10.0/1/6/8 断言)。

**验证:** `flutter test test/signer/` **111/111 GREEN**(含 field_labels reviewFields 覆盖);`flutter analyze` 5 改动文件零问题。动作码闭环:onchina `chain_action_code(10,6)=0x0A06`、`(10,8)=0x0A08` 与钱包常量逐位一致 → 严格两色判 matched 绿签。onchina/链端/citizenapp 均未改(前二正确,后者精简版 `QrProtocol` 不签占号)。
