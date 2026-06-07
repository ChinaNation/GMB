# 任务卡:修复 personal-manage 高风险与结构残留

## 任务需求

修复个人多签 `personal-manage` 审查发现的问题：

- 拒绝回调与手动 cleanup 重复发 `DuoqianCreateRejected`。
- 创建提案未快照 fee，执行和清理阶段重算 fee。
- 执行失败终态缺少业务清理和可靠事件。
- `remove_pending_admin_account` 吞错。
- `PersonalDuoqians` / `PersonalDuoqianInfo` creator 冗余。
- runtime 配置重复校验、死参数、重复 action 解码、零权重占位、事件字段不一致等残留。

## 预计修改目录

- `citizenchain/runtime/governance/personal-manage/`：修复个人多签创建、关闭、清理、回调、权重和测试；涉及 Rust 代码、测试、注释和残留清理。
- `wumin/lib/`：离线签名端只解码 extrinsic call data，本次 ProposalData 增加 `fee` 不影响冷钱包签名载荷；涉及残留确认。
- `wuminapp/lib/`：同步热钱包端对个人多签创建 action 的解码；涉及 Dart 代码。
- `memory/05-modules/`：更新个人多签/治理模块技术文档；涉及文档。
- `memory/08-tasks/`：记录执行过程并归档；涉及任务文档。

## 验收标准

- 拒绝后的重复 cleanup 不再重复发拒绝事件。
- 创建提案的 fee 使用提案时快照，执行/清理不重算历史 fee。
- 执行失败终态能释放创建 reserve、清理 Pending 状态，并可靠发失败事件。
- `PersonalDuoqianInfo` 冗余 storage 被移除或 creator 冗余被消除。
- 死参数、重复配置校验、重复 action 解码被清理。
- 权重不再为 0。
- 文档、注释和客户端解码同步。
- 目标 Rust 测试、格式化、残留扫描通过；若有既有无关阻塞需记录。

## 执行记录

- [x] 确认影响面：链端 `personal-manage`、`duoqian-transfer` 测试 mock、wuminapp ProposalData/storage codec、模块文档和统一协议文件。
- [x] 修复高风险终态链路：拒绝 cleanup 幂等、创建 fee 快照、执行失败终态清理和失败事件。
- [x] 清理结构债和死代码：删除 `PersonalDuoqianInfo`/`PersonalDuoqianMeta`、删除 `_callback_context`、删除重复 runtime 配置校验、统一 action 解码、`remove_pending_admin_account` 不再吞错。
- [x] 补测试、权重、文档和客户端解码：新增执行失败、重复拒绝事件、protected/reserved、fee 快照测试；weights 改为保守非零；wuminapp storage/proposal decoder 同步。
- [x] 执行验收。

## 验收结果

- `cargo test --manifest-path citizenchain/Cargo.toml -p personal-manage --lib`：23 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p duoqian-transfer --lib`：22 passed。
- `cargo check --manifest-path sfid/backend/Cargo.toml`：通过。
- `flutter test test/duoqian/duoqian_manage_service_test.dart test/duoqian/duoqian_storage_codec_test.dart test/duoqian/duoqian_manage_storage_test.dart`：All tests passed。
- `cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain --lib`：未执行成功，runtime build.rs 要求 `WASM_FILE` 环境变量，提示必须使用 CI 统一 WASM，本地直接 check 被仓库规则阻止。

## 结论

本任务完成。未修改 `spec_version`；创世前 schema 以当前代码和统一协议文件为准。
