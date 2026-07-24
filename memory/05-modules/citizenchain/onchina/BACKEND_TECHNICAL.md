# OnChina 后端技术文档

## 1. 功能需求

OnChina 后端负责多机构工作台、管理员身份、行政区、机构、公民、管理员、扫码签名、公开查询和链侧凭证。它运行在 `citizenchain/onchina/src/`，属于公民链产品内部能力。

## 2. 当前结构

```text
citizenchain/onchina/src/
├── main.rs                    # Axum 路由、AppState、StoreHandle 和后端入口
├── auth/                      # 管理员登录、安全动作、passkey 和会话鉴权
│   └── login/                 # 管理员登录、扫码登录、鉴权守卫和签名校验
├── cid/                       # CID 号编码、机构码、生成、校验和行政区 SQLite
│   └── china/                 # 中国行政区划 SQLite 真源
├── citizenapp/                # CitizenApp 查询和公民侧 BFF
├── core/                      # HTTP、安全、运行期工具、chain_* 和 QR 协议辅助
│   └── qr/                    # QR_V1 协议辅助和统一 sign_request 构造
├── crypto/                    # sr25519、公钥规范化和哈希辅助
├── domains/                   # 公权、私权、公民、资料库、地址等业务域
│   ├── address/               # 镇下地址库查询和 AddressRegistry call data 构造
│   ├── citizens/              # 公民档案、护照号和投票凭证
│   ├── docs/                  # 机构资料库入口
│   ├── gov/                   # 公权机构链上投影、链目录验收和公权机构接口
│   └── private/               # 私权机构入口和六类私权机构子模块
├── institution/               # 机构账户、机构管理员元数据和主体共享内核
│   ├── accounts/              # 机构账户入口
│   ├── admins/                # 本地管理员元数据缓存
│   └── subjects/              # 主体共享模型、注册内核、详情和非法人能力
├── indexer/                   # 链事件解析与索引 worker
├── platform/                  # 控制台能力、mDNS、TLS CA 和平台健康检查
├── scope/                     # 省/市可见范围与过滤规则
├── store/                     # Store 聚合体和结构化存储边界
└── workspace/                 # 机构工作台类型、三段式分区和登录态工作台清单
```

## 3. 目录铁律

- 禁止恢复旧独立身份系统产品目录。
- 禁止恢复旧 registry 目录。
- 禁止恢复 `backend/src/` 源码壳。
- 禁止恢复独立 `chain` 业务目录；链交互只能放在所属业务模块的 `chain_*.rs` 或 `core/chain_*`。
- 禁止恢复独立 `cid_number`、`models`、`login`、`qr` 等历史目录壳。
- `scope/` 只放权限范围规则，不放 HTTP handler 或公钥工具。
- 非法人机构能力统一归 `institution/subjects/unincorporated_org/`，不得放在单侧 `domains/gov/` 或 `domains/private/`。
- 机构工作台统一归 `workspace/`。`workspace` 只生成登录态可渲染清单，不保存管理员授权真源，不承载业务 handler。

## 4. Store 和表边界

后端只承认结构化 PostgreSQL 表为主数据。`store/` 可以封装访问和短期缓存，但不得保存第二份业务主数据。

