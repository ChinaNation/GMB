# OnChina 技术架构

## 1. 定位

OnChina 是公民链内置的链上中国多机构工作台，负责各机构通过“节点端 + 浏览器”进入本机构后台。注册局、司法院、立法院、行政机关、学校、公益组织、公司等机构共用同一套登录、绑定、权限和运行态框架；具体 UI 由当前登录钱包账户所属的链上机构管理员身份决定。

注册局不再是 OnChina 根 UI 的同义词，而是 `workspace` 机构工作台中的一种类型。OnChina 仍负责 CID 号、行政区、注册局既有业务、公民电子护照档案、机构登记、机构公开查询、链上公民身份提交和链侧调用生成；非注册局机构按自己的工作台能力进入“操作 / 显示 / 记录”三类页面，不复用注册局业务 UI。

OnChina 不是第五个产品。仓库产品只保留：

- 公民 `citizenapp`
- 公民链 `citizenchain`
- 公民钱包 `citizenwallet`
- 官方网站 `citizenweb`

### 1.1 与 CitizenApp 边缘架构的边界

OnChina 可以向 CitizenApp 或 Cloudflare 边缘层提供公开目录、链上投影、机构资料查询和受控服务端聚合能力，但不得成为 CitizenApp 的链上状态真源。

固定边界：

- CitizenApp 链上余额、身份、提案、投票和交易成功判断以端上轻节点读取的 finalized runtime storage 为准。
- OnChina / Citizen API 可以广播 CitizenApp 已经本地签名完成的 extrinsic，但不接触私钥、不修改交易载荷、不把广播成功解释为链上成功。
- OnChina 的主入口仍是机构内网工作台 `https://onchina.local:8964`；如后续提供公网投影服务，必须以独立受控服务节点、反向代理、白名单、限流和审计为边界，不直接暴露国储会核心节点 RPC。
- OnChina 不恢复旧独立后端目录，不新建 `backend/src/`、`backend/chain/` 或 `frontend/chain/` 业务壳。

## 2. 技术栈

- 后端：Rust + Axum + PostgreSQL
- 前端：React + TypeScript + Vite + Ant Design
- 链交互：Substrate RPC、SCALE、统一 QR_V1 扫码签名协议
- 行政区开发真源：`citizenchain/onchina/src/cid/china/china.sqlite`

## 3. 启动流程

0. 节点桌面端默认不启动 OnChina；用户在节点设置页“链上中国平台”行点击“启动”并二次确认后，节点才拉起 OnChina 子进程。
1. 读取 `DATABASE_URL`、`ONCHINA_CHINA_DB`、链 RPC 和安全配置。
2. 初始化 PostgreSQL schema 父表。
3. 将父表收敛到当前目标字段，新增缺失字段并删除废弃字段。
4. 校验关键字段存在、废弃字段不存在。
5. 创建当前目标索引。
6. 为 `subjects/citizens/citizen_documents/gov/private/accounts/docs/audit` 创建 `province_code` 分区。
7. 读取随包只读行政区 SQLite，并断言 SQLite 省表与 runtime primitives `PROVINCE_CODE_INFOS` 一致。
8. 初始化登录、节点绑定和链路运行态结构化表。
9. 启动链上交易索引 worker。

schema 初始化和业务目录初始化必须分离。服务启动只允许做结构收敛、分区创建、内置管理员投影和索引 worker 启动；确定性公权机构目录只能在显式维护命令中生成或对账。

局域网访问入口固定为 `https://onchina.local:8964`。OnChina 监听 `0.0.0.0:8964`，通过 mDNS 广告 `onchina.local`，TLS 自签证书目标主机为 `onchina.local`。节点设置页只负责启动服务，不自动打开浏览器；管理员在自己的电脑浏览器中访问该固定入口。

登录身份不由节点安装前预配置。管理员冷钱包扫码后，后端用 `verified_pubkey` 反查链上 active admin 所属机构；该钱包属于一个机构时直接进入本机构工作台，属于多个机构时先选择机构再进入对应工作台。未绑定节点返回候选机构并要求浏览器二次确认绑定，已绑定节点只允许该机构 active admin 登录。节点绑定只作为本机机构归属结果缓存，权限真源仍是链上 active admin 集合。解绑或换机构必须走 `NODE_BINDING_UNBIND` 冷签安全动作：当前本机会话管理员发起，冷钱包管理员签名确认，成功后停用 active binding 并清退本节点管理员会话，再重新登录绑定新机构。

### 3.1 机构工作台框架

