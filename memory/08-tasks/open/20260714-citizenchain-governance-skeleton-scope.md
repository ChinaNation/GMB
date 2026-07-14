# CitizenChain 创世治理骨架保护范围收缩

## 任务目标

- `NodeGuard::governance_skeleton` 只保护 89 个指定创世核心机构：1 个国家储委会、43 个省储委会、43 个省储行、1 个国家司法院、1 个联邦注册局。
- 受保护机构固定管理员人数、岗位代码与名称、岗位席位和管理员任职集合闭环，具体管理员允许依法轮换。
- 其他创世机构和运行期登记机构的管理员人数、岗位与组织结构完全由 runtime 合法交易和升级治理。
- 保护身份必须同时匹配机构码、CID 和主账户，禁止只按机构码扩大保护范围。

## 修改范围

- `citizenchain/runtime/primitives/`：89 个保护身份和岗位结构唯一清单。
- `citizenchain/runtime/admins/`：管理员人数按具体保护身份判断。
- `citizenchain/runtime/entity/public-manage/`：区分保护机构固定岗位与普通机构动态岗位。
- `citizenchain/node/src/core/node_guard/`：精确触发、共享类型解码和完整状态治理分区收缩。
- CitizenChain node/runtime 技术文档：更新保护边界、测试和运行口径。

## 明确不修改

- 不修改 `ConstitutionGuard`。
- 不删除或放宽 NodeGuard 的发行、PoW、CID、GenesisPallet 等其他策略。
- 不兼容旧 `AdminAccount` SCALE 数据，不保留双轨逻辑。
- 不执行 GitHub 推送或触发远端 CI。

## 验收

- 保护清单精确覆盖 89 个目标机构且 CID、主账户唯一。
- 保护机构人数、岗位和席位破坏被 runtime 与 NodeGuard 拒绝，合法换届通过。
- 普通机构管理员人数和岗位结构合法变化不触发治理骨架拒绝。
- 当前 runtime fresh block#0 通过 NodeGuard 与 ConstitutionGuard，真实 RPC 可用。
- 更新技术文档、补齐中文注释并清理旧镜像、旧触发和旧口径残留。

## 完成结果（2026-07-14）

- 已完成：治理保护清单按机构码、CID、主账户完整身份精确覆盖 89 个目标机构，CID 与主账户唯一性由测试固定。
- 已完成：`public-admins` 仅对完整保护身份固定管理员人数；同机构码但非保护 CID/主账户的机构按动态规则处理。
- 已完成：`public-manage` 仅对完整保护身份冻结岗位定义和席位；普通机构可以通过 runtime 治理结果增删改岗位、人数和组织结构。
- 已完成：NodeGuard 只收集和检查 89 个保护机构的管理员、岗位、任职 key；普通机构变化不触发，`:code` 变化才全量复核。
- 已完成：删除 NodeGuard 手写 SCALE 镜像，统一使用共享协议类型；删除原生层任期、来源和引用的重复业务判定。
- 已完成：技术文档与中文注释已更新，旧类别化保护、全表触发和手写镜像口径已清理。

## 验收记录（2026-07-14）

- `cargo test -p primitives governance_skeleton`：6/6 通过。
- `cargo test -p public-admins`：6/6 通过。
- `cargo test -p public-manage`：42/42 通过。
- `cargo test -p citizenchain`：37/37 通过。
- `cargo test -p node core::node_guard`：79/79 通过。
- `cargo test -p node governance_skeleton`：9/9 通过。
- 修改范围内 Rust 文件逐个 `rustfmt --check` 通过；全仓 `cargo fmt --all -- --check` 被既有无关文件 `citizenchain/crates/blockchain-test-harness/src/bin/harness.rs` 的格式差异阻断，本任务未改动该文件。
- `WASM_BUILD_FROM_SOURCE=1 cargo build -p node` 通过。
- 使用 `citizenchain-fresh --tmp --pool-type single-state --mining-threads 0` 真实启动成功；RPC `chain_getBlockHash(0)` 返回 `0x362e8055636a014a0a51f563d6dadb139d430bd1a991ee6569c5d8148fdbd4b0`，`system_health.isSyncing=false`，验收后节点正常退出。
- 用户当前桌面进程使用既有本地数据库真实启动成功并监听 `127.0.0.1:9944`；RPC `chain_getBlockHash(0)` 返回 `0x1f327586d8d3ffe02cc66f33097dfec5c037765e4ab66687e293abe21e7c1dee`，`system_health.isSyncing=false`，验证现有本地数据无需删除。
