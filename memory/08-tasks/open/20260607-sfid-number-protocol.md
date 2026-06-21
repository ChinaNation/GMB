任务需求：

将 SFID 系统身份 ID 协议改为目标态 `R5-K3P1C1-N9-D4`，统一所有主体身份字段为 `sfid_number`，删除历史主体属性段、历史身份字段别名、历史示例、历史注释和历史文档残留。机构账户地址继续统一按 `DUOQIAN` 规则派生。

所属模块：

- SFID
- citizenchain
- CPMS
- citizenwallet
- citizenapp

输入文档：

- `memory/07-ai/unified-required-reading.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/workflow.md`
- `memory/07-ai/definition-of-done.md`
- `memory/07-ai/pre-submit-checklist.md`
- `memory/07-ai/module-definition-of-done/sfid.md`
- `memory/05-modules/sfid/backend/number/NUMBER_TECHNICAL.md`
- `memory/01-architecture/sfid/SFID_TECHNICAL.md`

必须遵守：

- 不做历史格式兼容。
- 不保留历史字段、历史注释、历史示例或双轨逻辑。
- 不突破模块边界；SFID 负责身份号码协议生成与校验，citizenchain 负责链端常量、链上登记与 DUOQIAN 地址派生一致性，CPMS/citizenwallet/citizenapp 只消费目标态 `sfid_number`。
- SFID 不保存原始实名数据。
- 改代码后必须更新文档、完善中文注释、清理残留。

输出物：

- SFID 后端代码更新。
- SFID 前端代码更新。
- citizenchain 内置机构常量、清算行判定、节点端 DTO 更新。
- CPMS 导出、初始化样例与模块文档更新。
- citizenwallet/citizenapp 内置机构映射、签名 payload 样例、MyID 字段名更新。
- SFID 编号协议文档更新。
- 统一协议、统一命名登记与跨系统文档更新。
- 中文注释完善。
- 残留扫描与必要测试结果。

验收标准：

- 全系统只接受/展示 `R5-K3P1C1-N9-D4` 新格式。
- 历史五段身份号码格式不再作为有效格式存在。
- 全系统主体身份字段统一为 `sfid_number`。
- 当前代码、当前文档、测试样例和注释中不再出现历史字段别名、历史主体属性段或历史号码示例残留。
- 机构账户地址派生仍与链端 `DUOQIAN` 规则一致；内置机构地址随新 `sfid_number` 重算。
- 文档已更新，中文注释已完善，残留已清理。
- 已运行必要测试或说明未运行原因。

完成记录：

- 已按目标态删除历史主体属性段、历史字段别名、历史号码示例、历史注释和历史文档残留。
- 已删除历史生成/迁移脚本与历史 remap 质量文件。
- 已将 CPMS 安装码省市解析统一为从第一段 R5 读取，清理旧段位取值残留。
- 已将 SFID/citizenchain/CPMS/citizenwallet/citizenapp 的测试样例和 DTO 命名统一到 `sfid_number` 与 `subject_property`。
- 已重新生成 citizenchain 前端本地文档产物。
- 已执行源码/文档残留扫描：目标扫描项全部未命中。

验证结果：

- `cargo test --manifest-path sfid/backend/Cargo.toml` 通过。
- `cargo test --manifest-path cpms/backend/Cargo.toml` 通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p offchain-transaction --tests` 通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p duoqian-transfer --tests` 通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p sfid-system --tests` 通过。
- `cargo check --manifest-path sfid/backend/Cargo.toml` 通过。
- `cargo check --manifest-path cpms/backend/Cargo.toml` 通过。
- `cargo check --manifest-path citizenchain/node/Cargo.toml` 通过。
- `npm run build` 于 `sfid/frontend` 通过。
- `npm run build` 于 `citizenchain/node/frontend` 通过。
- `flutter test test/signer/payload_decoder_test.dart` 于 `citizenwallet` 通过。
- `flutter test` 于 `citizenapp` 指定相关测试集通过。
- `git diff --check` 通过。
