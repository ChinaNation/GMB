# ADR-031 CID 全局唯一与链上占号体系

## 标题

CID 号以链上写入时原子查重为唯一仲裁:占号先行、墓碑不删、幂等续用、校验单源;存量公权机构国/省级创世直铸、市/镇级按新码表重发号批量上链。

## 背景

2026-07-02 审计结论(证据均为 文件:行,基于当前工作区):

1. **创建无链上仲裁**:公民/机构 CID 在 onchina 本地生成、本地 PG 查重落库;链上登记要么缺失(公民建档零上链),要么后置(机构两步流)。每市自治节点各自一库互不可见,公民号 N9 桶按(省,"000",年)全省共享,跨市碰撞创建期无法发现。
2. **链端校验断裂(CRITICAL)**:`citizen-identity` 只查 `starts_with(b"CTZN")`(lib.rs:673),而真实公民号为 `GD000-CTZN1-…`(省码开头)→ 真实号必被全拒;被 `b"CTZN-0001"` 手造假夹具掩盖。`public/private-manage` 注册入口只查非空。校验单源 `primitives::cid::number::parse_cid_number_parts`(no_std)pallet 本可直接调用但未接。
3. **确定性种子无重试**:公民 CID 种子=姓名+性别+生日+省市镇+出生地(admin_entry.rs:383),无 nonce 无重试,同名同生日同镇真人碰撞即 409 建档失败无恢复路径。
4. **存量两类**:
   - 常量库 `china_*.rs` 国/省级公权机构 **282 个**(zf59/jc47/cb44/sf44/lf44/ch43/jy1;43 省结构),号符合现行规则;创世仅直铸 89 个(CB44 全量+CH43 全量+NJD+FRG,genesis/institution.rs:287-380),缺 193。
   - 嵌入式库(PG `sfid` 库 subjects 表)公权机构 **245,716 个**(全 ACTIVE 全唯一),全部为退役旧码表(GZF 233,917/GLF 3,004/GJC 2,919/GSF 2,916/GJY 2,873/GCB 44/SCH 43,均不在现行 92 码表),另 6 个号校验和坏;库 schema 仍是 `sfid_number` 旧列名,现行代码查 `cid_number` 列,属上一世代存量。其中 **GCB 44 = 国家储委会+43 省储委会、SCH 43 = 43 省储行**(2026-07-02 SQL 核实 name 列),即常量库储备体系的旧镜像,常量库已按新规则收编——真正的市/镇级存量 = 245,629(市 49,494+镇 196,135)。旧码 GZF 是聚合码(23.4 万条涵盖各类政府部门机构),无法按名映射到细分新码。**旧号与旧库整体废弃,不做映射。**
5. **机构关闭语义现状**:账户级物理删除(close.rs:285-287 删 InstitutionAccounts/CidRegisteredAccount/AccountRegisteredCid;整机构再删 AdminAccounts),机构级 `Institutions` 永不删且状态从未置 Closed(Closed 仅预留语义,types.rs:46);`register` 入口(call 2)不查本 pallet Institutions,存在「关闭后同号重建账户索引」缺口,`create` 入口(call 5)被残留条目双向永久拦截。
6. **onchina 无自动提交通路**:全部链上写=构造裸 call_data→管理员冷钱包扫码→钱包端提交;`ONCHINA_SIGNING_SEED_HEX` 只签平台挑战与链下凭证(凭证进 call 参数,origin 仍是管理员冷钱包)。node 桌面端有「后端组装+dry-run+author_submitExtrinsic」先例(node/src/governance/signing.rs:612-683)。citizens 表 `onchain_tx_hash` 等列无任何写入者(回写未闭环)。
7. **费用与吞吐**:五类费种 FeeChargeKind{VoteFlat,OnchainAmount,OffchainFee,Free,Unknown}(onchain-transaction/src/lib.rs:108-114),call→费类映射穷尽 match 编译期强制归类(configs/mod.rs:330-515);出块创世期 30s/运行期 6min(链上 TargetBlockTimeMs);runtime 无 utility.batch,但有批量先例 `submit_offchain_batch_v2`(MaxBatchSize=100_000,configs/mod.rs:1835);交易池默认 ready 8192 笔。

## 决策

### 铁则(用户已确认)

1. **查询 ≠ 占号**:唯一性仲裁只能是链上交易执行时的原子「验格式+查重+登记」;链下 RPC 预查仅作快速失败优化。
2. **占号先行**:建档 = 本地生成号 → 占号交易 InBestBlock → 才落本地档案。占号即终身绑定。
3. **墓碑不删除**:清档/关闭发吊销交易,链上状态 Active→Revoked/Closed,存储项永不删除、号码永不复用(对齐 ADR-021 行政区码墓碑)。
4. **幂等续用**:占号携档案承诺哈希(建档稳定字段 blake2_256);落库失败重试时链上查到「本注册局+同承诺」→ 直接落库不二次占号,孤号不产生。
5. **校验单源**:链上链下同一套 `primitives::cid` 规则,pallet 全量接入 `parse_cid_number_parts` + 家族谓词断言。

