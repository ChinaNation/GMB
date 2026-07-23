# OnChina 技术架构

## 1. 定位

OnChina 是公民链内置的链上中国多机构工作台，负责各机构通过“节点端 + 浏览器”进入本机构后台。注册局、司法院、立法院、行政机关、学校、公益组织、公司等机构共用同一套登录、绑定、权限和运行态框架；具体 UI 由当前登录 `account_id` 所属的链上机构管理员身份决定。

注册局不再是 OnChina 根 UI 的同义词，而是 `workspace` 机构工作台中的一种类型。OnChina 仍负责 CID 号、行政区、注册局既有业务、公民电子护照档案、机构登记、机构公开查询、链上公民身份提交和链侧调用生成；非注册局机构按自己的工作台能力进入“操作 / 显示 / 记录”三类页面，不复用注册局业务 UI。

OnChina 不是第五个产品。仓库产品只保留：

- 公民 `citizenapp`
- 公民链 `citizenchain`
- 公民钱包 `citizenwallet`
- 官方网站 `citizenweb`

### 1.1 账户标识目标契约

- PostgreSQL、Rust、TypeScript、JSON、缓存和链上调用中的单一账户字段统一为 `account_id`；多账户结构使用准确的 `<role>_account_id`。
- 账户与 32 字节公钥的文本形式统一为小写 `0x` 加 64 位十六进制；`ss58_address` 仅作派生展示值，不作为登录、权限或数据库关系真源。
- 登录验签必须从 `signer_public_key` 得到 `signer_account_id`，再读取链上 admins、有效岗位任职和岗位权限；节点绑定或本地投影不得产生第二套授权。
- OnChina PostgreSQL 业务库按最终 schema 重建，不写迁移、双读或兼容列；密钥和 Secret 不在删除范围。
- 2026-07-22 已删除并重建本机 `127.0.0.1:5433/onchina` 业务库；重建后旧账户列和旧账户索引均为零，账户与公钥格式约束按最终 schema 生效。
- 完整目标与进度见 ADR-040 和任务卡 `20260722-account-id-official-unify.md`。

### 1.2 与 CitizenApp 边缘架构的边界

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
2. 直接初始化 PostgreSQL 最终 schema、约束和索引，不执行迁移或字段回填。
3. 为 `subjects/citizens/citizen_documents/gov/private/accounts/docs/audit/institution_admins` 创建 `province_code` 分区。
4. 读取随包只读行政区 SQLite，并断言 SQLite 省表与 runtime primitives `PROVINCE_CODE_INFOS` 一致。
5. 初始化登录、节点绑定和链路运行态结构化表。
6. 初始化链上公权机构查询投影并启动交易索引 worker。

schema 初始化和链上业务投影必须分离。schema 入口只允许幂等创建最终结构与分区，不得携带 `ALTER/DROP`、旧列清理或兼容读取；公权机构目录只能从链上唯一真源投影。

局域网访问入口固定为 `https://onchina.local:8964`。OnChina 监听 `0.0.0.0:8964`，通过 mDNS 广告 `onchina.local`，TLS 自签证书目标主机为 `onchina.local`。节点设置页只负责启动服务，不自动打开浏览器；管理员在自己的电脑浏览器中访问该固定入口。

登录身份不由节点安装前预配置。管理员使用 CitizenWallet 扫码后，后端校验 `signer_public_key` 与签名，得到并严格比较 `account_id`，再查询链上 active admin 所属机构；该账户属于一个机构时直接进入本机构工作台，属于多个机构时先选择机构再进入对应工作台。未绑定节点返回候选机构并要求浏览器二次确认绑定，已绑定节点只允许该机构 active admin 登录。节点绑定只作为本机机构归属结果缓存，权限真源仍是链上 active admin 集合。解绑或换机构必须走 `NODE_BINDING_UNBIND` 扫码签名安全动作：当前本机会话管理员发起，CitizenWallet 签名一次并显示响应二维码，OnChina 回扫后停用 active binding 并清退本节点管理员会话，再重新登录绑定新机构。

### 3.1 机构工作台框架

- 后端 `citizenchain/onchina/src/workspace/` 是当前登录机构工作台清单的唯一生成层，只输出 `workspace_kind / workspace_title / workspace_sections / workspace_modules`，不保存第二份管理员授权真源。实例级模块必须按登录态准确 `institution_cid_number` 和 finalized 链状态判定。
- 前端 `citizenchain/onchina/frontend/workspace/` 是机构工作台挂载层。`RegistryWorkspace` 只承载注册局既有 UI，`PrivateInstitutionWorkspace` 只承载本私权机构信息、链上 `admins` 与授权模块，`judicial/` 承载司法院专属工作台，`GenericWorkspace` 只承载其它公权、立法和非法人机构的通用显示壳。
- 工作台顶层分类固定为 `operations`（操作）、`display`（显示）、`records`（记录）。操作放本机构可发起的提案、投票、管理员变更等动作；显示放本机构身份、权限和管理员；记录放登录、链写、投票、管理员变更等事实记录。
- 现有公权机构统一在创世阶段上链。OnChina 不在运行期生成既有公权机构目录，只读取链上机构和链上 active admins，并用本地 `subjects/accounts/admins` 投影补齐展示字段。
- 注册局工作台必须保持既有功能和 UI；私权机构不得看到注册局公民、机构目录和登记入口。新增司法院、立法院、学校、公司、公益组织等机构 UI 时，只能新增或扩展对应工作台目录，不得把机构差异重新塞回 `frontend/App.tsx`。
- 平台会员价格是准确 CID 实例模块：只有登录机构 CID 等于 finalized `SquarePost::PlatformCidNumber` 时，清单才包含 `platform_membership_price`。OnChina 只读取 finalized 价格，不在 PostgreSQL 保存第二份价格。

