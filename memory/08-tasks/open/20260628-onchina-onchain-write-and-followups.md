# 任务卡：onchina 链写通道 + 管理员/机构上链录入 + 09/10/12 收尾

> 承接 [20260628-onchina-console-refactor](20260628-onchina-console-refactor.md) 的全部后续待办。
> 该重构卡的**结构性重构 + scope 多档已完工并对抗式验证**;本卡只收**未完成项**。

## 任务需求

把 onchina(链上中国平台,原 registry)从"对链只读 + 本地元数据"补成**能把机构与机构管理员真正写上链**,并在此基座上完成 admin 泛化(09)、seed 泛化(10)、立法 web 提案(12)、能力覆盖位真源(R4)。

## 所属模块

`citizenchain/onchina`(后端 + 前端)+ 链端 `node/`/`runtime`(已有 extrinsic,需对接);自动分工 CID Agent(onchina) + Blockchain Agent(链对接)。

## 关键背景（理解 workflow 2026-06-28 查实,带证据)

**onchina 当前对链是「只读」的**:`onchina/src/core/chain_runtime.rs` 只 `.storage().at_latest().fetch()` 读链 + 离线签凭证,**全 crate 零 extrinsic 提交**(无 `.tx()`/`sign_and_submit`/`submit_extrinsic`)。后果链:

- **FRG 在 console 创建 CREG 市注册局管理员只写本地 postgres**:`apply_create_city_registry_conn`(onchina/src/auth/actions.rs:1186)→`repo::upsert_admin_conn`(:1205),**无任何上链动作**。
- 登录闸 `issue_session_after_onchain_gate`(onchina/src/auth/login/onchain_gate.rs)按**链上 Active 管理员集合**(`fetch_active_admins_onchain`)放行 → **console 创建出来的市注册局管理员实际登不进**(`NotOnchainAdmin`),除非链上 out-of-band 录入。
- 2026-06-30 口径更新：市注册局及其初始管理员不再走创世管理员模块的运行期特权入口；统一由注册局通过机构创建交易一次性写入机构与初始管理员。
- onchina **唯一**会构造的可上链凭证 = 机构注销 `build_institution_deregistration_credential`(onchina/src/core/chain_runtime.rs:312,由机构客户端冷钱包提交链)——**机构创建/管理员创建都没有这一步**。"构造 extrinsic→冷签→冷钱包提交链"这条凭证通道铺了一半,可复用。

## 待用户确认的意图（开工第一件事，先问再做）

**管理员/机构上链录入应该走哪条路?**
- (A) onchina 构造冷签凭证 → 冷钱包(CitizenWallet)提交链(复用注销凭证模式);
- (B) 走 `node/` 桌面端 `propose_admin_set_change`;
- (C) 走治理 votingengine 阈值;
- (D) 仅创世种子。

不同选择决定下面 1/2/4 的形态。**第一轮必须先做需求分析 + 确认意图,再创建子卡执行。**

> **已确认（2026-06-28）**：走 (A) 机制 = 三档鉴权最严档 `PasskeyColdSign`（passkey 二因子 + CitizenWallet 冷签），签名人=注册局管理员本人(origin)、零 op_tag。**数据契约 + 管理员资料上链 + 机构 pallet 精简已拆为前置卡** → [20260628-institution-admin-field-model-onchain](20260628-institution-admin-field-model-onchain.md)。该卡用户已显式授权链端改动（管理员 CID/姓名/职务/任期/来源上链供 CitizenApp 跨机构查看），**本卡 step1「零 runtime 改动」假设在机构/管理员存储范围内被前置卡取代**；本卡 step1 落地时直接对接前置卡定稿的新 storage 契约。

## 分步（建议顺序;1 是其余的基座）

### 🔴 1. 链写凭证基座 + 机构/管理员上链录入
- 给 onchina 建"构造特定 extrinsic 的 SCALE → 冷签 → 冷钱包提交链 → 状态回写"通道(复用注销凭证模式,SCALE 逐字节与链端对齐,零 runtime 改动)。
- 接通:① 机构上链注册;② 初始机构管理员随机构创建交易一次性上链；创建后的管理员变更再按各机构自治规则走对应管理员模块/投票引擎。
- 验收:console 创建的市注册局管理员能真正登录(进链上 Active 集合)。

### 2. card 09 admin 泛化（前置=1）
- `FederalRegistry/CityRegistry` → `Tier1/Tier2 + institution_id`;`federal_registry_scope` 表泛化或改 node env 派生(留意 FRG 一个扁平账户无省维度,需 admin_id→province 映射)。
- guards `require_admin_federal/city` → `require_admin_tier`;`catalog.rs`/`city_registry_admins.rs`/`repo.rs` 的 `*_federal_registry_*`/`*_city_registry_*` 泛化。
- **前端**:`onchina/frontend/admins/{RegistryAdminsView,ProvinceDetailView,FederalRegistryAdminSubTab,AddCityRegistryAdminModal}.tsx` 仍硬编码 `'FRG'`/`'CREG'` 字面(写权限/角色判定/锁市),需泛化到 Tier/能力位基础。
- **前置依赖**:各机构 capabilities(现 `platform/capability.rs` 除 FRG/CREG 外全 EMPTY),非注册局机构无 admin tab。