### 链端(runtime,breaking → 重新创世)

**D1 统一校验(卡1)**:三写入口(citizen-identity 四个 call、public-manage register/create、private-manage register/create)统一调 `parse_cid_number_parts`,家族断言复用现有谓词:citizen-identity ⇒ `is_person_code` 且码=CTZN;public-manage ⇒ `is_public_legal_code`;private-manage ⇒ `is_private_legal_code`/`is_unincorporated_code`。删 `starts_with(b"CTZN")` 残桩;全部测试夹具换 `generate_cid_number` 真实产物。

**D2 公民占号(卡2)**:citizen-identity 新增:

```text
storage CidRegistry: CidNumberBound → {
  registrar_institution_cid,  // 登记注册局机构号
  commitment: [u8;32],        // 档案承诺哈希
  status: Active | Revoked,   // 墓碑
  registered_at, revoked_at,
}
call occupy_cid(registrar_account, cid_number, commitment, province_code, city_code)
call occupy_cids_batch(registrar_account, items: BoundedVec<…, ≤10_000>)   // 批量,一次冷签占 N 号
call revoke_cid(registrar_account, cid_number)                              // Active→Revoked;若有绑定身份联动置 Revoked
```

- 授权复用 `CitizenIdentityAuthority` 省市 scope(标准 extrinsic 签名,遵守签名分层铁律零 op_tag)。
- 占号幂等:已存在且「同注册局+同 commitment+Active」→ Ok(重复提交安全);否则 `CidAlreadyOccupied`。
- `register_voting_identity` 新前置:`CidRegistry[cid].status == Active`。
- 费类:`occupy_cid`/`revoke_cid` 归 **Free**(公共登记服务,滥用由链上注册局授权门槛拦截;决策点 Q4 可改 VoteFlat)。

**D3 机构墓碑规范化(卡2)**:`Institutions` 永不删维持;补上状态语义——整机构关闭时置 `InstitutionLifecycleStatus::Closed`(落地预留语义,修 close.rs:207 注释与实现漂移);`register`(call 2)补本 pallet `Institutions` 检查:条目存在且 Closed ⇒ 拒绝(堵「关闭后同号重建索引」缺口);账户级物理删除行为维持(地址复用是既有设计)。

**D4 机构批量注册通道(卡3)**:仿 `submit_offchain_batch_v2` 新增 `register_public_institutions_batch`(注册局特权、每笔 ≤10,000 项、weight 随 len 线性、凭证按批签发一份覆盖整批 digest、费类 Free/VoteFlat 见 Q4)——24.6 万存量 ≈ 25-50 笔交易、每笔一次管理员冷签扫码,创世期 30s 档下纯链上时间 < 1 小时。

**D5 创世直铸 282(卡3)**:genesis/institution.rs 全量遍历 7 个 `china_*` 数组写入 Institutions+双账户+ProtectedGenesisAccounts(NJD/FRG 管理员特例保留);构建期逐号断言 `parse_cid_number_parts`+`is_public_legal_code`,坏号 chainspec 构建即 panic。市/镇级 24.6 万不进创世(state 体积 100MB+ 级不可行),走 D4 运行期批量通道。

### onchina 端

**D6 建档时序(公民)**:

```text
录入档案字段 → 生成号(种子+nonce=0..999 重试) → [可选 RPC 预查]
→ 构造 occupy_cid call → 管理员冷钱包扫码签名(一次/单;批量建档合并 occupy_cids_batch 一次扫码)
→ onchina 组装+dry-run+author_submitExtrinsic 提交(见 D7) → 等 InBestBlock
→ 成功:citizens 行落库(含占号 tx_hash/块高) 
→ CidAlreadyOccupied:nonce+1 重发号重签
→ 落库失败:重试;重启恢复=按种子重生成逐 nonce 查链,遇「本局+同 commitment」直接续用落库
```

机构创建同理:register extrinsic 即占号,InBestBlock 后本地 subjects 落库/转正(两步流严格「链上成功才转正」)。

**D7 onchina 补「组装+提交」通路**:参照 node/src/governance/signing.rs 骨架(fetch nonce 走 accountNextIndex、immortal era、system_dryRun 拒 Future/Stale、author_submitExtrinsic、submit-only+90s 后台 nonce 核对),QR 仍**只签不提交**(冷钱包边界不变,origin 仍是管理员),提交动作从钱包端移到 onchina 后端——顺带解决现状「onchina 裸 call_data QR 与钱包扩展尾解码器衔接不明」的悬空点。等待策略:占号/注册类等 InBestBlock(PoW 三件套:显式 nonce+immortal+只等 InBestBlock)。

