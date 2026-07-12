# 任务卡：CitizenApp 链连接与边缘服务架构冻结

## 1. 任务背景

本任务用于冻结“公民 App 快速稳定连接公民链网络，同时快速稳定使用聊天和广场，并降低国储会云节点暴露面”的产品架构口径。

用户已明确：

- 公民钱包不纳入本方案讨论，公民钱包只做冷钱包。
- 公民链是一个整体，由 `node`、`runtime`、`onchina` 组成，并打包成为一个安装包。
- 国储会等引导节点和云节点可部署在 Oracle Cloud 等云服务器上。
- 公民 App 是手机软件，目标是快速、稳定连接区块链网络，且在链连接异常时聊天和广场仍应尽量可用。
- 国储会区块链节点必须降低被攻击可能性，不能把核心 RPC 暴露成 App 直连入口。

## 2. 目标状态

架构目标固定为“四层协作”：

1. CitizenApp：内置 smoldot 轻节点，连接 CitizenChain P2P 网络；端上私钥只在本机签名。
2. Cloudflare 边缘层：提供 DNS/WAF/限流、聊天瞬时信封与信令转发、无内容推送、短期 TURN、广场媒体与 feed、轻节点启动清单和受控签名交易转发。
3. Citizen API / OnChina 投影能力：承接非链上查询、公开目录、业务聚合、受控链上事件投影和签名交易广播。
4. CitizenChain 云节点网络：由 `citizenchain/node + runtime + onchina` 安装包运行，Oracle Cloud 等服务器承载 bootnode/full node/archive/indexer/RPC 服务节点。

## 3. 范围

本步骤只做架构文档冻结：

- 新增 ADR，明确 CitizenApp 不是 API-only 客户端。
- 更新仓库、CitizenApp、CitizenChain、OnChina、Oracle Cloud 部署文档的架构口径。
- 清理“全部读写都通过 api.onchina.org”“Matrix 替代现有 Chat”“CitizenWallet 参与在线链路”等与目标状态冲突的文档表述。

本步骤不做：

- 不修改 `citizenchain/runtime/`。
- 不创建 Cloudflare bootstrap API 代码目录。
- 不部署 Cloudflare 或 Oracle Cloud。
- 不修改 CitizenWallet。
- 不触碰 GitHub 远端、PR、CI/CD。

## 4. 预计修改目录

- `/Users/rhett/GMB/memory/08-tasks/open/`
  - 用途：记录本次真实文档任务的任务卡。
  - 边界：只新增本任务卡，不创建任务拆分卡。
  - 类型：文档。
  - 残留清理：任务结束前补充执行记录与验收结果。
- `/Users/rhett/GMB/memory/04-decisions/`
  - 用途：新增架构 ADR，作为跨模块长期决策入口。
  - 边界：只冻结架构边界，不登记具体 API 字段。
  - 类型：文档。
  - 残留清理：移除与当前目标冲突的旧方案口径。
- `/Users/rhett/GMB/memory/01-architecture/gmb/`
  - 用途：更新仓库级产品矩阵与跨产品主流程。
  - 边界：只写产品边界和协作关系，不写实现细节。
  - 类型：文档。
  - 残留清理：避免把 OnChina 写成第五产品或把 CitizenWallet 写入在线链路。
- `/Users/rhett/GMB/memory/01-architecture/citizenapp/`
  - 用途：更新 CitizenApp 链连接、Cloudflare 边缘服务、P2P 失败降级口径。
  - 边界：不覆盖已有广场会员体系改动，不改 App 代码。
  - 类型：文档。
  - 残留清理：清理 API-only、Matrix、节点聊天等冲突表达。
- `/Users/rhett/GMB/memory/01-architecture/citizenchain/`
  - 用途：更新 CitizenChain 安装包、云节点角色、Oracle Cloud 安全边界。
  - 边界：不改 runtime，不改 node 代码，不改部署脚本。
  - 类型：文档。
  - 残留清理：强调国储会核心节点不直接开放公网 RPC。
- `/Users/rhett/GMB/memory/01-architecture/onchina/`
  - 用途：更新 OnChina 与 Citizen API / 投影能力的边界。
  - 边界：不把 OnChina 改成公民 App 的链上真源，不恢复旧独立后端。
  - 类型：文档。
  - 残留清理：避免把 OnChina 写成第五产品或通用公网 RPC 网关。

## 5. 风险点

- 如果把 CitizenApp 改成 API-only，会削弱轻节点可信边界，并与既有 ADR-004 冲突。
- 如果把国储会核心节点 RPC 暴露到公网，会显著增加攻击面。
- 如果 Cloudflare Worker 变成链上真源，会破坏链上最终性和端上签名边界。
- 如果引入 Matrix 替代当前 OpenMLS + 瞬时 WebSocket/WebRTC，会引入云端消息存储并与现有 Chat 技术路线冲突。
- 如果把公民钱包纳入在线链路，会破坏公民钱包冷钱包定位。

## 6. 分步实施方案

### Step 1：架构冻结

- 输出并落地 ADR。
- 更新产品级架构文档。
- 清理明显冲突的旧口径。
- 不改代码，不改 runtime。

