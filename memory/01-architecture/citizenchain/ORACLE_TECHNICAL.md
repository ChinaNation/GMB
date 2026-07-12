# CitizenChain Oracle Cloud 部署技术文档

## 1. 文档目的
- 固化 `citizenchain` 在 Oracle Cloud 云服务器上的标准部署流程。
- 统一“从空白服务器到节点启动”的操作口径，便于后续重复部署、迁移和运维。
- 明确 `citizenchain` 在 Oracle Cloud 上的依赖、端口、安全边界与常见排障方式。

## 2. 适用范围
- 适用于将 `GMB` 仓库中的 [`citizenchain`](/Users/rhett/GMB/citizenchain) 部署到 Oracle Cloud Linux 云服务器。
- 适用于普通全节点、参与 PoW 挖矿的节点，以及后续扩展为 `systemd` 常驻服务的节点。
- 默认服务器系统口径为 `Ubuntu`。

## 3. 部署对象说明

### 3.1 当前部署对象
- 部署对象为 `citizenchain/node` 原生链节点程序。
- 该节点为 Rust/Substrate 风格原生程序，不是通过 `apt` 或 Docker 直接安装的现成链客户端。
- 当前项目依赖自定义 `polkadot-sdk` 分支，必须通过源码编译获得节点二进制。

### 3.2 当前链运行口径
- 正式链标识：`citizenchain`
- 共识机制：`PoW + GRANDPA`
- 默认本地 RPC：`127.0.0.1:9944`
- 默认本地 Prometheus：`127.0.0.1:9615`
- 默认 P2P 端口：`30333`
- 首次启动若不存在 `powr` 密钥，节点会自动生成本地 PoW 作者密钥

### 3.3 生产节点角色口径

冻结 chainspec 固定包含 44 个权威引导节点。国储会节点是第 1 个权威 bootnode，其余 43 个后续逐步部署；权威节点和公开 bootnode 是同一台安装 CitizenChain 软件的服务器，不拆成两种节点。

- 公开入口：每个权威引导节点只开放 `30333/TCP` 的 WSS/libp2p，服务 CitizenApp 轻节点、普通全节点和其他权威节点。
- 本机入口：RPC `9944` 只监听 `127.0.0.1`；Prometheus 默认关闭；OnChina、数据库和管理端口不向公网开放。
- Cloudflare 链连接：首期 Worker 只通过 Access + 独立 Tunnel 访问国储会节点的本机 RPC，不需要另建独立 RPC 节点。Worker 侧必须使用成套的 RPC HTTPS URL 与 Access 服务令牌 Secret，缺失任一项时关闭 relay；当前代码不接受公网 HTTP、回环 HTTP 或无 Access 凭据的上游。
- 后续容灾：最多选择少量不同地区的权威引导节点作为 Worker 私有 RPC 备用，每个节点使用独立 Tunnel 和凭证，不连接全部 44 个节点。

CitizenApp 不直接依赖国储会节点 RPC；App 的链上真源是内置轻节点验证的 finalized 链状态。

相关实现参考：
- [`CITIZENCHAIN_TECHNICAL.md`](/Users/rhett/GMB/memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md)
- [`node/src/core/command.rs`](/Users/rhett/GMB/citizenchain/node/src/core/command.rs)
- [`node/src/core/service.rs`](/Users/rhett/GMB/citizenchain/node/src/core/service.rs)
- [`node/src/core/chain_spec.rs`](/Users/rhett/GMB/citizenchain/node/src/core/chain_spec.rs)

## 4. 总体部署步骤
- 第 1 步：准备 Oracle Cloud 云服务器
- 第 2 步：安装系统依赖与 Rust 工具链
- 第 3 步：获取 `GMB` 源码
- 第 4 步：编译 `citizenchain` 节点
- 第 5 步：手工启动并验证节点
- 第 6 步：配置防火墙与 Oracle Cloud 入站规则
- 第 7 步：配置 `systemd` 常驻运行
- 第 8 步：运行检查与常见排障

