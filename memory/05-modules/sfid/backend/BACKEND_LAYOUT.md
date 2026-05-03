# SFID 后端目录布局

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-backend-src平移根目录.md`
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`
  - `memory/08-tasks/done/20260502-sfid-institutions粗粒度整合.md`
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`
  - `memory/08-tasks/done/20260502-sfid-cleanup残留整改.md`
  - `memory/08-tasks/done/20260502-sfid-sheng-backup-admin-ui.md`

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
├── audit.rs                   # 审计日志查询 handler
├── citizens/                  # 公民身份模型、查询、绑定、投票凭证、CPMS 状态扫码
├── cpms/                      # CPMS 模型、安装、注册、匿名证书 rsa_blind、站点状态治理
├── crypto/                    # sr25519 派生、公钥规范化等低层加密辅助
├── indexer/                   # 链事件解析与索引 worker
├── institutions/              # 机构创建、机构资料、账户名称、机构链查询 chain_duoqian_info.rs
├── login/                     # 管理员登录、扫码登录、鉴权守卫、签名校验
├── models/                    # 全局共享模型、角色、响应包装、Store 结构
├── qr/                        # QR 协议辅助
├── scope/                     # 省/市可见范围与过滤规则,不放 handler
├── sfid/                      # SFID 生成、校验、省市代码、A3/机构码、admin 元信息 DTO
├── sheng_admins/              # 省管理员治理、市管理员操作员维护、三槽展示、本地 signing seed 管理
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
- CPMS 系统管理归 `sfid/backend/cpms/`,不得再放入 `sheng_admins/institutions.rs`。
- 后端不再维护 `sfid/backend/shi_admins/` 空壳转发目录;市管理员操作员维护归
  `sheng_admins/operators.rs`,CPMS 状态扫码归 `citizens/status.rs`。
- 公民 DTO 归 `citizens/model.rs`,CPMS DTO 归 `cpms/model.rs`,SFID 元信息 DTO 归
  `sfid/model.rs`,不得塞回 `models/`。
- `scope/` 只放权限范围规则,不得放 HTTP handler、CPMS 专用判断或 pubkey 工具。
- 省管理员只有“更换省管理员/主备交换”需要链交互时,才允许新增
  `sheng_admins/chain_replace_admin.rs`。
- 省管理员备用槽的本地姓名/账户保存归 `sheng_admins/roster.rs`,
  不因为页面新增按钮而创建 `chain_` 文件。
- 跨模块链底层工具只允许放在 `sfid/backend/app_core/chain_*`。
- 非源码目录 `db/`、`scripts/`、`tests/`、`target/` 不参与业务模块平铺。

## 验收口径

```text
test ! -d sfid/backend/src
test ! -d sfid/backend/chain
rg "mod chain;|crate::chain|chain::" sfid/backend -g '*.rs'
cd sfid/backend && cargo fmt && cargo check
```
