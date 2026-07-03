# ADR-031 CID 全局唯一与链上占号体系(终稿 v2)

## 标题

CID 号以链上写入时原子查重为唯一仲裁:校验单源、占号先行、墓碑不删、幂等续用;公权机构全量创世直铸(常量 282 + 模板派生 596,517 = 596,799),零交易;运行期新增(公民/机构)走占号先行流程。

> v1(2026-07-02)为批量交易上链方案;2026-07-03 按用户决策全面修订为本终稿。历史推演见 git 历史与 `memory/08-tasks/` 卡片。

## 一、铁则(用户已确认)

1. **查询 ≠ 占号**:唯一性仲裁只能是链上交易执行时的原子「验格式 + 查重 + 登记」;链下 RPC 预查仅作快速失败优化(TOCTOU 竞态防不住并发)。
2. **占号先行**:建档 = 本地生成号 → 占号交易 InBestBlock → 才落本地档案;占号即终身绑定。
3. **墓碑不删除**:清档/关闭发吊销交易,链上状态 Active→Revoked/Closed,存储项永不删除、号码永不复用(对齐 ADR-021 行政区码墓碑)。
4. **幂等续用**:占号携档案承诺哈希(建档稳定字段 blake2_256);落库失败重试识别「同注册局 + 同承诺」直接续用,孤号不产生。
5. **校验单源**:链上链下同一套 `primitives::cid`(码表/生成器/校验和/家族谓词),pallet 写入口全量接入。
6. **公权机构存量零交易**:全量创世直铸,与 NRC/PRC/PRB/FRG/NJD 同一模式。

## 二、已完成基座(2026-07-02/03)

| 项 | 状态 |
|---|---|
| **卡1 链端统一校验**(done/20260702-cid-occupy-card1) | ✅ citizen-identity 修 `starts_with(b"CTZN")` CRITICAL(真实号 `GD000-CTZN1-…` 曾被全拒);public/private-manage register/create 四入口接 `parse_cid_number_parts_bytes` + 家族断言(`is_person_code`=CTZN / `is_public_legal_code` / `is_private_legal_code`‖`is_unincorporated_code`)+ create 号码↔`institution_code` 参数一致;新增 `Error::InvalidCidNumber`;全仓夹具换真号 + 3 个家族拒绝用例;受影响 crate 测试全绿 |
| 码表 92 定稿 | ✅ 四级完整即制度设计:镇无立法/教委、省无省教委/省公安厅(死规则,绝不再提补码);A 国家 26/B 省 17/C 市 17/D 镇 14/E 私权 7/F 教育 6/G 个人 3/UNIN/PMUL |
| 命名规则定稿并程序验证 | ✅ 单源 = 确定性模板 gov-deterministic-v8:**简称 = 行政区显示名 + suffix,全称 = 行政区显示名 + full_suffix**;282 常量逆向验证零例外;模板覆盖 C 17/T 14/省部门 11/国家 NSN·NRP 全齐 |
| 嵌入式库清理 | ✅ 删旧公权 245,629(+账户 491,258+gov 目录);87 储备机构(NRC/PRC/PRB)对齐常量库(号/全称/简称/码/五类账户);旧码零残留 |
| 行政区真源 | china.sqlite:43 省 / 2,872 市 / 39,087 镇 |

## 三、链端设计(runtime breaking → 重新创世)

### 3.1 公民占号(citizen-identity 扩展)——卡2

```text
storage CidRegistry: CidNumberBound → {
  registrar_institution_cid,  // 登记注册局机构号
  commitment: [u8;32],        // 档案承诺哈希(建档稳定字段 blake2_256)
  status: Active | Revoked,   // 墓碑
  registered_at, revoked_at,
}
call occupy_cid(registrar_account, cid_number, commitment, province_code, city_code)
call occupy_cids_batch(registrar_account, items ≤ 10_000)   // 公民批量建档,一次冷签占 N 号
call revoke_cid(registrar_account, cid_number)               // Active→Revoked;有绑定身份则联动置 Revoked
```

- 校验:`parse_cid_number_parts_bytes` + 机构码 == CTZN(复用卡1 单源)。
- 授权:`CitizenIdentityAuthority` 省市 scope,标准 extrinsic 签名(签名分层铁律,零 op_tag)。
- 幂等:已存在且「同注册局 + 同 commitment + Active」→ Ok;否则 `CidAlreadyOccupied`。
- `register_voting_identity` 新前置:`CidRegistry[cid].status == Active`。
- 机构侧写入时查重已存在(`Institutions` + sibling `cid_exists`),不另建表;运行期新设机构经 register/create 即占号。

### 3.2 机构墓碑规范化(public/private-manage)——卡2

- `Institutions` 永不删(现状)+ 落地 `InstitutionLifecycleStatus::Closed` 语义:整机构关闭置 Closed(修 close.rs:207 注释与实现漂移)。
- 堵缺口:`register`(call 2)补本 pallet `Institutions` 检查——条目存在且 Closed ⇒ 拒绝(现状不查本表,关闭后同号可重建死索引)。
- 账户级物理删除行为维持(地址复用是既有设计)。