## 5. 第 1 步：准备 Oracle Cloud 云服务器

### 5.1 建议实例规格
- CPU：至少 2 核，建议 4 核及以上
- 内存：至少 4GB，建议 8GB 及以上
- 系统盘：至少 80GB，建议更大
- 网络：具备固定公网 IP，仅为 `30333/TCP` P2P 提供公网入口

### 5.2 登录服务器
只有临时运维窗口已经通过 Oracle Bastion 或固定来源 IP 放行 SSH 时，才直接登录：

```bash
ssh -i your-key.pem ubuntu@<ORACLE_SERVER_IP>
```

说明：
- 某些 Oracle Cloud 镜像用户名可能不是 `ubuntu`，也可能是 `opc`。
- 若首次连接，系统会要求确认主机指纹。
- 没有公网运维需求时不得开放 `22/TCP`；应保留 Oracle 控制台或 Bastion 作为恢复通道。

## 6. 第 2 步：安装系统依赖与 Rust 工具链

### 6.1 更新系统

```bash
sudo apt update
sudo apt upgrade -y
```

### 6.2 安装基础依赖

```bash
sudo apt install -y build-essential clang cmake pkg-config libssl-dev git curl protobuf-compiler
```

说明：
- `build-essential`、`clang`、`cmake`、`pkg-config` 用于编译 Rust 和 Substrate 相关依赖。
- `libssl-dev` 用于 TLS/加密相关编译依赖。
- `protobuf-compiler` 常见于 Rust 区块链依赖链路。

### 6.3 安装 Rust

```bash
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"
rustup default stable
```

### 6.4 验证 Rust 环境

```bash
rustc --version
cargo --version
```

## 7. 第 3 步：获取 GMB 源码

### 7.1 克隆仓库

```bash
git clone <YOUR_GMB_REPOSITORY_URL>
```

### 7.2 进入 CitizenChain 目录

```bash
cd GMB/citizenchain
```

### 7.3 源码依赖说明
- 当前工作区依赖 `https://github.com/ChinaNation/polkadot-sdk.git`
- 依赖分支为 `ss58-2027-fix`
- 因此服务器必须具备访问 GitHub 的能力，否则 `cargo build` 无法完成依赖拉取

相关参考：
- [`Cargo.toml`](/Users/rhett/GMB/citizenchain/Cargo.toml)
- [`node/Cargo.toml`](/Users/rhett/GMB/citizenchain/node/Cargo.toml)

## 8. 第 4 步：编译 CitizenChain 节点

### 8.1 执行编译

```bash
cargo build --release -p node
```

说明：
- `-p node` 表示只编译节点程序。
- 首次编译时间可能较长，因为需要下载并构建 Substrate 相关依赖。

### 8.2 编译输出位置
编译成功后，节点二进制位于：

```text
target/release/citizenchain
```

### 8.3 验证节点帮助信息

```bash
./target/release/citizenchain --help
```

若帮助信息正常输出，说明节点二进制已可运行。

## 9. 第 5 步：手工启动并验证节点

### 9.1 准备数据目录

```bash
mkdir -p /home/ubuntu/citizenchain-data
```

如果当前登录用户不是 `ubuntu`，请改成实际用户目录。

### 9.2 启动权威引导节点

```bash
./target/release/citizenchain \
  --chain citizenchain \
  --name oracle-citizenchain-01 \
  --base-path /home/ubuntu/citizenchain-data \
  --listen-addr /ip4/0.0.0.0/tcp/30333/wss \
  --rpc-port 9944 \
  --rpc-methods Safe \
  --no-prometheus \
  --in-peers 32 \
  --in-peers-light 100 \
  --out-peers 8 \
  --max-parallel-downloads 5 \
  --no-mdns \
  --mining-threads 2
```

