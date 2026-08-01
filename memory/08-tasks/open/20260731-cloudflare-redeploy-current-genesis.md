# CitizenApp Cloudflare 按当前创世重新部署

状态：**生产部署已完成（2026-07-31）**，剩余前置项：国储会 cloudflared 未安装

## 部署结果（2026-07-31 只读实测）

用户已在 CitizenConsole 执行生产部署。`GET https://www.crcfrcn.com/api/v1/chain/bootstrap`
返回 HTTP 200，链身份与仓库冻结值逐字节一致：

- `genesis_hash` = `0x278e68bced2dabf9690701188272da22d216fdaa2c617e7dcbe100df3e8bcbfa` ✅
- `state_root` = `0xa5386e7c0a0222fd030250b533bf73e78e947aec9f6a98dea7c1d5d64881c8c2` ✅
- `bootnodes` = 5 条：`nrcgch` / `prczss` / `prcgzs` / `prches` / `prchbs` ✅
  （本轮 bootNodes 变更已随该次部署一并上线）

**仍未打通的一段**：国储会 `nrcgch` 上 cloudflared **从未安装**（`which cloudflared` 未安装、
`cloudflared.service` Unit not found、`/etc/cloudflared` 不存在）——该机曾删除实例重建，
connector 没跟着装回。Tunnel `242190b5-…` 两端无 connector，边缘返回 502
`chain_rpc_http_failed`。网关与节点两段均正常（见下节实测），断点确切在 connector。

## 原始缺陷记录（已解决，保留追溯）

线上 Worker 曾停留在上一次创世的部署，与仓库冻结的当前创世不是同一条链。

只读实测（`GET https://www.crcfrcn.com/api/v1/chain/bootstrap`，2026-07-31）：

| 来源 | genesis_hash | state_root |
| --- | --- | --- |
| 线上 Worker | `0xe8f4067d…f8cf8eb45` | `0xbdc2593a…ae18f9` |
| 仓库 `wrangler.toml:74-75` | `0x278e68bc…3e8bcbfa` | `0xa5386e7c…81c8c2` |
| App 内置 `assets/chainspec.json` | — | `0xa5386e7c…81c8c2` |
| 本地链 block 0 实测 | `0x278e68bc…3e8bcbfa` | `0xa5386e7c…81c8c2` |

「正式创世冻结」改好了 `wrangler.toml`，但 Worker 从未随之重新部署。

## 影响

1. 设备子钥绑定失败 → 聊天/广场落 `IdentityRegistrationGate` 的 `bindFailed`
   「设备绑定未完成」。链路：`ensureDeviceSubkeyBound` → `takeoverCidDataRoot`
   → Worker `requireCurrentFinalizedBinding` 读链拿不到当前 CID 绑定 →
   401 `cid_binding_changed`（或 CHAIN_URL 未配时 503 `chain_rpc_not_configured`）。
2. `smoldot_client.dart` 的 `_bootstrapMatchesLocalSpec` 因 state_root 不匹配，
   每次启动都走「链启动清单与本地 chainspec 不一致，跳过远端 bootnodes」，
   只能用安装包内置 bootnodes。

## 已完成（本地预检，等价于部署脚本步骤 2-4）

- `npm run typecheck` 通过；`worker-configuration.d.ts` 无漂移（`types:check` 亦会过）。
- `npm test` 32 文件 / 222 用例全绿。
- `wrangler.toml` 的 `CHAIN_GENESIS_HASH` / `CHAIN_STATE_ROOT` 已确认等于当前创世。

### 追加：本卡待部署内容已含 bootNodes 变更（2026-07-31，另一线程）

按用户「bootNodes 更新为现在的 5 个节点」指令，`wrangler.toml` 的 `CHAIN_BOOTNODES`
与 `worker-configuration.d.ts` 已由 6 条改为 5 条：删除山东 `prcsds`、山西 `prcsxs`，
加入贵州 `prcgzs`（`12D3KooWC7t4V1Z2aQWS9HikBdXQgXEaTqeZ5YD78cnxtYBDn31M`），
顺序统一为 `nrcgch`、`prczss`、`prcgzs`、`prches`、`prchbs`。同批已同步节点 plain SSOT
与 App 轻形态 chainspec，`chainspec_hash` 因此变更（节点
`df2e5a28…`→`ce353fb3a7b078dce9a6da0c065a4a883df8892882a498336890aea5d04e29b5`，App
`f1e949e2…`→`9cb95a69368199f79172724e41ee1afa7593ece796cdbb5f58da3004d7fc8a19`），
`public_institutions/manifest.json` 的 `chainspec_hash` 已回写，冻结门禁通过。
**`genesis_hash` 与 `state_root` 未变**（bootNodes 在 chainspec 顶层，不进 `genesis`
字段，实测 `stateRootHash`、runtime `:code` blake2_256 `0x8d92e92c…`、patch 19 段均不变），
本卡的部署目标与复验基线不受影响。

