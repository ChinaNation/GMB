# CID 后端目录布局

- 最后更新:2026-06-14
- 任务卡:
  - `memory/08-tasks/done/20260502-citizencode-backend-src平移根目录.md`
  - `memory/08-tasks/done/20260502-cid-institutions粗粒度整合.md`
  - `memory/08-tasks/done/20260502-cid-models-scope边界整改.md`
  - `memory/08-tasks/done/20260502-cid-cleanup残留整改.md`
  - `memory/08-tasks/done/20260502-cid-sheng-backup-admin-ui.md`
  - `memory/08-tasks/open/20260530-cid-admins-module-unify.md`
  - `memory/08-tasks/open/20260530-cid-province-admin-governance-passkey.md`
  - `memory/08-tasks/done/20260530-cid-admin-permission-step2.md`
  - `memory/08-tasks/done/20260531-cid-admin-ui-closeout.md`
  - `memory/08-tasks/done/20260531-cid-admin-model-no-status.md`
  - `memory/08-tasks/open/20260603-cid-gov-private-subjects.md`
  - `memory/08-tasks/done/20260603-cid-remove-institutions-china-sqlite.md`
  - `memory/08-tasks/done/20260604-cid-core-number-store-refactor.md`
  - `memory/08-tasks/done/20260612-181650-重构-cid-私权机构架构-保留身份id格式-私权机构按个体经营-合伙企业-股权公司-股份公司-公益组织-注册协.md`
  - `memory/08-tasks/done/20260612-194131-cid-private-real-module-refactor.md`
  - `memory/08-tasks/open/20260613-cid-institution-list-audit-accounts.md`
  - `memory/08-tasks/open/20260614-cid-education-classification.md`

## 当前边界

CID 后端 `backend/src/` 源码壳已删除。CID 后端 Rust 源码不再放在 Cargo 默认
`src/main.rs` 下面,而是直接以 `citizencode/backend/` 为源码根目录。

`Cargo.toml` 使用显式入口:

```toml
[[bin]]
name = "citizencode-backend"
path = "main.rs"
```

## 当前结构

```text
citizencode/backend/
├── Cargo.toml                 # 显式声明 main.rs 为后端入口
├── main.rs                    # Axum 路由、AppState、StoreHandle 等后端入口
├── main_tests.rs              # main.rs 的测试模块
├── accounts/                  # 机构账户入口
├── admins/                    # 注册局机构 admins 治理、安全动作、Passkey 与登录认证
│   └── login/                 # 管理员登录、扫码登录、鉴权守卫、签名校验
├── audit.rs                   # 审计日志查询 handler
├── citizens/                  # 公民身份模型、查询、直接录入、绑定状态、投票凭证
├── core/                      # 跨业务底层工具,含 HTTP 响应、HTTP 安全、运行期工具、chain_*、QR 协议
│   └── qr/                    # QR_V1 协议辅助和统一 sign_request 构造
├── crypto/                    # sr25519 派生、公钥规范化等低层加密辅助
├── china/                     # 中国行政区划 SQLite 真源和省市查询接口
├── docs/                      # 机构资料库入口
├── gov/                       # 公权机构入口,含公安局和普通公权确定性列表路由归属
├── indexer/                   # 链事件解析与索引 worker
├── number/                    # 身份 ID 编码协议,SubjectProperty/机构码/生成/校验/admin 编码元信息 DTO
├── private/                   # 私权机构入口,按私权目标类型拆分子目录
│   ├── common/                # 私权机构共用类型规则:private_type → 主体属性/T2/P1/法人资格
│   ├── sole/                  # 个体经营:F+GT,无法人资格,负责人完全负责
│   ├── partnership/           # 合伙企业:无限合伙 F+GP / 有限合伙 S+LP
│   ├── company/               # 股权公司:S+GQ,股权有限公司/有限责任公司
│   ├── corporation/           # 股份公司:S+GF,股份有限公司
│   ├── welfare/               # 公益组织:S+GY,非营利法人
│   ├── association/           # 注册协会:S+AS,具有法人资格
│   └── participants/          # 参与人关系:负责人、合伙人、股东、成员等通用角色
├── scope/                     # 省/市可见范围与过滤规则,不放 handler
├── store/                     # Store 聚合体、省级进程内分片缓存和存储边界模型
├── subjects/                  # 身份主体共享模型、目标分区表、主体详情、非法人能力
│   ├── registration.rs        # 公权/教育通用注册内核;私权六类模块调用私权专用内核
│   └── unincorporated_org/               # 非法人机构从属关系能力
├── tests/                     # 集成/e2e 测试
└── target/                    # Cargo 构建产物,不得纳入源码整理
```

