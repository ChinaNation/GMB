# IM 双节点真实运行态互投验收

## 状态

done

## 任务需求

在已完成 `/gmb/im/1` request-response 接入 Spike 的基础上，补齐两个真实 headless 节点之间的运行态互投验收：启动两个不同 `base-path`、不同 P2P/RPC 端口、不同 owner 的节点，通过显式 `PeerId + multiaddr` 完成 A 节点向 B 节点私人 mailbox 投递密文信封，并验证 B owner 手机设备可以拉取、ack，同时拒绝第三方 mailbox。

## 边界

- 只做运行态验收闭环，不接 OpenMLS、Protobuf、持久化 mailbox、KeyPackage 池、近场和聊天窗口公民币转账。
- 调试 RPC 只能在 `GMB_IM_DEBUG_RPC=1` 的验收节点中注册，正式节点默认不暴露。
- 双节点脚本使用临时 `base-path`，不得清理或污染现有开发链数据目录。
- 通信全节点继续只服务 owner 自己，不互为节点、不做公共中继、不替第三方存消息。

## 预计修改目录

- `citizenchain/node/src/im/`：新增 IM debug RPC 模块，复用现有 owner-only mailbox 与 `/gmb/im/1` 直连投递逻辑；涉及代码和中文注释。
- `citizenchain/node/src/core/`：在节点 RPC 创建流程中按环境变量条件注册 IM debug RPC；涉及代码和中文注释。
- `citizenchain/scripts/`：新增双节点 smoke 脚本，启动两个临时节点并完成投递、拉取、ack、拒绝第三方验收；涉及脚本。
- `memory/05-modules/wuminapp/im/`：更新 IM 技术文档，记录双节点真实运行态验收路径和结论；涉及文档。
- `memory/05-modules/citizenchain/node/`：更新节点技术文档，记录 debug RPC 只限验收环境；涉及文档。
- `memory/07-ai/`：补充统一协议和命名登记；涉及文档与残留清理口径。

## 实施记录

- 新增 `citizenchain/node/src/im/rpc.rs`，实现 `GMB_IM_DEBUG_RPC=1` 条件启用的 IM debug RPC：能力查询、owner 设备登记、直连投递、待收拉取和 ack。
- 更新 `citizenchain/node/src/core/rpc.rs`，在 RPC module 创建时仅按环境变量注册 IM debug RPC，正式节点默认不暴露。
- 新增 `citizenchain/scripts/im-two-node-smoke.sh`，构建节点二进制，启动两个临时 `base-path` 的 headless 节点，使用显式 `PeerId + multiaddr` 走 `/gmb/im/1` 完成 A→B 投递。
- 更新 IM、node、统一协议和统一命名文档，清理“双节点运行态互投未完成”的旧口径。

## 验收记录

- `cargo test -p node im::`：通过，12 个 IM 单测全过。
- `cargo check -p node`：通过。
- `citizenchain/scripts/im-two-node-smoke.sh`：通过，两个真实 headless 节点完成 A→B 密文投递、B owner 拉取、ack 和第三方 mailbox 拒绝。
- `git diff --check`：通过。
