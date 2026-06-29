# 任务卡：机构 / 机构管理员 字段模型定稿 + 链上链下重构

> 字段方案经 2026-06-28 多轮需求分析定稿（链上越精简越好；公开实名资料必须上链供 CitizenApp 跨机构查看）。
> 本卡是 [20260628-onchina-onchain-write-and-followups](20260628-onchina-onchain-write-and-followups.md) 的**数据契约前置**：该卡 step1 原假设「零 runtime 改动」，本卡用户已显式授权链端改动（管理员资料上链 + 机构 pallet 精简），**该假设在机构/管理员存储范围内被本卡取代**。

## 任务需求

把「机构 + 机构管理员」字段模型按定稿方案落地：链上只存全国可见的权威/公开实名事实并极致精简，链下存私密/大文件/审计/同步索引/派生缓存。链上承载管理员公开实名资料（CID 号/姓名/职务/任期/来源），使 CitizenApp 能跨机构查看任一机构管理员，无需查各市本地库。

## 所属模块

- 链端 runtime（Blockchain Agent）：`citizenchain/runtime/private/organization-manage`、`citizenchain/runtime/admins/{admin-primitives,genesis-admins,public-admins,private-admins}`（`personal-admins` 不动）。
- onchina 后端（CID Agent）：`citizenchain/onchina/src/`（postgres schema + repo + DTO + 链写通道）。
- 客户端：`citizenwallet`（decoder）、`citizenapp`（管理员资料展示）。
- 协议登记：`memory/07-ai/unified-protocols.md`。

## 定稿字段方案（权威契约）

### 设计四原则
1. `cid_number` = 机构唯一身份主键，编码省/市/机构码/法人资格/盈利位，是一切派生的唯一源。
2. 链上 ≠ SQL 表，是 pallet 存储（SCALE、有界、按键直取、无 join）；链下 = postgres，按实体分表。
3. 同一字段绝不在两层都权威；postgres 的 `chain_*` 列只是只读投影。
4. 管理员链上实名锚 = `admin_cid_number`（账户是密码学身份不实名、姓名会重名，只有注册局 CID 号与真人一对一绑定）。

### 链上存储（pallet）
| 存储 | 键 | 值 |
|---|---|---|
| `Institutions` | `cid_number` | `cid_full_name`(仅公权)、`cid_short_name`(仅公权)、`status`、`created_at` |
| `InstitutionAccounts` | `(cid_number, account_name)` | `address`(=derive(cid,name))、`is_default`、`status`；+反向索引 `address → cid_number` |
| 机构管理员集合 | `cid_number` | `Vec<AdminProfile{ account, admin_cid_number, name, title, term_start, term_end, source }>` |

### 机构（Institution）字段
**机构主体**

| 字段 | 链上/链下 | 注释 |
|---|---|---|
| `cid_number` | 链上 | 机构唯一身份主键 |
| `cid_full_name` | 链上(公权)/链下(私权) | 公权全称上链供 CitizenApp 直读；私权存注册市本地 |
| `cid_short_name` | 链上(公权)/链下(私权) | 同上 |
| `institution_status` | 链上 | 生命周期 Pending/Active/Closed |
| `created_at` | 链上 | 创建区块号 |
| `institution_code` | 派生 | 由 cid_number 机构码段 |
| `province_code`/`city_code` | 派生 | cid_number r5 切片 |
| `province_name`/`city_name`/`town_name` | 派生 | code 经 china.sqlite 单源(ADR-021) |
| `has_legal_personality`/盈利位 | 派生 | 由机构码 / cid_number |
| `town_code` | 链下 | 不在 cid_number，仅镇级机构 |
| `parent_cid_number` | 链下 | CID 归属层，链上 payload 禁带 |
| `legal_representative_account` | 链下 | 标"哪个 admin 是法人代表" |
| `legal_rep_cid_number` | 链下 | account↔citizen 归 CID |
| `legal_rep_name` | 派生 | 法人代表 ∈ admins 时经 admin_name |
| `legal_rep_photo_*`(path/name/mime/size) | 链下 | 照片大文件 |
| `institution_document_*`(path/type) | 链下 | 材料大文件 |
| `institution_source_type` | 链下 | 创世性链上 kind 已含；FRG建/CREG建细分链下 |
| `issuer_cid_number` | 链下 | 签发注册局来源，链上验签不存 |
| `issuer_main_account` | 派生 | = derive(issuer_cid_number,"main_account") |
| `register_proposal_id` | 链下 | 注册提案审计索引 |
| `updated_at`/`created_by`/`updated_by` | 链下 | 本地编辑时间 + 操作员审计 |
| `chain_status`/`chain_tx_hash`/`chain_block_number` | 链下 | 链投影 + 索引 |
| `operation_log_id` | 链下 | 审计日志 |

