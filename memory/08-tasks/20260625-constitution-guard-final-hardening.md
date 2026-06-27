# 任务卡：宪法守卫最终加固与残留清理

## 任务需求

修复宪法守卫复查中剩余的两个改进点，并清理误导后续线程的残留文档：

- `legislation-yuan::write_law_version` 最终写入层补 commit-time 复校验，防止未来内部调用或回调载荷绕过提案入口校验。
- `node/src/core/constitution.rs` 补节点守卫包装测试，覆盖 warp 提交前校验、普通块 fail-closed 与 runtime upgrade 强制全检等关键边界。
- 清理文档中仍写 `LegislationApi`、`fail-open 保留`、旧测试数量等残留口径。

## 预计修改目录

- `citizenchain/runtime/public/legislation-yuan/`
  - 用途：补立法院模块最终写入层复校验与对应测试。
  - 边界：只处理宪法守卫相关写入安全，不扩展立法流程、不改投票引擎职责。
  - 类型：runtime 代码与测试，需用户二次确认后修改。

- `citizenchain/node/src/core/`
  - 用途：补宪法守卫节点侧包装测试。
  - 边界：只测试守卫行为，不改变导入链路业务逻辑。
  - 类型：节点代码测试。

- `memory/04-decisions/`
  - 用途：清理 ADR-027 中过期的 RPC 与守卫口径。
  - 边界：只更新当前目标态，不保留旧兼容叙述。
  - 类型：文档残留清理。

- `memory/05-modules/`
  - 用途：清理节点模块技术文档中的旧 `LegislationApi` 读取口径。
  - 边界：只更新公民宪法 tab 当前实现说明。
  - 类型：文档残留清理。

- `memory/08-tasks/`
  - 用途：记录本次任务执行与验收结果。
  - 边界：只更新本任务卡状态。
  - 类型：任务文档。

## 验收要求

- `cargo test --manifest-path citizenchain/runtime/public/legislation-yuan/Cargo.toml`
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`
- `cargo check --manifest-path citizenchain/runtime/public/legislation-yuan/Cargo.toml --no-default-features`
- `cargo fmt --manifest-path citizenchain/runtime/public/legislation-yuan/Cargo.toml --check`
- `cargo fmt --manifest-path citizenchain/node/Cargo.toml --check`

## 进度

- [x] runtime 二次确认完成
- [x] 最终写入层复校验完成
- [x] 节点守卫包装测试完成
- [x] 残留文档清理完成
- [x] 验收命令通过

## 完成摘要

- `write_law_version` 在任何 storage 写入前执行 commit-time 复校验,直接写入也会拒绝新立第二部宪法、废止宪法、Pending 重叠修法和不可修改条款变动。
- 节点宪法守卫抽出 `needs_full_invariant_check` 与导入态 key/value 预校验纯函数,补测 runtime `:code` 强制全检、warp 合法态通过、篡改态/缺键态拒绝。
- 文档残留已清理为当前口径:宪法展示走 `constitution_getDocument` RAW storage RPC;普通法律浏览可走 `LegislationApi`;普通块守卫错误为 fail-closed,不保留旧放行口径。

## 验收结果

- `cargo test --manifest-path citizenchain/runtime/public/legislation-yuan/Cargo.toml`:23 passed。
- `cargo test --manifest-path citizenchain/node/Cargo.toml constitution`:21 passed。
- `cargo check --manifest-path citizenchain/runtime/public/legislation-yuan/Cargo.toml --no-default-features`:通过。
- `cargo fmt --manifest-path citizenchain/runtime/public/legislation-yuan/Cargo.toml --check`:通过。
- `cargo fmt --manifest-path citizenchain/node/Cargo.toml --check`:通过。
