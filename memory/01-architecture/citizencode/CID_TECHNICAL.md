# CID 技术架构

## 目标状态

CID 后端源码直接以 `citizencode/backend/` 为根目录展开,不恢复 `backend/src/` 壳。前后端业务命名保持一致,公权机构使用 `gov`,私权机构使用 `private`,主体公共能力使用 `subjects`。

系统不维护历史兼容通道。所有业务主数据只存在于结构化数据库表,不得在运行期再维护第二套聚合主数据。
重构后不得保留不符合目标态的旧数据。私权机构拆分后,`PRIVATE_INSTITUTION` 行必须带有
非空 `name` 和六类之一的 `private_type`,否则必须在同一任务中删除其 `subjects/private/accounts/docs/ids/audit`
关联残留,不得补字段后作为兼容数据继续保留。

## 启动流程

1. 读取 `DATABASE_URL`、Redis 配置和行政区只读 SQLite 配置 `CID_CHINA_DB`。
2. 初始化 PostgreSQL schema 父表。
3. 将父表收敛到当前目标字段:新增缺失字段、删除废弃字段。
4. 校验关键目标字段存在、废弃字段不存在。
5. 创建当前目标索引。
6. 为 `subjects/citizens/gov/private/accounts/docs/audit` 创建按省分区。
7. 读取随包只读行政区 SQLite，为 `subjects/citizens/gov/private/accounts/docs/audit` 创建当前 43 个省级分区。
8. 初始化内置 43 个联邦注册局机构管理员。
9. 启动交易索引 worker。

schema 初始化和业务目录初始化必须分离。schema 收敛每次启动都可以执行,但只允许把数据库结构调整到当前目标状态,不得保留旧字段或旧接口作为兼容通道。任何依赖新字段的索引、约束或业务 SQL 都必须放在字段收敛和目标状态校验之后执行。

`citizencode-backend serve` 不自动全量生成公权机构目录,但启动前会校验全局 `gov_manifest`
是否匹配当前 `china.sqlite` hash 和目录 hash。目录过期时生产环境直接拒绝启动;本地开发如
明确设置 `CID_GOV_AUTO_RECONCILE=1`,才允许启动前自动执行一次 `reconcile-gov --changed-only`。

生产部署入口必须在安装新版后端二进制和 `/opt/citizencode/china/china.sqlite` 后、启动 `serve` 前执行
`reconcile-gov --changed-only` 和 `check-gov --strict`。`ensure-gov` 保留为幂等维护命令:
它检查 `gov_manifest` 与当前目录 hash,已初始化且完整则跳过;缺失、不完整或目录版本变化时
写入确定性公权机构和公安局。页面列表接口只能读取持久化结果,不得触发全量补数据。

行政区开发库权威源是 `citizencode/backend/china/china.sqlite`。正式部署时 `CID_CHINA_DB` 固定为
`/opt/citizencode/china/china.sqlite`,后端只读打开。行政区变更只能修改开发库并重新发布安装包,
不得在 CID 运行中改库或恢复行政区管理 tab。

## 数据表

### 主体身份

- `ids(cid_number, kind, province_code, city_code)`:全局身份 ID 索引。
- `subjects`:主体公共展示字段,按省分区;机构行保存 `name/cid_full_name/cid_short_name`、行政区、业务状态、私权分类和法定代表人资料。
- `citizens`:公民电子护照绑定字段,按省分区。
- `gov`:公权机构扩展字段,按省分区;只保存 `institution_code/org_code` 等机构类型细分。
  `source='GENERATED'` 表示由行政区和模板确定性派生,可被对账命令更新或删除;
  `source='MANUAL'` 表示管理员手动创建,不得被行政区对账当作 obsolete 删除。
- `private`:私权目标类型机构扩展字段,按省分区;分类字段为 `private_type/partnership_kind/has_legal_personality`。

`cid_number` 是唯一且不可变的身份标识。不得新增 `identity_key`、`generation_key` 等第二身份键。

### 账户与资料