**机构账户**（每个 = `cid_number + account_name`）

| 字段 | 链上/链下 | 注释 |
|---|---|---|
| `account_name` | 链上 | 账户名（键）：main_account/fee_account/自定义 |
| `account_address` | 链上 | =derive(cid_number, account_name)，注册+反向索引，供转账/反查归属 |
| `is_default` | 链上 | 主/费账户标记 |
| `account_status` | 链上 | 账户生命周期 |
| 余额 | 链上 | System.Account 原生账本，非自定义字段 |

### 管理员（Admin）字段
| 字段 | 链上/链下 | 注释 |
|---|---|---|
| `cid_number` | 链上 | 管理员所属机构（唯一身份键） |
| `admin_account` | 链上 | 管理员账户 ∈ 机构 admins |
| `admin_cid_number` | 链上 | 管理员本人实名锚（注册局 CID 号，一对一绑真人） |
| `admin_name` | 链上 | 姓名快照，来自注册局-公民列表的公民信息 |
| `admin_title` | 链上 | 对外职务（总统/议员/董事/局长），短串 |
| `admin_term_start_at`/`admin_term_end_at` | 链上 | 任期起止（类型见下方待确认项） |
| `admin_source_type` | 链上(1字节枚举) | 来源：创世/注册局/内部投票/互选/普选 |
| `admin_source_id` | 链下 | 选举/提案/登记明细 ID |
| `admin_profile_status` | 派生 | 在任由「∈admins + term」判定 |
| `admin_profile_updated_at` | 链下 | 链上 admin set 已带 updated_at 块高 |
| `admin_photo_*`(path/name/mime/size) | 链下 | 照片大文件 |
| `admin_department`/`admin_job` | 链下 | 内部部门/岗位（≠对外职务 admin_title） |
| `admin_contact_phone`/`admin_contact_email` | 链下 | 私密联系方式 |
| `admin_passkey_id` | 链下 | 登录安全（passkey 模块） |
| `admin_operation_log_id` | 链下 | 审计 |
| `chain_tx_hash`/`chain_block_number`/`chain_status` | 链下 | 同步索引 |

### 链下 postgres 三表（按实体分表，勿合并；1机构:N管理员）
- `institutions`（键 `cid_number`）：机构主体链下列。
- `institution_admins`（键 `(cid_number, admin_account)`）：管理员资料链下列。
- `institution_documents`（键 `cid_number` 或 `(cid_number, admin_account)`）：材料/照片。
- 统一同步审计列模式：`chain_status`/`chain_tx_hash`/`chain_block_number`/`created_by`/`operation_log_id`，各表共用。

### 上链录入路径（已确认）
PasskeyColdSign（三档鉴权最严档）：onchina 构造 extrinsic SCALE → passkey 二因子 → CitizenWallet 冷签 → 提交链 → 回写。签名人=注册局管理员本人(origin)，零 op_tag，符合签名铁律①。调用按目标分流：FRG→CREG `federal_set_city_registry_admins`；机构注册 `propose_create_institution`；其余机构管理员集对应 admin pallet `propose_admin_set_change`。

### 待确认实现细节（A2 开工前定）
- `term_start/term_end` 类型：推荐紧凑日期（u32，如天数自纪元）而非区块号——任期是日历语义、CitizenApp 按日期展示；区块号需换算且有出块漂移。

## 必须遵守
- 不碰：`QR_V1`、签名域 `GMB`、`primitives/cid/code.rs` 机构码表、`china.sqlite`、`CID_*` 身份 env、`personal-admins`。
- 链开发期：彻底改 + 不兼容 + 零残留；breaking 改走重新创世，不问 migration/spec_version。
- 改 extrinsic/storage 必先更新 `unified-protocols.md` 对应条目，再四方逐字节对齐（runtime/onchina/citizenwallet decoder/citizenapp）。
- 后端是唯一鉴权执行者；前端 capabilities 仅 UX 镜像。
- 注释描述当前实现，禁「从X改Y/原来/之前」历史措辞。