- 机构主写入只进入 `institution/subjects`、`domains/gov`、`domains/private`、`institution/accounts` 和 `domains/docs`。
- 公民主写入只进入 `domains/citizens`、`subjects`、`citizens`、`citizen_documents`、`passport_numbers` 和 `sequence_counters`。
- 管理员写入只进入 `admins`（本地展示元数据）和短生命周期安全运行态表；管理员字段固定为 `admin_account + family_name + given_name`，成员资格与岗位范围只来自链上，禁止建立本地管理员授权范围表。
- 旧机构直接创建 API 当前固定返回 501，不写 `chain_sign_sessions`、机构业务草稿、占用表或本地机构投影。第 6 步的新业务模块必须原子提交 CID 基础资料、admins、LR、初始治理岗位/权限/任职/投票规则和协议账户后，才能重新开放创建。
- 创建机构和创建公民只有两种业务结果：链上确认成功后写正式投影；未链上确认就是失败，并删除对应短期签名会话，不保留名称、CID、管理员或公民档案占用。
- 公权机构唯一真源是链上 `PublicManage`;`subjects/gov/accounts` 中的公权行只是本地查询投影,投影版本只记录在 `chain_projection_state`。
- 链上状态字段只作本地投影缓存(`subjects.chain_status`、`accounts.chain_status`),不得成为第二授权真源。
- `node_institution_bindings` 只保存本节点当前绑定的链上身份键：`candidate_id / institution_code / institution_cid_number / frg_province_code`。FRG 绑定始终是一个 FRG 机构身份，不得拆成虚拟省组身份；省级办理范围来自管理员钱包在 entity 中有效的 `PROVINCE_COMMISSIONER_<省码>` 任职，机构 CID/全称/简称/主账户来自 FRG 主体投影且只作身份与展示。绑定表不得保存名称或省市镇权限派生值。
- `admin_sessions.candidate_id` 必须与 active binding 严格一致；旧会话、解绑后会话、重绑前会话和候选不一致会话一律失效，不存在兼容回落。
- 审计写入统一走结构化审计入口，详情字段只保存事实，不保存 UI 文案。

### 4.1 公权机构链投影

- 显式 `sync-gov` 必须从链上 `PublicManage::Institutions` 与 `PublicManage::InstitutionAccounts` 全量读取,再写入本地 `subjects/gov/accounts` 投影。
- `serve` 启动时先读取链 `genesis_hash` 与 finalized head,再比对 `chain_projection_state(public-gov)` 的 `chain_genesis_hash / chain_block_hash / chain_block_number / item_count / account_count`;一致则直接启动并跳过全量同步,不一致或无投影才全量同步。链不可达、锚点无法确认或同步失败时 fail-closed。
- OnChina 不得在启动时从 `china.sqlite` 重新生成公权机构；`china.sqlite` 只提供行政区名称和镇级索引校验/展示。
- 投影状态写入 `chain_projection_state(projection_key='public-gov')`;旧 `gov_manifest`、`ensure-gov`、`reconcile-gov`、`check-gov` 均不得恢复。
- 普通列表、联邦注册局详情和本机构显示页只能读取 `gov.source='CHAIN'` 的公权投影；本地手工/pending 行不能冒充链上公权机构真源。
- `audit-chain-catalog` 只做创世链目录验收,不得用本地派生结果灌库。
- CitizenApp 公权机构接口只读取链上投影并下发 `chain_genesis_hash / chain_block_hash / chain_block_number / synced_at` 作为同步锚点;`manifest_version` 由 genesis hash + finalized block hash/number + 投影数量组成,不得使用本地 `synced_at` 单独推进版本,也不得把 OnChina PostgreSQL 当成公权机构真源。
- `PublicManage::InstitutionInfo` 按当前 runtime 精确字段序解码；机构存在即表示 active，
  不得在 OnChina 追加已删除的 lifecycle/status 尾字段，也不得用兼容分支吞掉尾随字节。
- 2026-07-16 创世准备验收使用 preview 块 0 的真实 node 和全新临时 PostgreSQL：启动投影
  49,593 个机构、99,231 个账户，33 项创世目录抽样对账通过，`/api/v1/health` 返回
  `UP`，公权目录版本锚定同一 genesis/block#0，前端首页真实返回“链上中国平台”。该
  preview 不替代正式冻结锚点，验收结束后节点、OnChina、PostgreSQL 与临时目录均已清理。

## 5. 公民录入和护照号

