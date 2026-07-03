# 存量公权机构:全量创世直铸(占号体系 卡3)

> 设计真源:`memory/04-decisions/ADR-031-cid-occupy-registry.md`(D5/D9/D4/D0)。2026-07-03 用户修订:**市/镇级同样创世直铸,与 NRC/PRC/PRB/FRG/NJD 同一模式,全程零交易**——此前"直铸不可行"只对 raw chainspec 形态成立,数据改常量+派生、部署改 plain spec 后直铸更简;批量上链驱动器与 `register_public_institutions_batch` 从存量路径整体删除(降级为未来增量场景备选,本轮不实现)。

## 背景与已完成前置

- 92 码表定稿(镇无立法/教委、省无教委/公安厅,2026-07-03 用户拍板);卡1 链端校验已完成归档。
- 嵌入式库旧机构清理已执行(2026-07-02):删旧公权 245,629+账户+gov 目录;87 储备机构对齐常量库;旧码零残留。
- 行政区真源 china.sqlite:43 省 / 2,872 市 / 39,087 镇。

## 处理决策

1. **国/省级 282:** 扩展 `runtime/genesis/src/institution.rs` 遍历全部 7 个 `china_*` 数组(现只铸 CB/CH/NJD/FRG 共 89)写入 Institutions+双账户+ProtectedGenesisAccounts;NJD/FRG 管理员特例保留。
2. **市/镇级 596,042(市 2,872×17 + 镇 39,087×14):数据不手写、不进 chainspec。** 行政区表(市/镇 code+名称,约 2MB)编进 primitives 常量;创世构建做「行政区 × 码表」双循环:号=生成器现场派生、主/费账户=派生原语现场派生、全称/简称=「行政区名+码名」拼装,全确定性;与 282 共用 `insert_public_institution`;创世机构不带管理员(后续走联邦特权直设市管理员既有通道)。
3. **构建期断言**:逐号 `parse_cid_number_parts`+`is_public_legal_code`,坏号创世构建即 panic。
4. **部署形态改造(必做)**:起始 state ≈0.6-0.8GB,raw chainspec(~1.5GB)不可行——节点从「raw include_bytes! 全量 state」改「plain spec + 首启 GenesisBuilder 本地构建创世」(冻结语义=冻结 runtime+常量,创世哈希唯一;首启一次性 ~420 万存储项,分钟级);CitizenApp/smoldot 侧 chainspec 用 `stateRootHash` 轻形态。
5. **onchina 只读对账(D9)**:不再生成机构、不再上链;各市节点首启从链上对账拉全国机构册写本地 subjects(`cid_number` 新 schema);旧 `sfid` 库整体删除。

## 目标

- primitives 内嵌行政区常量表(从 china.sqlite 导出生成,含一致性校验脚本,幂等可重生)。
- genesis 全量直铸 282 + 596,042,构建期断言全过;genesis 测试断言数量与「常量数组 + 行政区×码表」推导值逐一一致。
- 节点部署链路改 plain spec + 首启本地构建;smoldot 侧 stateRootHash;重新创世(6 节点 mesh)。
- 创世后重跑 citizenapp 机构注册表生成器(死规则:否则机构全断)。
- onchina 首启链上对账同步器(只读),验收「链上 ↔ 本地库」两方一致。
- 旧 `sfid` 库删除,全仓零 `sfid_number` 残留。

## 修改范围

- `citizenchain/runtime/primitives/`(行政区常量表 + 导出工具)
- `citizenchain/runtime/genesis/src/institution.rs` 与 `genesis/src/tests/`
- `citizenchain/node/`(chainspec 形态、首启构建、打包脚本)
- `citizenapp/`(smoldot chainspec 资产、机构注册表生成器重跑)
- `citizenchain/onchina/src/`(链上机构册对账同步器,只读)
- 本机/各节点旧 `sfid` 库删除

## 验收

- 创世后链上 Institutions 总数 = 596,324,与推导值一致(genesis 测试断言);内置号 100% 通过 parse 校验。
- 全网各节点创世哈希一致;首启构建耗时记录在案。
- onchina 对账:链上 ↔ 本地库两方一致;CitizenApp 机构册可读。
- 全程零交易零手续费;`cargo test -p citizenchain` 与 genesis 相关测试通过。

## 状态

- 2026-07-02:建卡(原批量交易上链方案)。
- 2026-07-03:按用户决策改为**全量创世直铸**终稿;批量驱动器/批量注册 extrinsic 从本卡删除。依赖卡2(占号 pallet 与 onchina 通路,服务运行期新增)。
