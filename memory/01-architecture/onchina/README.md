# OnChina 架构总览

OnChina 是公民链 `citizenchain` 内置的链上中国平台能力，不再作为独立产品存在。仓库当前只保留四个产品：公民、公民链、公民钱包和官方网站；OnChina 属于公民链产品内部的注册局身份、机构登记、行政区和管理后台能力。

## 产品归属

- 产品归属：公民链 `citizenchain`
- 源码目录：`citizenchain/onchina/`
- 产品级文档：`memory/01-architecture/onchina/ONCHINA_TECHNICAL.md`
- 模块级文档：`memory/05-modules/citizenchain/onchina/`
- 管理后台前端：`citizenchain/onchina/frontend/`

## 源码边界

- `citizenchain/onchina/src/core/`：数据库连接、HTTP 安全、统一响应、运行期维护、链交互和 QR 协议辅助。
- `citizenchain/onchina/src/cid/`：身份 ID 编码、机构码、CID 号生成和校验。
- `citizenchain/onchina/src/cid/china/`：中国行政区划 SQLite 开发真源。
- `citizenchain/onchina/src/admins/`：注册局机构 `admins`、登录、扫码二次确认和权限上下文。
- `citizenchain/onchina/src/gov/`：公权机构确定性目录和公权机构查询。
- `citizenchain/onchina/src/private/`：私权机构登记和六类私权机构能力。
- `citizenchain/onchina/src/subjects/`：主体公共模型、注册内核、主体详情、公开查询和非法人能力。
- `citizenchain/onchina/src/citizens/`：公民录入、电子护照档案、CitizenApp 查询和投票凭证。
- `citizenchain/onchina/src/accounts/`：机构账户管理。
- `citizenchain/onchina/src/docs/`：机构资料库。
- `citizenchain/onchina/src/audit/`：审计查询。
- `citizenchain/onchina/src/indexer/`：链上交易索引。

前端按同名业务边界放在 `citizenchain/onchina/frontend/` 下。某功能自己的 API 必须在功能目录内，通用 HTTP 封装只允许放 `frontend/utils/http.ts`。

## 数据真源

OnChina 以 PostgreSQL 结构化表作为唯一持久化真源。进程内缓存只允许承载短生命周期运行态和性能缓存，不得成为第二份业务主数据。

核心表：

- `ids`：全局唯一 CID 号索引。
- `subjects`：公民、公权机构、私权机构公共主体表，按 `province_code` 分区。
- `citizens`：公民档案、身份 CID、护照号、钱包地址和电子护照有效期，按 `province_code` 分区。
- `gov`：公权机构扩展表，按 `province_code` 分区。
- `private`：私权机构扩展表，按 `province_code` 分区。
- `accounts`：机构账户表，按 `province_code` 分区。
- `docs`：机构资料库元数据表，按 `province_code` 分区。
- `audit`：审计表，按 `province_code` 分区。
- `admins`：机构管理员本地元数据缓存；成员资格真源是链上 active admin 集合。
- `node_institution_bindings`：本节点首次登录确认后的机构绑定结果；限制本节点后续登录机构，不作为权限真源；解绑 / 换机构通过 `NODE_BINDING_UNBIND` 冷签安全动作停用 active binding 后重新绑定。
- `admin_*`：登录、会话、扫码签名和安全动作运行态。
- `chain_requests`、`chain_nonces`、`tx_records`、`tx_indexer_state`：链路幂等、防重放和索引运行态。

废弃快照表、旧机构行表、旧独立产品部署表不得保留为兼容数据源。

## 权限边界

管理员唯一真源为机构或个人多签的 `admins`。OnChina 管理端登录态只允许通过 `registry_org_code=FEDERAL_REGISTRY/CITY_REGISTRY` 表达注册局机构归属。

- 联邦注册局机构 `admins`：能读取联邦管理员目录，业务数据仍按所属省限制。
- 市注册局机构 `admins`：只能读取和写入所属市数据。
- SQL 查询必须在数据库层携带 `province_code` / `city_code` 范围条件，禁止取全量后在 Rust 或前端过滤。

## 公开接口

公开接口只读取结构化表，不要求管理员 token，由全局限流保护：

- 机构搜索、机构详情和机构账户读取 `subjects/accounts`。
- 电子护照状态按钱包公钥精确查询 `citizens`。
- 投票人数快照读取 `citizens` 聚合计数。
- 投票凭证只签发投票引擎已经定义的凭证，不实现投票流程。

## 禁止项

- 禁止恢复独立 旧独立身份系统产品、目录、CI、部署包或文档入口。
- 禁止恢复独立 `registry` 源码路径。
- 禁止恢复 `backend/src`、独立 `backend/chain`、独立 `frontend/api` 或独立 `frontend/chain`。
- 禁止在业务模块内复刻 QR 协议、扫码签名、验签或交易载荷解析。
- 禁止保留旧命名、旧文案、旧接口、旧部署脚本或旧文档作为兼容口径。
