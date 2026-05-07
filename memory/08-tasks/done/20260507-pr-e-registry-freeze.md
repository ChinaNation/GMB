# 任务卡:PR-E 命名与协议登记收口

## 任务需求

执行重新创世前收口项 4、6、7：

- 清理活跃代码注释中的 `InstitutionPalletId` 旧名。
- 补齐 `memory/07-ai/unified-protocols.md` 的统一协议登记。
- 补齐 `memory/07-ai/unified-naming.md` 的统一命名登记。

本任务只改活跃注释和 memory 规则文档，不改业务逻辑、协议字段顺序、storage 结构或测试 fixture。

## 预计修改目录

| 目录 | 用途、边界和修改类型 |
|---|---|
| `wuminapp/lib/duoqian/shared/` | 清理活跃 Dart 注释里的旧主体 ID 命名；只改注释，不改代码逻辑。 |
| `wuminapp/lib/proposal/` | 清理投票提案页面和服务注释里的旧主体 ID 表述；只改注释，不改代码逻辑。 |
| `wumin/lib/signer/` | 清理冷钱包 decoder / registry 注释里的旧主体 ID 表述；只改注释，不改代码逻辑。 |
| `citizenchain/runtime/votingengine/` | 清理反向索引注释里的旧主体 ID 表述；只改注释，不改 runtime 逻辑。 |
| `citizenchain/runtime/transaction/duoqian-transfer/` | 清理多签转账 pallet 和测试注释里的旧主体 ID 表述；只改注释，不改 runtime 逻辑。 |
| `memory/04-decisions/` | 修正 ADR-010 后续任务引用，指向已完成 done 任务；不改协议内容。 |
| `memory/07-ai/` | 补齐统一协议文件与统一命名文件登记；只改 AI 系统规则文档。 |
| `memory/08-tasks/` | 新增并归档本任务卡，更新任务索引；只改任务记录。 |

## 执行清单

- [x] 清理活跃代码注释旧名。
- [x] 补齐统一协议文件登记并删除“待纳入登记”空泛清单。
- [x] 补齐统一命名文件登记并删除“待补充登记”空泛清单。
- [x] 修正 ADR-010 后续任务引用。
- [x] 扫描验证旧名和待登记段落。
- [x] 归档任务卡并暂存。

## 验收标准

- `rg 'InstitutionPalletId' wuminapp/lib citizenchain sfid cpms wumin` 仅允许历史或非活跃目录命中；活跃 `wuminapp/lib` 不再命中。
- `rg '待纳入登记|待补充登记' memory/07-ai/unified-protocols.md memory/07-ai/unified-naming.md` 无命中。
- `git diff --check` / `git diff --cached --check` 通过。

## 执行结果

- 已把统一协议文件中的待登记清单替换为正式协议登记，覆盖投票载荷、人口快照、决议发行、多签转账、个人多签、storage 契约和 CPMS 四码契约。
- 已把统一命名文件中的待补充清单替换为正式命名登记，覆盖顶层配置、模块目录、SFID、wuminapp、citizenchain、API 字段和 QR display field key。
- 已清理本轮命中的旧主体 ID 注释残留，并修正 ADR-010 后续任务引用。

## 验证记录

- `rg -n "InstitutionPalletId|institution_pallet_id" wuminapp/lib citizenchain sfid cpms wumin -g '!**/build/**' -g '!**/target/**' -g '!**/node_modules/**'`：无命中。
- `rg -n "待纳入登记|待补充登记|待纳入|待补充" memory/07-ai/unified-protocols.md memory/07-ai/unified-naming.md`：无命中。
- `git diff --check`：通过。
- `git diff --cached --check`：通过。
- `flutter analyze lib/duoqian/shared/admin_institution_codec.dart lib/duoqian/shared/duoqian_manage_models.dart lib/proposal/runtime_upgrade/runtime_upgrade_detail_page.dart lib/proposal/transfer/transfer_proposal_service.dart`：通过。
- `flutter analyze lib/signer/pallet_registry.dart lib/signer/payload_decoder.dart`：通过。