### 3. card 10 seed 泛化（随 09）
- FRG 创世引导(china_zf 215 人 → 本地投影)抽象成 Tier1 seed;明确非 FRG 机构只从链上读、不走 china_zf 播种。

### 4. card 12 governance web 提案（本次重定位核心目标；与 1 共用链写基座，建议先做以建立链写通道）
- web 端构造 legislation extrinsic(对接链上 `legislation-yuan` idx27 / `legislation-vote` idx28)→ `PasskeyColdSign` 冷签 → 提交链;SCALE 逐字段对齐,零 runtime 改动。

### 5. R4 实例覆盖位链上配置真源（单独 ADR）
- 同机构码同层级跨地区能力可不同;`capability.rs` 覆盖位本期仅签名占位,配置真源(宪法/治理派生)留后续 ADR。

## 边界铁律（沿用重构卡）

- 链开发期:彻底改 + 不兼容 + 零残留;不问 migration/spec_version。
- 不碰:`QR_V1`、签名域 `GMB`、`primitives/cid/code.rs` 机构码表、`china.sqlite`、链上 pallet/事件名/index、`CID_*` 身份 env。
- 注释描述当前实现,禁"从 X 改 Y / 原来 / 之前"历史措辞。
- 后端是唯一鉴权执行者,前端 capabilities/useScope 仅 UX 镜像。
- 改 extrinsic 构造必与链端 SCALE 逐字节一致(参照注销凭证四方对齐做法)。

## 输入文档

- [ADR-030 onchina 多机构统一控制台](../04-decisions/ADR-030-onchina-multi-institution-console.md)
- [已完工重构卡 20260628-onchina-console-refactor](20260628-onchina-console-refactor.md)
- 长期记忆:`project_onchina_console_adr030`、`project_registry_onchain_auth_3b`(链上管理员供给 3a/3b)、`feedback_signing_layer_selection_rule`(签名分层)、`project_legislation_yuan_adr027`(立法院)。

## 验收标准

- 每子卡 `cargo test -p onchina` + `cargo check -p node` + 前端 `tsc` 绿。
- console 创建的机构/管理员真正上链,创建后能登录(过 onchain_gate)。
- 立法 web 提案 SCALE 与链端逐字节一致,冷签提交成功上链。
- 零残留:无 FRG/CREG 硬编码双角色死分支(前后端)。

## 进度

- [x] 第一轮需求分析 + 确认"上链录入意图"(A/B/C/D) → A=PasskeyColdSign;数据契约拆前置卡 20260628-institution-admin-field-model-onchain
- [x] **1 链写凭证基座 + 机构/管理员上链录入(2026-06-30 口径修正)**:`core/institution_call.rs` 只保留 `propose_create_institution` 公私双 pallet 编码；注册局创建机构时在创建输入携带 `admins` + `threshold`，创建接口返回链交易二维码，机构与初始管理员由同一笔链交易写入。市注册局管理员旧直设通道已由 20260630 卡清理；验收"创建管理员能真正登录"仍待重新创世后实跑。
- [x] **2 card 09 admin 泛化 —— 完成(2026-06-29)**:见 [20260629-onchina-09-10-admin-seed-generalization](20260629-onchina-09-10-admin-seed-generalization.md)。Tier 谓词单点(is_tier1/subordinate_registry + 前端 registryTier.ts)+ AdminActionType→Tier 中性名 + capability 加 can_view_own_admins;零 `=="FRG"/"CREG"` 字面。
- [x] **3 card 10 seed 泛化 —— 完成(2026-06-29,re-scope 为退役)**:删 seed.rs/run_seed_federal_admins/SeedFederalAdmins CLI/federal_registry_scope+provinces 表;FRG 管理员 + 省映射全走链读 FederalRegistryProvinceGroups(每节点单省);含 P0 修 FRG 登录。
- [ ] 4 card 12 governance web 提案 —— **未做**(单独窗口线程;核实:onchina src/frontend 零 legislation extrinsic 构造;立法 web 提案对接 legislation-yuan idx27/legislation-vote idx28 未建)
- [x] **5 R4 收尾 —— 完成(2026-06-29)**:09/10 完成后对抗审计(29 agent)确认无回归;capability.rs 已按机构类分发(非空能力占位)。覆盖位若需单独 ADR 可另起。

**逐项核对结论(2026-06-29)**:item 1 = 代码完成(本会话 B2/B3,链上往返待重新创世);**真未做 = 09 admin 泛化 / 10 seed 泛化 / 12 立法 web 提案 / R4 覆盖位 ADR**(共 4 项)。姊妹卡 console-refactor 的 09/10/12 即本卡 2/3/4(同物,延后),其余 01-08/11/13-17 已完工。文档迁移卡 20260629-ai-system-onchina-doc-migration 已完成(onchina arch/module 文档 + checklist/DoD 已建,citizencode 目录已删,07-ai 无旧产品口径残留)。