## 4. 行政区和 CID 号真源

- 国家码、省级行政区码和机构码常量唯一真源：`citizenchain/runtime/primitives/cid/code.rs`。
- 市、镇和地址段开发真源：`citizenchain/onchina/src/cid/china/china.sqlite`。
- CID 号生成和校验唯一源码目录：`citizenchain/onchina/src/cid/`。

生产环境中 `ONCHINA_CHINA_DB` 固定指向随包只读 SQLite。市镇地址段变更只能修改开发库并重新发布安装包，禁止运行期在线编辑行政区。

## 5. 结构化表

- `ids(cid_number, kind, province_code, city_code)`：全局身份 ID 索引。
- `subjects`：主体公共展示字段，按省分区；机构行缓存 `cid_full_name/cid_short_name`、行政区、业务状态、私权分类，以及链上公开法定代表人的 `family_name/given_name/cid_number/legal_representative_account_id`。法定代表人照片只属于 OnChina 链下资料，不进入 `InstitutionInfo`；数据库不保存拼接姓名列。
- `citizens`：公民档案、姓、名、身份 CID、护照号、可空 `account_id`、`province_code/city_code/town_code` 居住/办理地、`birth_*` 出生地和电子护照有效期字段，按省分区。数据库不保存 SS58；需要展示时由 `account_id` 派生 `ss58_address`。
- `citizen_documents`：公民独立资料库元数据,资料类型固定为“护照相片 / 出生证明 / 监护人护照 / 其他材料”,文件本体在磁盘；不得与机构 `docs` 共表。
- `gov`：公权机构扩展字段，按省分区；链上投影写 `source='CHAIN'`，人工公权机构写 `source='MANUAL'`。
- `private`：私权机构扩展字段，按省分区；分类字段使用 `private_type/partnership_kind/has_legal_personality`。
- `accounts`：机构账户，主键按 `(province_code, cid_number, account_name)` 收敛。
- `docs`：机构资料库元数据，文件本体在磁盘。
- `audit`：结构化审计记录，按省分区。
- `admins`：机构管理员本地元数据缓存；账户列统一为 `account_id`，创建来源账户为 `creator_account_id`。所有公权、私权机构的链上管理员项统一为 `account_id + cid_number + family_name + given_name`；本地缓存不是成员资格、公民身份或岗位权限真源。
- `chain_sign_sessions`：公民/机构链交易的短期签名会话，只保存 `actor_public_key`、签名 payload 和链上成功后写正式投影所需的上下文；它不是业务草稿，不参与 CID/名称占用，submit 成功或失败后都必须删除。
- `node_institution_bindings`、`node_binding_challenges`：本节点首次登录绑定机构和绑定确认挑战；绑定表使用 `bound_account_id`，挑战使用 `account_id`，并只保存链上身份键，禁止保存机构名称和省市镇权限派生值。解绑 / 换机构由 `NODE_BINDING_UNBIND` 冷签动作停用 active binding。
- `admin_sessions`：会话以 `account_id` 保存账户身份，并保存签发时的 `candidate_id`；每次鉴权与当前 active binding 严格比对，解绑、重绑或候选不一致时立即删除会话，不允许回落。
- `admin_login_sign_requests`、`admin_qr_login_results`、`admin_action_challenges`、`admin_security_grants`：登录和扫码签名运行态。管理员登录必须先扫描完整 `QR_V1/k=3 user_contact` 用户码，由后端从 `b.ss58_address` 派生规范 `account_id`、先查链上管理员名册，再生成 `QR_V1/k=1,a=1` 定向请求；`b.u` 必须是该账户公钥且数据库 `account_id` 不得为空。签名响应只能证明持有该目标账户私钥，不得改写目标账户。
- `chain_requests`、`chain_nonces`、`tx_records`、`tx_indexer_state`：链路幂等、防重放和索引运行态；交易发送方、接收方固定使用 `sender_account_id/recipient_account_id`。

`cid_number` 是唯一且不可变的身份标识。不得新增 `identity_key`、`generation_key` 等第二身份键。

