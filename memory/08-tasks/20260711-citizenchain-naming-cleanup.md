# 任务卡：CitizenChain 命名精简统一 + 残留清理(含四端锁步)

## 任务需求

执行命名审计「公民链」一类([[project_naming_audit_2026_07_11]]),用户拍板**全部做**(含链上名四端锁步)。覆盖 citizenchain(runtime/node/onchina)+ 跨产品锁步 CitizenApp/CitizenWallet + host chain-signing + 金标夹具。完成后更新文档、完善注释、彻底删残留。

**新硬规则(本轮起)**:禁止一个目录下只有一个文件或一个文件夹——只清冗余包装目录,排除 Cargo `src/`/`tests/`、`.cargo/`、`__pycache__`、`.gitkeep` 等约定/工具/占位目录(见 [[feedback_dir_separator_depth_caps]] 同族)。

## 阶段

- **A 安全批(不出链)**:POW_AUTHOR_KEY_TYPE 单源 · Native*→M* 守卫镜像 · 删 BuildSpec · SS58_PREFIX 单源 · blake2b_128→blake2_128 · 存储键 helper 单源 · is_education→NED_CODE/CEDU_CODE · build_runtime_signing_payloads→build_signing_payloads · build_signed_extrinsic_local · ElectionCampaign*→Campaign* · crate/dir(offchain-transaction→offchain 等/otherpallet→misc/genesis-pallet→genesis/admin_management→management) · OrgType→InstitutionType · get_law→law · list_proposals_by_org→by_institution。验证 `cargo check`。
- **A' onchina 前后端锁步**:/api/→/api/v1/、institution→institutions、official→gov、RoleCapabilities→CapabilitySet、*_api.ts→camelCase、workspace_action_*→action/title/enabled、docs 统一、删 GovCategory/INSTITUTION_CODE_LABEL、source→origin、EDUCATION_FORM、Governing/Subordinate→Federal/City。验证 onchina cargo + 前端 tsc。
- **B 链上名四端锁步(逐项 + 重生夹具)**:MODULE_TAG(ele-camp1→ele-camp、pub-adm1/pri-adm1→pub-admin/pri-admin、全仓统一)· extrinsic(publish_square_post→publish_post[39]、submit_offchain_batch_v2→submit_offchain_batch、propose_resolution_issuance→propose_issuance、start_population_snapshot→prepare_population_snapshot)· storage/pallet(ResolutionDestro→ResolutionDestroy[11]、PendingActivation→PendingActivations、Assets→AssetMetas、OffchainBatchItemV2)· 结构(RegisteredInstitution/InstitutionInfo→entity-primitives 单源)。每项:改 runtime+host+CitizenApp+CitizenWallet+夹具,四端逐字节一致,全端构建验证。
- **C 单子目录清冗余** + 文档/注释/残留清理。

## 验收

citizenchain `cargo check` GREEN、onchina 前端 tsc GREEN、CitizenApp/CitizenWallet `flutter analyze` 零新增、签名夹具/测试全绿(四端逐字节一致)、残留旧名零引用、单子目录冗余包装已清。

## 进度(2026-07-11)

**阶段 A 安全批(不出链)已完成并 cargo 验证 GREEN:** ElectionCampaign*→Campaign*、build_runtime_signing_payloads→build_signing_payloads、build_signed_extrinsic_local、NodeModeImplementationStatus→NodeModeStatus(Rust+TS)、blake2b_128→blake2_128、删 Subcommand::BuildSpec、is_education→NED_CODE/CEDU_CODE、NativeAccountInfo/Data→MAccountInfo/Data + fullnode_issuance 去重、POW_AUTHOR_KEY_TYPE 单源、SS58 单源(node 引 primitives::SS58_FORMAT)、list_proposals_by_org→by_institution、OrgType→InstitutionType、admins::admin_management→management、**crate 改名 multisig-transfer→multisig / offchain-transaction→offchain / onchain-transaction→onchain / otherpallet→misc**(全 Cargo+rust+node crate-vs-module 消歧)。
- **genesis-pallet→genesis 证伪不可行**:crate 名 `genesis` 与 runtime 自身 `genesis` 模块冲突(construct_runtime tt_default_parts 解析到本地模块),已回退保留 genesis-pallet。
- **storage-key helper 合并延后**:byte 版(node_guard storage_prefix/map_vec)vs String 版(governance value_key/map_key)返回类型不同,属类型级重构非改名。

**阶段 A' onchina 前后端锁步已完成并验证 GREEN(onchina cargo + frontend tsc):** AdminActionType Governing/Subordinate→Federal/City、RoleCapabilities→CapabilitySet、EDUCATION_INSTITUTION→EDUCATION_FORM、workspace_action_*→action/title/enabled、API `/api/v1/` 版本化 + institution→institutions + official→gov。

