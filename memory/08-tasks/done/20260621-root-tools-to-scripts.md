# 任务卡：根 tools 目录收敛到 scripts

- 状态：已完成
- 创建：2026-06-21
- 需求：删除仓库根目录 `tools/`，将其中仍有用途的脚本迁移到根目录 `scripts/`；与 `scripts/` 重复的脚本合并，无业务用途的脚本删除；同步文档、生成器注释和路径引用，清理残留。

## 范围

- `tools/`：迁移仍有用途的仓库级脚本，删除缓存和一次性临时脚本。
- `scripts/`：承接仓库级脚本和生成器。
- `docs/`、`memory/`、生成文件注释：更新根目录结构与脚本路径说明。
- `citizenchain/runtime/`：只记录发现的注释路径残留；按 runtime 二次确认硬规则，未获确认前不产生 runtime diff。

## 执行口径

- 根目录不再保留 `tools/`。
- 仓库级脚本统一在根 `scripts/` 下维护。
- 历史任务卡中的旧路径只作为历史记录，不作为当前结构真源；当前架构文档、白皮书、活跃任务卡和生成器注释必须改为新路径。

## 执行记录

- 已将 `duoqian.py`、`extract_whitepaper_images.py`、`fill_china_admins.py`、`generate_citizenapp_governance_registry.mjs`、`zhujichi.py` 迁移到根 `scripts/`。
- 已删除根 `tools/__pycache__` 和一次性临时脚本 `tools/resolve_stash_conflicts.py`，并删除根 `tools/` 目录。
- 已更新白皮书、仓库结构文档、命名登记、活跃任务卡和生成器注释中的当前路径。
- 已重新生成 `citizenchain/node/frontend/generated/local-docs.generated.ts`，保证区块链软件白皮书 Tab 使用同一结构说明。
- 已验证 `node scripts/generate_citizenapp_governance_registry.mjs` 可从新路径执行，并同步生成 CitizenApp / CitizenWallet 机构注册表。
- 已在用户二次确认后，将 `citizenchain/runtime/primitives/china/china_zb.rs` 第 6 行注释从 `tools/duoqian.py` 改为 `scripts/duoqian.py`，不涉及 runtime 业务逻辑。
- 已修正 `scripts/generate_citizenapp_governance_registry.mjs` 的输入/输出边界：读取 runtime 现有 `sfid_full_name/sfid_number` 字段，输出 CitizenApp / CitizenWallet 当前 `cidNumber` 字段。
- 已执行验证：Python 脚本语法检查、根 shell 脚本语法检查、`node scripts/generate_citizenapp_governance_registry.mjs`、Dart 生成文件格式化、白皮书本地缓存生成、根 `tools/` 残留扫描。
- 追补(2026-06-21)：统一 `scripts/duoqian.py` 与 BLAKE2 派生文档中 OP_AN/OP_HE 的 payload 表述，二者均为国储会 `cid_number`；同时补齐文档表格中的 `OP_HE=0x04`，恢复 `OP_PERSONAL=0x05`、`OP_INSTITUTION=0x06` 编号。

## 验收

- [x] 根目录 `tools/` 不存在。
- [x] 根 `scripts/` 包含迁移后的仍有用途脚本。
- [x] 当前文档和生成器注释不再把根 `tools/` 描述为当前目录。
- [x] 路径扫描确认无非历史、非第三方的根 `tools/` 当前引用。
