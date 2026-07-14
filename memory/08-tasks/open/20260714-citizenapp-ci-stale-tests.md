# 修复 CitizenApp CI 过期管理员变更测试

任务需求：
- 修复 CitizenApp CI 中与当前管理员变更边界不一致的测试。

所属模块：
- `citizenapp`

输入文档：
- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/03-security/security-rules.md`
- `memory/04-decisions/ADR-015-account-admin-internal-vote.md`
- `memory/05-modules/citizenapp/governance/GOVERNANCE_TECHNICAL.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`

必须遵守：
- `adminsChange` 只允许个人多签 `PMUL` 使用。
- 不得恢复机构管理员集合直接变更入口。
- 不修改 `citizenchain/runtime/`。
- 不触碰 GitHub 远端或重新触发 workflow。

输出物：
- 修正机构主体的过期测试断言。
- 增加个人多签管理员变更正向测试。
- 更新治理技术文档中的测试门禁说明。
- 完成测试、注释和残留检查。

验收标准：
- `citizenapp/test/common/institution_info_test.dart` 通过。
- CitizenApp 全量测试通过。
- `flutter analyze` 通过。
- 文档已更新，旧测试口径已清理。

当前进度：
- [x] CI 失败日志和根因已确认。
- [x] 新增任务卡已获用户明确许可。
- [x] 修正测试并补正向覆盖。
- [x] 执行局部测试、全量测试和静态分析。
- [x] 更新文档、检查注释和清理残留。

执行记录：
- 2026-07-14：将 NRC、城市注册局和私权机构的 `adminsChange` 断言改为禁止，并新增 `PMUL` 允许该能力的正向测试。
- 2026-07-14：局部测试 8 项全部通过；全量测试 546 项通过、5 项跳过。
- 2026-07-14：`flutter analyze` 成功完成，保留两条与本次无关的既有 info 级提示；`git diff --check` 通过。
- 2026-07-14：治理技术文档已登记对应回归测试门禁，未发现本次产生的临时文件、兼容分支或旧断言残留。
