# SFID 技术架构

## 目标状态

SFID 后端源码直接以 `sfid/backend/` 为根目录展开,不恢复 `backend/src/` 壳。前后端业务命名保持一致,公权机构使用 `gov`,私权机构使用 `private`,主体公共能力使用 `subjects`。

系统不维护历史兼容通道。所有业务主数据只存在于结构化数据库表,不得在运行期再维护第二套聚合主数据。

## 启动流程

1. 读取 `DATABASE_URL` 和 Redis 配置。
2. 初始化 PostgreSQL schema。
3. 为 `subjects/citizens/gov/private/accounts/docs/audit` 创建按省分区。
4. 初始化内置 43 个联邦管理员。
5. 启动交易索引 worker。

`sfid-backend serve` 不自动全量生成公权机构目录。确定性机构只在显式维护命令中写入。

部署入口必须在后端 schema 初始化后、启动 `serve` 前执行 `ensure-gov`。该命令检查 `gov_manifest` 与当前目录 hash,已初始化且完整则跳过;缺失、不完整或目录版本变化时才写入所有确定性公权机构和公安局。行政区划真源变化时再执行按省或按市对账。页面列表接口只能读取持久化结果,不得触发全量补数据。

## 数据表

### 主体身份

- `ids(sfid_number, kind, p_code, c_code)`:全局身份 ID 索引。
- `subjects`:主体公共展示字段,按省分区。
- `citizens`:公民电子护照绑定字段,按省分区。
- `gov`:公权机构扩展字段,按省分区。
- `private`:私权机构和非法人扩展字段,按省分区。

`sfid_number` 是唯一且不可变的身份标识。不得新增 `identity_key`、`generation_key` 等第二身份键。

### 账户与资料

- `accounts`:机构账户,主键为 `(p_code, sfid_number, account_name)`。
- `docs`:机构资料库元数据,文件本体存磁盘。

### 管理员与安全

- `admins`:联邦管理员/市级管理员。
- `sheng_admin_scope`:联邦管理员所属省。
- `admin_sessions`:登录会话。
- `admin_login_challenges`:签名登录挑战。
- `admin_qr_login_results`:扫码登录结果。
- `admin_passkeys`、`admin_passkey_challenges`:Passkey 凭据和挑战。
- `admin_action_challenges`、`admin_security_grants`:高风险操作二次确认。

登录挑战、二维码结果和会话属于短生命周期安全运行态。清理逻辑必须在 Rust 中先计算明确截止时间,再把时间点传给 SQL 比较;SQL 中不得使用 `$1 - interval '...'` 这类参数参与 interval 运算的写法,避免 PostgreSQL 无法推断绑定参数类型。数据库错误必须展开 PostgreSQL 的 SQLSTATE、message、detail 和 hint,不得只把 `db error` 传到前端或启动日志。

### CPMS 与公民绑定

- `cpms_sites`:CPMS 安装授权、安装密钥、状态和 CPMS 公钥哈希。
- `citizen_bind_challenges`:公民绑定签名挑战。
- `citizen_status_imports`:CPMS 年度状态导入幂等记录。

### 审计与链路

- `audit`:结构化审计记录,按省分区。
- `chain_requests`、`chain_nonces`:链路幂等与防重放。
- `tx_records`、`tx_indexer_state`:链上交易索引。

目标数据库不得保留废弃快照或旧机构行表;它们也不得作为兼容数据源。

## 权限模型

管理员分为联邦管理员和市级管理员:

- 联邦管理员:只能增删改查所属省数据。
- 市级管理员:只能增删改查所属市数据。

所有列表和 CRUD 接口必须把管理员范围转成 SQL 条件。禁止读取全量数据后在 Rust 或前端过滤。

市级管理员范围不单独建表。市级管理员记录保存在 `admins`,所属市使用 `admins.city`,所属省通过 `admins.created_by` 指向的联邦管理员和 `sheng_admin_scope` 解析。注册局页面统一显示 `联邦管理员列表`、`市级管理员列表`;市级管理员在该页面只读查看本人所属省的联邦管理员和本人所属市的市级管理员目录,不得显示新增、编辑或删除入口。

## 前端交互与提示

公权机构和公安局使用同一个 `GovView` 组件边界,但它们属于两个一级 tab。顶部 tab 点击必须生成重置信号,详情页本地状态必须在 `category` 或重置信号变化时清空,避免从某个机构详情页切换模块时仍停留在旧详情。

SFID 前端提示统一由 `sfid/frontend/utils/notice.ts` 管理。业务组件只允许调用 `notice.success/error/warning/info/confirm/warningModal`,不得直接调用 Ant Design `message.*`、`Modal.confirm`、`Modal.warning` 或浏览器 `alert`。统一入口负责:

- 同一时刻只显示一个提示。
- 将 WebAuthn、网络和后端错误翻译为中文。
- 将用户取消类错误显示为取消提示或静默,不得展示 W3C 英文原文。

业务组件捕获异常时必须把原始错误对象传给 `notice.error(error, '中文兜底提示')`,不得先取 `error.message` 再传入提示入口。后端 `ApiError.error_code` 和原始 `message` 的翻译只允许在 `notice.ts` 中实现;无法识别的英文错误必须在统一入口降级为中文兜底提示,不得原样显示给用户。

## 公权机构

`gov` 模块负责:

- 公安局确定性目录。
- 政府、立法院、司法院、监察院、公民教育委员会、公民储备委员会等宪法机构目录。
- 根据 `china` 行政区划变化执行目标范围对账。

公权机构不保存上下级字段。国家/部/省/市/镇只作为目录分类和行政区范围参与生成、查询和对账。主体表 `subjects` 保存身份、名称、行政区和状态;`gov` 只保存 `institution_code/org_code` 等机构类型细分;初始化批次只记录在 `gov_manifest`,不得写入初始化来源业务字段。

## 私权机构与非法人

私权机构由注册局管理员人工注册。学校属于私权机构的一种,机构类型使用教育委员会代码 `JY`。

非法人能力放在 `sfid/backend/subjects/uninorg`,因为公权机构和私权机构都可能拥有从属非法人机构。

## 公开接口

公开查询不要求管理员 token,由全局限流保护。接口只读取结构化表:

- `/api/v1/app/institutions/search`
- `/api/v1/app/institutions/:sfid_number`
- `/api/v1/app/institutions/:sfid_number/accounts`
- `/api/v1/app/clearing-banks/search`
- `/api/v1/app/clearing-banks/eligible-search`
- `/api/v1/app/voters/count`
- `/api/v1/app/vote/credential`
- `/api/v1/app/myid/status`

## 禁止项

- 不得恢复旧聚合快照目录或旧运行期分片缓存。
- 不得双写历史格式。
- 不得新增历史兼容接口。
- 不得在 SFID 业务模块内实现投票流程。投票流程只属于投票引擎,SFID 只签发其已定义的凭证。
