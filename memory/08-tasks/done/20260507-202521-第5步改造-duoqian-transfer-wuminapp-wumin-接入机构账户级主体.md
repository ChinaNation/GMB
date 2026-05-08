# 任务卡：第5步改造-duoqian-transfer-wuminapp-wumin-接入机构账户级主体

- 任务编号：20260507-202521
- 状态：done
- 所属模块：citizenchain/runtime
- 当前负责人：Codex
- 创建时间：2026-05-07 20:25:21

## 任务需求

先执行第5步，改造 duoqian-transfer 多签转账、wuminapp 在线端、wumin 冷钱包，统一接入 0x05 InstitutionAccount 机构账户级内部投票主体；保持一人一票一笔交易；完成文档、注释、残留清理，且不修改 spec_version。第4步 organization-manage 后续再执行。

## 必读上下文

- memory/00-vision/project-goal.md
- memory/00-vision/trust-boundary.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/agent-rules.md
- memory/07-ai/context-loading-order.md
- memory/01-architecture/citizenchain-target-structure.md
- citizenchain/CITIZENCHAIN_TECHNICAL.md
- citizenchain/runtime/README.md

## 模块模板

- 模板来源：memory/08-tasks/templates/citizenchain-runtime.md

### 默认改动范围

- `citizenchain/runtime`
- `citizenchain/governance`
- `citizenchain/issuance`
- `citizenchain/otherpallet`
- `citizenchain/transaction`
- 必要时联动 `primitives`

### 先沟通条件

- 修改 runtime 存储结构
- 修改资格模型
- 修改提案、投票、发行核心规则


## 模块执行清单

- 清单来源：memory/07-ai/module-checklists/citizenchain.md

# CitizenChain 模块执行清单

- 开工前先确认任务属于 `runtime`、`node`（含桌面端）或 `primitives`
- 关键 Rust 或前端逻辑必须补中文注释
- 改动链规则、存储或发布行为前必须先沟通
- 如果改动 `runtime` 且会影响 `wuminapp` 在线端或 `wumin` 冷钱包二维码签名/验签兼容性，必须先暂停单边修改，转为跨模块任务
- 触发项至少检查：`spec_version` / `transaction_version`、pallet index、call index、metadata 编码依赖、冷钱包 `pallet_registry` 与 `payload_decoder`
- 未把 `wuminapp` 在线端和 `wumin` 冷钱包的对应更新纳入本次执行范围前，不允许继续 runtime 改动
- 文档与残留必须一起收口

## 模块级完成标准

- 标准来源：memory/07-ai/module-definition-of-done/citizenchain.md

# CitizenChain 完成标准

- 改动范围和所属模块清晰
- 关键逻辑已补中文注释
- 文档已同步更新
- 影响链规则、存储或发布行为的点都已先沟通
- 残留已清理


## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通
- 改代码后必须更新文档和清理残留

## 输出物

- 代码
- 中文注释
- 文档更新
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- 任务卡已创建
- 已完成 `duoqian-transfer` 转账主体拆分：`0x03 PersonalDuoqian` 走 `PersonalQuery::is_active`，`0x05 InstitutionAccount` 走 `InstitutionQuery::is_active`，`0x02 SfidInstitution` 拒绝作为转账支出主体。
- 已补充链端测试：机构账户 `0x05` 可完成内部投票转账，`0x02` 不能作为转账来源；统一重试相关测试名称与注释已改为 `retry_passed_proposal` 语义。
- 已完成 wuminapp 账户级编码/发现/查询链路：`institution_data`、`duoqian_storage_codec`、`admin_institution_codec`、`duoqian_discovery_service`、`duoqian_manage_service`、`institution_admin_service` 均按 `0x03 / 0x05` 账户级主体读取。
- 已补齐 wuminapp 转账 QR 展示字段：`propose_transfer` 冷钱包 display.fields 改为 `institution / beneficiary / amount_yuan / remark`，`org` 仅作为链端路由，不作为展示字段。
- 已完成 wumin 冷钱包解码：`propose_transfer` 仅接受 `0x01 / 0x03 / 0x05` 可支出主体，拒绝旧裸 sfid 与 `0x02`；`propose_sweep_to_main` 只接受 `0x01 BuiltinInstitution`。
- 已更新文档：ADR-015、duoqian-transfer 技术文档、wuminapp governance/transfer 技术文档、QR action registry、QR protocol spec。
- 已清理残留注释：旧 `execute_X` 重试措辞、注册机构“主账户统一管理员”口径、`user_duoqian` 扫码加入口径。
- 本步骤未修改 `spec_version`。

## 验证记录

- `cargo test -p duoqian-transfer --lib`：22 passed。
- `flutter test test/duoqian test/institution`：49 passed。
- `flutter test test/signer`：61 passed。
- `flutter analyze lib/proposal/transfer/transfer_proposal_page.dart lib/institution/institution_data.dart lib/proposal/shared/internal_vote_service.dart`：No issues found。
