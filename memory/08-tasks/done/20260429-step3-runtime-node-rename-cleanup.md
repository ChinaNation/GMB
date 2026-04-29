# 任务卡：第3步 runtime + node 重命名后清理收口

## 状态

- done

## 背景

runtime 与 node 已完成一轮模块、文件夹、文件名称重命名，主要是命名和目录收口，不应在文档、注释中继续保留旧名称、旧路径和施工过程说明。

`memory/05-modules/citizenchain/` 当前只保留 `runtime/` 与 `node/` 等当前态目录，不再保留旧桌面节点目录。

## 目标

- 清理 runtime、node 代码注释和文档中的旧模块名、旧路径、旧 storage 名。
- 文档只描述当前态，不保留旧名称对照表。
- 清理旧桌面节点称谓残留，统一描述为 `node` 桌面节点前后端。
- 不改业务逻辑、不改 call index、不改签名协议。
- 更新文档、完善必要中文注释、清理残留。

## 范围

- `citizenchain/runtime/`
- `citizenchain/node/`
- `memory/05-modules/citizenchain/runtime/`
- `memory/05-modules/citizenchain/node/`
- `memory/08-tasks/`

## 验收标准

- 当前代码和文档中不再出现旧模块名残留。
- 当前代码和文档中不再出现旧桌面节点目录或旧桌面节点称谓。
- 相关 Rust 与前端构建验证通过，或明确记录无法执行的原因。

## 执行记录

- 已将旧桌面节点目录的残留统一收口为 `memory/05-modules/citizenchain/node/`。
- 已将 `memory/08-tasks/` 中含旧桌面节点称谓的任务卡和模板重命名为当前 `node` 命名，并同步任务索引。
- 已清理 runtime 与 node 相关代码注释、技术文档中的旧模块名、旧路径、旧入口、历史迁移说明。
- 已保持业务逻辑、call index、签名协议不变，仅做命名、注释、文档和残留清理。

## 验证记录

- `cargo fmt` 通过。
- `cargo test -p admins-change --lib` 通过，20 个测试通过。
- `cargo test -p duoqian-manage --lib` 通过，21 个测试通过。
- `cargo test -p duoqian-transfer --lib` 通过，20 个测试通过。
- `cargo test -p offchain-transaction --lib` 通过，23 个测试通过。
- `cargo test -p voting-engine --lib` 通过，49 个测试通过。
- `WASM_FILE=/Users/rhett/GMB/citizenchain/target/wasm/citizenchain.compact.compressed.wasm cargo test -p node offchain` 通过，61 个 offchain 相关测试通过。
- `npm run build` 在 `citizenchain/node/frontend` 通过。
- `rg` 检查确认当前 `memory` 与 `citizenchain` 中不再出现旧桌面节点称谓、旧 runtime 模块名和清理目标残留词。
