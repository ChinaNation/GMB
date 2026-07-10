# 任务卡：CitizenChain 云节点部署边界与端口安全

## 1. 任务背景

本任务用于记录后续部署 CitizenChain 云服务器节点时的安全边界。

用户已明确：

- 本步骤讨论的是安装区块链软件的云服务器节点，不讨论公民钱包。
- CitizenChain 是一个整体安装包，由 `node`、`runtime`、`onchina` 组成。
- 引导节点、验证节点等不是独立软件，而是安装区块链软件后按权限、启动参数和部署角色运行的服务器。
- 冻结 chainspec 固定包含 44 个权威引导节点；国储会节点是第 1 个，其他 43 个后续逐步部署。
- 国储会权威节点本身就是公开 bootnode，不再拆出另一台公开引导节点或首期独立 RPC 节点。
- 目标是让 CitizenApp 能快速稳定连接 P2P 网络，同时减少国储会区块链节点被攻击的可能性。

## 2. 目标状态

云节点部署优先通过服务器安全组、防火墙、内网绑定、启动参数和部署配置完成隔离。

默认目标：

- 不修改 `citizenchain/runtime/`。
- 不改变链上共识和业务逻辑。
- 不把任何权威引导节点 RPC 暴露给公网。
- 不把 RPC URL 下发给 CitizenApp。
- 不让 Cloudflare Worker 成为链上状态真源。
- 不把聊天或广场迁移到区块链节点。

只有当真实检查发现现有 `node` 或部署脚本把 RPC、Prometheus、bootnode、validator 参数硬编码到无法通过部署配置调整时，才另行提出代码或脚本修改方案，并按仓库规则单独确认。

## 3. 节点与端口边界

### 3.1 权威 Bootnode / 引导节点

- 定位：同一台 CitizenChain 节点同时承担权威身份、P2P 发现、链同步和其权限允许的共识职责。
- 必须开放：`30333/TCP` 的 WSS/libp2p 公网入口。
- 必须关闭公网：`9944` RPC、`9615` Prometheus、`8964` OnChina、`5433` 内嵌 PostgreSQL和管理端口。
- CitizenApp：可通过 bootstrap 清单获得公开 bootnode multiaddr。

### 3.2 国储会首发节点

- 国储会节点是 44 个权威引导节点中的第 1 个，不是隐藏在公开 bootnode 后面的另一台核心服务器。
- 首期 Worker 只需要一个链上游，通过 Access + 独立 Tunnel 访问国储会节点的 `127.0.0.1:9944`。
- Tunnel 由服务器主动出站连接 Cloudflare，Oracle 和主机防火墙不开放 `9944` 入站。
- CitizenApp 只直连国储会节点的 `30333/wss` P2P，不直连 RPC。

### 3.3 后续权威节点

- 其余 43 个节点部署后全部开放同样的 `30333/TCP` P2P 入口，其余端口默认关闭。
- Worker 稳定阶段最多选择少量不同地区的权威节点作为私有 RPC 备用，每个节点使用独立 Tunnel 和凭证，不连接全部 44 个节点。
- 广播成功只代表交易进入 RPC/交易池流程，不代表链上成功。

### 3.4 Archive / Indexer 能力

- 后续如在现有安装节点上启用历史、索引、审计能力，只允许本机或受控私网访问。
- 不得因此开放公网数据库、索引器管理端口、Prometheus 或裸 RPC；CitizenApp 不直接连接。

## 4. 后续部署前必须检查

部署执行前必须先读取仓库代码、启动脚本、配置文件和真实运行输出，不得猜测当前实现：

