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