- 后端 `citizenchain/onchina/src/workspace/` 是当前登录机构工作台清单的唯一生成层，只输出 `workspace_kind / workspace_title / workspace_sections`，不保存第二份管理员授权真源。
- 前端 `citizenchain/onchina/frontend/workspace/` 是机构工作台挂载层。`registry/` 只承载注册局既有 UI，`judicial/` 承载司法院专属工作台，`generic/` 承载尚未落专属 UI 的普通机构。
- 工作台顶层分类固定为 `operations`（操作）、`display`（显示）、`records`（记录）。操作放本机构可发起的提案、投票、管理员变更等动作；显示放本机构身份、权限和管理员；记录放登录、链写、投票、管理员变更等事实记录。
- 现有公权机构统一在创世阶段上链。OnChina 不在运行期生成既有公权机构目录，只读取链上机构和链上 active admins，并用本地 `subjects/accounts/admins` 投影补齐展示字段。
- 注册局工作台必须保持既有功能和 UI；新增司法院、立法院、学校、公司、公益组织等机构 UI 时，只能新增或扩展对应工作台目录，不得把机构差异重新塞回 `frontend/App.tsx`。

## 4. 行政区和 CID 号真源

- 国家码、省级行政区码和机构码常量唯一真源：`citizenchain/runtime/primitives/cid/code.rs`。
- 市、镇和地址段开发真源：`citizenchain/onchina/src/cid/china/china.sqlite`。
- CID 号生成和校验唯一源码目录：`citizenchain/onchina/src/cid/`。

生产环境中 `ONCHINA_CHINA_DB` 固定指向随包只读 SQLite。市镇地址段变更只能修改开发库并重新发布安装包，禁止运行期在线编辑行政区。

## 5. 结构化表

- `ids(cid_number, kind, province_code, city_code)`：全局身份 ID 索引。
- `subjects`：主体公共展示字段，按省分区；机构行缓存 `cid_full_name/cid_short_name`、行政区、业务状态、私权分类，以及链上公开的 `legal_representative_name/cid_number/account`。法定代表人照片只属于 OnChina 链下资料，不进入 `InstitutionInfo`。
- `citizens`：公民档案、姓、名、身份 CID、护照号、钱包地址、`province_code/city_code/town_code` 居住/办理地、`birth_*` 出生地和电子护照有效期字段，按省分区。
- `citizen_documents`：公民独立资料库元数据,资料类型固定为“护照相片 / 出生证明 / 监护人护照 / 其他材料”,文件本体在磁盘；不得与机构 `docs` 共表。
- `gov`：公权机构扩展字段，按省分区；自动目录写 `source='GENERATED'`，人工公权机构写 `source='MANUAL'`。
- `private`：私权机构扩展字段，按省分区；分类字段使用 `private_type/partnership_kind/has_legal_personality`。
- `accounts`：机构账户，主键按 `(province_code, cid_number, account_name)` 收敛。
- `docs`：机构资料库元数据，文件本体在磁盘。
- `audit`：结构化审计记录，按省分区。
- `admins`：机构管理员本地元数据缓存，成员资格真源在链上 active admin 集合。
- `node_institution_bindings`、`node_binding_challenges`：本节点首次登录绑定机构和绑定确认挑战；绑定不是权限真源，只限制本节点后续登录机构；解绑 / 换机构由 `NODE_BINDING_UNBIND` 冷签动作停用 active binding。
- `admin_login_sign_requests`、`admin_qr_login_results`、`admin_action_challenges`、`admin_security_grants`：登录和扫码签名运行态。
- `chain_requests`、`chain_nonces`、`tx_records`、`tx_indexer_state`：链路幂等、防重放和索引运行态。

`cid_number` 是唯一且不可变的身份标识。不得新增 `identity_key`、`generation_key` 等第二身份键。

## 6. 高并发策略

OnChina 高并发目标建立在结构化表、组合索引、省分区和省市范围查询之上。

必备原则：

- 后台列表必须在 SQL 层携带 `province_code` / `city_code` 条件。
- 联邦注册局机构 `admins` 查询本省业务数据时必须带 `province_code`。
- 市注册局机构 `admins` 查询本市业务数据时必须带 `province_code + city_code`。
- 页面列表只读持久化结果，禁止同步触发全量对账。
- 高频公开查询可增加短 TTL 缓存，但缓存不得成为主数据。

必备索引：

- `subjects(province_code, city_code, kind, status, cid_number)`
- `subjects(province_code, city_code, cid_full_name)`
- `citizens(province_code, city_code, created_at DESC, id DESC)`
- `citizens(province_code, city_code, cid_number, wallet_pubkey, wallet_address)`
- `citizens(province_code, city_code, town_code, created_at DESC, id DESC)`
- `citizen_documents(province_code, cid_number, uploaded_at DESC, id DESC)`
- `gov(province_code, city_code, town_code, institution_code)`
- `private(province_code, city_code, private_type, cid_number)`
- `accounts(province_code, cid_number)`
- `docs(province_code, cid_number, uploaded_at DESC)`
- `audit(province_code, city_code, created_at DESC)`
- `admins(registry_org_code, city_name)`
- `admins(lower(admin_account))`

## 7. 管理员和安全

管理员唯一字段统一为 `admins`。OnChina 不恢复独立管理员身份表、授权真源或授权分支。

