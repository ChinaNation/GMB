# 存量公权机构:国家/省/市创世直铸(占号体系 卡3)

> 设计真源:`memory/04-decisions/ADR-031-cid-occupy-registry.md`(D5/D9/D4/D0)。2026-07-04 部署口径更新:**创世只直铸当前国家/省/市公权机构**;镇级公权机构、国家/省/市后续新增公权机构与私权机构统一由注册局运行期注册上链。全节点继续使用 plain spec + 官方创世状态包;CitizenApp 使用 `stateRootHash` 轻形态 + 公权机构创世快照缓存 + 链投影增量。

## 背景与已完成前置

- 92 码表定稿(镇无立法/教委、省无教委/公安厅,2026-07-03 用户拍板);卡1 链端校验已完成归档。
- 嵌入式库旧机构清理已执行(2026-07-02):删旧公权 245,629+账户+gov 目录;87 储备机构对齐常量库;旧码零残留。
- 行政区真源 china.sqlite:43 省 / 2,872 市 / 39,087 镇。

## 处理决策

1. **国/省级 294:** 扩展 `runtime/genesis/src/institution.rs` 遍历全部 7 个 `china_*` 数组(现只铸 CB/CH/NJD/FRG 共 89)写入 Institutions+双账户+ProtectedGenesisAccounts;NJD/FRG 管理员特例保留。
2. **模板派生机构 49,299(2026-07-04 调整为创世到市):数据不手写、不进 chainspec。** 命名与机构集真源 = onchina `gov/service.rs` 确定性模板(gov-deterministic-v8,已由用户统一命名),**搬进 primitives 作为链上/链下单源**(onchina 改引用):
   - 组装规则(全 294 常量+模板逆向验证零例外):`简称 = 行政区显示名 + suffix`、`全称 = 行政区显示名 + full_suffix`;国家级 = label/手写;
   - 创世派生范围:国家参众议会 NSN/NRP 2 + 省级部门 11 类×43 省=473 + 市级 17 类×2,872=48,824;
   - 非创世范围:镇级 14 类×39,087 镇=547,218,由注册局按真实运行期需要注册上链;
   - 号=生成器现场派生、主/费账户=派生原语现场派生,全确定性;与 294 共用 `insert_public_institution`;创世机构不带管理员(后续走联邦特权直设通道)。
3. **构建期断言**:逐号 `parse_cid_number_parts`+`is_public_legal_code`,坏号创世构建即 panic。
4. **部署形态改造(必做)**:即便创世降到国家/省/市,正式链仍以 plain spec + 官方创世状态包冻结;正式安装包内置 `genesis-state/`,首启复制链数据库并等待 RPC ready;CitizenApp/smoldot 侧 chainspec 用 `stateRootHash` 轻形态。
5. **onchina 只读投影(D9)**:不再生成机构、不再上链;各市节点启动时只能从链上读取公权机构册并写本地 subjects(`cid_number` 新 schema)缓存;旧 `sfid` 库整体删除。

## 目标

- primitives 内嵌行政区常量表(从 china.sqlite 导出生成,含一致性校验脚本,幂等可重生)。
- genesis 直铸 294 + 49,299 = 49,593,构建期断言全过;genesis 测试断言数量与「常量数组 + 国家/省/市×码表」推导值逐一一致。
- 节点部署链路改 plain spec + 官方创世状态包;smoldot 侧 stateRootHash;重新创世(6 节点 mesh)。
- 创世后重跑 CitizenApp 公权机构快照包生成器(死规则:否则机构全断)。
- onchina 启动链上投影同步器(只读),验收「链上 ↔ 本地库」两方一致。
- 旧 `sfid` 库删除,全仓零 `sfid_number` 残留。

## 修改范围

- `citizenchain/runtime/primitives/`(行政区常量表 + 导出工具)
- `citizenchain/runtime/genesis/src/institution.rs` 与 `genesis/src/tests/`
- `citizenchain/node/`(chainspec 形态、创世状态包安装、打包脚本)
- `citizenapp/`(smoldot chainspec 资产、公权机构快照包生成器重跑)
- `citizenchain/onchina/src/`(链上机构册对账同步器,只读)
- 本机/各节点旧 `sfid` 库删除

## 验收

