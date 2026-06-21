# 修复 CitizenApp / CPMS CI 失败(预创世发布提交)

## 任务需求

- 提交「预创世发布」(b33f359f)后 CitizenApp CI 与 CPMS CI 双双失败,上一可比提交仍为绿。
- 两处失败同源:「统一管理员体系」轮的机械式 `role`→`user_group` / `AdminRole`→`AdminGroup` 全局替换误伤了不该改的目标。

## 根因

### CPMS CI(`npm run build` → `tsc -b` 失败,amd64/arm64 同)
- `citizenpassport/frontend/dangan/ArchiveList.tsx:170` 把原生 `<tr>` 的 ARIA 属性 `role="button"` 改成了 `user_group="button"`,干净类型里 `<tr>` 无 `user_group` → TS2322。
- `App.tsx:47/53` 的 `<ProtectedRoute user_group=...>` 合法(组件确有该 prop),非 bug。
- 附带发现:那次全局替换连本地 `node_modules` 也改了,`@types/react/index.d.ts` 里 `role?: AriaRole` 被改成 `user_group?: AriaUserGroup`(本地污染,node_modules 不入库,不影响 CI/仓库)。

### CitizenApp CI(`flutter test --concurrency=1` 全挂,`IsarError: IllegalArg: Collection id is invalid`)
- `citizenapp/lib/isar/wallet_isar.g.dart` 被手工 find-replace(`AdminRoleCacheEntity`→`AdminGroupCacheEntity`、`roleName`→`adminGroupName`),但 collection 级 `id`(name 的确定性哈希)没重算,仍是旧名哈希 `-7398263961586602634`,与 `name='AdminGroupCacheEntity'` 不匹配 → Isar 打开校验失败 → 凡打开 Isar 的测试全挂。
- CI 用仓库已提交的 .g.dart(不跑 build_runner),故提交的 .g.dart 必须正确。

## 修复

- CPMS:`ArchiveList.tsx:170` `user_group="button"` → `role="button"`(恢复 ARIA 角色,与"管理员角色"概念无关)。
- CitizenApp:`cd citizenapp && dart run build_runner build` 重生 .g.dart;collection id 由 `isar_community_generator` 按新名重算为 `-6431187929672259628`,与 name 一致。**铁律:生成文件只能重生,不能手改。**
- 本地清理:`cd citizenpassport/frontend && npm ci` 还原被污染的 @types/react。

## 验收

- CPMS:`npm ci` 后 `npm run build`(`tsc -b && vite build`)✓ 通过,无 TS 错。
- CitizenApp:之前失败的 Isar 测试子集(wallet_manager / attestation / reorder / governance proposal_local_store / im_isar_store)`flutter test --concurrency=1` 全过;全套件复跑结果见下。
- 残留:全仓 stale id `-7398263961586602634` 归零;cpms 前端仅 ArchiveList 一处 ARIA 误伤,已修。

## 待办

- 等用户 commit + push 后确认 CitizenApp CI / CPMS CI 转绿。
- (诊断收尾,本卡不含 runtime 改动)