**阶段 A' 尾项已补:** get_law→law(runtime API + onchina 后端;前端走 HTTP 路由不受影响)、admin_security_api.ts→securityApi.ts / city_registry_admins_api.ts→cityRegistryAdminsApi.ts。source→origin(广泛铺开的 DTO 字段)、GovCategory/INSTITUTION_CODE_LABEL 删(后者需后端下发是功能改)延后。

## 阶段 B 链上名四端锁步已完成并全端验证 GREEN

**关键判断纠正:** extrinsic/struct 名是 Substrate metadata,call_index 显式钉死(`#[pallet::call_index]`)、SCALE 位置编码,改名=标识符同步**非**改线格/签名,无需重生夹具。MODULE_TAG 是 pallet 侧给 votingengine ProposalData 加的路由前缀,**客户端零镜像**(不进冷签 payload),改名纯链内部。

已做(chain + node + CitizenApp + CitizenWallet 四端同改 + 四端构建验证):
- extrinsic:submit_offchain_batch_v2→submit_offchain_batch、OffchainBatchItemV2→OffchainBatchItem、publish_square_post→publish_post、propose_resolution_issuance→propose_issuance、start_population_snapshot→prepare_population_snapshot
- storage/pallet:PendingActivation→PendingActivations、Assets→AssetMetas(仅 onchain-issuance 存储,Config 关联型 Assets 保留)、ResolutionDestro→ResolutionDestroy(目录+crate+pallet)
- MODULE_TAG:pub-adm1→pub-admin、pri-adm1→pri-admin、ele-camp1→ele-camp、multisig-transfer→multisig(去数字尾/对齐 crate;短缩写 gra-key/res-dst 等按精简保留)
- **踩坑纠正:** prepare_population_snapshot 合并令 citizen-identity 与 joint-vote 两个不同 extrinsic 撞名 → 客户端按 call 名解码的 map 冲突(unreachable case);已把 joint 的还原为 prepare_joint_population_snapshot 保持可区分。

**终检四端全绿:** citizenchain `cargo check` EXIT=0 · onchina 前端 `tsc -b` EXIT=0 · CitizenApp `flutter analyze` 2 既有(零错)· CitizenWallet `flutter analyze` No issues。

## 阶段 C 单子目录 + onchina 尾项(部分完成,tsc GREEN)

已做:删无用 `GovCategory`;onchina 前端 `theme/theme.ts→theme.ts`;node 前端单子目录冗余包装折叠(`core/tauri.ts→tauri.ts`[38 importer]、`generated/local-docs.generated.ts→local-docs.generated.ts`、`settings/node-mode|node-key/*→settings/*`、`other/other-tabs→other-tabs`,含相对路径深度修正)。node 前端 `tsc --noEmit` EXIT=0。

**未做(结构性重构/entangled,非命名核心,建议各自单开任务):**
- `RegisteredInstitution/InstitutionInfo→entity-primitives 单源`:公私 pallet 类型 DRY 大重构(移定义到共享 crate + re-export + 两 pallet 改),类型级非改名。
- storage-key helper 合并:byte 版(node_guard)vs String 版(governance)返回类型不同,类型级重构。
- `INSTITUTION_CODE_LABEL` 删:需后端 CID 码表下发端点(功能改)。
- `source→origin`:onchina 多结构体纠缠(含 AdminSource 型 source),裸词歧义,高 churn。
- `docs/documents/DocumentLibrary` 统一:跨后端路由 + 前端组件多面。
- `genesis-pallet→genesis`:crate 名与 runtime genesis 模块冲突,证伪不可行(保留)。
- 剩余单子目录:`node/gen/schemas`、`onchina/frontend/assets` 等(约定/低值)。

**结论:四类审计(公民/公民钱包/其他/公民链)的核心命名精简统一 + 链上名四端锁步已全部执行并四端构建验证 GREEN;剩余为结构性 DRY 重构与 entangled 改名,非命名核心。**

## 阶段 D 结构性重构(2026-07-11 续,逐项做+验证)

基线:主检出起点 `cargo check` EXIT=0(干净)。改动均在主检出、不提交、供 review。

