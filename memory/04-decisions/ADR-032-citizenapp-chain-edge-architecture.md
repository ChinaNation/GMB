# ADR-032：CitizenApp 链连接与边缘服务架构

## 状态

Accepted（2026-07-08）

## 背景

CitizenApp 是手机软件，需要在移动网络下快速、稳定地连接 CitizenChain 网络，同时快速、稳定地使用聊天和广场功能。国储会等核心区块链节点可能部署在 Oracle Cloud 等云服务器上，但这些核心节点必须降低公网暴露面，避免成为公民端流量和恶意请求的直接入口。

本 ADR 同时固定以下边界：

- CitizenWallet 不参与本架构讨论；CitizenWallet 只做离线冷钱包和扫码签名。
- CitizenChain 是一个整体安装包，由 `node`、`runtime`、`onchina` 组成。
- CitizenApp 必须保留内置轻节点能力，不能改成只访问 HTTP API 的中心化客户端。
- Cloudflare 只承接边缘入口、缓存、广场媒体、Chat 瞬时转发、启动清单和受控转发，不运行 Substrate 节点，不保存用户私钥，不成为链上状态真源。

## 决策

### 1. 总体分层

目标架构采用四层协作：

```text
CitizenApp
  ↓
Cloudflare 边缘层
  ↓
Citizen API / OnChina 投影能力
  ↓
CitizenChain 云节点网络
```

各层职责固定如下：

- CitizenApp：内置 smoldot 轻节点，连接 CitizenChain P2P 网络；端上私钥只在本机签名；链上关键判断以 finalized 链状态为准。
- Cloudflare 边缘层：提供 DNS/WAF/限流、Worker API Gateway、广场 R2 媒体存储、D1/KV/DO 边缘数据、Chat 瞬时密文/信令转发、广场 feed、轻节点启动清单和服务健康信息。
- Citizen API / OnChina 投影能力：提供非链上查询、公开目录、业务聚合、受控链事件投影和已签名交易广播；不托管私钥，不替用户签名。
- CitizenChain 云节点网络：运行 `citizenchain/node + runtime + onchina` 安装包，承担 bootnode/full node/archive/indexer/RPC service node 等角色。

### 2. 链连接真源

CitizenApp 的链上真源仍是内置轻节点通过 P2P 获取并验证的 finalized 链状态。

Cloudflare 启动清单可以帮助 App 更快进入可用状态，例如提供：

- 当前推荐 bootnodes。
- lightSyncState/checkpoint。
- Worker/OnChina 投影服务健康状态。
- 可选的受控交易广播入口。

这些信息只用于启动加速、服务发现和降级提示，不替代轻节点验证。

### 3. P2P 失败时的产品状态

P2P 连接失败不等于整个 App 不可用。CitizenApp 运行态拆成三种：

- 正常：轻节点已连接 P2P，能够推进 best/finalized，链读写走轻节点。
- 降级：轻节点暂时无法连接 P2P，聊天、广场、公开目录和本地缓存继续可用；链上余额、投票资格、提案状态等关键状态标记为等待链同步或只显示最近 finalized 快照。
- 离线：网络不可用；只展示本地缓存和离线可完成的签名准备，不承诺链上状态。

已签名交易在降级状态下可以通过受控 API 广播到服务节点 RPC，但广播成功只表示节点已收到交易，不表示链上成功。链上成功仍必须以 finalized runtime storage 或事件确认为准。

### 4. 云节点安全边界

国储会核心节点不得作为公民 App 的公共 RPC 入口。

生产节点角色必须拆分：

- 核心/权威节点：持有必要出块、最终性或机构运行能力；只开放必要 P2P；RPC/Prometheus 只允许本机或内网访问。
- 公开 bootnode：只承担 P2P 发现和连接引导，尽量不持有关键业务私钥。
- RPC service node：供 Citizen API、Indexer、Worker 后端侧受控访问，必须经过反向代理、白名单、限流和审计。
- Archive/Indexer：服务历史查询、广场发布确认、公开投影和运维观测，不参与用户私钥保管。

### 5. 聊天和广场

聊天采用 OpenMLS + Cloudflare 瞬时密文转发 + WebRTC 设备附件 + 近场通信。消息、会话和附件只保存在设备；Cloudflare 只保存设备公钥、推送 Token、一次性 KeyPackage、防重放哈希和短期 TURN 索引。

广场继续采用当前正式路线：媒体文件存 Cloudflare R2，CDN 分发；CitizenChain 只保存发布所需的链上元数据、哈希、索引和费用结果；Worker/D1/KV/DO 承接登录、会员、上传、feed、推荐信号和发布确认。

当前 ADR 不引入 Matrix，不恢复区块链节点聊天，不把聊天消息写入链上。

### 6. OnChina 边界

OnChina 是 CitizenChain 安装包内置能力，不是第五个产品。OnChina 可以提供局域网机构工作台、链上投影、公开目录和受控服务端聚合能力，但不得成为 CitizenApp 链上状态真源，也不得恢复旧独立后端结构。

## 禁止项

- 禁止把 CitizenApp 改成所有链读写都依赖 `api.onchina.org` 的 API-only 客户端。
- 禁止让 Cloudflare Worker 持有或接触用户私钥。
- 禁止把国储会核心节点 RPC `9944` 或 Prometheus `9615` 直接暴露到公网。
- 禁止把 API 广播成功显示成链上交易成功。
- 禁止把 CitizenWallet 写入在线链连接、聊天或广场架构。
- 禁止把 Matrix 写成当前聊天目标路线。
- 禁止恢复区块链节点聊天、云端聊天内容存储或节点配对流程。

## 影响

后续实施必须分步确认：

1. Cloudflare 启动清单 API。
2. CitizenApp 链连接状态机。
3. 已签名交易受控广播兜底。
4. Oracle Cloud 节点角色拆分与防火墙口径。
5. 聊天和广场生产硬化。
6. 安全、审计、限流和可观测性。
7. 真实运行态验收。

任何涉及 `citizenchain/runtime/` 的修改，仍必须按 runtime 二次确认硬规则单独说明路径、内容和原因，并获得第二次确认。