参数说明：
- `--chain citizenchain`：显式加载冻结正式 chainspec
- `--name`：设置节点名称，便于识别
- `--base-path`：指定数据库、网络密钥、keystore 等本地数据目录
- `--listen-addr`：只把 `30333/TCP` 作为公网 WSS/libp2p 入口
- `--rpc-methods Safe`：RPC 保持回环监听并限制为安全方法；禁止追加 `--rpc-external`
- peer 数量参数：固定当前 SDK 默认安全基线，扩容前必须压测
- `--mining-threads 2`：启用 2 个 CPU 挖矿线程

### 9.3 关于链规格说明
- 生产部署必须使用 `citizenchain` 或省略 `--chain`。
- `dev`、`local`、`staging` 当前也会加载同一份冻结 chainspec，不会创建另一条临时链。
- `citizenchain-fresh` 只供本机 bake 流程使用；`mainnet` 不是内置链标识。

### 9.4 启动后预期行为
- 节点开始连接 chain spec 中内置的 bootnodes
- 首次启动若本地无 `powr` 密钥，会自动生成
- 节点开始同步区块，并在满足条件时参与 PoW 出块

### 9.5 验证节点是否正常运行
可观察控制台日志中是否出现以下类型信息：
- 已启动网络服务
- 已连接到其他 peers
- 正在同步或导入区块
- 已启动 RPC 服务

如需停止前台运行，使用：

```bash
Ctrl+C
```

## 10. 第 6 步：配置防火墙与 Oracle Cloud 入站规则

### 10.1 服务器本机防火墙

```bash
sudo ufw default deny incoming
sudo ufw default allow outgoing
sudo ufw allow 30333/tcp
sudo ufw enable
```

说明：
- `30333/tcp`：CitizenChain P2P 通信端口
- 没有运维需求时不开放 `22/tcp`；临时 SSH 必须限定固定来源 CIDR，并在操作完成后删除规则

### 10.2 Oracle Cloud 控制台安全规则
Oracle Cloud Network Security Group 只创建一条固定公网业务入站规则：

- 来源：`0.0.0.0/0`
- 协议：TCP
- 目标端口：`30333`

不得为 `9944`、`9615`、`8964`、`5433` 创建公网入站规则。没有运维需求时也不得为 `22` 创建公网规则。

### 10.3 RPC 与 Prometheus 安全边界
- 默认 RPC：`127.0.0.1:9944`
- Prometheus：权威引导节点默认使用 `--no-prometheus` 关闭
- OnChina：节点启动时默认不启动；启用时也只能绑定本机或受控私网

禁止直接暴露以下端口到公网：
- `9944`
- `9615`
- `8964`
- `5433`

### 10.4 权威引导节点安全边界

国储会和后续 43 个权威引导节点使用相同边界：

- `30333/tcp`：CitizenChain P2P 通信端口。
- `22/tcp`：默认关闭；确需运维时只通过 Oracle Bastion、Cloudflare Access 或固定来源临时放行。

权威节点的 `9944` 必须保持回环监听。后续 Worker 访问国储会本机 RPC 时使用服务器主动出站的 Cloudflare Tunnel，仍不开放任何 RPC 入站端口。

### 10.5 国储会私有 RPC Tunnel

国储会节点是 44 个权威 bootnode 中的第 1 个，也是首期唯一 Worker 链上游，不另建 Cloudflare 节点或独立 RPC 节点。Cloudflare 只运行边缘 Worker、Access 和 Tunnel 控制面；安装在国储会服务器上的 `cloudflared` connector 主动建立出站连接，并把受控请求转发到同机 `127.0.0.1:9944`。

2026-07-12 实查 Cloudflare 控制面后的当前状态：

