# ADR-004: wuminapp 采用 PoW 专用轻节点内核并取消 HTTP RPC 回退

## 背景

ADR-002 决定让 `wuminapp` 从 HTTP RPC 迁移到 `smoldot` 轻节点，并保留 `WUMINAPP_RPC_URL` 作为调试回退。

随着 `wuminapp` 实际接入 `smoldot` 并在 `citizenchain` 上验证，已经确认以下事实：

- `citizenchain` 当前共识不是 Aura / BABE，而是 `PoW + GRANDPA`
- `wuminapp` 当前余额、治理批量查询等主路径仍大量依赖 legacy JSON-RPC（如 `state_getStorage`、`state_queryStorageAt`）
- 在当前 PoW 链上，轻节点可以连网、拿到基础链信息，但 storage 读取链路没有跑通
- 根目录临时加入的 `smoldot/` 目录既是 `wuminapp` 真实构建依赖，又是未纳入主仓库治理的嵌套 Git 仓库，已经直接干扰提交流程

这说明：

- “原版 smoldot + 直接透传 legacy JSON-RPC + 可选 HTTP 回退” 不适合 `wuminapp` 的长期形态
- 如果 `wuminapp` 长期坚持轻节点模式，而 `citizenchain` 明确保留 PoW，不应继续把 HTTP RPC 当隐形兜底

## 决策

`wuminapp` 的链访问长期架构调整为以下模式：

1. `wuminapp` 只保留轻节点模式，不再保留 HTTP RPC 回退能力，不再保留 `WUMINAPP_RPC_URL` 这一类主干能力开关。
2. `citizenchain` 保持 `PoW + GRANDPA`，不为了适配上游轻节点而回退到 Aura / BABE。
3. 维护一份自有 GitHub fork，作为 `PoW` 轻节点内核的权威上游；主仓库通过 Git submodule 引用 `wuminapp/third_party/smoldot-pow/`，不再把整套内核源码作为普通文件跟踪。
4. `wuminapp/rust` 不再把 Flutter 侧暴露为“任意 JSON-RPC 方法透传器”，而是提供面向 App 业务的轻节点能力接口。
5. 轻节点 Rust 层负责：
   - PoW 头部校验
   - GRANDPA finalized 追踪
   - finalized 状态读取与 storage proof 校验
   - runtime metadata / version 获取
   - extrinsic 提交
   - 新区块订阅
6. Flutter 侧不再依赖 `state_getStorage` / `state_queryStorageAt` 这类字符串方法作为核心业务接口，而是改为调用 Rust FFI 暴露的 typed capability。
7. 对用户与开发者都不提供隐藏 HTTP 回退；如果轻节点不可用，App 必须显式展示“轻节点未同步 / P2P 不可达 / 校验失败”等真实状态。

## 影响

- `wuminapp/rust/Cargo.toml` 的依赖路径要从根目录临时 `smoldot/` 收敛到 `wuminapp/third_party/smoldot-pow/`
- `wuminapp/scripts/build-smoldot-native.sh` 及相关打包流程需要固定为 PoW fork 的构建路径
- `wuminapp/lib/rpc/` 需要从“JSON-RPC 兼容层”逐步改造成“轻节点能力适配层”
- `wallet`、`governance`、`trade/onchain` 相关读取逻辑都要迁移到新接口
- `app-run.sh` / `app-clean-run.sh` 等启动脚本不再接受 HTTP RPC 回退配置
- CI 与发布流程必须增加 PoW 轻节点专项验证，确保没有回退开关也能正常工作
- `GMB` 主仓库需要维护 `.gitmodules` 与 submodule 提交指针，发布时显式记录引用到的 `smoldot-pow` 提交

## 备选方案

| 方案 | 否决原因 |
| --- | --- |
| 继续使用原版 smoldot，并在 Flutter 层直接透传 legacy JSON-RPC | 现状已经证明在 PoW 链上主路径不稳定，且 legacy JSON-RPC 并不适合长期承担轻节点核心接口 |
| 保留隐藏的 HTTP RPC 调试后门 | 会掩盖真实轻节点缺口，导致主干始终无法证明“脱离 RPC 也可运行” |
| 把 citizenchain 改回 Aura / BABE | 与既定链设计冲突，不符合本次约束 |
| 把临时 `smoldot/` 目录直接作为嵌套仓库提交进主仓库 | 版本管理混乱，破坏主仓库提交与复现能力 |

## 后续动作

1. 建立自有 `smoldot` PoW fork，并固定上游基线提交。
2. 将主仓库中的临时 `smoldot/` 迁入 `wuminapp/third_party/smoldot-pow/`，完成独立仓库发布，并把主仓库引用方式收口为 Git submodule。
3. 在 `wuminapp/rust` 内实现 PoW 专用轻节点能力层，替代当前 JSON-RPC 透传。
4. 分阶段迁移 `wallet`、`governance`、`trade/onchain` 到 typed capability。
5. 删除 HTTP RPC 相关脚本、环境变量、文档与残留。
6. 建立轻节点专项验证矩阵，作为发布前阻断条件。