### 3.3 费类

- `occupy_cid`/`occupy_cids_batch`/`revoke_cid` → **Free**(公共登记服务,滥用由链上注册局授权门槛拦截);`configs/mod.rs` 穷尽 match 显式归类(编译期强制)。
- 创世直铸走创世块 state 写入,**不产生交易、不产生任何手续费**。

## 四、公权机构全量创世直铸——卡3

### 4.1 数据源(单源迁移,不进 chainspec)

- **模板表搬家**:`OfficialOrgTemplate { institution_code, suffix, full_suffix }`(国家 NSN/NRP + 省部门 11 + 市 17 + 镇 14)从 onchina `gov/service.rs` 迁入 `primitives`,onchina 改引用——链上创世与链下目录同一真源,杜绝漂移。
- **行政区常量表**:2,872 市 + 39,087 镇(code+显示名,约 2MB)由 china.sqlite 导出生成编进 primitives(幂等导出工具 + 一致性校验)。

### 4.2 创世构建(genesis/institution.rs)

- 常量 282:全量遍历 7 个 `china_*` 数组写入(现状只铸 89:CB44+CH43+NJD+FRG);NJD/FRG 管理员特例保留。
- 模板派生 596,517:「行政区 × 模板」双循环——号 = 生成器现场派生,主/费账户 = `derive_duoqian_account` 现场派生,名称 = 模板组装;与 282 共用 `insert_public_institution`;创世机构不带管理员(后续走联邦特权直设市管理员既有通道)。
- 构建期逐号断言 `parse_cid_number_parts` + `is_public_legal_code`,坏号创世构建即 panic(fail-fast)。

### 4.3 部署形态改造(必做)

- 起始 state ≈ 0.7-0.9GB(59.7 万机构 × 机构记录+双账户三索引 ≈1KB);raw chainspec JSON(≈1.5GB+)不可行。
- 节点:从「raw chainspec include_bytes! 全量 state」改「**plain spec + 首启 GenesisBuilder 本地构建创世**」——冻结语义 = 冻结 runtime+常量,创世哈希全网唯一;首启一次性写 ~420 万存储项,分钟级。
- CitizenApp/smoldot:chainspec 用 `stateRootHash` 轻形态。
- 重新创世部署(6 节点 mesh);创世后重跑 citizenapp 机构注册表生成器(死规则:否则机构全断)。

### 4.4 规模账(终态)

| 层 | 构成 | 数量 |
|---|---|---|
| 常量直铸 | 国家单体+联邦部委署局+省核心治理 6 类×43 | 282 |
| 模板派生 | 国家参众议会 NSN/NRP | 2 |
| 模板派生 | 省级部门 11 类 × 43 省 | 473 |
| 模板派生 | 市级 17 类 × 2,872 市 | 48,824 |
| 模板派生 | 镇级 14 类 × 39,087 镇 | 547,218 |
| **合计** | **全部创世,零交易零手续费** | **596,799** |

## 五、onchina 端

- **组装+提交通路(卡2)**:onchina 现状零自动提交(全部=裸 call_data→冷签→钱包提交,且回写断链)。按 node/src/governance/signing.rs 骨架补「验签后组装 + accountNextIndex 实时 nonce + immortal era + system_dryRun 拒 Future/Stale + author_submitExtrinsic + 90s 后台 nonce 核对」;**QR 仍只签不提交**,冷钱包安全边界不变。
- **事件回写闭环(卡2)**:indexer 补解析 `CidOccupied`/`CidRevoked`/机构注册事件 → 回写 citizens/subjects 的 `onchain_*` 列(修「无写入者」缺口)。
- **公民建档时序(卡2)**:

```text
录入档案 → 生成号(种子 + nonce 0..999 重试,治愈同名同生日碰撞死局) → [可选 RPC 预查]
→ 管理员冷签 occupy_cid(单笔;批量建档合并 occupy_cids_batch 一次扫码)
→ onchina 组装+dry-run+提交 → InBestBlock
→ 成功:citizens 落库(含 tx_hash/块高);CidAlreadyOccupied:nonce+1 重发号
→ 落库失败/重启恢复:按种子逐 nonce 查链,遇「本局+同 commitment」直接续用落库
清档 → revoke_cid(PasskeyColdSign)→ 链上墓碑 + 本地清档
```

- **机构册只读对账(卡3,2026-07-03 修订)**:创世机构册与 onchina **同源**(primitives 派生代码)——各节点首启用同一套 primitives 本地物化 596,799 条写 subjects/gov/accounts(零网络零交易),启动时随机抽样 32 号 fetch 链上核对(不一致 fail-closed),部署验收另有全量比对 CLI;不做每节点 ~200MB 的全量 WSS 拉取。运行期新增机构走占号先行路径落库;旧 `sfid` 库删除,全仓零 `sfid_number` 残留。

## 六、执行顺序

