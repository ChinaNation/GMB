# 任务卡：修复普通节点关闭 `grandpa-voter` 后网络连接被 GRANDPA 协议空接收端打断

- 任务编号：20260326-112500
- 状态：in-progress
- 所属模块：citizenchain/node
- 当前负责人：Codex
- 创建时间：2026-03-26 11:25:00

## 任务需求

1. 修复普通节点在未持有有效 GRANDPA 私钥时启动后 `peers=0`、无法接入现网的问题。
2. 明确区分“注册 GRANDPA 网络协议”和“启动 `grandpa-voter`”的角色边界。
3. 保证普通节点默认仅作为同步节点运行，不再因为空的 GRANDPA 协议接收端把已建立连接主动打断。

## 必读上下文

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/repo-map.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/agent-rules.md`
- `memory/07-ai/context-loading-order.md`
- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`

## 模块边界

- 允许改动：
  - `citizenchain/node`
  - `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
  - `memory/08-tasks/index.md`
- 不改动：
  - 链上 authority set 规则
  - UI 设置协议
  - 服务器现网配置

## 风险说明

- 普通节点不注册 GRANDPA 网络协议后，必须确认常规同步仍通过 `block-announces` / `sync/*` 路径正常建立。
- 不能把“是否参与 GRANDPA 投票”和“是否允许普通节点接网同步”再度耦合到同一个错误分支。
- 若保留任何未消费的网络协议句柄，仍可能再次出现 `EssentialTaskClosed` 异常。

## 已定位根因

- 当前代码会无条件注册 GRANDPA 网络通知协议。
- 但普通节点在没有匹配 GRANDPA 私钥时，不再启动 `grandpa-voter`。
- `grandpa_notification_service` 只在 `grandpa-voter` 分支被消费，普通节点因此留下一个已注册但无人消费的 GRANDPA 协议通道。
- `litep2p` 在向全部已注册协议广播 `ConnectionEstablished` 时，命中这个已关闭接收端后返回 `EssentialTaskClosed`，连接随即中断，最终导致 `block-announces` 无法建立、`peers=0`。

## 实施记录

- 已把 GRANDPA 角色判定提前到网络构建之前，统一基于“本地是否持有且匹配当前 authority set 的 GRANDPA 私钥”决定节点角色。
- 已修改 `citizenchain/node/src/service.rs`：
  - 只有真正的 GRANDPA 节点才注册 GRANDPA 网络通知协议。
  - 只有真正的 GRANDPA 节点才持有并消费 `grandpa_notification_service`。
  - 普通节点彻底不再暴露 GRANDPA 协议，避免空接收端导致的 `EssentialTaskClosed`。
- 已更新 `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`，明确“默认普通节点，导入并匹配 GRANDPA 私钥后才切换为 GRANDPA 节点”的运行规则。

## 验证结果

- `cargo fmt --manifest-path /Users/rhett/GMB/citizenchain/Cargo.toml --all` 通过。
- `cargo check -p node` 通过。
- 本轮未直接重打桌面运行时二进制，真实联网效果仍需在重新构建并重启桌面节点后复测。
