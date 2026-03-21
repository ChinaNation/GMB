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
- 默认链：`mainnet`
- 共识机制：`PoW + GRANDPA`
- 默认本地 RPC：`127.0.0.1:9944`
- 默认本地 Prometheus：`127.0.0.1:9615`
- 默认 P2P 端口：`30333`
- 首次启动若不存在 `powr` 密钥，节点会自动生成本地 PoW 作者密钥

相关实现参考：
- [`CITIZENCHAIN_TECHNICAL.md`](/Users/rhett/GMB/memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md)
- [`node/src/command.rs`](/Users/rhett/GMB/citizenchain/node/src/command.rs)
- [`node/src/service.rs`](/Users/rhett/GMB/citizenchain/node/src/service.rs)
- [`node/src/chain_spec.rs`](/Users/rhett/GMB/citizenchain/node/src/chain_spec.rs)

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
- 网络：具备公网 IP，允许 SSH 登录

### 5.2 登录服务器
如果本地已有 SSH 私钥：

```bash
ssh -i your-key.pem ubuntu@<ORACLE_SERVER_IP>
```

说明：
- 某些 Oracle Cloud 镜像用户名可能不是 `ubuntu`，也可能是 `opc`。
- 若首次连接，系统会要求确认主机指纹。

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
target/release/node
```

### 8.3 验证节点帮助信息

```bash
./target/release/node --help
```

若帮助信息正常输出，说明节点二进制已可运行。

## 9. 第 5 步：手工启动并验证节点

### 9.1 准备数据目录

```bash
mkdir -p /home/ubuntu/citizenchain-data
```

如果当前登录用户不是 `ubuntu`，请改成实际用户目录。

### 9.2 启动普通主网节点

```bash
./target/release/node \
  --chain mainnet \
  --name oracle-citizenchain-01 \
  --base-path /home/ubuntu/citizenchain-data \
  --mining-threads 2
```

参数说明：
- `--chain mainnet`：显式指定主网
- `--name`：设置节点名称，便于识别
- `--base-path`：指定数据库、网络密钥、keystore 等本地数据目录
- `--mining-threads 2`：启用 2 个 CPU 挖矿线程

### 9.3 关于链规格说明
- 当前节点接受 `mainnet`
- 省略 `--chain` 也会进入主网配置
- `dev` 与 `local` 当前已被显式禁用

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
sudo ufw allow 22/tcp
sudo ufw allow 30333/tcp
sudo ufw enable
```

说明：
- `22/tcp`：SSH
- `30333/tcp`：CitizenChain P2P 通信端口

### 10.2 Oracle Cloud 控制台安全规则
还需要在 Oracle Cloud 控制台中放通对应入站规则：
- `22/TCP`
- `30333/TCP`

如果只需要节点参与网络，不需要公网 RPC，则不建议放开 `9944`。

### 10.3 RPC 与 Prometheus 安全边界
- 默认 RPC：`127.0.0.1:9944`
- 默认 Prometheus：`127.0.0.1:9615`
- 这两个默认只监听本机，属于更安全的初始配置

不建议直接暴露以下端口到公网：
- `9944`
- `9615`

除非你明确配置了访问控制、反向代理、白名单和限流策略。

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
User=ubuntu
WorkingDirectory=/home/ubuntu/GMB/citizenchain
ExecStart=/home/ubuntu/GMB/citizenchain/target/release/node --chain mainnet --name oracle-citizenchain-01 --base-path /home/ubuntu/citizenchain-data --mining-threads 2
Restart=always
RestartSec=5
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

说明：
- `User`、`WorkingDirectory`、`ExecStart` 应按实际服务器用户名与目录调整
- `Restart=always` 可保证进程异常退出后自动重启

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
ps aux | grep target/release/node
```

### 12.2 检查端口监听

```bash
ss -lntp | grep 30333
ss -lntp | grep 9944
ss -lntp | grep 9615
```

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
nslookup nrcgch.wuminapp.com
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
- 不要把 RPC `9944` 直接暴露到公网
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
mkdir -p /home/ubuntu/citizenchain-data
./target/release/node --chain mainnet --name oracle-citizenchain-01 --base-path /home/ubuntu/citizenchain-data --mining-threads 2
```

## 15. 文档维护边界
- 若 `node` CLI 启动参数发生变化，应同步更新本文。
- 若 chain spec、默认端口、bootnodes 或运行模式发生变化，应同步更新本文。
- 若后续引入 Docker、安装器、自动化部署脚本，应新增专门部署文档，不直接覆盖当前源码编译部署口径。