- 检查 `citizenchain/node/` 的 RPC、WS、P2P、Prometheus、bootnode、validator 启动参数。
- 检查安装包或 systemd 启动配置是否只公开 P2P，并保持 RPC 回环监听。
- 检查 RPC 是否绑定到 `0.0.0.0`、内网地址或本机地址。
- 检查 Oracle Cloud 安全组、主机防火墙、Cloudflare Tunnel 或反向代理配置。
- 检查 bootstrap 清单只下发 bootnode multiaddr，不下发 RPC URL。
- 检查 Worker 只通过 Access + Tunnel 连接被选中权威节点的本机 RPC。
- 检查 Prometheus、数据库、管理界面是否仅内网可见。

## 5. 预计修改目录

- `/Users/rhett/GMB/memory/08-tasks/open/`
  - 用途：持续记录云节点部署边界、每步确认和验收结果。
  - 边界：修改现有任务卡，不新增任务卡。
  - 类型：文档。
  - 残留清理：删除旧角色拆分和公网 RPC 口径。
- `/Users/rhett/GMB/memory/01-architecture/citizenchain/`
  - 用途：记录权威引导节点、Oracle Cloud 安全组和 Cloudflare 私有连接拓扑。
  - 边界：只修改架构文档，不改 runtime。
  - 类型：文档。
  - 残留清理：清理核心节点、公开 bootnode 和独立 RPC 节点的错误拆分。
- `/Users/rhett/GMB/memory/05-modules/citizenchain/node/`
  - 用途：记录节点 RPC、P2P peer 上限和云部署安全基线。
  - 边界：本步骤只修改节点技术文档。
  - 类型：文档。
  - 残留清理：删除允许公网 Unsafe RPC 的旧建议。
- `/Users/rhett/GMB/citizenchain/scripts/`
  - 用途：维护不含密钥的权威引导节点 systemd 标准模板。
  - 边界：修改现有模板，不新增脚本，不写入服务器账户、密钥或 Tunnel token。
  - 类型：部署配置。
  - 残留清理：删除 `--rpc-external` 和 `--rpc-cors all`。
- `/Users/rhett/GMB/citizenapp/cloudflare/`
  - 用途：实现 Worker 到 Access + Tunnel 私有 RPC 的受控请求边界，并为后续单主多备上游预留统一调用层。
  - 边界：步骤 2 只修改现有 Worker、测试和配置注释，不创建 Tunnel、不部署远端、不改 CitizenApp 轻节点。
  - 类型：Worker 代码、测试、配置注释和文档。
  - 残留清理：已删除旧单一 RPC 变量名；未上线 bootnode 推荐留待单独步骤按真实部署状态处理。

## 6. 风险点

- 如果权威引导节点 RPC 暴露到公网，会显著增加国储会和后续节点的攻击面。
- 如果 bootnode 的 P2P 入口也被关闭，CitizenApp 轻节点和其他节点可能无法发现网络。
- 如果 Worker 绕过 Access + Tunnel 直接访问公网 RPC，会把权威节点拖进公网业务流量。
- 如果部署脚本把 RPC 绑定公网作为默认行为，需要先调整启动参数或脚本，而不能只依赖文档约束。
- 如果在未检查真实代码和运行输出前写死端口号，可能形成错误部署手册。

## 7. 后续实施步骤

1. [x] 只读检查 `node` 启动参数、部署脚本和现有文档。
2. [x] 输出权威引导节点与端口矩阵技术方案并取得确认。
3. [x] 修正权威引导节点 systemd 模板和 Oracle Cloud 端口口径。
4. [x] 收敛 Worker 链 RPC 调用层、Access 服务令牌和已签名交易广播边界。
5. [ ] 设计并建立国储会节点的 Access + 独立 Tunnel 私有 RPC 连接。
6. [ ] 用真实服务器验证 P2P 可连、RPC 不公网暴露、Worker 可通过 Tunnel 连接本机 RPC。

## 8. 验收标准

