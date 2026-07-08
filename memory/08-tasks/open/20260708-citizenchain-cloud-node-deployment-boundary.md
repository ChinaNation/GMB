# 任务卡：CitizenChain 云节点部署边界与端口安全

## 1. 任务背景

本任务用于记录后续部署 CitizenChain 云服务器节点时的安全边界。

用户已明确：

- 本步骤讨论的是安装区块链软件的云服务器节点，不讨论公民钱包。
- CitizenChain 是一个整体安装包，由 `node`、`runtime`、`onchina` 组成。
- 引导节点、验证节点等不是独立软件，而是安装区块链软件后按权限、启动参数和部署角色运行的服务器。
- 国储会等核心节点可部署在 Oracle Cloud 等云服务器上。
- 目标是让 CitizenApp 能快速稳定连接 P2P 网络，同时减少国储会区块链节点被攻击的可能性。

## 2. 目标状态

云节点部署优先通过服务器安全组、防火墙、内网绑定、启动参数和部署配置完成隔离。

默认目标：

- 不修改 `citizenchain/runtime/`。
- 不改变链上共识和业务逻辑。
- 不把 Validator RPC 暴露给公网。
- 不把 RPC URL 下发给 CitizenApp。
- 不让 Cloudflare Worker 成为链上状态真源。
- 不把聊天或广场迁移到区块链节点。

只有当真实检查发现现有 `node` 或部署脚本把 RPC、Prometheus、bootnode、validator 参数硬编码到无法通过部署配置调整时，才另行提出代码或脚本修改方案，并按仓库规则单独确认。

## 3. 节点角色边界

### 3.1 Bootnode / 引导节点

- 定位：为 CitizenChain P2P 网络提供可发现入口。
- 软件：仍是安装区块链软件的节点，只是按引导节点角色运行。
- 必须开放：必要的公网 P2P 入口。
- 不应开放：公网 RPC、WS RPC、Prometheus、管理端口。
- CitizenApp：可通过 bootstrap 清单获得公开 bootnode multiaddr。

### 3.2 Validator / 验证节点

- 定位：参与共识、出块或验证。
- 必须开放：共识所需 P2P 通信，具体范围以后续真实部署检查为准。
- 不应开放：公网 RPC、WS RPC、Prometheus、管理端口。
- Cloudflare Worker：不得直接使用核心 Validator RPC。
- CitizenApp：不得直连 Validator RPC。

### 3.3 RPC Service Node / 服务节点

- 定位：给 Worker / Citizen API 提供受控链交互能力。
- 可提供：受控 RPC，例如读取事件、读取公开 storage、广播完整 signed extrinsic。
- 访问范围：只允许 Cloudflare Tunnel、内网、固定出口或安全组白名单访问。
- 不应开放：面向公网 App 的裸 RPC。
- 注意：广播成功只代表交易进入 RPC/交易池流程，不代表链上成功。

### 3.4 Archive / Indexer 节点

- 定位：历史数据、索引、审计和查询。
- 访问范围：内网或后端服务访问。
- 不应开放：公网数据库、索引器管理端口、Prometheus、裸 RPC。
- CitizenApp：不直接连接。

## 4. 后续部署前必须检查

部署执行前必须先读取仓库代码、启动脚本、配置文件和真实运行输出，不得猜测当前实现：

- 检查 `citizenchain/node/` 的 RPC、WS、P2P、Prometheus、bootnode、validator 启动参数。
- 检查安装包或 systemd/docker 启动配置是否能区分 bootnode、validator、RPC service node、archive/indexer。
- 检查 RPC 是否绑定到 `0.0.0.0`、内网地址或本机地址。
- 检查 Oracle Cloud 安全组、主机防火墙、Cloudflare Tunnel 或反向代理配置。
- 检查 bootstrap 清单只下发 bootnode multiaddr，不下发 RPC URL。
- 检查 Worker 只连接 service node RPC，不连接核心 Validator RPC。
- 检查 Prometheus、数据库、管理界面是否仅内网可见。

## 5. 预计修改目录

- `/Users/rhett/GMB/memory/08-tasks/open/`
  - 用途：记录本云节点部署边界任务卡。
  - 边界：只新增本任务卡，不创建部署脚本。
  - 类型：文档。
  - 残留清理：后续部署完成后补充真实端口、节点角色和验收结果。
- `/Users/rhett/GMB/memory/01-architecture/citizenchain/`
  - 用途：后续记录 CitizenChain 云节点角色、Oracle Cloud 安全组和部署拓扑。
  - 边界：不改 runtime，不写未验证端口号。
  - 类型：文档。
  - 残留清理：清理“核心节点公开 RPC”等冲突描述。
- `/Users/rhett/GMB/memory/01-architecture/citizenapp/`
  - 用途：后续同步 CitizenApp 只使用 bootnode 和 Worker 受控接口的边界。
  - 边界：不把 CitizenApp 改成 API-only 客户端。
  - 类型：文档。
  - 残留清理：避免文档出现 App 直连 Validator RPC。
- `/Users/rhett/GMB/citizenchain/node/`
  - 用途：后续只读检查节点启动参数和实际端口绑定来源。
  - 边界：未单独确认前不改代码。
  - 类型：只读检查，必要时另行确认代码或配置修改。
  - 残留清理：不适用。
- `/Users/rhett/GMB/citizenchain/scripts/`
  - 用途：后续只读检查模块内非密钥部署脚本。
  - 边界：不得写入密钥；如需新增或修改脚本，必须另行列路径确认。
  - 类型：只读检查，必要时另行确认脚本修改。
  - 残留清理：密钥脚本必须留在根目录 `scripts/` 且被 Git 忽略。

## 6. 风险点

- 如果 Validator RPC 暴露到公网，会显著增加国储会核心节点攻击面。
- 如果 bootnode 的 P2P 入口也被关闭，CitizenApp 轻节点和其他节点可能无法发现网络。
- 如果 Worker 使用 Validator RPC，而不是 service node RPC，会把核心节点拖进公网业务流量。
- 如果部署脚本把 RPC 绑定公网作为默认行为，需要先调整启动参数或脚本，而不能只依赖文档约束。
- 如果在未检查真实代码和运行输出前写死端口号，可能形成错误部署手册。

## 7. 后续实施步骤

1. 只读检查 `node` 启动参数、部署脚本和现有文档。
2. 输出节点角色与端口矩阵技术方案，等待确认。
3. 如只需云服务器调整，则记录 Oracle Cloud 安全组、防火墙、内网绑定、Cloudflare Tunnel 建议。
4. 如需要修改安装包启动参数或脚本，则单独列出路径、原因和风险，等待确认后执行。
5. 部署时用真实服务器或本地等价服务验证 P2P 可连、RPC 不公网暴露、Worker 可连 service node。

## 8. 验收标准

- Bootnode 的公网 P2P 可用。
- Validator 不开放公网 RPC / WS RPC / Prometheus / 管理端口。
- RPC service node 只允许受控后端访问。
- Archive / Indexer / 数据库不公网开放。
- CitizenApp bootstrap 清单不返回任何 RPC URL。
- Worker relay 和链读取不连接核心 Validator RPC。
- 所有真实端口和启动参数必须来自实际检查或部署输出，不得凭经验填写。

## 9. 执行记录

- 2026-07-08：用户确认创建本任务卡；本次仅记录部署边界，不修改代码、不修改 `citizenchain/runtime/`、不修改 CitizenWallet、不部署云服务器。