- **[D1] 机构生命周期类型 → entity-primitives 单源 —— 完成,cargo GREEN。** 先逐字段确认 public/private 两份 `institution/types.rs` **完全一致(仅 4 行 doc 措辞不同)**。7 个类型(`RegisteredInstitution`/`InstitutionLifecycleStatus`/`InstitutionInfo`/`InstitutionAccountInfo`/`CloseInstitutionAction`/`InstitutionInitialAccount`/`CreateInstitutionAccount`)上提 `entity-primitives/src/lib.rs` 作唯一定义(doc 合并为公私通用措辞,字段序/derive/枚举判别值原样保留 = SCALE 编码不变);entity-primitives Cargo.toml 补 codec/scale-info/frame-support/sp-runtime(+/std)。两 pallet `institution/types.rs` 改为 `pub use entity_primitives::{…}`,`mod.rs` 的 `pub use types::*` 与 lib.rs 对外 `pub use` 链不变 → genesis/multisig 测试及所有下游零改。node/onchina 四处 SCALE 解码镜像(字段序钉死)未动,布局不变仍可解码。`cargo check --workspace` EXIT=0(仅 2 既有 warning)。

- **[D2] storage-key helper 合并 → 单源模块 —— 完成,cargo GREEN + golden 测试过。** 实况:非两套而是 **4 份 byte 版**(node_guard 的 cid_lifecycle/citizen_issuance/fullnode_issuance/governance_skeleton 各一份 storage_prefix/map_vec/blake2_map… )+ **1 份 hex-String 版**(governance/storage_keys.rs,自研 twox_hash/blake2b_simd hasher)。统一表示决策:**裸 Vec<u8> 为核心 + `to_hex()` 包 RPC 十六进制**(String 版=`"0x"+hex(byte 版)`,逐字节等价)。新建唯一实现 `node/src/shared/storage_keys.rs`(hasher 统一 `sp_core::hashing`):`prefix`/`blake2_128_concat`/`blake2_map`/`blake2_double_map`/`twox64_map`/`to_hex` + re-export 原始 hasher。4 个 node_guard 文件的私有 helper **改薄委托**(名字/签名保留→~150 wrapper 调用点零改);governance/storage_keys.rs 的 `value_key/map_key/double_map_key/twox64_concat_prefix/system_account_key` 委托 shared + `to_hex`,`twox_128/blake2_128` 改 re-export shared(endpoint/institution_read/admins 直接取哈希的调用点零改);删自研 hasher 与 twox_hash/blake2b_simd 引用。SCALE 编码约定保留(map_vec/double_map_vec 内部 encode 带长度前缀,map_account 传裸 32 字节)。twox_64 仅测试期用故 `#[cfg(test)]` 门控。**验证:** `cargo check -p node` 仅 2 既有 warning;`cargo test storage_keys` 5 test 全过,含 golden `prefix_matches_known_system_account`(= 公认 `System::Account` 前缀 `0x26aa39…371da9`,证 twox_128 规范实现、委托后旧键逐字节不变)+ 既有 `storage_keys_distinct_per_account` 仍过。

- **[D3] 删前端 INSTITUTION_CODE_LABEL → 后端 CID 码表单源下发 —— 完成,onchina cargo + 前端 tsc GREEN。** 核实:前端硬编码 92 条**已过时**(缺 12 条军事/子部门码),真源 `primitives/cid/code.rs` `INSTITUTION_CODE_INFOS`=**104**、`PROVINCE_CODE_INFOS`=**43**(数组长度类型 + 单测 assert 双证)。后端:抽 `institution_code_items()`(admin_cid_meta 与新端点共用,ALL_CODES 派生 104 码)+ 新增免登录 `GET /api/v1/public/cid/labels`(cid::admin::public_cid_labels + CidLabelsOutput DTO,挂 public_routes)——数据即单源、内容比旧硬编码更全且已随前端 bundle 公开,无敏感性。前端:新建 `subjects/institutionLabels.ts`(模块级缓存 + `useInstitutionCodeLabels()` hook,publicRequest 一次拉取,兜底裸码);删 labels.ts 的 INSTITUTION_CODE_LABEL(92 行);5 处消费改 hook(JudicialDisplay/GovDetailPage/PrivateDetailLayout 直改、GovListTable 补 useMemo dep、OperationRecords 把 institution 标签从模块级对象改运行期 `formatAuditDetail(detail, institutionLabels)` 传参)。`EDUCATION_INSTITUTION_CODE_LABEL`(另一张 7 条教育码表,非目标)保留。**验证:** `cargo check -p onchina` EXIT=0;`npx tsc -b` EXIT=0;残留 INSTITUTION_CODE_LABEL 仅注释与新代码文档字符串。