- Bootnode 的公网 P2P 可用。
- 44 个权威引导节点只开放公网 `30333/TCP`，不开放 RPC / Prometheus / OnChina / 数据库 / 管理端口。
- 被选中作为 Worker RPC 上游的节点仍不开放 RPC 入站，只允许 Access + Tunnel 访问本机 RPC。
- Archive / Indexer / 数据库不公网开放。
- CitizenApp bootstrap 清单不返回任何 RPC URL。
- Worker relay 和链读取只能通过受控 Tunnel 连接本机 RPC。
- 所有真实端口和启动参数必须来自实际检查或部署输出，不得凭经验填写。

## 9. 执行记录

- 2026-07-08：用户确认创建本任务卡；本次仅记录部署边界，不修改代码、不修改 `citizenchain/runtime/`、不修改 CitizenWallet、不部署云服务器。
- 2026-07-10：完成只读检查，确认 44 个 bootnode 中第 1 个为国储会权威节点；发现 systemd 示例含 `--rpc-external`、Oracle 文档使用无效的 `--chain mainnet`、网络概览依赖远程公网 `9944`。
- 2026-07-10：用户确认执行步骤 1，开始修正权威 bootnode 启动与端口安全基线；本步骤不修改 runtime、不连接 Cloudflare、不操作线上 Oracle 服务器。
- 2026-07-10：步骤 1 完成。标准 systemd 模板已删除 `--rpc-external` 和 `--rpc-cors all`，固定 `--chain citizenchain`、回环 RPC、关闭 Prometheus、`30333/TCP` WSS 和显式 peer 上限，并增加专用账户文件权限与 systemd 基础沙箱。
- 2026-07-10：使用当前 `target/debug/citizenchain` 设置 `CITIZENCHAIN_HEADLESS=1`，把模板完整参数原样传入并执行 `--help`，退出码为 0；`export-chain-spec --chain citizenchain` 真实加载冻结正式 chainspec 成功并返回 44 个 bootnode；`citizenchain --version` 返回 `1.0.0-f2100ee50e8`。`git diff --check` 通过，`citizenchain/runtime/` 无 diff。
- 2026-07-10：当前 macOS 工作区没有 `systemd-analyze`，且本步骤未登录或修改线上 Oracle 服务器；Linux unit 校验、真实监听地址和公网端口验收保留到部署步骤执行，不能记为已通过。
- 2026-07-10：用户确认执行步骤 2。Worker 删除旧单一 RPC 变量，改为 `CITIZEN_CHAIN_RPC_URL` 与两项 `CITIZEN_CHAIN_RPC_ACCESS_*` 远端 Secret；统一调用层只允许 `state_getStorage`、`author_submitExtrinsic`，强制 HTTPS、Access 请求头、3 秒超时、4 MiB 响应上限、禁止重定向且不自动重试。relay 只有在开关和三项 Secret 全部有效时才对 App 显示为启用。
- 2026-07-10：步骤 2 链相关 18 项 Vitest 通过，随后 Worker 全量类型检查和 18 个测试文件共 103 项测试全部通过；统一调用层的 Access 请求头、HTTPS 拒绝、响应限长、超时/传输/RPC 语义错误分类和 D1 失败审计均已覆盖。本步骤未创建或部署 Tunnel、Access 应用、远端 Secret，也未访问 Oracle 服务器；真实私有链路验收保留到步骤 3。
- 2026-07-10：步骤 2 完成真实本地运行态验收。Wrangler `4.107.0` dry-run 打包通过，本地 Worker 的 `/health` 与 `/v1/chain/bootstrap` 返回 200，三项临时配置使 relay 启用且响应不泄露 RPC URL，`/v1/chain/rpc` 返回 404。初次验收发现 workerd 不支持 `redirect=error`，已改为受支持的 `redirect=manual` 并继续拒绝所有 3xx，防止 Access 令牌随跳转外发。重新打包后，Miniflare/workerd 真实执行 relay 返回 202，HTTPS 上游截获 `author_submitExtrinsic` 和正确的两项 Access 头，D1 审计行为 `broadcast`、交易哈希一致。全部 `/tmp` 验收文件、证书、数据库和进程均已清理。