- 公民由注册局管理员在 OnChina 当前办理城市下一次交易录入,不再由前端手填 `cid_number`。
- 联邦注册局管理员必须先选择分管省内城市后才能录入公民;市注册局管理员直接锁定本市。
- 公民姓名统一为 `family_name` 和 `given_name`；展示姓名时由前端按中文顺序组合，数据库不保留拼接姓名或带 `citizen_` 前缀的别名列。
- 公民身份 CID 由 `src/cid/generator.rs` 生成,机构代码固定为 `CTZN`,个人码 R5 市段固定为 `000`。
- 护照号由 `src/domains/citizens/passport_no.rs` 生成,OnChina 自持完整算法。
- 创建公民不得要求 `wallet_account`;未成年人或暂未开户公民可以先建立本地电子护照档案。
- 推送链上公民身份时才录入 `wallet_account`;后端接受 SS58 地址或 0x 公钥,解析为内部 `wallet_pubkey`。请求必须显式提供 `identity_level=voting/candidate`：投票身份要求该钱包签 `VotingIdentityPayload`，参选身份要求该钱包签 `CandidateIdentityPayload`。
- 未满 16 周岁不得推送链上公民身份。OnChina 在生成签名二维码前校验年龄,runtime `citizen-identity` 在 `register_voting_identity / update_voting_identity / upgrade_to_candidate_identity` 再次校验 `citizen_age_years >= 16`。
- 出生省市镇必填,字段为 `birth_province_code / birth_city_code / birth_town_code`;创建后不得被普通编辑流程修改。
- 居住/办理行政区直接使用链上中国统一行政区字段 `province_code / city_code / town_code`;前端只允许在当前办理城市下选择 `town_code`,不得恢复旧的第二套居住字段。
- 护照有效期自动计算:创建时年满 16 周岁为 10 年,未满 16 周岁为 5 年,字段为 `passport_valid_from / passport_valid_until`。
- `citizens` 表当前字段只表达公民档案、身份 CID、护照号、可为空的钱包地址、出生地、居住地、护照有效期和投票资格。
- 公民资料库独立使用 `citizen_documents` 表和 `/api/v1/admin/citizens/:cid_number/documents` 接口,不得复用机构 `docs` 表或 `domains/docs` 逻辑。资料类型固定为“护照相片 / 出生证明 / 监护人护照 / 其他材料”,文件本体写入磁盘,表内只保存元数据和内容哈希。
- `passport_numbers` 是护照号全局索引表;`passport_number_recycle_pool` 只保存可回收护照号,不得保存旧公民个人资料。

## 6. 链交互边界

链交互按业务归属放置：

- 机构注册信息凭证、账户列表 DTO 和 handler：`institution/subjects/chain_*.rs`
- 投票资格提示查询：`domains/citizens/chain_vote.rs`
- 公民链上身份推送：`domains/citizens/chain_identity.rs`
  - `POST /api/v1/admin/citizens/:cid_number/onchain/prepare` 只消费一次 Passkey，建立 180 秒 `citizen_onchain_operations` 操作并生成 `a=2 citizen_identity` 签名请求；请求体必须包含 `wallet_account` 和 `identity_level`。
  - `identity_level=voting` 编码 `VotingIdentityPayload`，完成后生成 `0x0a00 register_voting_identity` 注册局管理员链上签名二维码。
  - `identity_level=candidate` 编码 `CandidateIdentityPayload`，完成后生成 `0x0a01 upgrade_to_candidate_identity` 注册局管理员链上签名二维码；该交易同时写入投票身份和参选身份。
  - `POST /api/v1/admin/citizens/:cid_number/onchain/complete` 不再二次认证；它按签名响应 `id` 校验管理员、机构、CID、钱包、身份级别和完整 payload，原子消费操作后生成管理员最终链签二维码。钱包绑定和上链投影只在最终链交易确认后一次性落库。