## 目录铁律

- 禁止恢复 `backend/src/` 源码壳。
- 禁止恢复独立 chain 业务目录。
- 后端新增功能模块直接放 `citizencode/backend/<功能名>/`。
- 功能模块如需和区块链交互,在所属目录中新建 `chain_*.rs`。
- 后端不再维护分散的省级/市级管理员双目录;
  联邦注册局机构 admins、市注册局机构 admins 和管理员治理写入口统一归 `admins/`。
- 公权机构前后端目录统一命名为 `gov`;后端不得再另建 `public` 或 `registry_admins`。
- 私权机构归 `private`,其下按 `common/sole/partnership/company/corporation/welfare/association/participants`
  拆分私权类型与参与人关系;公民继续使用 `citizens`;智能人功能当前不上线,不得预建智能人目录或表。
- 公民 DTO 归 `citizens/model.rs`,编码元信息 DTO 归 `number/model.rs`,不得恢复或塞回 `models/`。
- HTTP 响应包装归 `core/response.rs`;Store 聚合体归 `store/model.rs`;
  注册局机构 admins 列表 DTO 归 `admins/model.rs`;管理员 Passkey 和安全挑战模型归
  `admins/security_model.rs`;审计日志行模型归 `audit.rs`。
- 行政区划唯一真源归 `china/`;不得恢复 `citizencode/`、`province.rs`、`cities.rs`
  或 `city_codes/*.rs` 静态表。
- 非法人机构能力归 `subjects/unincorporated_org/`;不得放在单侧 `gov/` 或 `private/`。
- `scope/` 只放权限范围规则,不得放 HTTP handler 或公钥工具。
- 管理端操作权限类型只允许 `LOGIN_STATE / PASSKEY / PASSKEY_CHALLENGE`,统一登记在
  `admins/operation_auth.rs`;未登记或类型不匹配的操作必须拒绝。
- 新增、删除注册局机构管理员不得在列表查询 handler 暴露写入口;
  必须统一走 `admins/actions.rs` 的 `PASSKEY_CHALLENGE` 治理动作入口。
- 新增市注册局机构管理员必须由 `admins/actions.rs` 调用 `admins/city_registry_admins.rs` 的省市校验和数量统计；
  同一省同一市最多 30 名市注册局机构管理员，市名可能跨省重复，统计时必须带省份。
- 联邦/市注册局机构管理员姓名修改属于 `LOGIN_STATE`,使用登录态 PATCH handler,但仍必须做省域和注册局机构校验。
- 市注册局管理员账户属于身份根,`UPDATE_CITY_REGISTRY` 不接收 `admin_account`;修改市注册局管理员
  只允许调整管理员姓名。
- 联邦注册局机构管理员采用同级模型;新增、删除统一走
  `CREATE_FEDERAL_REGISTRY / DELETE_FEDERAL_REGISTRY` 安全动作;编辑姓名使用登录态 PATCH handler。
- 管理员不存在停用状态字段;删除管理员时必须同步清理会话、Passkey、短期挑战和安全 grant。
- `PASSKEY` 业务写操作必须先在 `admins/actions.rs` 发起安全动作,由 `admins/passkeys.rs`
  提供 WebAuthn 验证后换取一次性 `x-cid-security-grant`;`PASSKEY_CHALLENGE` 写操作必须再叠加
  当前管理员冷钱包 sr25519 签名。