- 远程管理 Tunnel `nrcgch-rpc` 健康，运行 1 个 connector。
- 唯一链入口为 `chain.crcfrcn.com` 的 Access 保护路径，Tunnel 转发到 `127.0.0.1:18080` 固定方法网关，网关再连接本机 `127.0.0.1:9944`。
- Access 使用 `chain` 自托管应用、`CitizenChain` Service Auth 策略和唯一链服务令牌；Worker 的 `CHAIN_URL`、`CHAIN_ID`、`CHAIN_SECRET` 只保存在远端 Secret。
- 不增加 `Everyone`、交互式 `Allow` 或 `Bypass`，也不把令牌、Tunnel token 或完整私有 URL 写入仓库、安装包、日志或命令文档。

服务器部署顺序：

1. 先在服务器本机确认 CitizenChain 正常运行，`9944` 只监听 `127.0.0.1`，`30333` 监听 `0.0.0.0`。
2. 使用 Cloudflare 控制台为既有 `nrcgch-rpc` Tunnel 添加 Linux connector；Tunnel token 只在服务器 root 会话中使用，不写入仓库或普通用户配置。
3. 把 `cloudflared` 安装为 systemd 服务并启动，确认服务开机自启、connector 状态为 Healthy。
4. 不携带 Access 服务令牌访问链保护路径必须被拒绝；携带 Worker 专用服务令牌后必须到达固定方法网关。
5. 从公网确认 `30333/TCP` 可连接且 `9944/TCP` 不可连接，再通过 staging Worker 验证固定链读取方法；全部通过前保持 `CHAIN_EXTRINSIC_RELAY_ENABLED=0`。

`cloudflared` 不是区块链节点，不参与共识、P2P、Runtime 或 RocksDB，也不要求 Oracle 开放任何 Cloudflare 入站端口。首期只连接国储会这一台上游；后续容灾节点必须使用独立 Tunnel、独立 Access 凭证和显式故障切换策略，不连接全部 44 个 bootnode。

## 11. 第 7 步：配置 systemd 常驻运行

### 11.1 创建服务文件

```bash
sudo nano /etc/systemd/system/citizenchain.service
```

写入：

```ini
[Unit]
Description=CitizenChain Node
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=citizenchain
# 权威引导节点只公开 P2P；RPC 保持回环监听。
ExecStart=/usr/bin/citizenchain \
  --chain citizenchain \
  --base-path /opt/citizenchain/data \
  --node-key-file /opt/citizenchain/data/node-key/secret_ed25519 \
  --rpc-port 9944 \
  --rpc-methods Safe \
  --no-prometheus \
  --state-pruning archive \
  --trie-cache-size 268435456 \
  --listen-addr /ip4/0.0.0.0/tcp/30333/wss \
  --in-peers 32 \
  --in-peers-light 100 \
  --out-peers 8 \
  --max-parallel-downloads 5 \
  --no-mdns
Restart=always
RestartSec=5
LimitNOFILE=65536
UMask=0077
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/citizenchain/data
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictSUIDSGID=true

[Install]
WantedBy=multi-user.target
```

说明：
- 安装前创建 `citizenchain` 专用系统账户和 `/opt/citizenchain/data` 数据目录。
- 把编译产物安装为 `/usr/bin/citizenchain`；节点只允许写入自己的数据目录。
- 禁止在 `ExecStart` 中恢复 `--rpc-external`、`--unsafe-rpc-external` 或 `--rpc-cors all`。
- 仓库中的标准模板为 `citizenchain/scripts/citizenchain-node.service`，服务器配置必须与其保持一致。

### 11.2 重新加载并启动服务

```bash
sudo systemctl daemon-reload
sudo systemctl enable citizenchain
sudo systemctl start citizenchain
```

### 11.3 查看服务状态

```bash
sudo systemctl status citizenchain
```

### 11.4 查看实时日志

```bash
journalctl -u citizenchain -f
```

## 12. 第 8 步：运行检查与常见排障

### 12.1 检查进程是否存活

```bash
ps aux | grep citizenchain
```

### 12.2 检查端口监听

```bash
ss -lntp | grep 30333
ss -lntp | grep 9944
ss -lntp | grep 9615
```