本卡执行生产部署时会一并把新 bootNodes 推上线，无需另开一卡。另需在 Cloudflare DNS
删除 `prcsds`、`prcsxs` 两条记录、新增 `prcgzs`，保留 7 条：`www`、`chain`、`nrcgch`、
`prczss`、`prcgzs`、`prches`、`prchbs`。

### 追加：节点侧三段实测与 cloudflared 前置（2026-07-31）

Tunnel 链路 `Worker → CHAIN_URL → Tunnel 242190b5-… → connector → 127.0.0.1:18080 网关
→ 127.0.0.1:9944 节点`，在国储会实测三段：

| 段 | 实测结果 |
| --- | --- |
| ① connector | **未安装** —— `which cloudflared` 无、Unit not found、`/etc/cloudflared` 不存在 |
| ② 网关 18080 | **正常** —— `/health` 返回 `ok`，RPC 隐藏路径 POST 返回 HTTP 200 |
| ③ 节点 9944 | **正常** —— 创世 `0x278e68bc…`，版本 `0.0.0-369cbc5a9a4`，finalized head 有值 |

**排查网关时不能用根路径探活**：`guo-rpc` 站点里 `location / { return 404; }` 是设计如此，
`curl http://127.0.0.1:18080` 返回 404 属正常，正确探活是 `/health`。

**`CHAIN_URL` 必须带完整路径**：Worker 端 `fetch(config.url, …)` 原样使用该值、不拼接
pathname，而 nginx 只在 `/rpc-d74b1d75c4e8efcef5fa417e181a2c71e5257773e1b6c4dc` 上转发
RPC。若 secret 只填到域名，装完 tunnel 后症状会从 502 变成打到根路径的 404，`error_code`
仍是 `chain_rpc_http_failed`，极易误判为"tunnel 没装好"。cloudflared 的 ingress 指向
`http://127.0.0.1:18080` 保留路径转发即可，**不要在 ingress 里写死该路径**。

**装 cloudflared 不需要开任何入站端口** —— Tunnel 是出站长连接。国储会当前入站仅
`22`（运维）+ `30333`（P2P），装完不变。

同批已完成的节点侧加固（供部署时参照，避免误判为异常）：内核升至
`6.17.0-1019-oracle`（机器重启过）、`rpcbind` 已关、nginx 默认站点已删（**`guo-rpc`
站点原样保留**）、root 登录已禁（用 `ubuntu` + NOPASSWD sudo）、已装 fail2ban
（`maxretry=10`/`bantime=600`）、`guo-node.service` 已加 systemd 加固。cloudflared 为独立
unit，与上述互不影响。

## 待执行（只能由用户操作）

CitizenConsole → 「☁️ CitizenApp Cloudflare」→ 「生产部署」。

部署走 `citizenconsole/actions/cloudflare.sh production`，凭证 `CF_DEPLOY_TOKEN` /
`CF_DATA_TOKEN` 由控制台原生安全代理在 Touch ID 后注入子进程。Claude 无法执行：
本机 `wrangler whoami` 未认证，且部署令牌属凭证，不得由 Claude 处理。

## 关键前置

脚本步骤 6 会把 Keychain 里 16 个 Secret 原样推到 Worker，其中 `CHAIN_URL` /
`CHAIN_ID` / `CHAIN_SECRET` 决定 Worker 读哪条链。**若 Keychain 里的 `CHAIN_URL`
指向的 RPC 网关仍连旧创世链，部署后 vars 正确但链读依旧错，「设备绑定未完成」不会消失。**
部署前必须确认该网关后面的节点跑的是 `0x278e68bc…` 这条链。

## 部署后复验

1. `GET /api/v1/chain/bootstrap` 的 `genesis_hash` / `state_root` 应等于
   `0x278e68bc…3e8bcbfa` / `0xa5386e7c…81c8c2`。
2. App 进聊天/广场，应通过设备子钥绑定（首次弹一次生物识别）而非「设备绑定未完成」。
3. App 日志不再出现「链启动清单与本地 chainspec 不一致，跳过远端 bootnodes」。

## 主要风险

- 步骤 6 会覆盖 Worker 上全部 16 个 Secret，Keychain 里任一项过期都会带上线。
- 部署脚本无灰度，路由 `www.crcfrcn.com/api/*` 直接切新版本。
- 与本卡同源的 `reset-formal-data`（清空重建全部数据）绝不在本卡范围内执行。