### Step 2：Cloudflare 启动清单 API

- 先输出技术方案并等待确认。
- 目标是让 CitizenApp 启动时获得 bootnodes、lightSyncState/checkpoint、服务健康状态、Worker/OnChina 投影入口。
- Cloudflare 只提供启动加速信息，不提供链上真源。

### Step 3：CitizenApp 链连接状态机

- 先输出技术方案并等待确认。
- 明确正常、降级、离线三种状态。
- P2P 可用时链读写以轻节点为准；P2P 不可用时聊天和广场继续走 Cloudflare，链上关键状态展示为降级。

### Step 4：签名交易受控广播兜底

- 先输出技术方案并等待确认。
- App 本地签名后可通过 Citizen API 转发交易到服务节点 RPC。
- API 不接触私钥，不修改交易，不把广播成功当成链上成功。

### Step 5：Oracle Cloud 节点角色拆分

- 先输出技术方案并等待确认。
- 区分国储会核心节点、公开 bootnode、公开 RPC/service node、archive/indexer。
- 核心节点只开放必要 P2P，不向公网开放 RPC/Prometheus。

### Step 6：聊天和广场生产硬化

- 先输出技术方案并等待确认。
- 继续使用 Cloudflare Worker/D1/R2/Durable Object/KV 的现有路线。
- 不切 Matrix，不恢复区块链节点聊天。

### Step 7：安全与可观测性

- 先输出技术方案并等待确认。
- 增加限流、审计、日志、节点健康探测、异常告警和访问边界。

### Step 8：真实运行态验收

- 先输出技术方案并等待确认。
- 使用真实手机、真实 Worker、真实云节点或本地等价服务验证连接、降级、广播、聊天和广场。

## 7. 执行记录

- 2026-07-08：用户确认按该方案设计并允许新增本任务卡和 ADR。
- 2026-07-08：已新增 `ADR-032`，并同步更新仓库总文档、CitizenApp、CitizenChain、OnChina、Oracle Cloud 部署文档；本步骤未修改代码、未修改 `citizenchain/runtime/`、未部署 Cloudflare 或 Oracle Cloud。
- 2026-07-08：第 2 步已落地 Cloudflare Worker `GET /v1/chain/bootstrap` 启动清单接口，新增 `citizenapp/cloudflare/src/chain/bootstrap.ts` 和 `citizenapp/cloudflare/test/chain_bootstrap.test.ts`，并登记统一协议 `P-API-CITIZENAPP-004`。该接口只提供公开启动清单，不返回 RPC URL，不代理 JSON-RPC，不接触私钥；CitizenApp 端接入留到第 3 步确认后执行。
- 2026-07-08：第 3 步已接入 CitizenApp 轻节点启动状态机，新增 `citizenapp/lib/rpc/chain_bootstrap_api.dart` 和 `citizenapp/test/rpc/chain_bootstrap_api_test.dart`，更新 `citizenapp/lib/rpc/smoldot_client.dart`。App 启动时先读取启动清单，校验 `chain_id`、`protocol_id`、`state_root`、`SS58` 与本地 chainspec 一致后才把推荐 bootnodes 注入内存版 chainspec；清单不可用、不匹配或违反安全位时继续使用本地 assets。API 清单仍不是链上真源，`signed_extrinsic_relay` 仍保持关闭。
- 2026-07-08：第 4 步已落地已签名交易受控广播兜底，新增 `citizenapp/cloudflare/src/chain/extrinsic_relay.ts`、`citizenapp/cloudflare/test/chain_extrinsic_relay.test.ts`、`citizenapp/cloudflare/migrations/0007_chain_extrinsic_relay.sql`、`citizenapp/lib/rpc/signed_extrinsic_relay_api.dart`、`citizenapp/test/rpc/signed_extrinsic_relay_api_test.dart`，并更新 `citizenapp/lib/rpc/chain_rpc.dart`、`citizenapp/lib/rpc/chain_bootstrap_api.dart`、`citizenapp/cloudflare/src/chain/bootstrap.ts`、`citizenapp/cloudflare/src/routes.ts`、`citizenapp/cloudflare/src/types.ts`、`citizenapp/cloudflare/wrangler.toml`。Worker 仅在 `CHAIN_EXTRINSIC_RELAY_ENABLED=1` 且服务节点 RPC 已配置时开放 `POST /v1/chain/extrinsics/relay`，只接受完整 signed extrinsic hex，只调用 `author_submitExtrinsic`，不接触私钥、不保存原始 extrinsic body、不返回 RPC URL；App 仅在轻节点 submit-only 失败且错误像链路故障时使用该兜底，交易本身 invalid/bad proof/stale/future/payment 类错误不兜底。
- 2026-07-10：后续 ANR/轻节点任务已把 bootstrap 契约彻底升级为 v2 并清除远端 checkpoint 分支；staging `ff19bc46-dc17-4f77-a53f-aed2739142a0` 与 production `00d836aa-9c43-4561-ba33-8730d780c1a0` 已全量发布。两端真实 HTTPS 均返回 schema v2、6 个 bootnodes、无 checkpoint/RPC URL，`/v1/chain/rpc` 为 404；生产 arm64 profile 已在 Pixel 8a 验证读取生产清单并恢复 finalized `#31` 本机缓存，无 CitizenApp ANR 或崩溃。