- `admins/passkeys.rs` 的 WebAuthn 配置读取 `CID_PASSKEY_RP_ID`、
  `CID_PASSKEY_ORIGIN` 和可选 `CID_PASSKEY_ALLOWED_ORIGINS`;未配置时开发默认
  `localhost / http://localhost:5179`,生产环境 `CID_ENV=prod|production` 启动期强制
  `cid.crcfrcn.com / https://cid.crcfrcn.com`。
- Passkey 注册流程固定为 `register/start -> register/confirm -> register/complete`;
  `start` 只生成 `QR_V1 / sign_request` 公民钱包签名请求,`confirm` 验证 sr25519 签名响应后
  才生成 WebAuthn creation options,`complete` 完成浏览器凭据 attestation 并替换当前管理员有效 Passkey。
- 通用 `QR_V1 / sign_request` envelope 构造归 `core/qr/sign_request.rs`;业务模块只传入
  已确定的签名原文、摘要和展示字段,不得在各业务模块复刻二维码协议包装。机器验真字段保留
  `0x` 公钥/哈希,人机展示字段必须转为中文和 SS58 地址。
- 跨模块链底层工具只允许放在 `citizencode/backend/core/chain_*`。
- 非源码目录 `tests/`、`target/` 不参与业务模块平铺;后端源码根下不得恢复空的
  `db/` 或 `scripts/` 目录。

## Store 边界

- 当前持久化按模块快照表拆分:
  - `store_citizens`:公民记录、绑定运行态和投票缓存。
  - `store_subjects`:机构、账户、机构资料文档。
  - `store_ops`:登录 challenge/session、扫码登录结果、审计、链幂等、回调任务、指标。
    同时保存管理员 Passkey 注册挑战、写操作挑战和短期安全 grant。
- `store/` 内的分片缓存只保留进程内按省访问 API,用于减少 handler 的跨省扫描和锁竞争;
  重启后由模块 Store 快照重新同步。
- 数据库当前目标结构由 `main.rs` 启动时创建；初始联邦注册局机构管理员唯一真源为
  `admins/province_admins.rs`。
- 关系型目标表从初始化阶段即按 `province_code` 创建省级分区,不得写成“数据量变大后再分区”:
  - `ids`:全局 `cid_number` 唯一约束表,不是第二身份键。
  - `subjects`:统一身份主体索引,`kind=CITIZEN/PUBLIC/PRIVATE`;机构全称/简称列只允许
    `cid_full_name/cid_short_name`,旧展示缓存列已废除并在启动期清理;同时保存
    `province_code/city_code/town_code/省市镇/status/education_type/private_type/partnership_kind/has_legal_personality`
    等主体展示、范围和私权分类字段。
  - `citizens`:公民详情,保留精简命名。
  - `gov`:公权机构详情,保存 `level/institution_code`；公安局只保留
    `level=CITY AND institution_code='CPOL'` 的市公安局。
  - `private`:私权机构详情,仅保存目标私权类型机构;字段以
    `private_type/partnership_kind/has_legal_personality` 表达分类,不得恢复旧分类列。
  - `accounts`:机构账户。所有机构默认有“主账户 / 费用账户”;省公民储备银行额外有“永久质押”;
    国储会额外有“安全基金 / 两和基金”。这些制度账户均为默认账户,不可按普通自定义账户删除。
  - `docs`:机构资料库。
  - `audit`:目标审计分区表。机构详情操作记录按 `target_cid` 精确读取,写入点覆盖机构创建、
    详情编辑、账户创建/删除、资料上传/下载/删除和公民护照直接录入。
- `CN` 与 43 个省代码的分区在启动建表时一次性创建。
- 机构主写入只进入 `subjects / gov / private / accounts / docs` 目标表;
  私权机构精确搜索从 `subjects + accounts + admins` 查询,且 handler 必须先把登录
  scope 翻译成 `province_code / city_code` 后再交给 StoreHandle,不得用中文省市字段或内存全量过滤。
