# citizenui 统一技术文档

## 1. 产品定位

- `citizenui` 是 `citizenchain` 的本地桌面管理端（UI 壳）。
- 四类角色工作台：
  - 国储会（NRC）
  - 省储会（PRC）
  - 省储行（PRB）
  - 全节点（FULL）

## 2. 部署与进程模型

- 统一安装包名称：`citizenchain`。
- 运行时双进程：
  - `citizenui-desktop-shell`（Tauri UI 进程）
  - `citizenchain node`（本地链节点 sidecar 进程）
- UI 不直接读写 RocksDB，通过本地 RPC (`ws://127.0.0.1:9944`) 与节点交互。
- 不依赖独立中心化后端服务。

## 3. 当前目录结构（实现）

```text
citizenui/
├── CITIZENUI_TECHNICAL.md
├── Cargo.toml
├── Cargo.lock
└── desktop/
    ├── package.json
    ├── src/                     # React + TS 前端
    ├── src-tauri/               # Tauri 壳（Rust）
    │   ├── src/main.rs
    │   ├── tauri.conf.json
    │   └── binaries/            # 打包时注入 citizenchain-node sidecar
    └── scripts/
        ├── prepare-sidecar.mjs
        └── sync-org-registry.mjs
```

## 4. Rust Workspace（实现）

- `Cargo.toml` workspace members：
  - `desktop`
  - `desktop/src-tauri`
- 不包含独立 `backend` crate。

## 5. 认证与验签（实现）

- 登录协议：`WUMINAPP_LOGIN_V1`。
- 登录流程：
  1. UI 生成 challenge 并展示二维码。
  2. 手机端签名后回传回执二维码。
  3. UI 解析回执并校验 `request_id`、过期时间、一次性消费。
  4. 通过 Tauri command 调 Rust 侧验签。
  5. 按公钥映射角色并路由到 NRC/PRC/PRB/FULL 页面。
- 验签算法支持：
  - `sr25519`
  - `ed25519`
  - `auto`（先 `sr25519` 再 `ed25519`）

## 6. 本地节点 sidecar 机制（实现）

- 构建阶段：
  - `desktop/scripts/prepare-sidecar.mjs` 执行 `cargo build -p node --release`
  - 将节点二进制复制到 `desktop/src-tauri/binaries/`
- 打包配置：
  - `desktop/src-tauri/tauri.conf.json`
  - `bundle.externalBin = ["binaries/citizenchain-node"]`
- 运行阶段：
  - `desktop/src-tauri/src/main.rs` 在 `setup` 中自动拉起本地节点
  - base path 位于 app data 目录下 `node-data`
  - 应用退出事件中回收子进程

## 7. 前端功能域映射（实现）

- `desktop/src/pages/Nrc`：国储会界面
- `desktop/src/pages/Prc`：省储会界面
- `desktop/src/pages/Prb`：省储行界面
- `desktop/src/pages/Full`：全节点界面
- `desktop/src/features/auth`：扫码登录、回执解析、角色进入
- `desktop/src/services/rpc/polkadot.ts`：链上信息读取与交易查询

## 8. 开发与构建命令（实现）

```bash
cd /Users/rhett/GMB/citizenchain/citizenui/desktop
npm install
npm run dev
```

```bash
cd /Users/rhett/GMB/citizenchain/citizenui/desktop
npm run build
```

```bash
cd /Users/rhett/GMB/citizenchain/citizenui/desktop
npm run tauri:dev
```

```bash
cd /Users/rhett/GMB/citizenchain/citizenui/desktop
npm run tauri:build -- --bundles app
```

## 9. 质量检查（当前可执行）

- 前端：`npm run build`
- Tauri Rust 壳：在 `desktop/src-tauri` 下执行 `cargo check`
