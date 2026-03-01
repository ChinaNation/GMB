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

## 6. 认证与路由规则（实现对齐）

- 协议统一：`WUMINAPP_LOGIN_V1`（与 SFID/CPMS 完全一致）。
- 离线双向扫码：登录端出挑战码，手机端确认签名并出回执码，登录端扫码完成验签。
- 手机端边界：仅执行挑战签名与回执码展示，不接收、不轮询登录结果回传。
- 手机端签名原文固定：
  - `WUMINAPP_LOGIN_V1|citizenchain|aud|origin|request_id|challenge|nonce|expires_at`
- 回执核心字段：
  - `proto/request_id/account/pubkey/sig_alg/signature/signed_at`
- 系统端顺序：
  1. 解析回执
  2. 读取挑战并重建原文
  3. `sr25519` 验签
  4. 校验时效与 `request_id` 一次性消费
  5. 角色识别与路由（NRC/PRC/PRB/FULL）
- 登录结果展示责任在登录端（citizennode），不回传到手机签名端。
- 角色规则：
  - 命中国储会、省储会、省储行管理员公钥进入对应界面。
  - 其他用户进入全节点界面。

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