## 分步任务（建议顺序；A 是基座）

### Phase A · 链端契约（基座，Blockchain Agent）
- **A1 机构 pallet 精简**：仅动 `citizenchain/runtime/private/organization-manage/src/` 一个目录 —— `InstitutionInfo` 11→5 字段：保留 `cid_full_name`(公权)/`cid_short_name`(公权,新增)/`institution_code`(4字节路由缓存)/`status`/`created_at`；删 6 字段 `main_account`/`fee_account`/`admins`/`admins_len`/`threshold`/`account_count`（admins/threshold 真源在 admin pallet+internal-vote；main/fee=derive 且在 InstitutionAccounts；account_count 前缀计数）。名称分档 `is_public_legal_code(institution_code)` 公权填私权空；`propose_create_institution` payload 加 `cid_short_name`（→P-TX-001）；账户/反向索引/cid_number 建键不变。**node 的 `OnChainInstitution` 镜像不在 A1 改，随 B0 整体删**。
- **A2 admin pallet 资料化**：`admin-primitives` + genesis/public/private —— admins 从 `Vec<AccountId>` → `Vec<AdminProfile{account, admin_cid_number, name, title, term_start, term_end, source}>`；机构类按 `cid_number` 建键（personal 保持 AccountId）；`source` 枚举；`term` 类型定稿。
- **A3 协议登记 + 重新创世**：更新 `unified-protocols.md`（机构/账户/管理员 storage 契约 + `propose_create_institution`/`propose_admin_set_change` 载荷格式）；`cargo test` 全绿；重新创世。

### Phase B · onchina 链写 + 链下表（CID Agent）
> 架构定调（2026-06-28）：机构管理归 onchina，**onchina 独占机构读（subxt dynamic，与 `fetch_active_admins_onchain` 同风格）+ 写（PasskeyColdSign）**；node 桌面端 = 纯矿工不承接机构业务。
- **B0 机构管理下沉 onchina + 删 node 残留**：onchina 实现机构读（subxt dynamic，取代 node 手写 `OnChainInstitution` 镜像）；**删** `citizenchain/node/src/private/organization_manage/` + `citizenchain/node/frontend/private/organization-manage/`（清算行命名已废、ADR-030 前遗留）；删前核 node 桌面无活引用。**保留**：node `transaction/offchain_transaction/`（清算结算）、`governance/proposal.rs`（提案生命周期）。
- **B1 postgres 三表 + repo + DTO**：`institutions`/`institution_admins`/`institution_documents` + 统一同步审计列；删现 `subjects` 中派生字段冗余列（province_name 等）。
- **B2 机构链写通道**：复用注销凭证模式，构造 `propose_create_institution` 新载荷 SCALE → PasskeyColdSign 冷签 → 提交 → 回写 `chain_*`。
- **B3 管理员上链录入**：构造 admin set 录入 extrinsic（FRG→CREG / 公权 / 私权 路由），含 cid/name/title/term/source；回写；验收 = console 创建的管理员能登录（进链上 Active 集合）。

### Phase C · 客户端
- **C1 CitizenWallet decoder**（Wallet）：新机构注册 / admin set 载荷逐字节解码，无剩余字节。
- **C2 CitizenApp 展示**（Mobile Agent）：读链上管理员资料展示姓名/职务/任期/CID/来源。

### Phase D · 收尾
- **D1 残留清理 + 回写**：零残留校验；`unified-protocols.md`、本卡、ADR（如需）、长期记忆回写。

## 输入文档
- [ADR-030 onchina 多机构统一控制台](../04-decisions/ADR-030-onchina-multi-institution-console.md)
- [20260628-onchina-onchain-write-and-followups](20260628-onchina-onchain-write-and-followups.md)
- `memory/07-ai/unified-protocols.md`（P-STORAGE-001/002、P-TX-001/007）
- 长期记忆：`project_onchina_console_adr030`、`project_registry_onchain_auth_3b`、`feedback_signing_layer_selection_rule`、`project_institution_name_single_source_2026_06_21`、`feedback_china_code_immutable`。

