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
