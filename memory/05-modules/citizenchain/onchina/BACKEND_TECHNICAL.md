# OnChina 后端技术文档

## 1. 功能需求

OnChina 后端负责注册局身份、行政区、机构、公民、管理员、扫码签名、公开查询和链侧凭证。它运行在 `citizenchain/onchina/src/`，属于公民链产品内部能力。

## 2. 当前结构

```text
citizenchain/onchina/src/
├── main.rs                    # Axum 路由、AppState、StoreHandle 和后端入口
├── accounts/                  # 机构账户入口
├── admins/                    # 注册局机构 admins、安全动作和登录认证
│   └── login/                 # 管理员登录、扫码登录、鉴权守卫和签名校验
├── audit/                     # 审计日志查询
├── cid/                       # CID 号编码、机构码、生成、校验和行政区 SQLite
│   └── china/                 # 中国行政区划 SQLite 真源
├── citizens/                  # 公民身份、账户绑定、投票凭证和 CitizenApp 查询
├── core/                      # HTTP、安全、运行期工具、chain_* 和 QR 协议辅助
│   └── qr/                    # QR_V1 协议辅助和统一 sign_request 构造
├── crypto/                    # sr25519、公钥规范化和哈希辅助
├── docs/                      # 机构资料库入口
├── gov/                       # 公权机构确定性目录和公权机构接口
├── indexer/                   # 链事件解析与索引 worker
├── private/                   # 私权机构入口和六类私权机构子模块
├── scope/                     # 省/市可见范围与过滤规则
├── store/                     # Store 聚合体和结构化存储边界
├── subjects/                  # 主体共享模型、注册内核、详情和非法人能力
└── tests/                     # 集成和端到端测试
```

## 3. 目录铁律

- 禁止恢复旧独立身份系统产品目录。
- 禁止恢复旧 registry 目录。
- 禁止恢复 `backend/src/` 源码壳。
- 禁止恢复独立 `chain` 业务目录；链交互只能放在所属业务模块的 `chain_*.rs` 或 `core/chain_*`。
- 禁止恢复独立 `cid_number`、`models`、`login`、`qr` 等历史目录壳。
- `scope/` 只放权限范围规则，不放 HTTP handler 或公钥工具。
- 非法人机构能力统一归 `subjects/unincorporated_org/`，不得放在单侧 `gov/` 或 `private/`。

## 4. Store 和表边界

后端只承认结构化 PostgreSQL 表为主数据。`store/` 可以封装访问和短期缓存，但不得保存第二份业务主数据。

- 机构主写入只进入 `subjects/gov/private/accounts/docs`。
- 公民主写入只进入 `citizens/subjects`。
- 管理员写入只进入 `admins`(本地元数据缓存)和短生命周期安全运行态表;成员资格真源在链上(`federal_registry_scope` 表已退役,见 [[project_onchina_registry_tier_chainread_2026_06_29]])。
- 链上状态只属于 `accounts.chain_status`，机构主体本身不保存链上状态。
- 审计写入统一走结构化审计入口，详情字段只保存事实，不保存 UI 文案。

## 5. 链交互边界

链交互按业务归属放置：

- 机构注册信息凭证、账户列表 DTO 和 handler：`subjects/chain_*.rs`
- 公民投票凭证：`citizens/chain_*.rs`
- 联合投票人口快照凭证：`citizens/chain_*.rs`
- 通用 SCALE、genesis hash、RPC URL 和交易提交辅助：`core/chain_*.rs`

业务模块不得新增全局链目录，不得在 handler 内手写 pallet/call 字节或二维码动作码。动作码、payload、签名/验签规则以 `memory/07-ai/unified-protocols.md` 为唯一登记入口。

## 6. 错误码和提示边界

后端统一通过 `ApiError.error_code` 暴露稳定业务错误码。HTTP `401` 只表示管理员登录态无效；公民绑定 challenge 过期、账户不匹配、签名失败等业务错误不得返回 `401`。

数据库错误必须展开 PostgreSQL SQLSTATE、message、detail 和 hint，禁止只向前端或日志传 `db error`。

## 7. 管理员写操作

管理员新增、替换、Passkey 更新、节点解绑和链写动作必须使用 `PASSKEY_COLD_SIGN` 二次确认。业务 handler 只负责构造业务动作，二维码协议包装和签名结果识别归 `core/qr/`。

联邦注册局机构 `admins` 不允许本地新增或删除，只允许在同省范围内替换。市注册局机构 `admins` 每省每市最多 30 人，统计必须同时带省和市，不能只按市名统计。

## 8. 验收

```text
rg "mod chain;|crate::chain|chain::" citizenchain/onchina/src -g '*.rs'
cargo check --manifest-path citizenchain/Cargo.toml -p onchina
curl -kfsS https://onchina.local:8964/api/v1/health
curl -ksS -i https://onchina.local:8964/api/v1/admin/auth/check -H "authorization: Bearer <token>"
```

涉及数据库、登录、管理员列表、机构详情和扫码签名的变更必须跑真实 HTTP 接口。只通过 `cargo check` 不能证明连接池、SQL 字段顺序和扫码验签流程正确。
