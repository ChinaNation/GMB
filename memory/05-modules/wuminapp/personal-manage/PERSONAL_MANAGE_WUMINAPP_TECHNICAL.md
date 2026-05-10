# wuminapp personal-manage 技术文档

## 1. 模块定位

`wuminapp/lib/governance/personal-manage/` 是 wuminapp 端个人多签主业务目录，对齐 runtime `citizenchain/runtime/governance/personal-manage/`。

本目录只处理个人多签，不承载机构多签、机构 SFID 账户、多签转账业务。

## 2. 当前边界

### 负责

- 个人多签创建页面：`personal_duoqian_create_page.dart`
- 个人多签关闭页面：`personal_duoqian_close_page.dart`
- 个人多签账户列表页：`personal_manage_account_list_page.dart`
- 个人多签账户详情页：`personal_manage_account_info_page.dart`
- 个人多签反向索引发现服务：`personal_manage_discovery_service.dart`
- 个人多签管理员激活列表：`personal_admin_list_page.dart`
- 待激活创建提案反查：`personal_pending_create_lookup.dart`
- 个人提案历史聚合与 Isar 持久化：`personal_proposal_history_service.dart`
- 详情页提案列表组件：`personal_proposal_list_section.dart`
- PersonalManage call data、ProposalData 解码和链上查询：`personal_manage_service.dart`
- PersonalManage storage key 与 SCALE 解码：`personal_manage_storage_codec.dart`
- PersonalManage 提案详情模型：`personal_manage_models.dart`

### 不负责

- 机构多签创建、关闭、SFID 机构账户查询：继续由 `wuminapp/lib/governance/organization-manage/` 机构路径处理。
- 多签转账：唯一实现目录仍是 `wuminapp/lib/transaction/duoqian-transfer/`。
- Isar schema 定义：仍在 `wuminapp/lib/isar/`，本模块只使用既有实体。
- 通用投票、签名、RPC：仍使用 `proposal/shared`、`signer`、`rpc` 等共用能力。
- 个人/机构多签管理提案投票详情页：共用入口位于 `wuminapp/lib/governance/duoqian_manage_detail_page.dart`，本模块只提供 PersonalManage 解码服务。

## 3. 链上契约

PersonalManage 交易载荷：

- `PersonalManage::propose_create`：pallet `7`，call `0`。
- `PersonalManage::propose_close`：pallet `7`，call `1`。

PersonalManage ProposalData：

- `MODULE_TAG = b"per-mgmt"`。
- `ACTION_CREATE = 0`：`duoqian_address + proposer + amount + fee`。
- `ACTION_CLOSE = 1`：`duoqian_address + beneficiary + proposer`。

PersonalManage storage：

- `PersonalManage::PersonalDuoqians` 保存 `creator / account_name / created_at / status`。
- 管理员和阈值真源仍是 `AdminsChange::Subjects`，SubjectKind 使用 `0x03 PersonalDuoqian`。

## 4. 与 organization-manage 目录关系

`wuminapp/lib/governance/organization-manage/` 不再承载 PersonalManage 主业务。当前仅保留：

- 机构多签 OrganizationManage 服务与机构 storage codec。
- `AdminInstitutionCodec` 等跨个人/机构都需要读取的底层 Subject 解码能力。

个人账户列表、账户详情、反向索引发现、创建、关闭、管理员激活和提案历史均不得回流到 `organization-manage`。
`AdminInstitutionCodec` 只属于底层 Subject 解码能力，不承载 PersonalManage 主业务。

## 5. 测试

个人多签测试集中在：

- `wuminapp/test/governance/personal-manage/personal_manage_service_test.dart`
- `wuminapp/test/governance/personal-manage/personal_manage_storage_codec_test.dart`
- `wuminapp/test/governance/personal-manage/personal_manage_discovery_service_test.dart`
- `wuminapp/test/governance/personal-manage/personal_pending_create_lookup_test.dart`
- `wuminapp/test/governance/personal-manage/personal_proposal_history_service_test.dart`

本轮拆分验收命令：

```bash
cd wuminapp
flutter analyze
flutter test test/organization-manage test/personal-manage
```