- 创世后链上 Institutions 总数 = 49,593(常量 294 + 模板派生 49,299),与推导值一致(genesis 测试断言);内置号 100% 通过 parse 校验。
- 全网各节点创世哈希一致;正式首启复制创世状态包后 RPC ready 记录在案。
- onchina 对账:链上 ↔ 本地库两方一致;CitizenApp 机构册可读。
- 全程零交易零手续费;`cargo test -p citizenchain` 与 genesis 相关测试通过。

## 进展

- 2026-07-03:**代码核心完成并测试通过**(primitives 45 测试全绿、genesis lib + runtime + onchina 编译过、runtime 30 lib 测试过、onchina 134 测试过):
  - **命名/机构集模板单源迁 primitives**:新建 `primitives/cid/official_template.rs`(OfficialOrgTemplate + short_name/full_name 组装 + 省部门 11/市 17/镇 14/国家两院 2 全表);onchina `gov/service.rs` 删本地副本改引用,消除 D0 漂移风险。镇级模板保留给运行期注册使用,不进入创世枚举。
  - **行政区表嵌入 primitives**:`gen_area_data.py` 从 china.sqlite 生成紧凑二进制 `area_data.bin`(43 省/2872 市/39087 镇,578KB);新建 `primitives/cid/china/area.rs` no_std 零拷贝解析器(`for_each_area` 单回调 AreaItem 枚举 + `area_counts`)。
  - **创世派生单源**:新建 `primitives/cid/official_derive.rs`——`for_each_public_institution(f)` 用 seed+generator 确定性派生国家/省/市公权机构(与 onchina official_institution_cid 同源,年份固定 2026,国家两院落中枢省主市);`public_institution_derived_count()`。
  - **genesis 直铸**:`genesis/institution.rs` 新增 `insert_derived_public_institution`(号确定性派生账户、构建期 parse+公权家族断言、不进 ProtectedGenesisAccounts 避免 59 万双倍保护)+ `build_template_institutions` 调 primitives 枚举;`build()` 末尾接入。常量 294 全量直铸维持。
  - **测试**:primitives 断言派生数 =49,299、每号 parse 合法+公权家族+全局唯一、名称组装、国家两院国名前缀、区划计数 43/2872/39087。终态链上 = 294 + 49,299 = **49,593**,零交易。
- **剩余 = 部署操作(用户 重新创世 步)**:①chainspec 形态改造(plain spec + genesis-state 创世状态包;smoldot 侧 stateRootHash)②重新创世部署(6 节点 mesh)③onchina 链上机构册只读投影 ④citizenapp 公权机构快照包重跑 ⑤旧 sfid 库删除。genesis-pallet 自身 test mock 为上一轮 runtime 重构预留断链(缺 public_manage/public_admins 等 impl),故派生断言落 primitives(编译/测试均绿);创世 state 物化在 bake 阶段发生,正式用户首启复制创世状态包。

## 进展与审计(2026-07-03)

- **链端派生已完成**:primitives 新增 `official_template.rs`(命名模板单源:国 2/省部门 11/市 17/镇 14)、`official_derive.rs`(创世派生枚举,`public_institution_derived_count()==49,299`)、`china/area.rs + area_data.bin`(行政区常量:43 省/2,872 市/39,087 镇,与 china.sqlite 一致);genesis `insert_derived_public_institution`(构建期 parse+公权家族断言;派生机构 Active、主/费双账户,**不打 ProtectedGenesisAccounts 封存标记**,留给后续治理)。
- **体积口径更新**:镇级机构不进创世,移出曾导致首启重物化的 547,218 条;正式 bake 仍产 genesis-state,但创世规模降为 49,593 机构。
- **覆盖核实**:cities 表无 "000" 保留市(count=0,无多铸);村级无机构码(制度定稿)不铸;国家/省/市 × 对应码族创世直铸;镇级码族只服务运行期注册。
- **账户操作语义核实**:机构账户=派生地址无私钥,唯一操作路径=管理员集合经 internal_vote(multisig-transfer 要求 `InstitutionQuery::is_active` + 发起人 `is_internal_admin`);无管理员=完全不可操作(创世余额亦为 0);创世带管理员的仅 NJD/FRG/CB/CH,其余待联邦特权直设。

## 剩余工作清单

