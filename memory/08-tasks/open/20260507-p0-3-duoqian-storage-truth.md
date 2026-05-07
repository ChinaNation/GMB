# P0-3 清理 DuoqianAccounts 旧 storage 真源

## 任务目标

执行重新创世前总审计 P0-3：`wuminapp` 不得再读取已删除的 `OrganizationManage::DuoqianAccounts`。注册机构多签、个人多签、管理员阈值查询全部切到当前 runtime 真源。

## 当前真源

- 注册机构地址反查：`OrganizationManage::AddressRegisteredSfid(duoqian_address)`
- 注册机构账户状态：`OrganizationManage::InstitutionAccounts(sfid_number, account_name)`
- 注册机构基本快照：`OrganizationManage::Institutions(sfid_number)`
- 个人多签账户状态：`PersonalManage::PersonalDuoqians(personal_address)`
- 个人多签反向信息：`PersonalManage::PersonalDuoqianInfo(personal_address)`
- 管理员与阈值：`AdminsChange::Subjects(subject_id)`

## 预计修改目录

- `wuminapp/lib/duoqian/shared/`：新增或调整链上 storage key/decoder，替换旧 `DuoqianAccounts` 查询；涉及 Dart 代码，不改链端 runtime。
- `wuminapp/lib/institution/`：修管理员和阈值查询分流；涉及注册机构、个人多签、内置机构三类 subject。
- `wuminapp/test/duoqian/`：增加 storage key、subject id 与 decoder 回归测试；确保旧 storage 不复发。
- `wuminapp/test/institution/`：如需要，补管理员服务查询路由测试。
- `memory/08-tasks/open/`：回写 P0-3 执行结果和总审计记录；只涉及文档。

## 执行清单

- [x] 新增当前 storage 真源的 key 构造和 SCALE decoder。
- [x] `fetchPersonalMeta` 改读 `PersonalManage::PersonalDuoqianInfo`。
- [x] `fetchDuoqianAccount` 改为先查注册机构路径，再查个人多签路径。
- [x] `InstitutionAdminService` 改为按 `duoqian:` / `personal:` / 内置机构三类 subject 查询。
- [x] 删除或停用 `_buildDuoqianAccountsKey`。
- [x] 清理 `wuminapp/lib` 和 `wuminapp/test` 中旧 `DuoqianAccounts` 引用。
- [x] 增加回归测试并运行验收。

## 验收标准

- `rg -n "DuoqianAccounts|OrganizationManage.*PersonalDuoqianInfo" wuminapp/lib wuminapp/test` 无输出。
- `flutter test test/duoqian` 通过。
- `flutter analyze lib/duoqian/shared lib/institution test/duoqian` 通过。
- `git diff --cached --check` 通过。

## 执行结果

2026-05-07 已执行：

- 新增 `wuminapp/lib/duoqian/shared/duoqian_storage_codec.dart`，统一维护 `AddressRegisteredSfid`、`Institutions`、`InstitutionAccounts`、`PersonalDuoqians`、`PersonalDuoqianInfo`、`AdminsChange::Subjects` 的 storage key 与 SCALE 解码。
- `DuoqianManageService.fetchPersonalMeta` 改读 `PersonalManage::PersonalDuoqianInfo`。
- `DuoqianManageService.fetchDuoqianAccount` 改成先走注册机构路径 `AddressRegisteredSfid -> Institutions + InstitutionAccounts`，未命中再走个人多签 `PersonalManage::PersonalDuoqians`。
- `InstitutionAdminService` 改为统一读取 `AdminsChange::Subjects`：`duoqian:` 先反查 SFID 再派生 subject，`personal:` 按账户地址派生 subject，内置机构按机构 id 派生 subject。
- 已清理 `wuminapp/lib` 与 `wuminapp/test` 中旧 `OrganizationManage::DuoqianAccounts` 活跃引用。
- 已补 `wuminapp/test/duoqian/duoqian_storage_codec_test.dart`、`wuminapp/test/duoqian/duoqian_manage_storage_test.dart`、`wuminapp/test/institution/institution_admin_service_test.dart`。

验收记录：

- `flutter test test/duoqian/duoqian_storage_codec_test.dart test/duoqian/duoqian_manage_storage_test.dart test/institution/institution_admin_service_test.dart test/duoqian/duoqian_manage_service_test.dart`：通过。
- `flutter test test/duoqian test/institution`：通过。
- `flutter analyze lib/duoqian/shared lib/institution test/duoqian test/institution`：通过。
- `rg -n "DuoqianAccounts|OrganizationManage.*PersonalDuoqianInfo|AdminsChange\\.Institutions|admins-change Institutions" wuminapp/lib wuminapp/test`：无输出。
