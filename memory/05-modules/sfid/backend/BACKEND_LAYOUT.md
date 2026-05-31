# SFID 后端目录布局

- 最后更新:2026-05-31
- 任务卡:
  - `memory/08-tasks/done/20260502-sfid-backend-src平移根目录.md`
  - `memory/08-tasks/done/20260502-sfid-cpms-sheng目录整改.md`
  - `memory/08-tasks/done/20260502-sfid-institutions粗粒度整合.md`
  - `memory/08-tasks/done/20260502-sfid-models-scope边界整改.md`
  - `memory/08-tasks/done/20260502-sfid-cleanup残留整改.md`
  - `memory/08-tasks/done/20260502-sfid-sheng-backup-admin-ui.md`
  - `memory/08-tasks/done/20260525-sfid-cpms-store.md`
  - `memory/08-tasks/open/20260530-sfid-admins-module-unify.md`
  - `memory/08-tasks/open/20260530-sfid-province-admin-governance-passkey.md`
  - `memory/08-tasks/done/20260530-sfid-admin-permission-step2.md`
  - `memory/08-tasks/done/20260531-sfid-admin-ui-closeout.md`
  - `memory/08-tasks/done/20260531-sfid-admin-model-no-status.md`

## 当前边界

SFID 后端旧源码壳已删除。SFID 后端 Rust 源码不再放在 Cargo 默认
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
├── cpms/                      # CPMS 安装授权、ARCHIVE 验真、档案导入、站点状态治理
├── crypto/                    # sr25519 派生、公钥规范化等低层加密辅助
├── indexer/                   # 链事件解析与索引 worker
├── institutions/              # 机构创建、机构资料、账户名称、机构链查询 chain_duoqian_info.rs
├── login/                     # 管理员登录、扫码登录、鉴权守卫、签名校验
├── models/                    # 全局共享模型、角色、响应包装、Store 结构
├── qr/                        # QR 协议辅助
├── scope/                     # 省/市可见范围与过滤规则,不放 handler
├── sfid/                      # SFID 生成、校验、省市代码、A3/机构码、admin 元信息 DTO
├── admins/                    # 省/市管理员治理和安全分级,含 actions.rs / passkeys.rs / 冷钱包 grant
├── store_shards/              # 进程内省分片缓存,不再持久化到旧 store_shards 表
├── db/                        # 数据库迁移和 seed,不是 Rust 源码模块
├── scripts/                   # 后端开发脚本,不是 Rust 源码模块
├── tests/                     # 集成/e2e 测试
└── target/                    # Cargo 构建产物,不得纳入源码整理
```

## 目录铁律

- 禁止恢复旧后端源码壳。
- 禁止恢复独立 chain 业务目录。
- 后端新增功能模块直接放 `sfid/backend/<功能名>/`。
- 功能模块如需和区块链交互,在所属目录中新建 `chain_*.rs`。
- CPMS 系统管理归 `sfid/backend/cpms/`,不得放入管理员目录。
- 后端不再维护旧省级/市级管理员双目录;
  省级管理员列表、市级管理员列表和管理员治理写入口统一归 `admins/`。
- 公民 DTO 归 `citizens/model.rs`,CPMS DTO 归 `cpms/model.rs`,SFID 元信息 DTO 归
  `sfid/model.rs`,不得塞回 `models/`。
- `scope/` 只放权限范围规则,不得放 HTTP handler、CPMS 专用判断或 pubkey 工具。
- 省管理员治理写操作不得直接在 `operators.rs` 或 `catalog.rs` 暴露写 handler;
  必须统一走 `admins/actions.rs` 的治理动作入口,Passkey 注册与 WebAuthn 工具归
  `admins/passkeys.rs`。
- 市级管理员地址属于身份根,`UPDATE_OPERATOR` 不接收 `admin_pubkey`;修改市级管理员
  只允许调整管理员姓名。
- 省级管理员采用同级模型;新增、编辑、删除省级管理员统一走
  `CREATE_SHENG_ADMIN / UPDATE_SHENG_ADMIN / DELETE_SHENG_ADMIN` 安全动作。
- 管理员不存在停用状态字段;删除管理员时必须同步清理会话、Passkey、短期挑战和安全 grant。
- 一般业务写操作必须先在 `admins/actions.rs` 发起安全动作,由 `admins/passkeys.rs`
  提供 WebAuthn 验证后换取一次性 `x-sfid-security-grant`;重要业务写操作必须再叠加
  当前管理员冷钱包 sr25519 签名。
- `admins/passkeys.rs` 的 WebAuthn 配置读取 `SFID_PASSKEY_RP_ID`、
  `SFID_PASSKEY_ORIGIN` 和可选 `SFID_PASSKEY_ALLOWED_ORIGINS`;未配置时开发默认
  `localhost / http://localhost:5179`,生产环境 `SFID_ENV=prod|production` 启动期强制
  `sfid.crcfrcn.com / https://sfid.crcfrcn.com`。
- CPMS 安装授权、安装码重签发、禁用、启用、吊销、删除归省级管理员;
  市级管理员不得通过 CPMS handler 操作授权治理。
- 跨模块链底层工具只允许放在 `sfid/backend/app_core/chain_*`。
- 非源码目录 `db/`、`scripts/`、`tests/`、`target/` 不参与业务模块平铺。

## Store 边界

- SFID 后端不再使用旧 `runtime_store`、`runtime_misc`、`runtime_cache_entries`
  或旧 `store_shards` JSONB 表保存整包 Store。
- 当前持久化按模块快照表拆分:
  - `store_citizens`:公民记录、绑定 challenge、状态扫码短期池、投票缓存。
  - `store_cpms`:CPMS 安装授权和授权状态。
  - `store_institutions`:机构、账户、机构资料文档。
  - `store_ops`:登录 challenge/session、扫码登录结果、审计、链幂等、回调任务、指标。
    同时保存管理员 Passkey 注册挑战、写操作挑战和短期安全 grant。
- `store_shards/` 只保留进程内按省缓存访问 API,用于减少 handler 的跨省扫描和锁竞争;
  重启后由模块 Store 快照重新同步。
- `db/migrations/015_store_reset.sql` 明确删除旧整包 JSON 表,不做旧数据迁移。

## 验收口径

```text
test ! -d sfid/backend/src
test ! -d sfid/backend/chain
rg "mod chain;|crate::chain|chain::" sfid/backend -g '*.rs'
cd sfid/backend && cargo fmt && cargo check
```

## 错误码边界

SFID 后端统一通过 `ApiError.error_code` 暴露稳定业务错误码。HTTP `401` 只表示管理员
登录态无效;公民绑定 challenge 过期、账户不匹配、签名失败、ARCHIVE 验真失败等业务错误
不得返回 `401`。完整规则见 `memory/05-modules/sfid/ERROR_CODES.md`。
