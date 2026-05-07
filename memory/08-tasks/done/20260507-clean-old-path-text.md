# 任务卡:清理旧路径文字残留

## 任务需求

回答全量验收中 5 个问题，并按用户要求清理仓库中的旧路径文字残留。

本任务只处理文字、注释、配置说明和任务记录中的旧路径字符串，不改业务逻辑、不恢复旧目录、不迁移源码。

## 预计修改目录

| 目录 | 用途、边界和修改类型 |
|---|---|
| `citizenchain/node/src/` | 清理活跃 Rust 注释中的 SFID 旧路径引用；只改注释。 |
| `cpms/backend/src/` | 清理 CPMS 注释中的 SFID 旧路径引用；只改注释。 |
| `wuminapp/lib/` | 清理 Dart 注释中的 SFID / Isar 旧路径引用；只改注释。 |
| `memory/` | 清理架构文档、模块文档、done 任务卡和 AI 规则文档中的旧路径字符串；只改文档。 |
| `memory/05-modules/sfid/backend/` | 将链交互归属文档移出旧目录片段；只改文档路径。 |
| `memory/08-tasks/` | 新增并归档本任务卡，更新任务索引；只改任务记录。 |

## 执行清单

- [x] 清理旧路径字符串：SFID 后端旧源码壳、SFID 前端旧源码壳、旧 chain/api 业务目录、wuminapp 旧 Isar 目录。
- [x] 复查旧路径字符串无命中。
- [x] 复查 open 任务卡为空。
- [x] 运行必要语法/构建检查。
- [x] 归档任务卡并暂存。

## 验收标准

- 5 类旧路径字符串无命中。
- 活跃代码只发生注释改动。
- `git diff --check` 通过。

## 执行结果

- 已清理活跃代码注释、memory 架构文档、模块文档、历史任务卡和反馈文档中的旧路径字符串。
- 已将 SFID 后端链交互归属文档移到 `memory/05-modules/sfid/backend/CHAIN_TECHNICAL.md`，避免文档路径本身继续携带旧目录片段。
- 已保留规则语义：仍禁止恢复旧后端源码壳、独立 chain 业务目录、旧全局 API 目录和旧大写 Isar 目录。

## 验证记录

- 旧路径内容扫描：无命中。
- 旧路径文件路径扫描：工作区无命中。
- `flutter analyze lib/rpc/sfid_public.dart`：通过。
- `cargo check --manifest-path cpms/backend/Cargo.toml`：通过；仍有 19 个既有 warning。
- `git diff --check`：通过。