- `accounts`:机构账户,主键为 `(province_code, cid_number, account_name)`。
- `docs`:机构资料库元数据,文件本体存磁盘。

机构本身不保存链上状态。链上状态只属于 `accounts.chain_status`,用于账户是否已在链上激活、注销或等待同步。机构详情页不得展示机构链上状态字段。

机构法定代表人资料归属 `subjects`,包括:

- `legal_rep_name`:法定代表人姓名,由管理员输入。
- `legal_rep_cid_number`:法定代表人身份ID,只能从正常状态公民中选择。
- `legal_rep_photo_path/legal_rep_photo_name/legal_rep_photo_mime/legal_rep_photo_size`:证件照元数据。

初始化生成的公权机构允许法定代表人资料暂为空;任何机构进入编辑保存时必须补齐姓名、身份ID和证件照。新增人工机构必须在创建时填写三项。候选公民搜索接口只返回 `cid_number` 字符串,不得返回姓名等 CID 公民模型不存在的字段。

### 管理员与安全

- `admins`:注册局机构管理员账户。
- `federal_registry_scope`:联邦注册局机构管理员所属省。
- `admin_sessions`:登录会话。
- `admin_login_challenges`:签名登录挑战。
- `admin_qr_login_results`:扫码登录结果。
- `admin_passkeys`、`admin_passkey_challenges`:Passkey 凭据和挑战。
- `admin_action_challenges`、`admin_security_grants`:高风险操作二次确认。

登录挑战、二维码结果和会话属于短生命周期安全运行态。清理逻辑必须在 Rust 中先计算明确截止时间,再把时间点传给 SQL 比较;SQL 中不得使用 `$1 - interval '...'` 这类参数参与 interval 运算的写法,避免 PostgreSQL 无法推断绑定参数类型。数据库错误必须展开 PostgreSQL 的 SQLSTATE、message、detail 和 hint,不得只把 `db error` 传到前端或启动日志。
任何会在持有数据库连接锁时读取 `postgres::Row` 的代码,必须保证 SELECT 字段顺序和 `row.get(index)`
逐项一致;越界 panic 会污染连接池并导致后续接口出现 `postgres client lock poisoned`。

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

管理员唯一真源为机构或个人多签的 `admins`。CID 管理端按登录账户所属注册局机构判定权限:

- `registry_org_code=FEDERAL_REGISTRY`:联邦注册局机构的 admins,只能增删改查所属省数据。
- `registry_org_code=CITY_REGISTRY`:市注册局机构的 admins,只能增删改查所属市数据。

所有列表和 CRUD 接口必须把管理员范围转成 SQL 条件。禁止读取全量数据后在 Rust 或前端过滤。

市注册局机构管理员范围不单独建表。管理员记录保存在 `admins`,所属市使用 `admins.city_name`,所属省通过 `admins.created_by` 指向的联邦注册局机构管理员和 `federal_registry_scope` 解析。注册局页面统一显示联邦注册局机构 admins、市注册局机构 admins;市注册局机构 admins 在该页面只读查看本人所属省和所属市的管理员目录,不得显示新增、编辑或删除入口。

## 前端交互与提示

公权机构和公安局使用同一个 `GovView` 组件边界,但它们属于两个一级 tab。顶部 tab 点击必须生成重置信号,详情页本地状态必须在 `category` 或重置信号变化时清空,避免从某个机构详情页切换模块时仍停留在旧详情。

公权机构、公安局、私权机构列表必须显示连续序号。机构详情页的身份字段统一显示为 `身份ID`,不得使用代码框包裹,不得展示 `SubjectProperty 类型` 或机构链上状态。账户列表可以展示账户链上状态。

CID 前端提示统一由 `citizencode/frontend/utils/notice.ts` 管理。业务组件只允许调用 `notice.success/error/warning/info/confirm/warningModal`,不得直接调用 Ant Design `message.*`、`Modal.confirm`、`Modal.warning` 或浏览器 `alert`。统一入口负责:

- 同一时刻只显示一个提示。
- 将 WebAuthn、网络和后端错误翻译为中文。
- 将用户取消类错误显示为取消提示或静默,不得展示 W3C 英文原文。

业务组件捕获异常时必须把原始错误对象传给 `notice.error(error, '中文兜底提示')`,不得先取 `error.message` 再传入提示入口。后端 `ApiError.error_code` 和原始 `message` 的翻译只允许在 `notice.ts` 中实现;无法识别的英文错误必须在统一入口降级为中文兜底提示,不得原样显示给用户。

管理员扫码登录的端侧职责固定为:CID 页面生成 `CITIZEN_QR_V1 / login_challenge`,
`citizenwallet` 公民钱包扫描并生成 `login_receipt`,CID 页面再扫描该登录回执。`citizenapp`
不承担管理员登录 QR 职责,前端文案不得引导用户使用 citizenapp 处理登录挑战。

## 公权机构

`gov` 模块负责:

- 公安局确定性目录。
- 政府、立法院、司法院、监察院、公民教育委员会、公民储备委员会等宪法机构目录。
- 根据 `china` 行政区划变化执行目标范围对账。

公权机构不保存上下级字段。国家/部/省/市/镇只作为目录分类和行政区范围参与生成、查询和对账。主体表 `subjects` 保存身份、名称、行政区和状态;`gov` 只保存 `institution_code/org_code` 等机构类型细分和自动/手工边界;初始化批次只记录在 `gov_manifest`,不得写入批次来源业务字段。
自动目录的同步边界以 `gov.source` 区分:确定性目录统一写 `GENERATED`,手工公权机构统一写
`MANUAL`。行政区删除、改名或 code tombstone 只会清理 `GENERATED` 目录及其
`subjects/gov/accounts/docs/ids/audit` 派生残留,不得误删手工机构。

## 私权机构与非法人

私权机构由注册局管理员人工注册。私权入口拆成个体经营、合伙企业、股权公司、股份公司、
公益组织、注册协会六类;身份 ID 格式不变,后端按 `private_type` 锁定
`subject_property + institution_code + p1`。

教育委员会学校机构统一归教育机构入口管理,机构类型使用教育委员会代码 `JY`,不在私权六类 Tab 中出现。

非法人能力放在 `citizencode/backend/subjects/uninorg`,因为公权机构和私权机构都可能拥有从属非法人机构。
个体经营 `F+GT` 和无限合伙 `F+GP` 是独立非法人,不选择所属法人;其它从属非法人仍按
`subjects/uninorg` 校验所属法人、地域和盈利属性继承。

## 公开接口

公开查询不要求管理员 token,由全局限流保护。接口只读取结构化表:

- `/api/v1/app/institutions/search`
- `/api/v1/app/institutions/:cid_number`
- `/api/v1/app/institutions/:cid_number/accounts`
- `/api/v1/app/voters/count`
- `/api/v1/app/vote/credential`
- `/api/v1/app/myid/status`

清算行属于链上组织治理概念,不属于 CID 身份设计;CID 不提供清算行相关公开接口。

## CI 与发布边界

- `citizencode-ci.yml` 的 push / pull_request 自动 CI 只允许执行后端编译、后端测试、前端依赖安装和前端构建。
- 正式 `citizencode.deb` 只允许在 GitHub 页面手动 `Run workflow` 时构建和上传;push 自动 CI 不生成正式发布包。
- 当前 CID workflow 不部署服务器,不得新增 push 自动部署或读取部署 SSH 密钥的步骤。

## 禁止项

- 不得恢复旧聚合快照目录或旧运行期分片缓存。
- 不得双写历史格式。
- 不得新增历史兼容接口。
- 不得保留旧数据、旧注释、旧文档或旧 UI 文案作为兼容口径。
- 涉及 API、数据库、登录、扫码或页面展示的任务,不得只用编译或 build 作为完成验收;必须打真实服务接口或检查真实页面。
- 不得在 CID 业务模块内实现投票流程。投票流程只属于投票引擎,CID 只签发其已定义的凭证。