- 联合投票本地人数查询：`domains/citizens/chain_joint_vote.rs`
- 地址变更调用：`domains/address/chain_call.rs`
- 立法法律只读链读：`domains/legislation/law/chain_read.rs` 负责读取 `Law`、`LawVersion`、`LawVersionLabels` 和宪法不可修改条款 manifest；`LawView.version_title/version_title_en` 只能来自链上 `LawVersionLabels[(law_id, version)]`。
- 立法提案写入：`domains/legislation/law/chain_propose.rs` 的新法、修法、废法载荷必须在 `actor_cid_number` 后紧接必填 `proposer_role_code`。后端只做 1..64 字节和 SCALE 顺序校验，真正发起权限由 runtime 按完整岗位主体校验；不得从管理员登录态推导岗位码或恢复无岗位码旧载荷。
- `domains/legislation/chain_read_proposal.rs` 必须按当前核心 `Proposal` 布局解码；人口分母已独立存于投票引擎 `ProposalPopulationSnapshots`，不得在 Proposal 镜像恢复 `citizen_eligible_total` 尾字段。
- 通用 SCALE、genesis hash、RPC URL 和交易提交辅助：`core/chain_*.rs`

业务模块不得新增全局链目录，不得在 handler 内手写 pallet/call 字节或二维码动作码。动作码、payload、签名/验签规则以 `memory/07-ai/unified-protocols.md` 为唯一登记入口。

PublicManage/PrivateManage call 5 及 `0x1e05/0x1f05` 已永久留洞，后端不得继续编码旧 `cid_number + cid_full_name + cid_short_name + town_code + admins + actor_cid_number` 载荷。新创建协议由第 6 步独立业务模块另行登记；必须携带注册局完整 `RoleSubject`，并原子绑定目标机构的岗位、权限、任职和投票规则，不能只凭注册局 admins 或在创建后补权限。

## 7. HTTPS 和机构 CA

正式入口固定为 `https://onchina.local:8964`。OnChina 启动时在 `ONCHINA_TLS_DIR` 生成并持久化本机构节点私有 CA：

- `onchina-org-root-ca.crt`：员工浏览器可下载和安装的 CA 公钥证书。
- `onchina-org-root-ca.key`：仅保存在节点服务器本地的 CA 私钥，禁止通过 HTTP、日志或前端接口暴露。
- `onchina-server.crt` / `onchina-server.key`：由本机构 CA 签发的 `onchina.local` 服务证书。
- `onchina-cert-profile.txt`：证书策略标记；旧超长期默认有效期证书没有该标记，下次启动必须自动重建。

CA 有效期固定到 2036-01-01；服务证书每次 OnChina 启动时用当前 CA 重新签发，有效期 397 天以内，避免 macOS / Safari / Chrome 拒绝超长 TLS 服务证书。

未登录公共接口 `/api/v1/platform/ca-certificate` 只返回 CA 公钥证书 PEM，用于员工首次访问时下载并导入浏览器/系统受信任根证书；`/api/v1/platform/ca-certificate/info` 只返回文件名、证书主题、SHA-256 指纹和有效期展示信息。

## 8. 错误码和提示边界

后端统一通过 `ApiError.error_code` 暴露稳定业务错误码。HTTP `401` 只表示管理员登录态无效；公民档案不存在、账户不匹配、签名失败等业务错误不得返回 `401`。

注册局机构主账户缺失必须返回稳定错误码 `ONCHINA_REGISTRY_MAIN_ACCOUNT_MISSING`;正常目标态下该错误只能在链投影异常或绑定自愈失败时出现,不得再被前端降级成通用“请求内容不正确”。

数据库错误必须展开 PostgreSQL SQLSTATE、message、detail 和 hint，禁止只向前端或日志传 `db error`。

## 9. 管理员写操作

市注册局本地目录维护、Passkey 更新、节点解绑和链写动作必须使用相应安全档。业务 handler 只负责构造业务动作，二维码协议包装和签名结果识别归 `core/qr/`。

公民身份上链(`CITIZEN_ONCHAIN_PUSH`)固定为一次业务操作：管理员 Passkey 一次、目标公民钱包签名一次、管理员最终链交易签名一次。最终链签已经承担管理员钱包授权，不得再叠加安全 grant 冷签；`complete` 依靠一次性 `citizen_onchain_operations` 防串单、防过期和防重放。

