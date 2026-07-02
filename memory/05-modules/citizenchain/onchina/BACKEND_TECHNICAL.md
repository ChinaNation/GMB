# OnChina 后端技术文档

## 1. 功能需求

OnChina 后端负责多机构管理员身份、行政区、机构、公民、管理员、扫码签名、公开查询和链侧凭证。它运行在 `citizenchain/onchina/src/`，属于公民链产品内部能力。

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
│   ├── gov/                   # 公权机构确定性目录和公权机构接口
│   └── private/               # 私权机构入口和六类私权机构子模块
├── institution/               # 机构账户、机构管理员元数据和主体共享内核
│   ├── accounts/              # 机构账户入口
│   ├── admins/                # 本地管理员元数据缓存
│   └── subjects/              # 主体共享模型、注册内核、详情和非法人能力
├── indexer/                   # 链事件解析与索引 worker
├── platform/                  # 控制台能力、mDNS、TLS CA 和平台健康检查
├── scope/                     # 省/市可见范围与过滤规则
└── store/                     # Store 聚合体和结构化存储边界
```

## 3. 目录铁律

- 禁止恢复旧独立身份系统产品目录。
- 禁止恢复旧 registry 目录。
- 禁止恢复 `backend/src/` 源码壳。
- 禁止恢复独立 `chain` 业务目录；链交互只能放在所属业务模块的 `chain_*.rs` 或 `core/chain_*`。
- 禁止恢复独立 `cid_number`、`models`、`login`、`qr` 等历史目录壳。
- `scope/` 只放权限范围规则，不放 HTTP handler 或公钥工具。
- 非法人机构能力统一归 `institution/subjects/unincorporated_org/`，不得放在单侧 `domains/gov/` 或 `domains/private/`。

## 4. Store 和表边界

后端只承认结构化 PostgreSQL 表为主数据。`store/` 可以封装访问和短期缓存，但不得保存第二份业务主数据。

- 机构主写入只进入 `institution/subjects`、`domains/gov`、`domains/private`、`institution/accounts` 和 `domains/docs`。
- 公民主写入只进入 `domains/citizens`、`subjects`、`citizens`、`citizen_documents`、`passport_numbers` 和 `sequence_counters`。
- 管理员写入只进入 `admins`(本地元数据缓存)和短生命周期安全运行态表;成员资格真源在链上(`federal_registry_scope` 表已退役,见 [[project_onchina_registry_tier_chainread_2026_06_29]])。
- 链上状态只属于 `accounts.chain_status`，机构主体本身不保存链上状态。
- 审计写入统一走结构化审计入口，详情字段只保存事实，不保存 UI 文案。

## 5. 公民录入和护照号

- 公民由注册局管理员在 OnChina 当前办理城市下一次交易录入,不再由前端手填 `cid_number`。
- 联邦注册局管理员必须先选择分管省内城市后才能录入公民;市注册局管理员直接锁定本市。
- 公民姓名拆为 `citizen_family_name` 和 `citizen_given_name`;展示姓名时由前端按中文顺序组合,数据库不保留姓名单字段。
- 公民身份 CID 由 `src/cid/generator.rs` 生成,机构代码固定为 `CTZN`,个人码 R5 市段固定为 `000`。
- 护照号由 `src/domains/citizens/passport_no.rs` 生成,OnChina 自持完整算法。
- 创建公民不得要求 `wallet_account`;未成年人或暂未开户公民可以先建立本地电子护照档案。
- 推送链上公民身份时才录入 `wallet_account`;后端接受 SS58 地址或 0x 公钥,解析为内部 `wallet_pubkey` 并要求该钱包对 `VotingIdentityPayload` 签名。
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
- 公民投票资格查询：`domains/citizens/chain_vote.rs`
- 公民链上身份推送：`domains/citizens/chain_identity.rs`
  - `POST /api/v1/admin/citizens/:cid_number/onchain/prepare` 生成 `a=2 citizen_identity` 签名请求,由目标公民钱包签名。
  - `POST /api/v1/admin/citizens/:cid_number/onchain/complete` 验证公民钱包签名,落库钱包绑定,并生成 `0x0a00 register_voting_identity` 注册局管理员链上签名二维码。
- 联合投票本地人数查询：`domains/citizens/chain_joint_vote.rs`
- 地址变更调用：`domains/address/chain_call.rs`
- 立法法律只读链读：`domains/legislation/law/chain_read.rs` 负责读取 `Law`、`LawVersion`、`LawVersionLabels` 和宪法不可修改条款 manifest；`LawView.version_title/version_title_en` 只能来自链上 `LawVersionLabels[(law_id, version)]`。
- 通用 SCALE、genesis hash、RPC URL 和交易提交辅助：`core/chain_*.rs`

业务模块不得新增全局链目录，不得在 handler 内手写 pallet/call 字节或二维码动作码。动作码、payload、签名/验签规则以 `memory/07-ai/unified-protocols.md` 为唯一登记入口。

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

数据库错误必须展开 PostgreSQL SQLSTATE、message、detail 和 hint，禁止只向前端或日志传 `db error`。

## 9. 管理员写操作

管理员新增、替换、Passkey 更新、节点解绑和链写动作必须使用 `PASSKEY_COLD_SIGN` 二次确认。业务 handler 只负责构造业务动作，二维码协议包装和签名结果识别归 `core/qr/`。

联邦注册局机构 `admins` 不允许本地新增或删除，只允许在同省范围内替换。市注册局机构 `admins` 每省每市最多 30 人，统计必须同时带省和市，不能只按市名统计。NJD、普通公权机构、私权机构和非法人组织本期只能查看本机构链上 active admin 列表，不能在 OnChina 内维护管理员集合。

管理员列表 API 的展示字段统一来自链上 `AdminProfile` 投影：`admin_cid_number / name / admin_role / term_start / term_end / source / source_label`。本地 `admins.admin_name` 只用于登录态缓存、创建市注册局管理员和审计历史，不作为联邦注册局、市注册局、本机构管理员列表的展示真源；旧的本地管理员姓名 PATCH 动作和前端入口不得恢复。

## 10. 控制台能力映射

控制台 tab 能力由 `src/platform/capability.rs` 单源下发给前端。runtime 已经实现 FRG 省级组登记权高于 CREG 本市登记权，OnChina 能力表必须只镜像这个目标状态，不能另行降权：

- `FRG` 是 Tier1 创世注册局，能力必须是 `CREG` 的超集：可进入公民、私权、教育、公权机构、市注册局和联邦注册局，并可在本省范围内登记机构、写业务、维护市注册局管理员、维护本省联邦注册局管理员。
- `CREG` 是 Tier2 下级注册局，保留本市公民/机构/业务写入能力；同时必须能进入“联邦注册局”tab，只读查看本省联邦注册局管理员列表，不得发起联邦注册局管理员编辑或更换。
- `NJD`、普通公权机构、私权机构和非法人组织只获得 `can_view_own_admins`，只读查看本机构链上 active admin 列表。
- `NRC`、`PRC`、`PRB` 走节点桌面端，不获得 OnChina 网页能力。
- `PMUL` 和其它个人主体不获得 OnChina 网页能力。
- 前端 tab 展示只使用后端下发的 `capabilities`；后端 handler、scope 和链上 active admin 校验仍是安全边界。

## 11. 验收

```text
rg "mod chain;|crate::chain|chain::" citizenchain/onchina/src -g '*.rs'
cargo check --manifest-path citizenchain/Cargo.toml -p onchina
curl -kfsS https://onchina.local:8964/api/v1/health
curl -kfsS https://onchina.local:8964/api/v1/platform/ca-certificate/info
curl -kfsS -o /tmp/onchina-org-root-ca.crt https://onchina.local:8964/api/v1/platform/ca-certificate
curl -ksS -i https://onchina.local:8964/api/v1/admin/auth/check -H "authorization: Bearer <token>"
```

涉及数据库、登录、管理员列表、机构详情和扫码签名的变更必须跑真实 HTTP 接口。只通过 `cargo check` 不能证明连接池、SQL 字段顺序和扫码验签流程正确。
