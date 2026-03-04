# nodeui 技术文档

## 1. 产品定位

- `nodeui` 是公民币区块链本地节点控制台。

## 2. 运行模型

- 安装包/桌面壳：Tauri（`nodeui-desktop-shell`）。
- 本地节点进程：`citizenchain-node` sidecar。
- UI 通过本地 RPC 与节点通信（默认本机 `ws://127.0.0.1:9944`）。
- 节点状态展示为前端到本地 RPC 的连接状态（已连接/连接中/连接异常/未连接）。

## 3. 当前目录结构

```text
nodeui/
├── Cargo.toml
├── Cargo.lock
├── NODEUI_TECHNICAL.md
├── package.json
├── scripts/
│   └── prepare-sidecar.mjs
├── src/
│   ├── App.tsx
│   ├── main.tsx
│   ├── pages/Full/FullDashboard.tsx
│   ├── features/chain/
│   │   ├── NodeStatusBanner.tsx
│   │   └── useAutoConnect.ts
│   ├── services/rpc/polkadot.ts
│   ├── stores/session.ts
│   ├── constants/node.ts
│   ├── utils/rpcEndpoint.ts
│   ├── theme/antdTheme.ts
│   └── assets/styles/global.css
└── src-tauri/
    ├── Cargo.toml
    ├── tauri.conf.json
    └── src/main.rs
```

## 4. 已实现功能

- 启动应用时自动尝试连接本地节点 RPC。
- 顶部展示本机节点连接状态（中文标签）。

## 5. sidecar 机制

- `scripts/prepare-sidecar.mjs` 会在构建前执行：
  - `cargo build -p node --release`
  - 把节点二进制复制到 `src-tauri/binaries/`
- `src-tauri/src/main.rs` 在 `setup` 阶段自动拉起本地节点子进程。
- 应用退出时回收节点子进程。

## 6. Rust Workspace

- `Cargo.toml` 中 workspace member 为：
  - `src-tauri`
- 根 `Cargo.toml` 同时包含 `nodeui-frontend` 包定义（前端模块元信息）。

## 7. 启动与构建

```bash
cd /Users/rhett/GMB/citizenchain/nodeui
npm install
npm run tauri:dev
```

```bash
cd /Users/rhett/GMB/citizenchain/nodeui
npm run build
```

```bash
cd /Users/rhett/GMB/citizenchain/nodeui
npm run tauri:build -- --bundles app
```

## 8. 维护边界

- 该目录仅负责 UI 壳与本地节点进程编排。
- 不修改链协议逻辑（runtime/node 共识与业务规则）。