## 验收标准
- 每阶段 `cargo test`（runtime / `-p onchina`）+ `cargo check -p node` + 客户端 decoder/UI 测试绿。
- 链上管理员可读到 cid/姓名/职务/任期/来源；CitizenApp 跨机构查看任一机构管理员成功。
- console 创建的机构/管理员真正上链，创建后能登录（过 onchain_gate）。
- 四方载荷逐字节一致；decoder 无剩余字节。
- 零残留：无冗余链上快照、无派生字段第二真源、无历史化注释。

## 进度
- [x] 字段模型需求分析 + 定稿（2026-06-28）
- [x] A1 机构 pallet 精简（2026-06-28）：`InstitutionInfo` 11→5 字段(cid_full_name/cid_short_name 仅公权 + institution_code + created_at + status)；删 main_account/fee_account/admins/admins_len/threshold/creator/account_count；`propose_create_institution` payload 加 cid_short_name；名称分档 `is_public_legal_code`；`resolve_admin_account_for_account` 改派生主账户；事件改用 stored_full_name(私权不上链)；协议登记 P-TX-001/P-STORAGE-002 已更；`cargo test -p organization-manage` 32 passed(含公权存名/私权空名/公权拒空简称三新例)。node 镜像随 B0 删。**遗留待 B2**：CID 凭证签名需纳入 cid_short_name。
- [~] A2 admin pallet 资料化（进行中）：
  - [x] A2.1 admin-primitives 基座（2026-06-28，compiles，workspace 仍绿因向后兼容）：新增 `AdminProfile<AccountId>{account,admin_cid_number,name,title,term_start,term_end,source}` + `AdminSource{Genesis/Registry/InternalVote/MutualElection/PopularElection}` + 常量 `ADMIN_NAME_MAX_BYTES=128`/`ADMIN_CID_NUMBER_MAX_BYTES=CID_NUMBER_MAX_BYTES`；`AdminAccountLifecycle<AccountId, AdminItem=AccountId>` 泛型化(create_pending/set_active_direct 收 `Vec<AdminItem>`，personal 走默认 AccountId 不动)；`AdminAccountQuery` 加默认 `active_account_admin_profiles→None`(institution 覆盖)。`active_account_admins` 签名不变(出 `Vec<AccountId>`)→投票/多签/阈值消费方零改。
  - [x] A2.2 三机构 pallet（genesis/public/private）：`AdminProfilesOf<T>=BoundedVec<AdminProfile>` 作 storage；`active_account_admins` 抽 `.account`(投票/多签/阈值零改)；新增 `active_account_admin_profiles`；`propose_admin_set_change`/`federal_set` 收 profiles；callback decode/apply profiles。**personal-admins 未动**(仍 Vec<AccountId>，走 trait 默认 AdminItem=AccountId)。
  - [x] A2.3 organization-manage：`propose_create_institution` admins→profiles，create.rs 转发(投票快照抽 account)，`CreateInstitutionActionOf`/事件带 profiles，Config 绑定 `AdminAccountLifecycle<_, AdminProfile>`(runtime/configs 与 genesis Config 自动满足)。
  - [x] A2.4 genesis 种子 `genesis_build`(CHINA_ZF→Genesis-source 空 meta profiles，逐人资料留后期上链充实)。
  - [x] A2.5 测试：public/private/genesis 各 6、organization-manage 34(含 profile 存取往返 + `CreateInstitutionAction` SCALE 非空 meta 往返 + account 路径不变三新例)；`cargo check -p citizenchain` 绿；fmt 绿。3-agent 对抗式审查全 sound，medium(创建提案往返用空 meta)已补非空往返测试闭环。
  - 协议登记已更：P-STORAGE-001(admins→AdminProfile/personal 仍 AccountId) + P-TX-007(机构布局 AdminProfile) + P-TX-001(admins→profiles)。
  - [x] A2.6 残留(全 workspace 验证发现):workflow 只测 4 crate,遗漏 `multisig-transfer/src/tests/mod.rs` + `runtime/src/tests/mod.rs` 的旧形态 `InstitutionInfo`(11字段)/`AdminAccount`(Vec<AccountId>) 测试种子;已改新 5 字段 InstitutionInfo + 机构 AdminAccount 存 profiles。`cargo test --workspace --no-run` 绿;`cargo test -p citizenchain`(35)/votingengine(87)/multisig-transfer(23) 全过。