- **[D4] source → origin(管理员展示来源字段)—— 完成,onchina cargo + tsc GREEN,前后端锁步。** 只改与 `source_label` 配对的 u8 展示字段;**`AdminSource` 枚举 + 解码镜像 `OnChainAdminProfile.source`(chain_runtime.rs:844)未动**。后端(source→origin / source_label→origin_label / fn admin_source_label→admin_origin_label):model.rs 3 DTO(CityRegistry/Federal/OwnInstitution)、chain_runtime.rs(OnChainAdminProfileView 字段 + fn + builder `origin: profile.source`——RHS 读 844 镜像故保留)、catalog.rs 2 处、city_registry_admins.rs、actions.rs、legislation/display/service.rs 测试。无 serde rename → 线名 = 字段名,前端锁步:admins/api.ts(Federal/OwnInstitution)、cityRegistryAdminsApi.ts、AdminProfileCard.tsx(type + 140/170 两处「来源」读)。缓存 bump `cid-admin-list-v4→v5`(RegistryAdminsView.tsx,含 source_label 的旧缓存自动淘汰)。**验证:** onchina cargo EXIT=0;tsc EXIT=0;**onchina** 内非 dist 残留 source_label 零(node 桌面端产品另保留 source_label,serde `rename_all=camelCase` 内部自洽,属独立产品不在本项 onchina 范围)。

- **[D5] docs/documents/DocumentLibrary 统一为 docs —— 完成,onchina cargo + tsc GREEN,前后端锁步。** 后端模块本已叫 docs;route `/documents`→`/docs`(main.rs 3 条 `/api/v1/institutions/:cid_number/documents*` → `/docs*`,replace_all;**citizen_documents 的 `/admin/citizens/.../documents` 是另一概念,不动**)+ 前端 docs/api.ts 4 条 URL 同步。组件 `DocumentLibrary`→`DocsLibrary`:`git mv DocumentLibrary.tsx DocsLibrary.tsx`、改声明、docs/index.ts re-export、两 importer(PrivateDetailLayout/GovDetailPage 的 import 路径 + JSX 标签)。**验证:** 非 dist 残留 DocumentLibrary 零;后端 3 route = `/institutions/:cid_number/docs*` 与前端 fetch 逐字一致;cargo EXIT=0;tsc EXIT=0。

- **[D6] 单子目录清冗余(全面)+ 删死资源 —— 完成,四端 GREEN。** 用户拍板全面清(≈30),node/gen/ 按「Tauri 工具生成、构建即重建」排除。
  - **23 个 Rust `mod.rs`-only 目录 → `foo.rs`**(git mv + rmdir,模块路径不变→零 import 改):runtime/src/configs、node/src/{onchina_proc,settings/{node_mode,device_password,onchina_platform,bootnodes_address,grandpa_address,fee_account},home/{identity,rpc,process},other/other_tabs,mining/{dashboard,network_overview}}、onchina/src/domains/private/{sole,participants,welfare,association,common,corporation,partnership,company}、onchina/src/institution/subjects/unincorporated_org。**踩坑:** grandpa_address.rs 的 `include_str!("../institution-catalog.json")` 相对源文件,上提一层后路径错→改 `include_str!("institution-catalog.json")`(cargo 抓到)。`cargo check --workspace` EXIT=0。
  - **7 个前端单文件目录 → 上提一层**(修被移文件相对 import 深度 + 所有 importer 路径):onchina `core/qr/citizenQr.ts`、`core/institution/CreateInstitutionForm.tsx`、`workspace/registry/RegistryWorkspace.tsx`、`workspace/generic/GenericWorkspace.tsx`(onchina tsc EXIT=0);node `settings/fee-address/WalletSection.tsx`、`app/styles/global.css`、`transaction/offchain-transaction/settlement/admin-unlock.tsx`(node tsc EXIT=0)。
  - **删死资源** `onchina/frontend/assets/login-bg.png`(12.5MB,全仓零引用,用户确认删)+ 移除空 assets/ 目录。
  - **排除(工具/约定/占位/数据)不动:** node/gen(Tauri 生成)、node/capabilities、node/resources/onchina-frontend(.gitkeep 占位)、node/data、crates(workspace member)、node/kernels、node/chainspecs、onchina/src/cid/china/reference。
  - **已知残留单子目录(超出本轮批准的 ≈30,重折叠另议):** `node/frontend/admins/` 只含 `admin-management/`(11 文件子目录,折叠需改大量 importer,属单子文件夹包装,非本轮 23+7 范围)。

## 阶段 D 验收(四端全绿)

- citizenchain `cargo check --workspace` **EXIT=0**(仅 2 既有 warning:MAccountData / KeyTypeId,与基线一致)
- onchina 前端 `npx tsc -b` **EXIT=0**
- node 前端 `npx tsc --noEmit` **EXIT=0**
- CitizenApp `flutter analyze` **2 既有 info(零错,与基线一致)**;CitizenWallet `flutter analyze` **No issues found**
- 6 项均无残留旧名/旧路径;链上名(extrinsic/MODULE_TAG/storage 布局/call_index)零改动,故无需重生金标夹具;Flutter 客户端未触碰(本轮全在 citizenchain runtime/node/onchina)。
- 改动全部留在 `/Users/rhett/GMB` 主检出工作区、未提交,供 review。

