# 任务卡：CID 市列表/市卡片 显示空白与无法进入 修复

创建：2026-06-23

## 根因（已诊断）

提交 `bf187d53 统一命名修复` 把后端 cities 接口返回字段 `name`→`city_name`（`CidCityItem` 无 serde rename，JSON 键跟随字段名），但前端缓存版本号 `CID_CITY_CACHE_VERSION='cid-cities-v2'` 未 bump。`metaCache.readCache` 只比对 version → 旧结构缓存（`city_name` 全空）静默残留，永不回源。

两症状同根因：
- 市卡片（CityGrid，被 gov/education/private tab 复用）：`city_name||code` → 空 → 显示 code(001/002)，点击传空无法进入。
- 市注册局列表（ProvinceDetailView）：`registryRows.find(r=>r.city_name===city_item.city_name)` join 落空 → registry=null → 身份ID/名称列空、点击失效。

后端 china.sqlite 数据正常（001|锦程市，name 空值=0/2872），字段名已修对 → 残留纯前端缓存。

## 方案（用户确认：治本+加固）

- **治本**：`CID_CITY_CACHE_VERSION` `'cid-cities-v2'`→`'cid-cities-v3'`，丢弃全部旧缓存强制回源。
- **加固**：`metaCache` 城市缓存增加形状校验——缓存若存在 `city_name` 缺失项即判脏弃缓存回源，使缓存对未来字段漂移自愈。
  - 注：原报告提的「join 改 code」前端不可行（`InstitutionListRow`/`CityRegistryAdminRow` 只有 city_name 无 city code，需后端 DTO 加字段才能做，留作 follow-up）。两端 city_name 同源 china.sqlite，治本+形状校验后 join 必成立。

## 预计修改

- `citizencode/frontend/china/metaCache.ts`：bump version + citiesCacheUsable 形状校验（loadCachedCidCities/readCachedCidCities）。

## 验收

- [ ] tsc 通过。
- [ ] 逻辑：旧 v2 缓存被弃；含空 city_name 的缓存被判脏回源；正常数据正常缓存命中。

## 执行记录

- [x] 改 metaCache.ts：`CID_CITY_CACHE_VERSION` v2→v3 + `citiesCacheUsable` 形状校验（loadCachedCidCities 命中前校验、脏缓存 removeItem 回源；readCachedCidCities 校验后才返回）。
- [x] tsc 验证 ✅ `npx tsc -b` TSC_EXIT=0。
- [x] 教训沉淀 memory：改 DTO 字段必 bump 缓存版本号（[[feedback-dto-field-rename-bump-cache-version]]）。
- [ ] follow-up（非阻塞）：join 改 code 需后端 InstitutionListRow/CityRegistryAdminRow 增 city_code 字段。
- [ ] CityGrid.tsx 的 `|| c.code` 治标补丁（未提交）：治本后不再触发，保留作最后防线，不动（不扩范围）。

## 验证说明
- 静态：tsc 0 error；逻辑三场景自检通过（旧 v2 弃 / 脏缓存回源 / 正常命中）。
- 运行时最终验证：登录 CID 控制台进省→市，市卡片显示市名（非 001/002）可点入、市注册局列表身份ID/名称有值可点入即闭环（旧浏览器无需手动清缓存，v3 自动弃旧）。