预期结果：
- `30333` 监听 `0.0.0.0`，可从公网完成 TCP/WSS 连接。
- `9944` 只能监听 `127.0.0.1` 或 `::1`，不得出现 `0.0.0.0:9944`。
- `9615` 不应存在监听。
- 从外部网络连接 `9944`、`9615`、`8964`、`5433` 和未启用运维时的 `22` 必须失败。

### 12.3 常见问题一：编译失败
可能原因：
- 系统依赖缺失
- Rust 工具链未正确安装
- 无法访问 GitHub 拉取依赖
- 机器内存不足导致编译过程中断

优先检查：

```bash
rustc --version
cargo --version
free -h
df -h
```

### 12.4 常见问题二：节点无法连入网络
可能原因：
- Oracle Cloud 安全组未放行 `30333/TCP`
- 本机 `ufw` 未放行 `30333/TCP`
- 节点无公网出口，无法访问 bootnodes
- DNS 解析异常，导致 `/dns4/...` bootnode 地址无法解析

优先检查：

```bash
ping github.com
nslookup nrcgch.crcfrcn.com
ss -lntp | grep 30333
```

### 12.5 常见问题三：节点启动后马上退出
可能原因：
- `ExecStart` 路径错误
- `WorkingDirectory` 不正确
- 数据目录权限错误
- 端口被占用

优先检查：

```bash
sudo systemctl status citizenchain
journalctl -u citizenchain -n 100 --no-pager
```

### 12.6 常见问题四：同步慢或资源占用高
可能原因：
- CPU 核数不足
- 内存不足
- 磁盘性能较差
- 同时开启的 `mining-threads` 过多

建议：
- 先把 `--mining-threads` 设置为 `1` 或 `2`
- 确保至少 4GB 内存，最好 8GB 及以上
- 尽量使用性能更好的块存储

## 13. 生产部署建议
- 使用固定 `--base-path`，不要用 `--tmp`
- 使用 `systemd` 进行常驻托管
- 国储会和后续 43 个节点都是权威 bootnode，统一只开放公网 `30333/TCP`
- 不把任何权威节点 RPC `9944`、Prometheus、OnChina、数据库或管理端口暴露到公网
- Worker 首期只通过 Access + 独立 Tunnel 访问国储会节点的本机 RPC，不另建独立 RPC 节点
- 定期检查磁盘使用量、日志和同步状态
- 在升级节点版本前，先保留数据目录和服务配置备份

## 14. 最小可执行部署清单
如果只需要最短路径完成部署，可按以下顺序执行：

```bash
sudo apt update
sudo apt upgrade -y
sudo apt install -y build-essential clang cmake pkg-config libssl-dev git curl protobuf-compiler
curl https://sh.rustup.rs -sSf | sh -s -- -y
source "$HOME/.cargo/env"
rustup default stable
git clone <YOUR_GMB_REPOSITORY_URL>
cd GMB/citizenchain
cargo build --release -p node
sudo useradd --system --user-group --home-dir /opt/citizenchain --shell /usr/sbin/nologin citizenchain
sudo install -m 0755 target/release/citizenchain /usr/bin/citizenchain
sudo install -d -o citizenchain -g citizenchain -m 0700 /opt/citizenchain/data
sudo install -m 0644 scripts/citizenchain-node.service /etc/systemd/system/citizenchain.service
sudo systemctl daemon-reload
sudo systemctl enable --now citizenchain
```

若 `citizenchain` 系统账户已经存在，跳过 `useradd`，不得删除或重建现有账户。

## 15. 文档维护边界
- 若 `node` CLI 启动参数发生变化，应同步更新本文。
- 若 chain spec、默认端口、bootnodes 或运行模式发生变化，应同步更新本文。
- 若后续引入 Docker、安装器、自动化部署脚本，应新增专门部署文档，不直接覆盖当前源码编译部署口径。
