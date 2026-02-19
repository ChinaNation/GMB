# citizennode 目录结构

```text
citizennode/
├── backend/                 # 统一后端（鉴权、审计、业务网关）
├── desktop/                 # 统一桌面前端（Tauri + React）
├── docs/                    # 技术与接口文档
├── scripts/                 # 构建与同步脚本
├── Cargo.toml               # Rust workspace（应用层）
└── README.md
```

## desktop 功能域

- `pages/Nrc`：国储会界面
- `pages/Prc`：省储会界面
- `pages/Prb`：省储行界面
- `pages/Full`：全节点挖矿界面
- `features/auth`：挑战签名登录与角色识别

## 说明

- 这是统一前端管理软件，不是区块链节点代码库。
- `GMB/primitives` 保持在原位置，不迁移。
