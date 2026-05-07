# P0-5 统一 Step2D fixture 真源

## 任务目标

执行重新创世前总审计 P0-5：统一 `wumin` / `wuminapp` Step2D fixture，消除 `cast_referendum` 旧 `VotingEngine(9).call_index=2` 与当前 `JointVote(23).cast_referendum(1)` 的漂移。

## 当前真源

- Runtime pallet：`citizenchain/runtime/src/lib.rs` 中 `JointVote = pallet_index 23`
- Runtime call：`citizenchain/runtime/votingengine/joint-vote/src/lib.rs` 中 `cast_referendum = call_index 1`
- 统一协议入口：`memory/07-ai/unified-protocols.md`
- 统一 fixture：`memory/06-quality/fixtures/step2d_credential_payload.json`

## 预计修改目录

- `memory/07-ai/`：登记 `JointVote.cast_referendum` 交易载荷协议；只涉及文档。
- `memory/06-quality/fixtures/`：新建 Step2D 统一 fixture 真源；只放测试数据。
- `memory/08-tasks/open/`：记录 P0-5 执行范围、结果和验收；只涉及文档。
- `wumin/test/`：修改冷钱包测试读取统一 fixture，并断言 `23.1 / 0x1701`；涉及测试。
- `wuminapp/test/`：修改热钱包测试读取统一 fixture，并断言 `23.1 / 0x1701`；涉及测试。
- `wumin/test/fixtures/`：删除重复 Step2D fixture；残留清理。
- `wuminapp/test/fixtures/`：删除重复 Step2D fixture；残留清理。

## 执行清单

- [x] 在统一协议文件登记 `P-TX-002：JointVote.cast_referendum`。
- [x] 新建 `memory/06-quality/fixtures/step2d_credential_payload.json` 统一 fixture。
- [x] 修正 `cast_referendum` metadata 为 `pallet_index=23 / call_index=1`。
- [x] `wumin` 测试改读统一 fixture 并补 `23.1 / 0x1701` 断言。
- [x] `wuminapp` 测试改读统一 fixture 并补 `23.1 / 0x1701` 断言。
- [x] 删除 `wumin/test/fixtures/step2d_credential_payload.json`。
- [x] 删除 `wuminapp/test/fixtures/step2d_credential_payload.json`。
- [x] 回写审计文档并运行验收。

## 验收标准

- `rg -n '"pallet_index": 9|"call_index": 2|0x0902|test/fixtures/step2d_credential_payload\\.json' wumin/test wuminapp/test memory/06-quality` 不再命中旧 Step2D 漂移。
- `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart` 通过。
- `flutter test test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart` 通过。
- `flutter analyze test/signer test/proposal` 通过。
- `git diff --cached --check` 通过。

## 执行结果

2026-05-07 已执行：

- 新增统一 fixture：`memory/06-quality/fixtures/step2d_credential_payload.json`。
- 删除两端重复 fixture：
  - `wumin/test/fixtures/step2d_credential_payload.json`
  - `wuminapp/test/fixtures/step2d_credential_payload.json`
- `cast_referendum` 已统一为 `JointVote(23).cast_referendum(1)`，fixture 前缀固定 `0x1701`。
- `wumin/test/signer/payload_decoder_test.dart` 改读统一 fixture，并断言 `pallet_index=23 / call_index=1 / 0x1701`。
- `wuminapp/test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart` 改读统一 fixture，并断言 `pallet_index=23 / call_index=1 / 0x1701`。
- 统一 fixture 时同步修正 `propose_resolution_issuance` 测试数据中第二个 `recipient` 多 1 字节的问题：该条现在实际长度与 `expected_byte_length=253` 一致。

验收记录：

- `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart`：通过。
- `flutter test test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`：通过。
- `flutter analyze test/signer`：通过。
- `flutter analyze test/proposal`：通过。
- `rg -n '"pallet_index": 9|"call_index": 2|0x0902|test/fixtures/step2d_credential_payload\\.json' wumin/test wuminapp/test memory/06-quality`：无输出。
