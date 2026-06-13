# 任务卡:公权机构卡A — 数据包 + Isar + 增量同步 + 订阅(数据层)

属 ADR-018 §九(混合模式)。公权机构功能域的数据底座,纯客户端零链读。依赖 SFID BFF 卡(目录接口)。

状态:未开始。

## 设计(混合模式三层)
1. **数据包(基线)**:发布期从 SFID 导出接口拉全量目录 → 打进 `assets/public_institutions/`(按省分片,减小首屏解析)→ 首次启动/版本升级载入 Isar。一次性零运行时调用。
2. **版本/增量(运行时)**:开 app 低频(TTL 1d)调 `/wuminapp/public-institutions/version` 比对各省 manifest_version;有变的省(尤其浏览/订阅到的)走 `since_version` 增量,upsert + apply deleted。**懒同步:只同步用户实际打开或有订阅的省**,负载有界。
3. **Isar(合并真源)**:基线 + 增量 = 当前目录;UI 永远读 Isar(零延迟、离线、零链读)。

## 完工清单
- [ ] 生成脚本 `tools/generate_public_institution_bundle.mjs`:调 SFID 导出 → 写 `wuminapp/assets/public_institutions/<province>.json` + `catalog_version` 戳。
- [ ] Isar 实体(新增,**先沟通过结构**):
  - `PublicInstitutionEntity { sfidNumber(unique), institutionName, sfidName, shortName, province, city, town, institutionCode, orgCode, status, accountCount, customAccountNamesJson, catalogVersion, updatedAtMillis }`
  - `PublicInstitutionSubscriptionEntity { walletPubkeyHex+sfidNumber(composite unique), subscribedAtMillis }`
  - 版本戳:复用 AppKvEntity 或新 `PublicCatalogMetaEntity`(全局 catalog_version + 各省 manifest_version)。
- [ ] 数据包载入器:首次/版本升级幂等 upsert 进 Isar。
- [ ] 增量同步服务:version 比对 → 按省 delta 拉取(走 api_client + Isar/TTL,复用卡⑥ E 类缓存框架)→ upsert/delete → 更新版本戳。懒触发(打开省/有订阅才同步)。
- [ ] 订阅 store:按钱包公钥本地增删订阅、查"关注"集合(纯本地)。
- [ ] 目录查询 API(供卡 B/C):listProvinces / listCitiesByProvince / listInstitutionsByCity / getBySfid / listSubscribed —— **全部 Isar 本地查询,R1 安全(无长前缀 keysPaged)**。

## 单测
- [ ] 数据包载入幂等;catalog_version 升级触发重载。
- [ ] 增量:upsert + deleted 应用正确;版本戳持久化。
- [ ] 订阅增删 + 关注集合查询;多钱包隔离。
- [ ] custom_account_names 空/非空解析。

## 验收
- [ ] flutter analyze 0 + flutter test 全过。
- [ ] 与 SFID BFF 卡联调:全量载入 + 一次增量闭环;离线可读目录。

## 不做(边界)
- 不做 UI(卡 B);不做详情动态(卡 C)。
- 不扫链、不碰 SfidRegisteredAddress 长前缀(R1)。

## 改动目录(中文注释)
- 新增 `wuminapp/lib/citizen/public/data/`:目录 repo / 载入器 / 增量同步 / 订阅 store,代码。
- 改 `wuminapp/lib/isar/`:新增公权机构 + 订阅 + 版本戳实体(**Isar 结构变更,先沟通**),代码。
- 新增 `wuminapp/assets/public_institutions/` + `tools/generate_public_institution_bundle.mjs`:数据资源 + 生成脚本。
- 改 `wuminapp/lib/wallet/capabilities/api_client.dart`:公权目录接口 + Isar/TTL(与卡⑥ 共用缓存框架),代码。
