# 20260625 管理员字段扩展 + 护宪大法官生产成员解析(E1+E2 合并卡)

承接 `20260625-legislation-signing-5type-revision.md` 的 E 收尾。本卡把原 E1(管理员字段扩展)+ E2(护宪生产成员解析)合并实现;E3(重新创世 + 真机 QA)不在本卡(用户指定跳过,随整套上链统一处理);E4(残留文档清理)已在母卡随手做完。

## 背景 / 目标

护宪大法官终审(F)链端状态机已落地。本卡当前目标态是:管理员集合统一为 `AdminProfile{account, admin_cid_number, name, admin_role, term_start, term_end, source}`;护宪生产成员由 `RuntimeInternalAdminProvider::constitution_guard_members()` 从国家司法院 `NJD` 的 active admins 中过滤 `admin_role=护宪大法官` 取得,且成员数必须恰好 7 人。国家司法院进入创世机构,创世治理账户固定 15 人,角色为 7 名护宪大法官、1 名首席大法官、2 名次席大法官、5 名大法官,内部治理固定阈值为 8/15。

## 已确认范围

- **E1 管理员字段扩展**:机构 admins 记录从「账户/SS58 单值」扩为 `AdminProfile` 结构;个人多签仍为 `AccountId`。
- **E2 护宪生产成员解析**:用 `AdminProfile.admin_role` 实现 `constitution_guard_members()` 生产体——读国家司法院 `NJD` admins,过滤 `admin_role=护宪大法官`,成员数必须恰好 7 人。
- 同一 `admin_role` 字段后续可复用于权责划分(如 NJD 全部 admins 中划出 7 名护宪大法官)与 B3 法定代表人(职务=机构首脑)对齐。

## E1 设计(链端,runtime 二次确认)

1. **AdminProfile 结构**(单一真源,`admin-primitives`):`account`、`admin_cid_number`、`name`、`admin_role`、`term_start`、`term_end`、`source`。
2. **护宪职务常量**:`ADMIN_ROLE_CONSTITUTION_GUARD = 护宪大法官`;护宪成员解析只认该常量。
3. **admin pallets 存储改造**:genesis/public/private 机构管理员集合使用 `Vec<AdminProfile>`;personal 个人多签保持 `Vec<AccountId>`。
4. **链写/链读 DTO**:OnChina `AdminProfileArg`、链镜像 `OnChainAdminProfile`、CitizenApp `AdminProfile` 均使用 `admin_role` 字段名。
5. **B3 法定代表人对齐（已被 2026-07-13 模型取代）**：法定代表人三字段现归 entity `InstitutionInfo`，admins 不保存法定代表人副本；管理员岗位与法定代表人不互相派生。
6. **客户端读取**:CitizenApp 读 admins 的解码结构已同步为 `adminRole` / JSON `admin_role`;字段名不保留 `title` 兼容分支。

## E2 设计(链端)

1. `constitution_guard_members()` 生产实现放 `runtime/src/configs/mod.rs` 的 `RuntimeInternalAdminProvider`(现委托 admins-change 处)。
2. 逻辑:取 NJD 机构 admins → `filter(admin_role == 护宪大法官)` → 必须恰好 7 人。
3. NJD 机构码常量走单一源(china 常量库);护宪大法官职务取值走 `ADMIN_ROLE_CONSTITUTION_GUARD` 单一源。
4. 测试:mock 注 `admin_role` 字段,验证生产解析正好取出 7 名护宪大法官;非护宪职务 admins 不入选;数量异常按硬约束处理。

## 硬规则约束

runtime 二次确认 / 禁止兼容(admins 结构全切,无过渡) / 单一真源(`ADMIN_ROLE_CONSTITUTION_GUARD` + `NJD` 码) / 字段改动同步客户端缓存结构 / 真实运行态验收(随 E3 上链后,不在本卡)。

## 验收

- 聚焦验证已过:`cargo test -p legislation-vote`、`cargo test -p onchina`、`flutter test test/governance/admins-change/admins_change_codec_test.dart`。
- 管理员/机构/多签回归已过:`cargo test -p public-admins -p private-admins -p public-manage -p private-manage -p multisig-transfer`。
- 残留扫描已过:本任务范围内无 `AdminProfile.title`、`title/term`、护宪“多数/过半”旧口径残留。
- 整 runtime check 与真机:留 E3(重新创世与固定治理创世写入收口后)统一验,本卡不跨线程代改。

## 已关闭问题

1. **字段名**:管理员公开职务字段统一为 `admin_role`,不得再使用 `title`。
2. **护宪人数策略**:NJD 中 `admin_role=护宪大法官` 必须恰好 7 人;不足、超出或重复均视为无效成员集。
3. **legal_representative 与 admin_role**：法定代表人公开信息归 entity；当前管理员岗位资料仍待机构岗位/任职模型收口，二者不互相派生。

## 进度

- [x] E1 AdminProfile 结构 + `admin_role` 字段统一
- [x] E1 admin pallets / OnChina / CitizenApp DTO 同步
- [x] E2 `RuntimeInternalAdminProvider::constitution_guard_members()` 生产解析
- [x] E2 护宪 7 人 + 4 名及以上赞成/反对规则
- [x] NJD 创世机构 + 15 名创世治理账户角色 + 8/15 固定阈值
- [x] 聚焦编译 + 单测 + 残留扫描
- [ ] E3 重新创世 + 真机 QA(随整套上链统一处理)