联邦注册局机构 `admins` 和岗位任职不得本地直接改库；换届只能构造链上治理或注册局登记动作后由 entity 写入。市注册局本地登记目录每省每市最多 30 人，统计必须同时带省和市，但该目录不是链上管理员资格真源。NJD、普通公权机构、私权机构和非法人组织的本机构管理员/岗位维护也必须走链上 `propose_institution_governance`，不得在 OnChina 内建立第二套管理员集合。

`INSTITUTION_CREATE` 旧扫码授权链路当前不允许进入 prepare/submit；创建 handler 在鉴权后固定返回 501，不能生成安全 grant、链签会话或旧 call data。第 6 步启用新业务时必须重新登记完整原子载荷和权限主体，不能沿用本段已删除字段集。

`PASSKEY_COLD_SIGN` 正式提交的安全门统一在 `auth/actions.rs::require_admin_security_grant`：先消费 `X-Passkey-Assertion`，再消费 `x-cid-security-grant`，任一缺失、过期、归属不匹配或 payload hash 不匹配都 fail-closed，不允许降级为 SESSION 或只验冷签 grant。机构资料上传、资料删除、机构详情更新等链下写操作虽然不直接提交链交易，也必须按各自后端 `grant_payload` 逐字段绑定授权：上传资料为 `target/file_name/doc_type/file_size`，删除资料为 `target/doc_id/file_name`，机构详情更新为 `target/cid_number/cid_full_name/parent_cid_number/family_name/given_name/legal_representative_cid_number/legal_representative_photo_path`。

机构管理员列表 API 联合读取链上 `admins(admin_account + family_name + given_name)` 人员集合与 entity 岗位、`InstitutionRolePermissions` 和有效任职。`institution/admins/chain_roles.rs` 负责公权/私权岗位路由、任职合并和 FRG 省专员范围解析；管理员即使没有岗位也必须保留人员行，姓名只展示。本地联系方式、照片和 Passkey 不得成为管理员资格或岗位真源；业务授权必须由完整 `RoleSubject + BusinessActionId + operation` 查询，不按账户或前端标签推断。

链上机构唯一查询先读取 `PublicManage::Institutions[cid_number]`，未命中再读取 `PrivateManage::Institutions[cid_number]`，不建立本地分流真源；公私权 CID 不重复由 runtime 与 NodeGuard 的链上不变式保证。创世公权目录全量投影仍精确为 49,593 个机构和 99,231 个协议账户；非营利法人“公民链技术发展基金会” `GZ018-SFGYR-201206100-2026` 属私权创世机构，只参加独立私权存在性审计，不冒充公权目录行，也不增加公权投影计数。启动抽样当前覆盖 32 个派生公权机构、1 个公权常量机构和该基金会，共 34 项。

链上机构管理员无论来自 `PublicAdmins` 还是 `PrivateAdmins`，都统一解码为 `account_id + cid_number + family_name + given_name`。非空 CID 的最终绑定由 runtime 向 `citizen-identity` 校验，OnChina 本地公民库只能用于输入补全和展示，不能变成链上授权真源。

机构治理链写入口：

- `POST /api/v1/admin/institution/governance/prepare`：请求必须独立携带必填 `proposer_role_code`；后端只接受当前节点绑定机构 CID，按 `cid_number + action + register_nonce + signature + actor_cid_number + proposer_role_code + credential_signer_pubkey + scope` 的 runtime 顺序构造签名载荷并写入 `chain_sign_sessions`。管理员登录态不产生业务权限，最终由 runtime 校验完整岗位主体。管理员集合、岗位、任职和法定代表人任命/更换/解除都只进入链上 call data，不写本地正式投影；解除时提交 `clear_legal_representative=true`，不得同时提交 `legal_representative_cid_number`。
- `POST /api/v1/admin/institution/admins/register/prepare`：注册局管理员发起 `register_institution_admins`，目标机构 CID 从请求读取，actor CID 只来自当前节点绑定注册局 CID。
- 提交阶段复用统一链签会话 submit。机构治理 purpose 进块后只记录审计；OnChina 读侧继续读取链上 `admins / InstitutionRoles / InstitutionRoleAssignments`，禁止在提交成功后本地直接改管理员或岗位真源。
- 创建动态岗位时前端不得提交岗位码；runtime 使用所属 pallet 的 `MODULE_TAG`、CID、单调 nonce 和真实 proposal_id 生成 `R_<32 位大写十六进制>`，删除后永久不复用。
- 法定代表人治理使用 runtime `InstitutionLegalRepresentativeChange::Set/Clear`，任命/更换时三字段同时写入，解除时三字段同时清空。

