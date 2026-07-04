# 任务卡:修复本机清链脚本为最新 CI WASM 重新创世

> 2026-07-04 口径更新:当前正式重新创世不再生成 raw SSOT,而是由
> `bake-chainspec.sh` 生成 plain SSOT、CitizenApp `stateRootHash` 轻形态和
> `genesis-state/` 创世链状态包。本卡下方 raw 描述只记录当时历史需求。

## 任务需求

`citizenchain/scripts/clean-run.sh` 必须执行真正的本机重新创世:

- 下载最新成功的 GitHub `citizenchain-wasm` artifact。
- 使用该 WASM 生成新的创世锚点(历史口径为 raw chainspec/genesis;当前口径为 plain + genesis-state)。
- 清空本机节点数据。
- 启动桌面端时使用新生成的 fresh chainspec。
- fresh chainspec 不得携带旧网络 bootnodes,避免清链后重新接回旧链。

## 预计修改目录

- `citizenchain/scripts/`：修复本机清链启动脚本；涉及脚本。
- `citizenchain/node/src/core/`：补充 fresh genesis chain spec 生成入口；涉及 Rust 代码。
- `citizenchain/node/src/desktop/`：允许桌面内嵌节点从环境变量读取本次 chain spec；涉及 Rust 代码。
- `citizenchain/node/src/home/`：把环境变量 chain spec 传入节点启动器；涉及 Rust 代码。
- `memory/05-modules/citizenchain/node/`：同步节点技术文档；涉及文档。
- `memory/08-tasks/`：记录验收结果；涉及任务文档。

## 验收标准

- `clean-run.sh` 不再默认使用冻结主网 chainspec 重新接回旧网络。
- `clean-run.sh` 生成的 fresh chainspec 的 bootNodes 必须为空。
- fresh chainspec 的 genesis `:code` 必须等于下载的最新 CI WASM。
- 桌面内嵌节点能读取 `CITIZENCHAIN_CHAIN_SPEC` 并使用 fresh chainspec 启动。
- Rust 格式化和节点编译检查通过。

## 执行记录

- [ ] 修改 fresh chain spec 生成入口。
- [ ] 修改桌面节点启动参数传递。
- [ ] 修改清链脚本。
- [ ] 更新文档。
- [ ] 运行验证。