**D8 事件回写闭环**:indexer 补解析 `CidOccupied`/`CidRevoked`/机构注册事件 → 回写 citizens.onchain_* 与 subjects 对应列(修「onchain_tx_hash 无写入者」缺口)。

**D9 清库重建(卡3,2026-07-02 用户定):不迁移、不映射,旧库整删、按新规则直生**:
1. 旧 `sfid` 库整体删除(零残留死规则;旧号聚合码 GZF 本就无法按名细分映射);
2. 市/镇级公权机构按补齐后的码表(D0)**直接生成标准全配集**:每市 C 族全 17 类、每镇 D 族补码后全类——新号新档案,与旧数据无对账关系;
3. 生成即走 D4 批量通道占号/注册上链(断点续传 checkpoint、幂等、限速灌池 ≤ ready 8192),InBestBlock 后写新库 subjects(`cid_number` 新 schema);
4. 验收只对「新库 ↔ 链上」两方一致,不对旧库。

**D0 机构码四级完整性(已定稿,2026-07-03 用户拍板)**:92 码表即四级完整,**不补码**。镇级不设立法/教委、省级不设省教委/省公安厅是制度设计(教育=国 NED+市 CEDU;公安=市 CPOL+镇 TPOL);注册局=国 FRG+市 CREG(ADR-029)。名称已统一(市立法会/市教委会/市自治会/镇自治会等),`cargo test -p primitives --lib cid` 28 项全绿,87 储备机构库↔常量库零不一致,生成条件核验通过(2026-07-03)。

### 部署顺序

卡1(链端校验)与卡2(占号+墓碑+批量)同一 runtime 版本 → 卡3 创世直铸 282+重生 raw chainspec(include_bytes! 冻结)→ 重新创世部署(6 节点 mesh)→ **创世期 30s 出块档内**跑 D9 存量迁移 → 重跑 citizenapp 机构注册表生成器(死规则)→ 终态对账:链上登记 = 282 + 迁移数。

## 影响

- runtime breaking,重新创世,零兼容零残留。
- 公民建档由纯本地变为依赖链活性(fail-closed);每单一次管理员冷签扫码(批量入口摊薄)。
- 链上可枚举每省建档量与全国机构册(号内无姓名生日隐私,已确认边界)。
- CitizenApp/CitizenWallet:公民确认签名(ACTION_CITIZEN_IDENTITY)与扩展尾规则不变;提交动作移回 onchina 后端。
- 终态链上登记:282(国/省,创世直铸)+ 市/镇级全配集 815,329(运行期批量生成)= 815,611;旧 `sfid` 库整体删除零残留。

## 备选方案

- **建档只做链下查询防重**:被否——TOCTOU 竞态防不住并发,查询不是仲裁。
- **清档删除链上号**:被否——删除即可复用,污染历史指认;墓碑成本极小。
- **市/镇级 24.6 万也创世直铸**:被否——state 100MB+、raw chainspec 300MB+ 不可行;批量交易通道数十笔即完成。
- **占号交易由节点热键自动签**:被否——违背「鉴权真源=链上 Active 管理员集合+冷签 origin」既有安全边界;用批量占号摊薄扫码成本。

## 后续动作

任务卡(memory/08-tasks/open/):
- `20260702-cid-occupy-card1-runtime-validation.md`(D1)
- `20260702-cid-occupy-card2-occupy-first-flow.md`(D2/D3/D6/D7/D8)
- `20260702-cid-occupy-card3-genesis-legacy-backfill.md`(D4/D5/D9)

决策点(2026-07-02 全部已决):
- **Q1 已决**:四级机构码补齐完整(见 D0)。
- **Q2/Q3 已消解**:旧库 GCB 44/SCH 43 经 SQL 核实 = 国家储委会+省储委会/省储行旧镜像(非银行新增、非学校),常量库已收编,清库即消。
- **Q4 已决**:创世直铸是创世块直接写 state,**不产生交易、不产生任何手续费**;费类问题只针对运行期交易——占号/吊销/批量注册归 **Free**(滥用由链上注册局授权门槛拦截)。
- **Q5 已决**:清库重建(见 D9),不迁移不映射,旧 `sfid` 库整体删除,按新规则直接生成新 CID。

规模账(按全配集,D 族补 2 码后 16 类):市级 3,185×17=54,145;镇级 47,574×16=761,184;合计 815,329,批量 10,000 项/笔 ≈ 82 笔交易;终态链上登记 = 282(创世)+ 815,329 = 815,611。每镇标配集为卡3执行参数(默认全配,如另有每镇标配清单以清单为准)。

## 状态

- 2026-07-02:方案定稿待执行;审计与四路代码核查证据见正文。