旧机构直接创建入口已关闭：OnChina 当前创建 API 固定返回 501，前端创建按钮固定禁用，不生成 `0x1e05/0x1f05` 或旧签名会话。后续如恢复机构登记，公权与私权机构管理员必须统一使用 `Admin { account_id, cid_number, family_name, given_name }`，机构治理阈值必须作为 entity 机构配置独立提交，不能由管理员人数或岗位数推导；具体登记规则需另立方案确认。

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
- `citizens(province_code, city_code, cid_number, passport_no, account_id)`
- `citizens(province_code, city_code, town_code, created_at DESC, id DESC)`
- `citizen_documents(province_code, cid_number, uploaded_at DESC, id DESC)`
- `gov(province_code, city_code, town_code, institution_code)`
- `private(province_code, city_code, private_type, cid_number)`
- `accounts(province_code, cid_number)`
- `docs(province_code, cid_number, uploaded_at DESC)`
- `audit(province_code, city_code, created_at DESC)`
- `admins(institution_code, city_name)`
- `admins(account_id)`（严格规范文本，不做 `lower(...)` 兼容）

## 7. 管理员和安全

管理员唯一字段统一为 `admins`。OnChina 不恢复独立管理员身份表、授权真源或授权分支。

- 登录态与本地缓存保留 `account_id`、公民 CID、姓、名展示字段；链上机构管理员解码无论来自公权还是私权 pallet，都统一为 `account_id + cid_number + family_name + given_name`。数据库不建立第二套公民 CID 或授权真源。
- 机构管理员列表联合读取链上 `admins` 人员集合与 entity 机构岗位任职；没有岗位的管理员仍必须返回，但管理员账户本身不具备机构业务权限。本地联系方式、照片和 Passkey 仅是私密资料，不得成为管理员资格或岗位真源。
- 联邦注册局管理员目录从 `PublicAdmins::AdminAccounts` 读取账户集合，并从 `PublicManage::InstitutionRoleAssignments` 的 `PROVINCE_COMMISSIONER_<省码>` 岗位取得 43 省归属；本地不保存第二份省组权限真源。
- FRG 管理员的 `scope_province_name` 只由其链上省岗位码派生；FRG 机构 CID 的登记地址只作机构展示元数据，禁止覆盖管理员授权省份。
- 联邦注册局和市注册局不提供“编辑本地管理员姓名”的入口。联邦注册局岗位任职目录完全只读，换届由治理业务写入 entity；下级市注册局本地登记目录新增/删除仍走安全动作，但不能替代链上管理员资格校验。
- 登录态：用于普通读取和低风险操作。
- `PASSKEY_COLD_SIGN`：用于管理员安全写操作、Passkey 更新、管理员集合变更、节点解绑和链写入二次确认。
- 扫码请求：统一使用 `QR_V1 / k=1 sign_request`。
- 签名响应：统一使用 `QR_V1 / k=2 sign_response`。

业务模块只传入动作码、签名原文、摘要和展示字段，不得自己包装二维码协议。

链交易提交必须先走 `system_dryRun` 预检。预检返回 `InvalidTransaction`、RPC
`RuntimeApi` 错误或 wasm trap 时，OnChina 直接返回“交易未提交”的失败结果，禁止继续调用
`author_submitExtrinsic`。这样浏览器只看到一次明确失败，不会把 runtime 校验崩溃再次透传成提交阶段错误。

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
- 修改节点桌面端 OnChina 启动入口：同步检查 `citizenchain/node/src/onchina_proc.rs`、`citizenchain/node/src/settings/onchina_platform.rs` 和节点设置页，确认 OnChina 不随节点默认启动。
- 涉及 QR、签名、链交易载荷、CID 号格式时：必须同步更新 `memory/07-ai/unified-protocols.md` 和相关端实现。

## 10. 验收口径

涉及 API、数据库、登录、权限、扫码或页面展示的任务，必须使用真实本地服务、真实 PostgreSQL、真实 HTTP 接口或真实页面验收。只通过编译、类型检查或前端 build 不算完成。

账户标识统一第 4 步已在 2026-07-23 完成真实验收：从当前 Runtime 源码生成的
`citizenchain.compact.compressed.wasm` SHA-256 为
`6fdd4cf2f7b5b884a63c680ecde5fd4dada73ea7df3e816b9b115129b68afcbb`，隔离
`citizenchain-fresh` 创世块为
`0x49f1da82260414adbfb72ce085d8520dbf56d1413b60f583af7722955e877458`。
真实 PostgreSQL、HTTPS OnChina 和索引器完成 49,593 个机构、99,231 个机构账户投影；
规范 `account_id` 可进入当前流程，大写、无前缀、SS58、旧 JSON 字段和旧路由被拒绝。
使用开发密钥生成的有效 sr25519 签名通过验签后，非链上管理员仍被链上管理员门禁拒绝，
证明签名身份不会绕过链上授权。验收后业务数据库再次清空重建，OnChina、节点和
PostgreSQL 均已停止，端口 `8964`、`9944`、`5433` 关闭。

最低检查：

```text
rg "旧独立身份系统名" memory AGENTS.md citizenchain/onchina --glob '!memory/08-tasks/**' --glob '!memory/04-decisions/**'
cargo check --manifest-path citizenchain/Cargo.toml -p onchina
npm --prefix citizenchain/onchina/frontend run build
```

如果工作区存在其它线程的未完成改动，验收必须说明受影响的命令和原因，不得把其它线程的失败混入本任务结论。
