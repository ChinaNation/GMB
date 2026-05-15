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

- `PersonalManage::propose_create`：pallet `7`，call `0`，字段顺序固定为
  `account_name / duoqian_admins / regular_threshold / amount`。
- `PersonalManage::propose_close`：pallet `7`，call `1`。
- `regular_threshold` 为用户输入的普通提案阈值，App 侧校验范围为
  `floor(admin_count / 2) + 1 ..= admin_count`；注册提案通过阈值固定为全员同意。

PersonalManage ProposalData：

- `MODULE_TAG = b"per-mgmt"`。
- `ACTION_CREATE = 0`：`duoqian_address + proposer + amount + fee`。
- `ACTION_CLOSE = 1`：`duoqian_address + beneficiary + proposer`。

PersonalManage storage：

- `PersonalManage::PersonalDuoqians` 保存 `creator / account_name / created_at / status`。
- 管理员真源是 `AdminsChange::Subjects`，SubjectKind 使用 `0x03 PersonalDuoqian`。
- 普通业务动态阈值真源是 `InternalVote.ActiveDynamicThresholds`；创建/注销生命周期阈值由投票引擎按管理员快照写成全员同意。

## 3.1 wuminapp 本地注销显示规则

- 账户列表页标题显示为“账户列表”。
- 已注销个人多签账户继续留在账户列表，状态显示“已注销”，不显示金额。
- 详情页链上明确查不到 `PersonalManage::PersonalDuoqians` 时，写入本机
  `PersonalDuoqianLocalState.statusClosed`，页面状态显示“已注销”。
- 已注销详情页不显示余额，不再从创建提案快照显示旧入金金额，也不显示“未找到”。
- 已注销详情页右上角三点菜单显示按钮“删除”；确认后删除
  `PersonalDuoqianEntity`、该账户全部 `PersonalDuoqianProposalEntity` 和本地状态键。
- 链路异常只显示加载失败，不把网络失败写成已注销。
- 链上 votingengine 90 天终态提案清理保持不变，wuminapp 不修改链上清理周期。
- 发起创建/注销提案后，runtime 投票引擎会在同一事务自动给发起人记一票赞成；wuminapp 本地提案记录初始 `yesVotes = 1`，不再显示发起人还需要第二次投票。

## 3.2 创建 / 注销阈值 UI

- 新增个人多签页面提供“普通提案阈值”输入框。
- “阈值规则”右侧浅色文案显示“注册须全员同意”。
- 注销个人多签页面“阈值规则”右侧浅色文案显示“注销须全员同意”。
- 扫码添加管理员使用 `assets/icons/scan-line.svg`，不使用二维码图标。
- 账户列表右上角加号弹窗：
  - 新增个人多签：副文案“无需身份ID”。
  - 新增机构多签：副文案“需要身份ID”，图标使用建筑/机构类图标。

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

2026-05-11 第 1 步验收已执行：

```bash
cd wuminapp
flutter analyze
flutter test test/governance/personal-manage
```