1. **node 部署形态改造(D5 必做)**:chain_spec.rs 已转 plain spec + genesis-state 创世状态包;CitizenApp/smoldot 侧 stateRootHash 轻形态需随新 CI WASM 重新烘焙。
2. **onchina 机构册只读投影同步器(D9)**:启动期从链上拉全国机构册写本地 subjects 缓存,链不可达 fail-closed。
3. **测试补课**:genesis-pallet 自带测试 mock 既有断链(Config 现要求 public_manage/public_admins 全栈);runtime 级 49,593 总数断言需随本轮更新。
4. 部署操作:重新创世(6 节点)→ 重跑 CitizenApp 公权机构快照包 → 各节点旧 `sfid` 库删除。

## 收尾与部署技术方案(2026-07-03 定稿,下一步执行序)

技术支点(已实码核验):runtime 已实现 `GenesisBuilder` API(runtime/src/apis.rs:319);`fresh_genesis_config()` 已是 plain 构建器形态(ChainSpec::builder(wasm)+with_genesis_config_patch+复用冻结 bootnodes);`institution::build` 挂在 genesis-pallet `BuildGenesisConfig`(创世构建期在 WASM 内执行);citizenapp chainspec 资产在 citizenapp/assets/chainspec.json(smoldot-pow 分支)。

### A. node 部署形态改造(plain spec + 官方创世状态包)
- A1 生成冻结 plain spec:`fresh_genesis_config()` → `as_json(raw=false)` 落 `node/chainspecs/citizenchain.plain.json`(内嵌 WASM+genesis patch+bootnodes+properties,MB 级);新增导出入口(CLI 子命令或 clean-run.sh 步骤)。
- A2 `chain_config()` 切换:include_bytes 冻结 plain JSON(替换 raw);raw 文件删除零残留;clean-run.sh / fuwuqi.sh 打包链路适配。
- A3 bake 行为:`bake-chainspec.sh` 启动临时节点经 WASM `GenesisBuilder_build_state` 物化 49,593 机构,导出 `target/chainspec/genesis-state/`。
- A4 正式首启行为:安装包内置 `genesis-state/`,节点启动前复制 `chains/citizenchain/db` 到本地数据目录;没有内置包时才允许开发/排障回退到 GenesisBuilder。
- A5 冻结语义:冻结的是 plain JSON(runtime WASM + patch + bootnodes)与同一次物化出的创世状态包,创世哈希由其唯一决定,全网一致(派生全确定性)。

### B. citizenapp/smoldot 轻形态
- B1 新脚本从首个已启动节点读 genesis header(块 0 哈希 + stateRoot),产出 `assets/chainspec.json` 轻形态:name/id/bootNodes/properties + `genesis.stateRootHash`(不含完整 state,不含 runtimeGenesis)。
- B2 smoldot-pow 分支对 `genesis.stateRootHash` 解析的兼容性验证(真机,验收项)。
- B3 公权机构快照包生成器重跑(死规则)。

### C. onchina 机构册投影(D9 修订:链上唯一真源)
- C1 启动投影:从链上 `PublicManage::Institutions` / `InstitutionAccounts` 全量读取,批量写 subjects/gov/accounts 缓存。
- C2 fail-closed:链不可达、创世哈希不匹配、链上目录不可读或本地写库失败,都拒绝启动工作台。
- C3 验收工具:一次性 CLI 全量遍历链上 Institutions ↔ 本地缓存比对(部署验收用)。
- C4 运行期增量:占号先行创建的新机构走卡2 已建路径落库。
- 修订理由:公权机构真源必须唯一在链上;OnChina primitives/china.sqlite 只能辅助解析,不得反向生成机构目录。

### D. 测试补课
- D1 runtime 级总数断言:runtime tests 调 `genesis_pallet::institution::build::<Runtime>()` 后 `Institutions::iter().count()==49,593` + 抽查(市级名称组装、账户派生、NJD/FRG 管理员在位;镇级走运行期注册)。
- D2 genesis-pallet 自带 mock 修复:镜像 public-manage tests 全栈 mock(system/balances/votingengine/admins/entity),恢复其相位切换测试。