- 机构管理员列表以链上 `admins` 账户集合与 entity 机构岗位任职关系联合查询为真源。岗位、权限、任期和来源不再从 `AdminProfile` 读取；本地联系方式、照片和 Passkey 仅是私密资料，不得成为管理员资格或岗位真源。
- 联邦注册局管理员目录从链上 `PublicAdmins::FederalRegistryProvinceGroups` 全量读取 43 个省级组；本省 5 人置顶且可更换，其它省份只读。`federal_registry_admin_scopes` 只缓存省级组归属用于展示和同省预检，不是权限真源。
- 联邦注册局和市注册局不再提供“编辑本地管理员姓名”的前端入口或 PATCH 动作；联邦注册局管理员更换、下级市注册局新增/删除仍走安全动作和冷钱包确认。
- 登录态：用于普通读取和低风险操作。
- `PASSKEY_COLD_SIGN`：用于管理员安全写操作、Passkey 更新、管理员集合变更、节点解绑和链写入二次确认。
- 扫码请求：统一使用 `QR_V1 / k=1 sign_request`。
- 签名响应：统一使用 `QR_V1 / k=2 sign_response`。

业务模块只传入动作码、签名原文、摘要和展示字段，不得自己包装二维码协议。

## 8. 前端规则

前端所有用户提示统一走 `citizenchain/onchina/frontend/utils/notice.ts`。业务组件不得直接调用 Ant Design `message.*`、`Modal.confirm`、`Modal.warning` 或浏览器 `alert`。

机构详情页身份字段统一显示为 `身份ID`，不得使用代码框包裹，不得展示 `SubjectProperty 类型` 或机构链上状态。机构链上状态只属于机构账户，允许在账户列表展示。

扫码确认页的左侧分类名必须是中文，右侧内容是用户可核对的值；机器字段不得直接渲染给用户。

### 8.1 法律文库展示

- 法律文库只读详情页展示公民宪法时，`law_id=0` 的标题、章、节、条、款均以链上结构化正文为唯一展示真源。
- 章、节、条标题直接显示链上 `title/titleEn`，只有空标题才使用兜底标题；款正文直接显示链上 `Clause.text/textEn`，不得在 UI 层额外拼接“第 x 款 / Paragraph x”。
- 后端 `LawView.immutableArticleNumbers` 只从 `LegislationYuan.ConstitutionImmutableManifest` 投影宪法不可修改条款号，普通法律保持空数组。
- 后端 `LawView.versionTitle/versionTitleEn` 只从 `LegislationYuan.LawVersionLabels[(law_id, version)]` 投影版本标签；公民宪法创世版本显示“创世版 / Genesis Edition”，无标签版本继续显示 `vN`。
- 前端不可修改条款徽章中文固定显示“不可修改条款”，英文固定显示“Immutable Clause”。
- 法律详情页切换中英文时，徽章必须紧跟当前语言的条标题：中文模式只在中文条标题后显示“不可修改条款”，英文模式只在英文条标题后显示“Immutable Clause”，禁止在同一个徽章内混排中英文。
- 徽章必须与当前语言条标题行内垂直居中；中文徽章使用更小字号，避免抢占条标题视觉层级。
- 法律编辑弹窗里的结构定位只允许显示“章序 / 节序 / 条序 / 款序”，不得把结构序号伪装成正文标题。

## 9. 发布和 CI 边界

OnChina 属于 `citizenchain`。不再保留独立 旧独立身份系统 CI、独立 旧独立身份系统安装包 或独立产品发布入口。

- 修改 `citizenchain/onchina/src/**`：执行 OnChina 后端编译、测试和真实 HTTP 验收。
- 修改 `citizenchain/onchina/frontend/**`：执行前端 build，并通过真实页面检查关键流程。
- 修改节点桌面端 OnChina 启动入口：同步检查 `citizenchain/node/src/onchina_proc/mod.rs`、`citizenchain/node/src/settings/onchina_platform/` 和节点设置页，确认 OnChina 不随节点默认启动。
- 涉及 QR、签名、链交易载荷、CID 号格式时：必须同步更新 `memory/07-ai/unified-protocols.md` 和相关端实现。

## 10. 验收口径

涉及 API、数据库、登录、权限、扫码或页面展示的任务，必须使用真实本地服务、真实 PostgreSQL、真实 HTTP 接口或真实页面验收。只通过编译、类型检查或前端 build 不算完成。

最低检查：

```text
rg "旧独立身份系统名" memory AGENTS.md citizenchain/onchina --glob '!memory/08-tasks/**' --glob '!memory/04-decisions/**'
cargo check --manifest-path citizenchain/Cargo.toml -p onchina
npm --prefix citizenchain/onchina/frontend run build
```

如果工作区存在其它线程的未完成改动，验收必须说明受影响的命令和原因，不得把其它线程的失败混入本任务结论。
