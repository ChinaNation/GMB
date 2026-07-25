# Node Clippy 零警告清理

## 任务目标

- 将 Node 的严格 Clippy 验收收口为零警告：
  `cargo clippy -p node --no-deps --all-targets --all-features -- -D warnings`
- 区分机械性告警、错误处理告警和接口约束告警，逐项修复或在不可改变外部接口时添加最小范围豁免。
- 不借清理告警改变现有业务逻辑、协议格式、授权规则或存储结构。

## 预计修改目录

- `citizenchain/node/src/`
  - 清理 Node 源码中的 Clippy 告警。
  - 仅涉及代码质量、错误处理与必要的局部 lint 说明。
  - 不修改 Runtime 业务逻辑。
- `citizenchain/node/tests/`
  - 仅在既有测试需要同步调整时修改。
  - 本任务不新增测试文件；如确需新增，另行确认。
- `memory/05-modules/citizenchain/node/`
  - 更新既有 Node 技术文档，记录严格 Clippy 验收口径和必要的接口边界说明。
- `memory/08-tasks/open/20260725-node-clippy-zero-warning.md`
  - 记录任务范围、实施进度和验收结果。
  - 任务完成后按仓库规则移动至已完成目录。

## 明确边界

- 不修改 `citizenchain/runtime/`。
- 不修改投票引擎和业务 pallet。
- 不修改或提交 `citizenconsole/`。
- 不清理、覆盖或回退其他线程及用户已有改动。
- 不通过全局关闭 Clippy 规则掩盖问题。
- 对外部 trait、命令框架或前端调用契约造成的告警，只允许最小作用域豁免，并补充中文原因。

## 实施步骤

- [x] 生成全目标精确告警清单并分类；确认 `--all-features` 被上游不兼容组合阻断。
- [x] 清理格式、借用、迭代器和可简化表达式等机械性告警。
- [x] 清理生产代码中的 `unwrap`、`expect` 和错误传播等安全性告警。
- [x] 处理框架接口、参数数量、错误体积等结构性告警。
- [x] 运行格式化、严格 Clippy、编译与 294 项 Node 测试验收。
- [x] 执行不破坏现有链数据的真实 Node RPC 与链规格导出验收。
- [x] 更新既有文档、完善中文注释并清理残留。

## 验收标准

- 可构建特性集严格 Clippy 命令零警告、零错误。
- `cargo check -p node` 通过。
- Node 相关测试通过。
- Node 可执行程序完成真实运行态检查；如受既有本地链状态阻塞，必须保留数据并记录真实错误，不得擅自清链。
- 全仓搜索确认没有为消除告警引入宽泛 lint 关闭或遗留表述。

## 状态

- 创建日期：2026-07-25
- 完成日期：2026-07-25
- 当前状态：已完成

## 验收结果

- `cargo fmt -p node -- --check`：通过。
- `cargo clippy -p node --no-deps --all-targets -- -D warnings`：通过，零警告。
- `cargo check -p node`：通过。
- `cargo test -p node`：294 通过、0 失败。
- `cargo build -p node`：通过。
- `target/debug/citizenchain export-chain-spec --chain dev`：真实导出成功，临时产物已删除。
- 本机既有节点进程保持运行，`system_health` 与 `chain_getHeader` RPC 均真实返回；未清理或覆盖链数据。
- `cargo clippy -p node --no-deps --all-targets --all-features -- -D warnings`：
  在上游 `pallet-staking` 编译阶段因缺少 `peek_disabled` 实现失败，尚未进入 Node lint；
  未越界修改 polkadot-sdk 或 Runtime。
