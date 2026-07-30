# fork/vendor 基线规则

## 目的

GMB 自有代码和自有文档必须清除待办、修复、临时占位、调试打印等残留标记。

收编的第三方 fork/vendor 目录只做单独统计，不参与自有代码清零门禁。原因是这些目录保留上游注释和实现脉络，机械删除会制造大范围无业务价值 diff，并增加未来同步上游的冲突成本。

## 当前 fork/vendor 范围

- `citizenapp/smoldot-pow/`：收编的 smoldot PoW 轻节点 fork。
- `citizenchain/node/vendor/`：本地覆盖的 GRANDPA voter 相关 vendor 实现。

## 门禁规则

1. 自有代码和自有文档扫描必须为 0。
2. fork/vendor 目录允许保留上游遗留标记，但必须单独统计数量。
3. fork/vendor 中 GMB 新增代码不得再添加待办类标记。
4. `docs/logo.svg` 等 base64 资源和 lock 文件哈希不参与文字残留门禁。
5. 如果后续决定深度维护 fork/vendor，必须另建专项任务卡，不在普通清理任务中机械改上游注释。
6. 第三方依赖 future-incompat 报告不计入自有 Rust warning 清零；例如 `trie-db 0.30.0` 由 polkadot-sdk 依赖链带入，等待上游版本统一升级处理。仓库根 `.cargo/config.toml` 只关闭该类第三方报告频率，不压制自有代码 warning。
7. **残留标记不阻断 ≠ 编译 lint 不阻断（2026-07-30 澄清）**：本文第 1—5 条只管「待办/修复/占位/调试打印」这类**文字残留标记**。`citizenchain/node/vendor` 既是 workspace 成员（`citizenchain/Cargo.toml`），又在自己的 `Cargo.toml` 里写了 `[lints] workspace = true`——它是**自愿纳入** clippy 门禁的，`cargo clippy --workspace` 也无法绕开它。因此：
   - `cargo clippy --workspace --all-targets -- -D warnings` **对 vendor 同样阻断**，不得用本文第 5 条当豁免理由；
   - GRANDPA finality 属共识关键路径，刻意保留 lint 覆盖是有意为之，不要为了「让文档自洽」去 `--exclude` 它；
   - vendor 里的 lint 修复只做**零行为变化**的最小改动（例如删冗余借用），不重排上游逻辑、不批量改上游注释，保持同步成本可控。

   历史教训：本条澄清前，文档说 vendor 不阻断、实配却让它阻断，导致 `finality_proof.rs` 的一条 `useless_borrows_in_formatting` 从 2026-03-26 一直红到 2026-07-30 创世前审计才被发现。

## 推荐扫描边界

自有范围扫描时排除：

- `.git/`
- `target/`
- `build/`
- `node_modules/`
- `.dart_tool/`
- `citizenapp/smoldot-pow/`
- `citizenchain/node/vendor/`
- `memory/08-tasks/done/`
- `docs/logo.svg`
- `package-lock.json`

fork/vendor 范围只输出统计报告，不作为阻断条件。