- 公安局和公权机构确定性列表是只读查询:启动或显式 reconcile 负责生成/对账,GET 列表接口
  只按 `province_code / city_code` 读取目标表,不得在 GET 中执行 backfill、reconcile、写库或分片同步。
  公权机构列表允许 `institution_code` 精确过滤,用于市注册局等确定性细类列表一次性读取完整身份ID,
	  不得让前端先读取省级公权目录分页再自行过滤。确定性公权目录版本为 `gov-deterministic-v6`;
	  内置机构全称/简称直接投影 `china_*.rs` 的 `cid_full_name/cid_short_name`,
	  区划模板目录也必须同时写入这两列,不得恢复旧展示缓存列。
- 国家/市公民教育委员会仍由 `gov/service.rs` 确定性生成和对账,但必须写入
  `subjects.education_type=NATIONAL_CITIZEN_EDU_COMMITTEE/CITY_CITIZEN_EDU_COMMITTEE`,
  并从公权机构浏览目录排除,由教育机构 tab 直接展示。
- `subjects/registration.rs` 承接公权/教育通用注册和列表内核;六类私权机构从
  `private/<type>/mod.rs` 传入固定规则后调用私权专用内核,不得恢复 `private/handler.rs`。
- `subjects/http.rs` 承接跨 `gov/private/accounts/docs/subjects` 的 HTTP 辅助函数,包括
  `ServiceError` 响应转换、CID 省市解析、机构 scope 可见性和默认账户 best-effort。
  审计写入统一调用 `core/runtime_ops.rs` 的 `append_audit_log`,detail 只存结构化事实。

## 验收口径

```text
test ! -d citizencode/backend/src
test ! -d citizencode/backend/chain
test ! -d citizencode/backend/institutions
test ! -d citizencode/backend/cid
test ! -d citizencode/backend/cid_number
test ! -d citizencode/backend/store_shards
test ! -d citizencode/backend/models
test ! -d citizencode/backend/login
test ! -d citizencode/backend/qr
rg "mod chain;|crate::chain|chain::" citizencode/backend -g '*.rs'
cd citizencode/backend && cargo fmt && cargo check
set -a; source citizencode/.env.dev.local; set +a
curl -fsS http://127.0.0.1:8899/api/v1/health
curl -sS -i http://127.0.0.1:8899/api/v1/admin/auth/check -H "authorization: Bearer <token>"
curl -sS -i http://127.0.0.1:8899/api/v1/admin/federal-registry -H "authorization: Bearer <token>"
curl -sS -i http://127.0.0.1:8899/api/v1/admin/city-registry-admins -H "authorization: Bearer <token>"
curl -sS -i http://127.0.0.1:8899/api/v1/institutions/federal-registry -H "authorization: Bearer <token>"
```

涉及数据库、登录、管理员列表、机构详情的变更必须跑真实 HTTP 接口。只通过
`cargo check` 不能证明连接池未被 panic 污染;验收时不得出现 `postgres client lock poisoned`。

## 错误码边界

CID 后端统一通过 `ApiError.error_code` 暴露稳定业务错误码。HTTP `401` 只表示管理员
登录态无效;公民绑定 challenge 过期、账户不匹配、签名失败等业务错误
不得返回 `401`。完整规则见 `memory/05-modules/citizencode/ERROR_CODES.md`。
管理员新增入口必须以规范化 `admin_account` 做全局唯一校验；重复账号按已有注册局机构返回
`CID_ADMIN_ACCOUNT_EXISTS_AS_FEDERAL_REGISTRY` 或 `CID_ADMIN_ACCOUNT_EXISTS_AS_CITY_REGISTRY`。
联邦注册局机构 admins 每省最多 5 人；市注册局机构 admins 每省每市最多 30 人；`federal_registry_scope.province_name` 只能建普通索引,不得建唯一约束。
管理员安全写操作必须在返回成功前显式完成 Store 持久化；持久化失败返回
`CID_STORE_PERSIST_FAILED`。
