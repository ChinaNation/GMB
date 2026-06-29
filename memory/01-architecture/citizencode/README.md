# CID 架构总览

CID 是注册局运营的身份 ID 系统。系统登记三类主体:

- 自然人:当前实现注册局直接录入公民护照身份,并由公民账户完成绑定。
- 公法人:公权机构,其中宪法确定性机构由系统按行政区划和宪法常量生成。
- 私法人/非法人:由注册局管理员按权限人工注册。

## 源码边界

- `citizenchain/registry/src/core`:数据库连接、HTTP 安全、运行期维护、二维码等通用能力。
- `citizenchain/registry/src/cid/china`:中国行政区划 SQLite 真源。
- `citizenchain/registry/src/cid`:身份 ID 编码协议。
- `citizenchain/registry/src/admins`:注册局机构 admins、登录、冷钱包扫码二次确认和权限上下文。
- `citizenchain/registry/src/gov`:公权机构确定性目录,CPOL 与其它市级公权机构同模板生成。
- `citizenchain/registry/src/private`:个体经营、合伙企业、股权公司、股份公司、公益组织、注册协会六类私权机构能力。
- `citizenchain/registry/src/subjects`:主体公共模型、通用注册内核、主体详情和公开查询。
- `citizenchain/registry/src/citizens`:公民录入、账户绑定、CitizenApp 查询和投票凭证。
- `citizenchain/registry/src/accounts`:机构账户管理。
- `citizenchain/registry/src/docs`:机构资料库。
- `citizenchain/registry/src/audit`:审计查询。
- `citizenchain/registry/src/indexer`:链上交易索引。

前端按同名边界拆分为 `citizencode/frontend/gov`、`citizencode/frontend/private`、`citizencode/frontend/subjects`、`citizencode/frontend/admins` 等目录。功能模块自己的 API 放在所属模块内,通用 HTTP 封装只放 `frontend/utils/http.ts`。

## 数据真源

后端以 PostgreSQL 结构化表作为唯一持久化真源。进程内只允许保存短生命周期局部变量,不得再维护第二套业务主数据。

启动期 schema 初始化负责把结构化库收敛到当前目标状态。流程必须先创建缺失父表,再补齐当前字段、删除废弃字段、校验目标状态,最后才创建依赖字段的索引。字段变更不得要求清空数据库;清库只属于本地重置环境动作,不是正式部署或覆盖部署的默认方案。

核心表:

- `ids`:全局唯一身份 ID 索引,保证一个 `cid_number` 只属于一个主体大类。
- `subjects`:公民、公权机构、私权机构公共主体表,按 `province_code` 省分区;机构行保存名称、行政区、业务状态和法定代表人资料。
- `citizens`:公民电子护照绑定结果,按 `province_code` 省分区。
- `gov`:公权机构扩展表,按 `province_code` 省分区,不区分初始化录入和人工新增。
- `private`:六类私权机构扩展表,按 `province_code` 省分区。
- `accounts`:机构账户表,按 `province_code` 省分区。
- `docs`:机构资料库元数据表,按 `province_code` 省分区。
- `audit`:审计表,按 `province_code` 省分区。
- `admins`、`federal_registry_scope`:注册局机构 admins 与权限范围。市注册局机构 admins 范围由 `admins.created_by + city_name` 解析,不再维护第二张市注册局机构管理员范围表。
- `citizen_bind_challenges`:公民绑定挑战运行态。
- `admin_*`:登录、会话、冷钱包扫码二次确认运行态。
- `chain_requests`、`chain_nonces`:链路幂等和防重放状态。

废弃快照表和旧机构行表不得保留在目标库中。目标结构只承认上方列出的精简表。

机构本身没有链上状态字段。链上状态只属于机构账户,即 `accounts.chain_status`。机构详情页不得展示机构链上状态;账户列表可以展示账户链上状态。

机构法定代表人资料写入 `subjects`:姓名由管理员输入,身份ID从正常状态公民中选择,证件照保存为文件元数据。候选公民搜索只返回 `cid_number`,不得返回姓名等 CID 公民模型不存在的字段。初始化生成的公权机构允许该资料暂为空;人工新增机构创建时必填,既有机构编辑保存时必须补齐。

## 权限与查询

联邦注册局机构 admins 的业务数据只查询自己省的数据;联邦注册局管理员目录是全量只读目录,必须带 `province_name` 列,写操作仍只允许更换自己省的管理员。市注册局机构 admins 只查询自己市的数据。后端必须在 SQL 层携带 `province_code` / `city_code` 条件,不得先取全量再在内存过滤。

公权机构属于确定性目录。列表接口直接从 `subjects/gov/accounts` 读取目标范围数据,不会在页面进入时全量重建。

注册局页面只显示两个管理员目录入口:联邦注册局机构 admins 和市注册局机构 admins。市注册局机构 admins 可以只读查看本人所属省和本人所属市的管理员,但不能新增、编辑或删除管理员。联邦注册局机构 admins 列表显示全部省份管理员,但写操作只允许同省范围内更换;联邦注册局机构 admins 不允许本地新增或删除。

管理员列表接口必须按登录管理员范围执行 SQL 查询:联邦注册局机构 admins 对联邦管理员返回全量并带 `province_name` 列,对市注册局管理员只返回本省;市注册局机构 admins 按联邦注册局机构归属和 `admins.city_name` 查询。不得在前端用旧登录态里的 `全国` 伪省份参与查询。

## 前端状态与提示

公权机构和公安局共用 `gov` 前端边界,但顶部 tab 是一级模块切换。进入机构详情页后再次点击 `公权机构` 或 `公安局` 必须重置详情状态并切换到目标模块入口,不得因复用组件实例停留在旧详情页。

公权机构、公安局、私权机构列表都必须展示连续序号。机构详情页身份字段统一为 `身份ID`,不得使用代码框包裹,不得展示 `SubjectProperty 类型` 或机构链上状态。

CID 前端所有用户提示统一走 `citizencode/frontend/utils/notice.ts`。业务组件不得直接调用 Ant Design 的 `message.*`、`Modal.confirm`、`Modal.warning` 或浏览器 `alert`。提示必须为中文,同一时刻只显示一个提示;网络错误、扫码签名错误和后端错误必须在统一入口翻译后再展示。

## 自动生成机构

`citizencode/backend/gov` 按以下输入生成目标目录:

- `citizenchain/registry/src/cid/china` 的行政区划 SQLite。
- `citizenchain/runtime/primitives/cid/china/*.rs` 中的宪法机构常量和既有 `cid_number`。

对账只在显式维护动作中执行。后端 `serve` 只初始化结构化 schema 和内置联邦注册局机构管理员,不得全量写入确定性机构目录。

部署入口必须在启动 `serve` 前执行 `ensure-gov`:它先检查 `gov_manifest` 和当前确定性目录 hash,已初始化且完整则跳过;缺失、不完整或目录版本变化时才写入 `subjects/gov/accounts/gov_manifest`。行政区划变化时执行按省或按市对账,不得让页面请求触发全量补数据。

## 公民护照

注册局管理员直接录入公民护照身份。公民绑定结果写入 `citizens` 和 `subjects`，CitizenApp 按钱包公钥查询电子护照状态。

## 链端公开查询

公开查询接口只读结构化表:

- 机构搜索/详情/账户读取 `subjects/accounts`。
- 机构注册信息凭证只返回 `cid_number / cid_full_name / account_names[]` 和验签包装字段。
- 投票人数快照读取 `citizens` 聚合计数。
- 投票凭证和电子护照状态按钱包公钥精确查询 `citizens`。