- [x] A2 admin pallet 资料化（2026-06-28，完成并全 workspace 对抗式验证）
- [ ] A3 协议登记 + 重新创世
- [x] B0 node 清算行/机构 解耦（2026-06-28，完成并对抗式验证）：用户澄清 **清算行=链下支付(L2/L3 结算)=node 保留**、**机构管理→onchina**、两者不同业务。落地：① 删机构创建(build/submit_propose_create_institution + 前端 create-multisig，→onchina)；② 清算行要的 4 个机构**读**命令(search_eligible_clearing_banks/fetch_clearing_bank_institution_detail/proposals/registration_info)+SCALE 镜像 **移入** `node/src/transaction/offchain_transaction/institution_read/`，**链上直读**；③ 镜像更新到 A1(InstitutionInfo 5字段)+A2(AdminAccount profiles)，institution-detail 的 admins/threshold/account_count 改从 admin pallet/internal-vote/InstitutionAccounts 派生(**顺带清掉 A1 欠的 node 镜像债**)；④ 前端 3 读页移入 `offchain-transaction/institution/`，section.tsx 重连，check-multisig 缺失分支引导去 onchina，删 wait-vote 死视图；⑤ 删 `node/src/private/`(organization_manage+mod)+`node/frontend/private/`+main.rs `mod private`。**验证**:`cargo check -p node` 绿 + institution_read 测试 4/4 + 前端 tsc 0 + 零残留 + 清算/结算命令全在。3-agent 对抗式审查全 sound + 自查通过。**清算行(链下支付)完整保留不删**。
- [x] B1 onchina 机构链下 schema 对齐（2026-06-28，完成并对抗式验证）：摸清真实结构=`subjects`(三 kind 共享身份核心) + `citizens`(公民人,独立不碰) + `gov`/`private`(公权/私权明细) + `accounts`/`docs` + `admins`(控制台登录,不碰)。落地:① `subjects` 删派生地名 `province_name`/`city_name`/`town_name`(ADR-021,改 DTO 层经 china.sqlite `area_display_names` 派生,前端名称字段不变);② `subjects` 加 `updated_by`/`issuer_cid_number`/`institution_source_type`/`register_proposal_id`/`legal_representative_account`/`chain_status`/`chain_tx_hash`/`chain_block_number`;③ **新建 `institution_admins` 表**(PK `(province_code,cid_number,admin_account)` 分区)只存链下私密资料(department/job/contact/photo/passkey_credential_id/source_id/profile_status/profile_updated_at + 同步审计),**姓名/职务/任期/CID/来源在 A2 已上链不进此表**;④ repo+DTO+前端类型+缓存版本 bump+自愈。**坑(对抗式审查 CRITICAL 抓到,我修)**:`admin.rs check_cid_full_name` 公权查重残留**裸 `city_name`**(非 `s.city_name`,agent grep 漏)迁移后运行时 `column does not exist`→改用 ctx 省级作用域转 `province_code+city_code` 比对。**决策**:subjects 与 private 重复列(private_type 等)**保留+注释标记**(读路径全直读 subjects,改 join 风险大,private 表仍单源权威,留后续)。验证:`cargo check/test -p onchina` 68 过+前端 tsc 0+fmt 绿+零地名 SQL 残留。3-agent 审查(1 sound,2 CRITICAL 同一处已修)。`citizens`/`admins` 隔离不碰。
- [x] B2 机构链写通道（2026-06-28，完成并对抗式验证）：已存件复用=`build_institution_registration_credential`(OP_SIGN_INST=0x13 服务端 issuer 签,chain_runtime.rs:219)+InstitutionCreate PasskeyColdSign 动作+dereg 通道模板。真缺口=onchina 无 propose_create_institution SCALE 编码器。落地:① **新建 `core/institution_call.rs` SCALE 编码器**(pallet17/call5 前缀[0x11,0x05],15 字段 post-A1/A2:cid_short_name+admins=Vec<AdminProfile>;[u8;N]裸/Vec带Compact;accounts=InstitutionInitialAccount{name,amount:u128}非[u8;32]——agent 纠正我 spec 并按 runtime 实现)+`registration_call.rs` 组装;② **跨类型对拍 4 测试**(dev-dep admin-primitives,构造真 `AdminProfile<[u8;32]>`/`AdminSource`/arg tuple `.encode()` 逐字节==手写编码器,非自证 golden);③ QR `ACTION_CID_INSTITUTION_CREATE=4`+`build_sign_request_bytes`(b.d=SCALE call data);④ 接 PasskeyColdSign prepare(建凭证+call data+QR);⑤ indexer 写回(发现 dereg 写回本不存在,新建对称 `InstitutionCreated`→subjects/accounts `chain_status=ACTIVE_ON_CHAIN`+tx_hash/block;worker 双路径);⑥ 状态用 `subjects.chain_status`(B1 列)PENDING→ACTIVE,不新建表。**onchina 零 extrinsic 提交(grep .tx/sign_and_submit=0,提交在钱包)**。**deviation(良性)**:call data 走独立响应字段 `institution_create_sign_request`(自己的 QR k=1 b.a=4),与 step-up 治理文本 sign_request 并存,二者不互毁。验证:`cargo test -p onchina` 72 过(含 4 对拍)+前端 tsc 0+fmt 绿+零 runtime/citizens/clearing 改动。3-agent 审查全 sound。凭证未覆盖 cid_short_name(公权可派生,留 runtime 不动)。next=C1 CitizenWallet decoder(propose_create_institution post-A1/A2,与 B2 b.d 逐字节对齐)。
- [~] B3 管理员上链录入（进行中，2026-06-28，直接手写未用子智能体）：
  - [x] **B3.1 CRITICAL 修登录死锁**：onchina `OnChainAdminAccount` 镜像(chain_runtime.rs:694)`admins: Vec<[u8;32]>`(旧 Vec<AccountId>)→改 A2 `Vec<OnChainAdminProfile{account,admin_cid_number,name,title,term_start,term_end,source}>`;`fetch_active_admins_onchain` 抽 `.account` 给登录闸。A2 是 runtime-only、B0 只修了 node 镜像,**onchina 镜像本来 stale→重新创世后没人能登录**。配跨真类型对拍测试(真 `AdminAccount<Vec<AdminProfile>>`.encode()→镜像 decode,断言 account/status)。删旧手写 golden 测试(被真类型对拍取代)。
  - [x] **B3.2 admin-set SCALE 编码器**：`core/institution_call.rs` 加 `encode_admin_set_call`(复用 B2 `encode_admin_profile`)+ pallet 索引常量(genesis12/public29/private30,federal_set call1/propose_admin_set_change call0)。配跨真类型对拍(真 `(InstitutionCode,AccountId,Vec<AdminProfile>,u32)` tuple `.encode()`==手写,断前缀[12,1])。
  - [ ] **B3.3 三动作上链 wiring（剩余，路径已解非数据缺口）**：CreateCityRegistry/DeleteCityRegistry/ReplaceFederalRegistry 现 apply_*_conn 只写 postgres→仿 B2 `build_institution_create_sign_request` 加 `build_admin_set_sign_request`(QR k=1 b.d=admin-set call data,新动作码)+ prepare 返回。**CREG 机构 cid 解析已查实**(我先前误判为缺口):CREG 是确定性官方机构 cid——`official_institution_cid::<Infallible>(seed_scope, province_code, city_code, "", "CREG", province_name, city_name, |_|Ok(false))`(gov/service.rs:683 同款)→ `derive_account(creg_cid,"主账户")`=main_account=federal_set 的 account。当前 CREG 集合=`repo::list_city_registry_admins_by_scope_conn`;逐人 cid+姓名=复用 B2 `resolve_admin_identity_conn`(registration_call.rs)。待定 2 值:CREG `seed_scope` 字符串(gov 模板找) + CREG admin-set 链上 `threshold` 约定(满足 `2*threshold>admins_len`,默认多数 m-of-n)。
  - [ ] **B3.4 indexer 写回**：订阅 admin pallet `AdminAccountActivated`→回写 institution_admins/auth.admins chain_status(event_parser 现未订阅 admin 事件)。
  - 已验证:`cargo test -p onchina` 73 过(含镜像对拍+admin-set 对拍)+fmt 绿+零 `.tx` 提交。基础设施(镜像+编码器)就绪,wiring 待 CREG 机构 cid 解析确认。
- [ ] C1 CitizenWallet decoder
- [ ] C2 CitizenApp 展示
- [ ] D1 残留清理 + 回写
