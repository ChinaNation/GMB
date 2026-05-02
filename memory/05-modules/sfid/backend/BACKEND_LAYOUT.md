# SFID 后端目录布局

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-backend-src平移根目录.md`

## 当前边界

`sfid/backend/src/` 已删除。SFID 后端 Rust 源码不再放在 Cargo 默认
`src/main.rs` 下面,而是直接以 `sfid/backend/` 为源码根目录。

`Cargo.toml` 使用显式入口:

```toml
[[bin]]
name = "sfid-backend"
path = "main.rs"
```

## 当前结构

```text
sfid/backend/
├── Cargo.toml                 # 显式声明 main.rs 为后端入口
├── main.rs                    # Axum 路由、AppState、StoreHandle 等后端入口
├── main_tests.rs              # main.rs 的测试模块
├── app_core/                  # 跨业务底层工具,含 HTTP 安全、运行期工具、chain_* 通用链工具
├── citizens/                  # 公民身份、绑定、投票凭证、CPMS 状态扫码
├── crypto/                    # 加密与 sr25519 辅助
├── indexer/                   # 链事件解析与索引 worker
├── institutions/              # 机构创建、机构资料、账户名称、机构链交互 chain_duoqian_info*
├── login/                     # 管理员登录、挑战、签名校验
├── models/                    # 共享模型、角色、状态、Store 结构
├── qr/                        # QR 协议辅助
├── scope/                     # 省/市可见范围与写权限判断
├── sfid/                      # SFID 生成、校验、省市代码、A3/机构码
├── sheng_admins/              # 省管理员后台业务和省管理员链交互 chain_*
├── shi_admins/                # 市管理员模块
├── store_shards/              # 分片 Store 与迁移辅助
├── db/                        # 数据库迁移和 seed,不是 Rust 源码模块
├── scripts/                   # 后端开发脚本,不是 Rust 源码模块
├── tests/                     # 集成/e2e 测试
└── target/                    # Cargo 构建产物,不得纳入源码整理
```

## 目录铁律

- 禁止恢复 `sfid/backend/src/` 源码壳。
- 禁止恢复独立 `sfid/backend/chain/` 业务目录。
- 后端新增功能模块直接放 `sfid/backend/<功能名>/`。
- 功能模块如需和区块链交互,在所属目录中新建 `chain_*.rs`。
- 跨模块链底层工具只允许放在 `sfid/backend/app_core/chain_*`。
- 非源码目录 `db/`、`scripts/`、`tests/`、`target/` 不参与业务模块平铺。

## 验收口径

```text
test ! -d sfid/backend/src
test ! -d sfid/backend/chain
rg "mod chain;|crate::chain|chain::" sfid/backend -g '*.rs'
cd sfid/backend && cargo fmt && cargo check
```