## 10. 机构工作台能力映射

工作台类型和工作台入口由 `src/workspace/` 生成，底层能力位由 `src/platform/capability.rs` 单源下发给前端。runtime 已经实现 FRG 省级组登记权高于 CREG 本市登记权，OnChina 能力表必须只镜像这个目标状态，不能另行降权：

- `FRG` 是 Tier1 创世注册局，进入 `registry` 工作台，能力必须是 `CREG` 的超集：可进入公民、私权、教育、公权机构、市注册局和联邦注册局，并可在本省范围内登记机构、写业务、维护市注册局管理员、维护本省联邦注册局管理员。
- `CREG` 是 Tier2 下级注册局，进入 `registry` 工作台，保留本市公民/机构/业务写入能力；同时必须能进入“联邦注册局”入口，只读查看本省联邦注册局管理员列表，不得发起联邦注册局管理员编辑或更换。
- `NJD` 进入 `judicial` 工作台，不复用注册局 UI。当前工作台按 `operations / display / records` 分类：显示页可只读查看本机构信息和链上 active admin 列表；管理员变更和岗位治理入口必须构造 `propose_institution_governance` 链动作，护宪终审按专属业务能力接入。
- 立法机构进入 `legislation` 工作台或通用工作台的立法入口，立法能力由 `domains/legislation/category.rs` 和 `can_*_legislation` 位决定。
- 私权机构进入 `private` 工作台，只下发本机构信息、链上 active admin 与准确 CID 授权模块；普通公权、立法和非法人机构分别使用 `public`、`legislation`、`unincorporated` 工作台种类，可共用通用显示壳但不得恢复 `generic` 权限语义。
- 登录、扫码登录轮询、鉴权检查和工作台返回统一携带 active binding 的准确 `institution_cid_number`；后端未能解析准确 CID 时 fail-closed，前端不得根据 `institution_code` 猜测。
- 平台会员模块只在准确 CID 等于同一 finalized 区块的 `SquarePost::PlatformCidNumber` 时下发。`domains/membership/` 只读取 finalized 价格、构造 `propose_set_platform_price` 和校验链上 `admins`，不保存价格、不实现投票。
- 所有链交易签名响应统一提交到 `POST /api/v1/admin/chain/submit`；业务域只 prepare，不得新建第二套 submit handler。平台调价 prepare 必须携带 `proposer_role_code`，按 `actor_cid_number + proposer_role_code + membership_level + new_price_fen` 编码；prepare/submit 复核节点绑定和准确平台 CID，业务授权最终只认链上岗位任职与 `sqr-sub/5` 权限，不把 active admins 当授权真源。
- `NRC`、`PRC`、`PRB` 走节点桌面端，不获得 OnChina 网页能力。
- `PMUL` 和其它个人主体不获得 OnChina 网页能力。
- 前端工作台展示只使用后端下发的 `workspace` 和 `capabilities`；后端 handler、scope 和链上 active admin 校验仍是安全边界。

### 10.1 本机构只读接口