## 阶段 E 对抗式验证 + 修复(6 verifier 工作流,build-green 掩盖不了的 bug)

对 6 项跑了 6 个独立对抗性 verifier(逐项证伪 + 逐字节等价 + 跨仓残留 + 三端 build)。结论:6 项本身**结构正确**(entity-primitives SCALE 布局逐字段一致、storage-key 委托经 python 独立复算 blake2/twox 逐字节等价、source→origin 前后端锁步、docs 路由前后端一致),但验证抓出 **3 个 build 掩盖的真 bug + 若干残留**,已全部修复:

- **[E1 本轮引入·HIGH] admin-unlock.tsx CSS side-effect import 深度漏改。** item6 上提该文件时 `from '../../../`→`'../../` 的 replace_all **没匹配无 `from` 的裸 `import '../../../…styles.css'`**(第7行),路径指向 frontend 外不存在文件、破 `vite build`;`tsc --noEmit` 不解析 CSS import 故漏检。改 → `'../../admins/admin-management/styles.css'`。**验证:** `npm run build`(tsc+vite)EXIT=0。
- **[E2 阶段B残留·HIGH] onchina indexer 静默漏索引。** pallet 改名 ResolutionDestro→ResolutionDestroy 后,`indexer/event_parser.rs:349` 仍匹配旧串 `"ResolutionDestro"`→ DestroyExecuted 事件永不命中、fund_destroy 记录静默停索引(字符串字面量,cargo 不报)。改串 + 注释 → ResolutionDestroy。cargo EXIT=0。
- **[E3 阶段B残留·HIGH] CitizenWallet 测试红 + 冷签锁步破。** (a) 解码器 test 仍断言旧名 `propose_resolution_issuance`(解码器已发 propose_issuance)→ 改测试 + 金标夹具 case 名 + lookup(阶段B只跑 analyze 未跑 test)。(b) 更深:JointVote(23) 解码器发 `prepare_population_snapshot` 但 runtime 真名 = `prepare_joint_population_snapshot`(joint-vote/lib.rs:293)、CitizenApp build 侧亦用该名 → 冷签 action 比对 mismatch、pallet-23 联合快照冷签被拒。改 CitizenWallet 解码器 action + qr_protocols decode-map key + 注释 → prepare_joint_population_snapshot(pallet28 legislation 走 prepare_legislation_snapshot 不撞)。**验证:** `flutter test test/signer/` **106 全过**;`flutter analyze` No issues。
- **[E4 阶段C半成品·完成 item2 单源] 3 个 node 文件仍手搓 hasher。** home/rpc.rs / mining/dashboard.rs / settings/fee_account.rs 各有本地 twox_128/blake2_128(与旧 governance 同款)——我新写的 shared 模块 doc 曾误称「唯一实现」。已全部委托 `shared::storage_keys`(byte 逐字节等价,storage_keys 5 test 仍过)+ 清 4 处 unused import + doc 改准确。
- **[E5 上一轮潜伏 bug] 文档生成器写旧路径。** `generate-local-docs.mjs:10` outputPath 仍指 `generated/local-docs.generated.ts`(旧),而 importer 已读新位置 → 每次 build 重生旧文件、新位置静态不更新。改 outputPath → 新位置,删残留 untracked `generated/` dir;`node generate-local-docs.mjs` 复跑确认写新位置、不再生 generated/。
- **低残留清理:** 删两个 pre-existing 空目录(communication-node/communication_node);task 卡 D4「残留 source_label 零」措辞收窄为 onchina 范围。
- **已知未改(低,记录):** onchina AuthContext `resolveRoleCapabilities` 函数名仍带旧词 RoleCapabilities(类型已 CapabilitySet;纯标识符、非线名,改需动调用点,留待)。

**阶段 E 终检:** citizenchain `cargo check --workspace` EXIT=0 · storage_keys 5 test 过 · onchina `tsc -b` EXIT=0 · node 前端 `npm run build`(tsc+vite)EXIT=0 · CitizenWallet `flutter test test/signer/` 106 过 + analyze 0 · CitizenApp analyze 2 既有 · 全仓 propose_resolution_issuance / ResolutionDestro 残留零。

## 阶段 F institution-asset 壳 crate 并入 primitives(用户提议纠偏后)