1. **卡2**(`20260702-cid-occupy-card2-occupy-first-flow.md`):3.1 + 3.2 + 3.3 + onchina 提交通路/回写/建档流程——与卡1 同一 runtime 版本;
2. **卡3**(`20260702-cid-occupy-card3-genesis-legacy-backfill.md`):4.1-4.3 全量直铸 + 部署形态改造 → 重新创世 → 注册表生成器重跑 → onchina 对账同步;
3. 终态对账:链上 Institutions = 596,799(genesis 测试断言与推导值一致),onchina 本地库 ↔ 链上两方一致。

## 七、影响

- runtime breaking,重新创世,零兼容零残留。
- 公民建档依赖链活性(fail-closed),每单一次管理员冷签(批量入口摊薄);链上可枚举每省建档量与全国机构册(号内无姓名生日)。
- CitizenApp/CitizenWallet:公民确认签名(ACTION_CITIZEN_IDENTITY)与扩展尾规则不变;链交易提交动作从钱包端移到 onchina 后端。
- 节点首启一次性创世构建(分钟级)与 ~0.8GB 起始 state(桌面矿工可承受);耗时记录在案。

## 八、备选方案(均已否)

- **建档只做链下查询防重**:TOCTOU 竞态防不住并发,查询不是仲裁。
- **清档删除链上号**:删除即可复用,污染历史指认;墓碑成本极小。
- **存量走批量交易上链**(v1 方案):数十笔批量交易+冷签+迁移窗口,全是多余复杂度;"直铸不可行"的旧判断只对 raw chainspec 形态成立,改 plain spec + 首启本地构建后直铸零交易更简。
- **占号交易由节点热键自动签**:违背「鉴权真源=链上 Active 管理员集合+冷签 origin」安全边界。

## 九、风险与防护

- **模板/行政区常量漂移**:导出工具幂等 + genesis 测试断言数量/名称与推导值一致 + 构建期逐号 parse 断言。
- **smoldot 轻端 chainspec 形态**:stateRootHash 形态需在 CitizenApp 真机验证(卡3 验收项)。
- **首启构建性能**:~420 万存储项写入,验收记录耗时;不可接受时退化方案为发布「官方预构建创世 DB 快照」(仍零交易)。
- **占号规模**:公民占号随建档线性增长,`CidRegistry` 条目 ~100B/人,亿级人口 ≈ 数十 GB 级远期 state——链上极简字段已是下限,属注册局链上化的固有账,创世期无感。

## 状态

- 2026-07-03:**卡3 代码全部完成**——plain spec 部署形态(chain_spec 切换/bake 重写/宪法检查 RPC 模式/首启物化实测 301s·2.7GB)、smoldot 轻形态(fork 已确认支持 StateRootHash)、onchina 启动抽样对账+audit-chain-catalog 全量比对、同源年份钉死+596,517 交叉测试、runtime 全量断言(抓修 193 常量漏铸)。测试:runtime 31/onchina 135/primitives 45 全绿。剩部署 runbook(方案 E)。

- 2026-07-02:v1 定稿(批量交易方案);同日完成嵌入式库旧机构清理。
- 2026-07-03:Q1-Q5 已决;卡1 完成归档;命名规则统一并验证;**v2 终稿 = 全量创世直铸 + 运行期占号先行**。
- 2026-07-03:**卡2 链端完成**(§3.1 CidRegistry+occupy/batch/revoke、§3.2 机构 Closed 墓碑+register 缺口封堵、§3.3 费类 Free)——citizen-identity 21/21、entity 34+34、citizen-issuance 12+5、runtime 30/30 全绿;全 runtime benchmarks 编译过(顺修 4 处既有断链)。
- 2026-07-03:**卡2 完工(onchina 侧 D6/D7/D8 完成,归档 done/)**——`core/chain_submit.rs`(组装+dry-run+提交+等进块+区块回查,QR 只签不提交)、`domains/citizens/occupy.rs`(两阶段占号 prepare/submit,nonce 碰撞重试+承诺哈希幂等续用+吊销墓碑,`chain_sign_sessions` 会话表)、D8 提交路径同步回写 onchain_* + `cid_registry_lookup` 链上预查、chain_identity complete 切 D7 会话、前端 useChainSign+两阶段 api+建档/吊销 UI;onchina 134 测试全绿、前端 tsc+build 过、node 不受影响。
- 2026-07-03:**卡3 代码核心完成(§4.1/4.2 全量创世直铸)**——命名/机构集模板迁 `primitives/cid/official_template.rs` 单源(onchina 改引用)、区划表嵌入 `primitives/cid/china/area.rs`+`area_data.bin`(43/2872/39087,gen_area_data.py 重生)、`primitives/cid/official_derive.rs` 确定性派生全部机构、genesis `insert_derived_public_institution`+`build_template_institutions` 直铸 282+596,517=**596,799**;primitives 45 测试全绿(含 596k 号 parse+唯一+计数断言)、runtime 30+onchina 134 过。**剩 §4.3 部署操作(chainspec 形态改造 + 重新创世 + onchina 只读对账 + 注册表重跑 + 旧库删除)属用户 重新创世 步**。
