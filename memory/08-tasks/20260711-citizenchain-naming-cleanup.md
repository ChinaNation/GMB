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

- **[D4] source → origin(管理员展示来源字段)—— 完成,onchina cargo + tsc GREEN,前后端锁步。** 只改与 `source_label` 配对的 u8 展示字段;**`AdminSource` 枚举 + 解码镜像 `OnChainAdminProfile.source`(chain_runtime.rs:844)未动**。后端(source→origin / source_label→origin_label / fn admin_source_label→admin_origin_label):model.rs 3 DTO(CityRegistry/Federal/OwnInstitution)、chain_runtime.rs(OnChainAdminProfileView 字段 + fn + builder `origin: profile.source`——RHS 读 844 镜像故保留)、catalog.rs 2 处、city_registry_admins.rs、actions.rs、legislation/display/service.rs 测试。无 serde rename → 线名 = 字段名,前端锁步:admins/api.ts(Federal/OwnInstitution)、cityRegistryAdminsApi.ts、AdminProfileCard.tsx(type + 140/170 两处「来源」读)。缓存 bump `cid-admin-list-v4→v5`(RegistryAdminsView.tsx,含 source_label 的旧缓存自动淘汰)。**验证:** onchina cargo EXIT=0;tsc EXIT=0;非 dist 残留 source_label 零。

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
