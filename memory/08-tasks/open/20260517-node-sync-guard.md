# 任务卡：修复桌面节点同步层脱钩后区块不更新的问题

## 任务需求

本机桌面节点出现底层 P2P 仍有连接、交易可以广播到其他节点，但 `system_peers` 为空且区块同步不继续的问题。需要先修复 node 端，在不清链、不删除数据库、不改回 `ws`、不常规请求参考节点的前提下，让节点发现 `network_service` 与 `sync_service` 脱钩并自动受控重启恢复。

## 影响范围

- `citizenchain/node/src/home/`：新增/接入同步守护，检测本机 sync/network 脱钩并触发受控重启。
- `citizenchain/node/src/shared/`：如有必要补充本地 RPC 采样工具；不承载业务判断。
- `citizenchain/node/src/desktop/`：如有必要注册守护状态查询命令；不写恢复逻辑。
- `memory/05-modules/citizenchain/node/`：更新 node/home 技术文档，记录守护边界和误判防护。

## 修复目标

- 只通过本机 `127.0.0.1` RPC 做常规自检，不定时请求公网参考节点。
- 不以区块高度是否增长作为重启条件，避免网络无交易时误判。
- 命中 `shouldHavePeers=true`、`system_peers=0`、raw connected peer 有版本和 ping 的持续异常后，受控重启进程内 Substrate 服务。
- 重启前保存待处理 extrinsics，重启后按限额重新提交。
- 加入冷却和降级状态，避免自动重启风暴。

## 验收方式

- 单元测试覆盖故障判定、防误判、冷却和 pending extrinsics 限额。
- `cargo test` 覆盖 node 侧新增逻辑。
- 文档同步记录本次根因、守护条件、不使用区块高度增长判定的原因。
- 残留扫描确认没有临时调试输出、无旧方案描述残留。

## 状态

- [x] 创建任务卡
- [x] 实现本地同步守护
- [x] 接入节点生命周期
- [x] 更新文档和注释
- [x] 残留检查与验证
