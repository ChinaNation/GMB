任务需求：
citizenapp 电子护照页面展示身份ID号码与身份ID状态，并将“绑定账户”改为“投票账户”；SFID 状态接口补充身份ID状态。当前任务只完成 citizenapp 与 SFID 的直接绑定展示闭环，不做投票拦截。

状态：
已执行

所属模块：
citizenapp 电子护照 / SFID 公民身份绑定

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/unified-required-reading.md
- memory/07-ai/agent-rules.md
- memory/07-ai/chat-protocol.md
- memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md
- memory/01-architecture/sfid/SFID_TECHNICAL.md

必须遵守：
- 绑定状态与身份ID状态是两个不同字段，不得混用。
- 绑定状态继续表示 `unset / pending / bound`。
- 身份ID状态由 SFID 返回，`NORMAL` 显示正常，其他状态显示异常。
- 当前任务不做 citizenapp 投票拦截、SFID 公民数量拦截、SFID 投票凭证拦截、链上投票引擎快照拦截。
- 改代码后同步更新文档、完善必要中文注释、清理残留。

输出物：
- SFID 公民记录保存身份ID状态。
- SFID citizenapp 状态接口返回身份ID号码与身份ID状态。
- citizenapp 电子护照页展示身份ID、投票账户、身份ID状态。
- 相关测试、文档和残留清理。

验收标准：
- 未绑定时身份ID显示“未绑定”。
- 已绑定时身份ID显示 SFID 返回的身份ID号码。
- 投票账户标签替换原“绑定账户”。
- 身份ID状态为 `NORMAL` 时显示“状态：正常”，其他值显示“状态：异常”。
- 绑定状态徽标仍按原绑定状态展示。
- 相关测试和静态检查通过，若无法运行需说明原因。

执行记录：
- 已在 SFID `CitizenRecord` 增加 `identity_status`，绑定 CPMS 档案二维码时保存 CPMS `cs` 状态；`NORMAL` 保存正常，其他状态保存异常。
- 已在 `GET /api/v1/app/vote-account/status` 返回 `identity_status`，并保持 `status` 只表示绑定状态。
- 已在 citizenapp `MyIdStatusResponse` / `MyIdState` / `SharedPreferences` 缓存中增加 `sfidNumber` 与 `identityStatus`。
- 已将电子护照页面新增“身份ID”显示，将“绑定账户”改为“投票账户”，并在账户地址下方显示“状态：正常/异常”。
- 已同步更新 citizenapp 与 SFID 架构文档、citizenapp 用户模块文档。
- 已新增 SFID 单测覆盖绑定状态与身份ID状态分离序列化。
- 已新增 citizenapp 页面测试覆盖身份ID、投票账户和正常状态展示。
- 验证：`cargo test --manifest-path sfid/backend/Cargo.toml` 通过；`flutter test test/myid_page_test.dart test/sfid_api_config_test.dart` 通过；`flutter analyze` 通过；格式化和残留搜索已完成。
