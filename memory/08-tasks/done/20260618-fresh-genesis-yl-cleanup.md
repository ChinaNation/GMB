任务需求：
走重新创世路线，彻底把旧省命名、旧省域名和旧省派生地址从全仓库代码、注释、文档、数据、生成资产和缓存中清理，统一更新为 YL/伊犁省，并重新生成依赖行政区与公权机构的数据包。

所属模块：
- citizenchain/runtime
- citizenchain/node
- sfid/backend/china
- sfid/backend/gov
- wuminapp
- wumin
- cpms
- memory

输入文档：
- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/workflow.md
- memory/07-ai/definition-of-done.md
- memory/07-ai/pre-submit-checklist.md
- memory/05-modules/sfid/backend/china/CHINA_TECHNICAL.md
- memory/05-modules/citizenchain/node/NODE_TECHNICAL.md
- memory/01-architecture/wuminapp/WUMINAPP_TECHNICAL.md
- memory/07-ai/chainspec-frozen.md
- memory/04-decisions/ADR-021-admin-district-single-source.md

必须遵守：
- 不可突破模块边界。
- 不可保留旧省命名、旧省域名、旧地址、旧注释、旧文档或旧数据残留。
- 不走兼容迁移；本任务按重新创世处理。
- 行政区唯一真源仍为 sfid/backend/china/china.sqlite，不恢复 sfid/backend/china/data/。
- 涉及 citizenchain/runtime/** 的任何写入必须先单独列明完整路径、预计改动内容和原因，并取得用户第二次确认。
- 修改代码后必须更新文档、完善中文注释、清理残留。
- 涉及数据库、链端资产、客户端数据包时必须执行真实校验。

输出物：
- 更新后的行政区 SQLite 开发库。
- 更新后的链端常量、派生地址和重新创世 chainspec。
- 更新后的 wuminapp 行政区包、公权机构包和治理机构注册表。
- 更新后的 wumin 公权机构表。
- 更新后的 memory 技术文档、ADR 和任务记录。
- 残留扫描报告。

验收标准：
- `rg`/`git grep` 全仓不再命中旧省 SFID、旧省中文名、旧省域名。
- `sfid/backend/china/china.sqlite` 业务表和二进制内容均不再含旧省文案。
- `sfid/backend/china/check_code_immutable.py` 通过。
- runtime 新地址由 `YL001-*` 派生，`china_zb.rs` 不含旧地址。
- node raw chainspec 与 wuminapp chainspec 使用同一份重新创世结果。
- wuminapp 行政区与公权机构数据包由修正后的唯一真源重新生成。
- 必要 Rust/Dart/Node 校验通过；涉及服务读取的部分完成真实本地验收或记录阻塞原因。

执行记录：
- 2026-06-18：创建跨模块任务卡并完成执行前必读文档装载。
- 2026-06-18：行政区开发库完成旧省命名残留清理并执行 `VACUUM`;后续重新创世基线任务已将行政区版本恢复为 1。
- 2026-06-18：通过 `check_code_immutable.py`，重新生成 `wuminapp/assets/admin_divisions/` 行政区随包快照。
- 2026-06-18：节点权威目录和两份 chainspec 的伊犁省 bootnode 域名更新为 `prcyls.crcfrcn.com`。
- 2026-06-18：更新行政区、chainspec 与重新创世相关文档，清理文档中的旧省字面残留。
- 2026-06-19：用户二次确认 runtime 修改后，更新 `citizenchain/runtime/primitives/china/` 中伊犁省 6 个保护机构的名称、SFID 号和派生地址；`python3 tools/duoqian.py` 干运行确认 0 变更。
- 2026-06-19：重新生成 fresh raw chainspec，`citizenchain/node/chainspecs/citizenchain.raw.json` 与 `wuminapp/assets/chainspec.json` 完全一致，sha256 均为 `cdf74fd89148ab8d681b020c65f59ff8f93e238a1404da44a7b47fae8bb4757a`。
- 2026-06-19：执行 SFID 公权机构运行库对账：`reconcile-gov --changed-only` 结果 `scopes=43 inserted=6 updated=249403 account_inserted=498861 removed=0`；`check-gov --strict` 结果 `ok=true manifest_current=true target_count=249409 active_count=249409 missing=0 mismatched=0 missing_accounts=0 obsolete=0`。
- 2026-06-19：通过真实 SFID HTTP 接口重新导出 `wuminapp/assets/public_institutions/`;后续任务 `20260619-wuminapp-public-all-gov` 已修正为完整公权目录，manifest `version=1`，43 省共 249343 条公民端公权机构，伊犁省 1737 条。
- 2026-06-19：删除 `wuminapp/build/` 中旧 chainspec 生成缓存；全仓含缓存残留扫描不再命中旧省中文名、旧省 SFID、旧省域名或旧派生地址。
- 2026-06-19：完成验收：runtime release build、node fresh chainspec 导出、chainspec SSOT 守卫、行政区不可变校验、SFID gov strict 校验、wuminapp 公权/行政区测试、wumin 签名端测试、runtime primitives 测试均通过。