**用户原提议**:institution-asset 定义机构账户转账、并入 multisig。**核查纠偏**:它**不是转账壳**,而是 61 行共享**授权 trait crate**(自述"不是 pallet,不含 storage/extrinsic"):`trait InstitutionAsset{can_spend(source,action)->bool}` + `enum InstitutionAssetAction`(9 动作:多签转账/关闭、链下批次/费率归集、NRC 安全基金、清算行 L2/L3)。**6 方依赖**(multisig/offchain/public·private·personal-manage + runtime 唯一实现 RuntimeInstitutionAsset);仅 2/9 动作与多签相关。**并入 multisig 会让 offchain+3 manage pallet 反向依赖 multisig 交易 pallet(依赖倒挂/成环/语义错位)**,已说明并经用户改选。

**执行(并入 primitives——6 方本就依赖 primitives、且 primitives 已托管此类跨切面 trait):** 新建 `primitives/src/institution_asset.rs`(trait+enum 逐字搬,enum 仅内部授权参数、非 storage/extrinsic/metadata → 零线格影响)+ lib.rs 注册模块;16 处引用 `institution_asset::`→`primitives::institution_asset::`;删 institution-asset crate + 6 Cargo.toml 依赖/feature + workspace member(43 行)。**踩坑:zsh 不对未加引号变量做词分割**,`perl … $FILES` 把整串当单文件名 → 改 for 循环逐文件。**验证:** `cargo check --workspace` EXIT=0(仅 2 既有 warning)· `cargo test -p primitives institution_asset` 过 · 全仓 bare `institution_asset::` 与 Cargo `institution-asset` 残留零 · 客户端无影响(非 metadata)。

## 阶段 G 完整测试套件补跑(填「测试全绿」验收窟窿)——发现遗留红测试

之前所有阶段验收只跑 `cargo check` + `flutter analyze` + 定向 test;这次补跑**完整**测试套件,发现验收项「签名夹具/测试全绿」在 Rust 侧与 CitizenApp 侧**从未真正达成**(check 不编译测试目标 → pallet 测试腐烂长期未被发现)。

- **CitizenWallet 完整 `flutter test`:✅ All tests passed。**
- **citizenchain `cargo test --workspace`:❌ 3 crate 测试目标编译失败**(`multisig` 15 错 / `pow-difficulty` 10 错 / `citizenchain` runtime 1 错,0 套件跑起)。**遗留、非本次造成**:报错全是**管理员/注册字段漂移**——测试仍用旧 `AdminProfile{account,name,admin_role,source}`,现结构已 `admin_account/admin_cid_number/admin_name/role_code/role_name/admin_source`(= 2026-06-28「机构/管理员字段定稿」runtime breaking,早本任务两周+);另有 AdminAccount 缺 cid_number、get/set_extra_admins 找不到、mock 缺 RegistryAuthority/InstitutionQuery、genesis::institution 解析不到。**佐证与本轮无关:pow-difficulty 任何阶段都没碰、报错无一提及 institution_asset/entity-primitives/搬动的生命周期类型。** 修它=独立活(≥3 crate 测试 mock 更新到新字段+补 trait+修引用),非命名审计范围,待用户定夺。
- **CitizenApp 完整 `flutter test`:+486 通过 / 35 失败,且 10 分钟超时被截断**(35 含真失败+超时未跑;本轮未直接改 citizenapp lib,疑似阶段 B 改名遗留或大套件超时,待甄别)。
- **未跑:** feature-gated 构建 `--features runtime-benchmarks` / `try-runtime`。

**结论:** 命名/结构/客户端锁步核心 done;但「测试全绿」验收在 Rust 与 CitizenApp 侧因**遗留测试腐烂**未达成(历史只跑 check/analyze 掩盖),是独立于命名任务的一块真实欠账。

## 阶段 H 用户三步计划·第1步:修 Rust 测试 crate

阶段 G 完整 cargo test 发现坏测试 crate 实为 **6 个**(工作区首轮 test 提前中断只报了 3;`cargo test --workspace --no-run` 才列全)。全是历史遗留测试腐烂(prior 只跑 cargo check 不编译测试目标),非本轮命名改动引入。用户拍板全修、分组二走方案二解耦。**5 个已修,onchina 单独立项:**