## 8. 验收要求

- 架构文档必须统一：CitizenApp 内置轻节点，Cloudflare 负责边缘服务和启动加速，Oracle Cloud 运行实际链节点，国储会核心节点不暴露公网 RPC。
- 文档不得把 CitizenWallet 写入在线连接架构。
- 文档不得把 Matrix 写成当前聊天目标路线。
- 文档不得把 API-only 作为 CitizenApp 链上读写目标状态。
- 本步骤只做文档验收，执行 `git diff --check` 并检查关键冲突词。

验收结果：

- `git diff --check` 已通过。
- 关键冲突词检查仅命中 ADR 和任务卡中的禁止项/风险项，以及产品文档中“不得恢复/不是目标路线”的约束说明。
- 第 2 步已通过 `npm --prefix citizenapp/cloudflare run typecheck`、`npm --prefix citizenapp/cloudflare test -- chain_bootstrap.test.ts`、`npm --prefix citizenapp/cloudflare test`。
- 第 2 步已通过 Wrangler dry-run：top-level、`--env staging`、`--env production` 均完成打包和配置解析；未部署远端。
- 第 2 步已完成真实本地运行态 smoke；当前 bootstrap 契约已在后续 ANR/轻节点任务中升级为 v2，只治理链身份、bootnodes 和服务发现，不再包含远端 checkpoint 字段；`/v1/chain/rpc` 仍必须返回 404。
- 第 3 步已通过 `flutter analyze lib/rpc/chain_bootstrap_api.dart lib/rpc/smoldot_client.dart test/rpc/chain_bootstrap_api_test.dart`。
- 第 3 步已通过 `flutter test test/rpc`，覆盖启动清单解析、拒绝 API-only / RPC proxy / RPC URL 字段、HTTPS 与本地 HTTP 配置、bootnodes 注入匹配校验、既有 RPC 缓存和签名交易构造测试。
- 第 3 步已通过 `npm --prefix citizenapp/cloudflare test -- chain_bootstrap.test.ts` 与 `git diff --check`。
- 第 3 步已完成真实本地运行态 smoke：`wrangler dev --local --port 8787` 后，App 侧 `ChainBootstrapApi` 对真实 `GET /v1/chain/bootstrap` 响应解析成功，结果为 `chain_id=citizenchain`、`bootnodes=6`、`api_is_truth=false`、`rpc_proxy=false`、`signed_extrinsic_relay.enabled=false`、`isSafeForLightClient=true`；精确扫描未发现 `rpc_url`、`validator_rpc_url`、`archive_rpc_url`、`chain_rpc_url`、`square_chain_rpc_url`；探测 `/v1/chain/rpc` 返回 404。
- 第 4 步已通过 `npm --prefix citizenapp/cloudflare run migrate:local`，本地 D1 成功应用 `0007_chain_extrinsic_relay.sql`。
- 第 4 步已通过 `npm --prefix citizenapp/cloudflare run typecheck`。
- 第 4 步已通过 `npm --prefix citizenapp/cloudflare test -- chain_bootstrap.test.ts chain_extrinsic_relay.test.ts`，覆盖 relay 默认关闭、开启时 bootstrap 只暴露 path 不暴露 RPC URL、只调用 `author_submitExtrinsic`、重复 signed extrinsic 去重、非法 hex/私钥字段拒绝、按 IP hash 限流。
- 第 4 步已通过 `flutter analyze lib/rpc/chain_bootstrap_api.dart lib/rpc/chain_rpc.dart lib/rpc/signed_extrinsic_relay_api.dart test/rpc/chain_bootstrap_api_test.dart test/rpc/signed_extrinsic_relay_api_test.dart`。
- 第 4 步已通过 `flutter test test/rpc`，覆盖 relay API 解析、错误码透传、错误 `chain_success_source` 拒绝、启动清单固定 relay path 校验、既有 RPC 缓存和签名交易构造测试。
- 第 4 步已通过 Wrangler dry-run：top-level、`--env staging`、`--env production` 均完成打包和配置解析；未部署远端。首次在仓库根目录误跑 dry-run 被 Wrangler 自动识别为静态项目并失败，已在 `citizenapp/cloudflare` 正确目录重跑通过。
- 第 4 步曾以已废弃的本地 HTTP 单 Secret 完成 relay 历史 smoke；2026-07-10 起当前 Worker 不再接受该配置方式，只接受 Access 保护的 HTTPS URL 与两项服务令牌 Secret。原 smoke 仍证明 relay 路由、私钥字段拒绝、通用 RPC 404 和 App 响应解析在当时通过，但不能替代新的 Access + Tunnel 运行态验收。
- bootstrap v2 已完成远端发布验收：staging 与 production 均为 100% 流量，真实响应只包含公开启动清单；生产手机未保留 staging base URL。正式链 finalized 仍为 `#31`，真实 GRANDPA warp 必须等待 `#33` 或更高后用干净数据环境单独验收。
