# citizennode 技术基线

## 1. 产品定位

- `citizennode` 是统一前端管理软件（非区块链节点本体）。
- 覆盖四类角色：国储会、省储会、省储行、全节点。

## 2. 技术栈

- 桌面端：`Tauri + React + TypeScript + Vite`
- 后端：`Rust`
- 协议：`REST + JSON`（必要时可补 WebSocket）

## 3. 登录与路由

- 统一挑战签名登录
- 输入公钥/SS58 自动识别角色
- 机构地址进入机构工作台
- 非机构地址进入 `全节点挖矿系统`

## 4. 边界

- 区块链工程不并入 `citizennode`
- `GMB/primitives` 保持原位置，通过路径依赖或同步脚本使用

## 5. 工程质量

- Rust：`cargo fmt`、`cargo clippy`、`cargo test`
- 前端：`npm run build`（可按需补 lint/test）