### E. 部署 runbook(代码完成后,用户执行)
1. 生成并冻结 plain spec → 提交;
2. prepack 内置 genesis-state → 出安装包 → 6 节点部署;首启复制状态包后 `chain_getBlockHash(0)` 全网一致核对;
3. citizenapp 轻形态 chainspec 替换 + 公权机构快照包重跑 + 真机 smoldot 验证;
4. onchina 各节点链投影同步通过;本机与各节点旧 `sfid` 库删除(零残留);
5. 终态验收:链上创世 Institutions=49,593、创世哈希一致、onchina 全量比对 CLI 通过、CitizenApp 机构册可读、公民建档占号端到端(真链)walkthrough。

### F. 风险
- WASM 创世构建耗时/内存(退化=DB 快照);smoldot stateRootHash 真机未验;bootnodes 复用已由 fresh_genesis_config 处理;创世哈希决定性=同 WASM+同 patch(派生代码确定性已测)。

## 收尾执行完成(2026-07-03)

- **A node 部署形态改造完成**:`chain_spec.rs` 切换冻结 plain spec(`citizenchain.plain.json`,10MB=WASM 5.3MB+patch+44 bootnodes;raw 已删零残留);`bake-chainspec.sh` 重写为 plain 流程(导出 plain → CITIZENCHAIN_HEADLESS 临时节点物化创世并记录耗时 → RPC 宪法检查 → 读块 0 头产 App 轻形态 → finalize 同步双 SSOT);`check-constitution-genesis.py` 新增 `--rpc/--at` 模式(临时节点按键查询,文件模式保留);run.sh/clean-run.sh raw 引用清零。
- **旧首启物化冒烟实测已废弃**:旧方案的首启重物化数据不再作为验收依据;本轮改为 49,593 机构后必须用 CI WASM 重新 bake。
- **B smoldot 轻形态**:bake 脚本自动产 `genesis.stateRootHash` 形态 App chainspec;**smoldot-pow 分支已确认原生支持 StateRootHash**(chain_spec.rs:72/317),真机验证留部署验收。
- **C onchina 链上对账完成**:新 `domains/gov/chain_audit.rs`——启动抽样 32+1 号对链上 `Institutions` 核对(名称/Active,fail-closed 拒绝启动;链暂不可达重试 6×10s;`ONCHINA_GOV_CHAIN_AUDIT=0` 开发逃生门);`audit-chain-catalog` 子命令全量双向比对(部署验收);`chain_runtime` 新增 `institution_lookup`/`for_each_chain_institution`。**同源加固**:onchina `official_institution_cid` 直调 primitives 生成器并钉死创世年份 2026(修掉"按当前年份"的跨年漂移炸弹);本轮同源交叉测试需按 49,299 创世派生重跑。
- **D1 runtime 全量断言抓到真缺口并修复**:旧断言曾抓出常量机构漏铸;本轮总数应更新为 49,593(含派生首条逐字节抽查+NJD 管理员在位),plain spec 需用 CI WASM 重生。
- **测试终态**:runtime 31/31(含 3 分钟全量直铸断言)、onchina 135/135(含同源交叉)、primitives 45/45、node 编译过。
- **D2 遗留(不阻塞部署)**:genesis-pallet 自带 test mock 为上轮 runtime 重构预留断链(Config 要求 public_manage/public_admins 全栈),其相位切换测试待镜像 entity 全栈 mock 修复;派生/直铸断言已由 primitives 45 + runtime 31 覆盖。

## 剩余 = 部署 runbook(用户执行,方案 E 节)

①`bake-chainspec.sh --finalize --wasm <CI_WASM>` 重生 plain SSOT、CitizenApp 轻形态和 genesis-state → 提交;②prepack 内置 genesis-state、6 节点部署、`chain_getBlockHash(0)` 全网一致核对;③citizenapp 真机 smoldot 验证 + 49,593 创世公权机构快照包重跑;④onchina 各节点链投影同步通过 + `audit-chain-catalog` 全量比对;⑤删本机与各节点旧 `sfid` 库;⑥公民建档占号端到端 walkthrough。

## 状态

- 2026-07-02:建卡(原批量交易上链方案)。
- 2026-07-03:曾改为扩大创世范围方案;代码核心完成。
- 2026-07-04:按用户最终口径调整为**创世只到国家/省/市**;镇级公权机构和后续新增机构统一由注册局运行期注册上链。