- `GET /api/v1/admin/own-institution` 返回当前 active binding 对应机构的 `InstitutionDetailOutput`，用于非注册局工作台“显示”页。
- 本接口不接受前端传入 `cid_number`；后端只从当前节点 active binding 的 `institution_cid_number` 定位本机构，避免变成任意机构详情读取入口。
- 返回数据仍来自结构化 `subjects/accounts` 投影；管理员资格由登录守卫、节点绑定和链上 active admins 校验决定。
- `GET /api/v1/admin/own-institution-admins` 目标返回链上 active `admins` 账户与 entity 有效机构岗位任职的联合结果。

### 10.2 CitizenApp 公权机构只读接口

- `GET /api/v1/app/public-institutions` 提供匿名只读公权机构链上投影分页;请求字段为 `province_name / city_name / since_version / after_cid / page_size`。
- `GET /api/v1/app/public-institutions/version` 返回当前 scope 的 `manifest_version / chain_genesis_hash / chain_block_hash / chain_block_number / synced_at / count`。
- `manifest_version` 由 `chain_projection_state(public-gov)` 的 `chain_genesis_hash / chain_block_hash / chain_block_number / item_count / account_count` 生成,只作为 CitizenApp 本地缓存游标;链上 `PublicManage` 仍是唯一真源。
- 接口只下发行政区 code,不下发行政区名称副本;CitizenApp 通过内置行政区字典按 `province_code / city_code / town_code` join 名称。
- 接口不得读取 `china.sqlite` 运行态派生公权机构,也不得把本地 `subjects/gov/accounts` 投影作为授权真源。

## 11. 验收

2026-07-17 机构治理运行态补验：当前源码 `citizenchain-fresh --tmp` 使用 `WASM_BUILD_FROM_SOURCE=1` 构建后启动成功，OnChina 使用临时内嵌 PostgreSQL 和 `ONCHAIN_WS_URL=ws://127.0.0.1:19944` 连接 fresh 链启动成功；启动期完成公权链投影 `49,593` 个机构与 `99,231` 个账户，首页 HTTP 返回 200，`subjects` 表旧 `legal_rep_*` 列为 0，新 `legal_representative_*` 三字段列齐备。交互式 CitizenWallet 扫码签名需要真实管理员登录会话和扫码设备，本次仅完成链、数据库、服务和页面基础运行态，不伪造扫码签名结果。

正式创世前曾使用管理员三字段布局；该布局现已全部废弃。OnChina 当前对公权、私权机构统一按四字段 `Admin` 解码与治理/登记编码，不兼容任何旧三字段机构布局；个人多签同样使用统一四字段 SCALE 结构。

2026-07-19 私权创世公民链基金会第 6 步验收：`institution_lookup` 已实现在相同 CID 主键下依次读取 `PublicManage` 和 `PrivateManage`，公权全量目录迭代继续只读取公权 storage；启动抽样固定增加公民链基金会，全量公权审计先独立核验基金会存在，再执行 49,593 个公权机构的双向比对。OnChina 137 项测试通过；没有把基金会复制进本地公权投影、没有读取本地公民数据库生成法定代表人，也没有新增第二套基金会身份常量。

```text
rg "mod chain;|crate::chain|chain::" citizenchain/onchina/src -g '*.rs'
cargo check --manifest-path citizenchain/Cargo.toml -p onchina
ONCHINA_EMBEDDED_PG=0 DATABASE_URL=<local_pg> ONCHAIN_WS_URL=<chain_ws> cargo run --manifest-path citizenchain/Cargo.toml -p onchina -- sync-gov
curl -kfsS https://onchina.local:8964/api/v1/health
curl -kfsS https://onchina.local:8964/api/v1/platform/ca-certificate/info
curl -kfsS -o /tmp/onchina-org-root-ca.crt https://onchina.local:8964/api/v1/platform/ca-certificate
curl -ksS -i https://onchina.local:8964/api/v1/admin/auth/check -H "authorization: Bearer <token>"
```

涉及数据库、登录、管理员列表、机构详情和扫码签名的变更必须跑真实 HTTP 接口。只通过 `cargo check` 不能证明连接池、SQL 字段顺序和扫码验签流程正确。