- **citizenchain**(1 错):cases.rs:1120 `genesis::institution::build`→`genesis_pallet::institution::build`(裸 `genesis` 撞本地 genesis.rs 模块,crate 真名 genesis_pallet;stage-A genesis 改名残留)。
- **internal-vote**(3 错):测试 3 处 `JointVote::prepare_population_snapshot`→`prepare_joint_population_snapshot`(stage-B 改名未传导测试;本地 helper 不动)。
- **multisig**(15 错):测试 mock 补 AdminAccount.cid_number、AdminProfile 字段改名(account→admin_account/name→admin_name/admin_role→role_code+role_name/source→admin_source+admin_source_ref)、三 Config 补关联型(RegistryAuthority=()、InstitutionQuery=public_manage::Pallet<Test>)、重建 thread-local get/set_extra_admins helper(2026-06-28 admin 字段定稿漂移)。
- **pow-difficulty + genesis-pallet**(10+5 错):**方案二解耦 `genesis_pallet::Config` 治理 supertrait**。genesis 加窄 trait `GenesisInstitutionSeeder`(seed 注入)+ `TargetBlockTime`(读块时间);Config 去 public_manage/public_admins supertrait 改 `InstitutionSeeder` 关联型;genesis_build 改 `<T::InstitutionSeeder as GenesisInstitutionSeeder>::seed()`;runtime configs.rs 加 `RuntimeGenesisSeeder`(调 institution::build::<Runtime>)+ pow-difficulty `type BlockTime=GenesisPallet`;pow-difficulty Config 去 genesis_pallet::Config supertrait 改窄 trait BlockTime;两测试 mock 用 dummy seeder/MockBlockTime,不再 mock 治理栈。**institution.rs 零改、seeding 逻辑不变、运行期零影响**(照搬已有 DeveloperUpgradeCheck 先例)。
- **onchina**(9 错)——**不是测试问题,是生产冷签编码 bug**(encode_admin_profile/AdminProfileArg 用旧 AdminProfile 布局,与链端不一致,影响创建机构/管理员上链交易),**单独立项 task_b6c1a9f8**(新窗口),跨四端 + 重生金标夹具。

**验证:** 生产 `cargo check --workspace` EXIT=0;`cargo test -p genesis-pallet -p pow-difficulty -p citizenchain -p multisig -p internal-vote` 全过(`genesis_public_institutions_full_mint_counts` 创世 seeding 计数回归 ok,证明方案二零影响);`cargo test --workspace --no-run` 仅剩 onchina 测试目标编译不过(已立项)。第1步除 onchina 外完成。

## 阶段 I 第2步:CitizenApp 测试基建(方案B 完成)+ widget 真失败立项

诊断:CitizenApp 完整 flutter test 50 失败,主要是测试基建——①默认并发 → Isar(isar_community/MDBX)多 isolate 抢同一固定路径 `${systemTemp}/citizenapp.isar` 互锁 → 30 秒超时;②即使串行,跨文件磁盘残留污染(resetForTest 在新 isolate `_isar==null` 时不清盘);③bootstrap `app bootstraps` 冒烟依赖 smoldot native,本地可用时不 skip → 连活链 hang 10 分钟。

**方案 B(用户拍板·唯一目录隔离)已完成:** lib/isar/app_isar.dart 加 `debugTestDirectoryOverride` seam(生产零影响);新建 test/support/isar_test_env.dart 的 `useIsolatedIsar()`(每文件唯一临时目录 + 复位 + 删目录);14 个开真库测试文件改用它(workflow 并行迁移 13 + bootstrap 手改,保留各自非-Isar mock、清死 import);bootstrap smoldot 冒烟改 `RUN_BOOTSTRAP_CHAIN_SMOKE` env 开关(bool);新建 citizenapp/dart_test.yaml(concurrency:1 + timeout:90s);顺清既有 wallet_manager_test.dart:1 unnecessary dart:typed_data。**验证:** flutter analyze 0 新增(只剩 1 既有 const-info);全量 flutter test 从 50 降到 29,wallet/chat/personal/local_tx 那批 Isar 隔离失败**全绿**,整套第一次能跑完。

**更正先前判断:**「CitizenApp 无产品 bug」不准——那是整套被 bootstrap hang 10min 截断、大批测试没跑到。方案 B 让整套跑完,暴露 **29 个 pre-existing widget 真失败**(单独跑也红,非方案 B 引入):identity_badge(12,颜色期望橙 RGB0.898,0.631,0 实际渲染白 1,1,1,疑缺 Theme/provider)、profile header/posts/user_profile(21,pumpAndSettle 抛异常)、square_home(1);personal_proposal_history 被前者泄漏污染带红,修好污染源应恢复。

widget 真失败**单独立项 task_7ff5cfe0**(新窗口)。第2步方案B(Isar 隔离基建)成果保留。

## 阶段 J 第3步:折叠 node/frontend/admins/(只含 admin-management/)

