# 存量公权机构:创世直铸 + 清库重建(占号体系 卡3)

> 设计真源:`memory/04-decisions/ADR-031-cid-occupy-registry.md`(D4/D5/D9/D0)。2026-07-02 用户已决:不迁移、不映射,旧库整删,按新规则直接生成新 CID(Q5);创世直铸零交易零手续费,运行期占号/批量注册费类 Free(Q4)。

## 存量审计(2026-07-02,本机 PG `sfid` 库 subjects 表)

- 公权机构存量 245,716 个(kind=PUBLIC),全 ACTIVE、号全唯一;全部为退役旧码表(GZF 233,917/GLF 3,004/GJC 2,919/GSF 2,916/GJY 2,873/GCB 44/SCH 43),6 个号校验和坏,库 schema 仍是 `sfid_number` 旧列名。
- **GCB 44 = 国家储委会+43 省储委会、SCH 43 = 43 省储行**(SQL 核实 name 列),即常量库储备体系旧镜像,常量库已按新规则收编;真正市/镇级存量 = 245,629。
- 旧码 GZF 为聚合码(23.4 万条涵盖各类政府部门),无法按名细分映射——佐证清库重建为唯一合理路径。

## 处理决策

1. **常量库国/省级 282 个:创世直铸全量收编。** 扩展 `runtime/genesis/src/institution.rs` 遍历全部 `china_*` 数组(现只铸 CB/CH/NJD/FRG 共 89)写入 Institutions+双账户+ProtectedGenesisAccounts;构建期逐号断言 `parse_cid_number_parts`+`is_public_legal_code`,坏号 chainspec 构建期 panic。创世块直接写 state,零交易零手续费。
2. **市/镇级:清库重建。** 旧 `sfid` 库整体删除(零残留);按补齐后码表(卡1 D0)生成标准全配集——每市 C 族全 17 类、每镇 D 族补码后全类(默认全配;如另有每镇标配清单以清单为准)——新号新档案,经 `register_public_institutions_batch` 批量通道占号/注册上链(仿 `submit_offchain_batch_v2`,≤10,000 项/笔、weight 随 len 线性、凭证按批签发、费类 Free),InBestBlock 后写新库 subjects(`cid_number` 新 schema)。
3. **规模账**(2026-07-03 按行政区真源 china.sqlite 修正:43 省/2,872 市/39,087 镇):市级 2,872×17=48,824;镇级 39,087×14=547,218(按当前 T 族 14 类;若补 TLEG/TEDU 则 ×16=625,392);合计 596,042 ≈ 60 笔批量交易,安排在重新创世后的创世期 30 秒出块档内完成;终态链上登记 = 282 + 596,042 = 596,324。
4. 灌池节流:交易池默认 ready 8192 笔,批量交易按块回执逐笔推进,断点续传 checkpoint、幂等可重跑。

## 目标

- genesis 全量直铸 282(含构建期断言)。
- 重新创世:重生 raw chainspec(include_bytes! 冻结),部署链路先重生再出 deb。
- 创世后重跑 citizenapp 机构注册表生成器(死规则:否则机构全断)。
- onchina「标准机构集生成器 + 批量上链驱动器」:按行政区真源(china.sqlite)×补齐后码表生成全配集,批量占号/注册,写新库;幂等可重跑,验收只对「新库 ↔ 链上」两方一致。
- 旧 `sfid` 库整体删除,全仓零 `sfid_number` 残留。

## 修改范围

- `citizenchain/runtime/genesis/src/institution.rs` 与 `genesis/src/tests/`(数量一致性断言)
- `citizenchain/runtime/entity/public-manage/`(`register_public_institutions_batch`,与卡2同一 runtime 版本)
- chainspec 重生 + 部署脚本
- `citizenapp/tools/generate_public_institution_bundle.mjs`(重跑)
- `citizenchain/onchina/src/`(标准机构集生成器 + 批量驱动器 CLI)
- 本机/各节点旧 `sfid` 库删除

## 验收

- 创世后链上 Institutions 含全部 `china_*` 282 个,数量与常量库逐一对账一致(genesis 测试断言)。
- 内置号 100% 通过 parse 校验(构建期断言)。
- 市/镇级全配集生成数 = 链上登记数 = 新库行数(两方对账,不对旧库)。
- 批量驱动器断点续传:中断重跑不产生重复占号。
- `cargo test -p citizenchain` 与 genesis 相关测试通过;注册表生成器产物与链上一致。

## 进展

- 2026-07-02:**嵌入式库旧机构清理已执行完毕**(用户指令,单事务提交):删除旧公权机构 245,629 条(连带账户 491,258 行、gov 目录 245,629 行);87 个与常量库同体的储备机构(国家储委会 NRC 1+省储委会 PRC 43+省储行 PRB 43)按常量库更新到位——号/全称/简称/机构码/主·费·永久质押·安全基金·两和基金账户地址逐字节对齐;终态:公权机构 87、旧码残留 0;常量库零改动。备份:scratchpad/sfid_pre_cleanup.dump(32MB,会话级临时)。注意:库 schema 仍为旧 `sfid_number` 列名,schema 切新与市/镇级全配集生成按本卡后续执行。

## 状态

- 2026-07-02:建卡;同日按用户五项决策(Q1-Q5)改为「创世直铸+清库重建」终稿。依赖卡1(码表补齐+校验)、卡2(占号+批量通道)。
