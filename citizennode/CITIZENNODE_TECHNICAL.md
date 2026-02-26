# citizennode 统一技术文档

## 1. 产品定位

- `citizennode` 是统一前端管理软件，不是区块链节点本体。
- 服务四类角色工作台：
  - 国储会（NRC）：治理/投票
  - 省储会（PRC）：治理/投票
  - 省储行（PRB）：链下交易
  - 全节点（FULL）：挖矿操作

## 2. 系统边界

- 区块链工程代码不并入本仓库。
- `GMB/primitives` 保持原位置，不迁移。
- `citizennode` 通过 RPC/API 连接链与后端。

## 3. 技术架构与栈

### 3.1 架构分层

- 桌面端：`desktop/`（Tauri + React 前端）
- 统一后端：`backend/`（Rust，鉴权/审计/业务网关）
- 文档：`docs/`（保留扩展文档目录）

### 3.2 技术栈

- 桌面端：`Tauri + React + TypeScript + Vite`
- 后端：`Rust`
- 通信协议：`REST + JSON`（必要时补充 WebSocket）

## 4. 目录结构（核心）

```text
citizennode/
├── backend/                 # 统一后端（鉴权、审计、业务网关）
├── desktop/                 # 统一桌面前端（Tauri + React）
├── docs/                    # 技术与接口文档扩展目录
├── Cargo.toml               # Rust workspace（应用层）
└── TECHNICAL.md             # 统一技术文档（本文件）
```

## 5. 前端功能域映射

- `desktop/src/pages/Nrc`：国储会界面
- `desktop/src/pages/Prc`：省储会界面
- `desktop/src/pages/Prb`：省储行界面
- `desktop/src/pages/Full`：全节点挖矿界面
- `desktop/src/features/auth`：挑战签名登录与角色识别

## 6. 认证与路由规则

- 统一挑战签名登录（与 SFID/CPMS 一致）。
- 登录页由系统生成一次性登录二维码（challenge）。
- 手机扫码后回传签名结果，签名内容中携带用户公钥。
- 系统从签名内容提取公钥并识别角色。
- 命中机构公钥才允许登录并进入对应工作台；未命中直接拒绝登录。

## 7. 开发与运行

### 7.1 前端开发

```bash
cd /Users/rhett/GMB/citizennode/desktop
npm install
npm run dev
```

- 默认链连接地址：`ws://127.0.0.1:9944`

### 7.2 桌面壳开发（Tauri）

```bash
cd /Users/rhett/GMB/citizennode/desktop
npm install
npm run tauri:dev
```

### 7.3 桌面打包

```bash
cd /Users/rhett/GMB/citizennode/desktop
npm run tauri:build
```

## 8. 工程质量基线

- Rust：`cargo fmt`、`cargo clippy`、`cargo test`
- 前端：`npm run build`（可按需补充 lint/test）