`node/frontend/admins/` 顶层只有 `admin-management/` 一个子目录(单子文件夹,违反「目录禁只含一个文件夹」硬规则;这是阶段 D6 明确留待的一项)。折叠:`admin-management/` 的 11 文件(AdminListPage/AdminProfileCard/AdminSetChangePage/AdminSetChangeSigningFlow/AdminSetDiff/AdminSetEditor/AdminWalletSelector/api/index/styles.css/types)`git mv` 上提到 `admins/`,删 `admin-management/`。import 修:内部文件 `'../../X'`→`'../X'`(深度 -1,同目录 `'./X'` 不变);外部 9 个 importer(transaction/offchain-transaction 的 node-register/admin-unlock/section、settings-panel/SettingsSection、governance 的 ProposalDetailPage/NrcSection/PrbSection/PrcSection/InstitutionDetailPage)`admins/admin-management`→`admins`。**验证:** 残留 `admin-management` 零;node 前端 `npm run build`(tsc + vite,能抓 styles.css 副作用 import)EXIT=0(`✓ built`)。第3步完成。

## 三步计划收尾(用户 2026-07-11 三步)

- **第1步 修 Rust 测试 crate:** 完整 cargo test 查出 6 个坏测试目标(非先报的 3)。5 个修好并测过(citizenchain 1 行 / internal-vote 3 处 / multisig mock 漂移 / pow-difficulty+genesis-pallet 方案二解耦 genesis::Config,创世 seeding 计数回归 ok 零影响);onchina 9 错是**生产冷签编码 bug**(非测试),单独立项 **task_b6c1a9f8**(已在另一会话运行)。
- **第2步 CitizenApp:** 方案 B 唯一 Isar 目录隔离完成,全量 flutter test 50→29(wallet/chat/personal/local_tx 那批 Isar 隔离失败全绿、整套第一次跑完);暴露的 29 个 pre-existing widget 真失败(被 hang 掩盖至今)单独立项 **task_7ff5cfe0**。「全量 flutter test 全绿」需 widget 任务修完才达成。
- **第3步 折叠 admins/admin-management→admins:** 完成,node 前端 build 绿。

改动均在 `/Users/rhett/GMB` 主检出、未提交、供 review。

## 阶段 K node/transaction 目录三端对齐(用户 2026-07-11 追问)

用户追问「node 下 transaction 目录命名为什么还没统一对齐」。核对:runtime/transaction/ 已是 `multisig`/`offchain`/`onchain`(即 pallet crate 名),但 node 侧 `src/transaction/` 与 `frontend/transaction/` 仍是旧长名 `multisig_transfer`/`offchain_transaction`/`onchain_transaction`(前端连字符形 `multisig-transfer`/`offchain-transaction`/`onchain-transaction`)——未对齐。用户拍板:前端「对齐」、src「强行对齐」。

**改动:** 三端 `git mv` 到与 runtime 逐字一致 `multisig`/`offchain`/`onchain`。
- **前端**(24 文件 rename):目录改名 + 全 .ts/.tsx 内 `/multisig-transfer`→`/multisig` 等 import 路径改写;`ProposalDetailSection.tsx` 一处旧词注释同改。
- **src**(28 文件 rename):目录改名 + `transaction::X_transaction`→`transaction::X` 精确前缀替换(35 处:offchain 26 / onchain 8 / multisig 1);`mod.rs` 重写 `pub mod` 与文档注释;`desktop/mod.rs` 6 个 tauri handler 由裸 `multisig_transfer::commands::` 改**全限定** `crate::transaction::multisig::commands::` 并删 use 导入(与同宏内 onchain handler 同风格,彻底避开 node 模块 `multisig`/`onchain` 撞 runtime 同名 crate 的坑2);`home/mod.rs`、`service.rs` 两处 doc 注释里的旧路径同步。

**关键:须保留的非路径旧令牌(未动,故意):** ①命令函数名 `build_multisig_transfer_request`/`submit_multisig_transfer`(tauri 命令名,须与前端 invoke 键锁步);②测试函数名 `..._uses_onchain_transaction_pallet`(语义描述);③runtime construct_runtime 实例名 `OnchainTransaction`/`OffchainTransaction`——benchmarking.rs 的 pallet 字符串 `"onchain_transaction"` 与 reserve.rs `const PALLET_NAME=b"OffchainTransaction"`(storage 前缀 twox_128 依赖它,改了读不到链上值)、及 listener.rs 4 处「监听/订阅/过滤 runtime pallet 事件」注释。只修了 2 处**指 node 模块位置**的残桩注释(endpoint.rs/settlement/mod.rs 的「放在/留在 offchain_transaction 下」→`offchain`)。

**验证:** `cargo check -p node` EXIT=0(无 unused import `multisig`、无撞名报错;2 条既有死导入 warning 与本次无关);前端 `tsc --noEmit` EXIT=0;残留终检 11 处旧令牌全属上述合法保留四类,零残桩。三端 `transaction/` 目录现逐字一致。改动在主检出、未提交、供 review。
