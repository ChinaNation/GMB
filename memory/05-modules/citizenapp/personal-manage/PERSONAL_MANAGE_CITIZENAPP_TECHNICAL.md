# citizenapp personal-manage 技术文档

## 1. 模块定位

`citizenapp/lib/transaction/personal-manage/` 是 citizenapp 端个人多签主业务目录，对齐 runtime `citizenchain/runtime/admins/personal-admins/`。

本目录只处理个人多签，不承载机构多签、机构 CID 账户、多签转账业务。

## 2. 当前边界

### 负责

- 个人多签创建页面：`personal_account_create_page.dart`
- 个人多签关闭页面：`personal_account_close_page.dart`
- 个人多签列表展示：统一由 `citizenapp/lib/citizen/shared/institution_account_list_page.dart` 与机构多签合并展示
- 个人多签账户详情页：`personal_manage_account_info_page.dart`
- 个人多签反向索引发现服务：`personal_manage_discovery_service.dart`
- 个人多签管理员激活列表：`personal_admin_list_page.dart`
- 待激活创建提案反查：`personal_pending_create_lookup.dart`
- 个人提案历史聚合与 Isar 持久化：`personal_proposal_history_service.dart`
- 详情页提案列表组件：`personal_proposal_list_section.dart`
- `PersonalAdmins` call data、ProposalData 解码和链上查询：`personal_manage_service.dart`
- `PersonalAdmins` storage key 与 SCALE 解码：`personal_manage_storage_codec.dart`
- `PersonalAdmins` 提案详情模型：`personal_manage_models.dart`

### 不负责

- 机构多签创建、关闭、CID 机构账户查询：继续由 `citizenapp/lib/transaction/organization-manage/` 机构路径处理。
- 多签转账：唯一实现目录仍是 `citizenapp/lib/citizen/proposal/transaction/`。
- Isar schema 定义：仍在 `citizenapp/lib/isar/`，本模块只使用既有实体。
- Isar 读写队列：由 `citizenapp/lib/isar/wallet_isar.dart` 统一提供，本模块不得直接打开 DB 实例。
- 通用投票、签名、RPC：仍使用 `citizen/shared/proposal`、`signer`、`rpc` 等共用能力。
- 个人/机构多签管理提案投票详情页由 `citizen/proposal/transaction` 的提案聚合和各管理模块解码服务共同支撑；本模块只提供 `PersonalAdmins` 解码服务。

## 3. 链上契约

PersonalAdmins 交易载荷：

- `PersonalAdmins::propose_create`：pallet `7`，call `0`，字段顺序固定为
  `account_name / admins / regular_threshold / amount`。
- `PersonalAdmins::propose_close`：pallet `7`，call `1`。
- `PersonalAdmins::cleanup_rejected_proposal`：pallet `7`，call `2`。
- `PersonalAdmins::propose_admin_set_change`：pallet `7`，call `3`，字段顺序固定为
  `institution_code / account_id / admins / new_threshold`，`institution_code` 必须为 `PMUL`。
- `regular_threshold` 为用户输入的普通提案阈值，App 侧校验范围为
  `floor(admins_len / 2) + 1 ..= admins_len`；注册提案通过阈值固定为全员同意。

PersonalAdmins ProposalData：

- `MODULE_TAG = b"per-mgmt"`。
- `ACTION_CREATE = 0`：`account + proposer + amount + fee`。
- `ACTION_CLOSE = 1`：`account + beneficiary + proposer`。

PersonalAdmins storage：

- `PersonalAdmins::PersonalAccounts` 保存 `creator / account_name / created_at / status`。
- 管理员真源是 `PersonalAdmins::AdminAccounts`，AdminAccountKind 使用 `PersonalMultisig`。
- 普通业务动态阈值真源是 `InternalVote.ActiveDynamicThresholds`；创建/注销生命周期阈值由投票引擎按管理员快照写成全员同意。
- 详情页和管理员列表不得从 `PersonalAdmins::AdminAccounts` 后续字段解阈值；该 storage
  的管理员列表后面是 `creator / created_at / updated_at / status`，错位读取会出现
  类似 `1478971204/2` 的异常阈值显示。

个人多签创建提交规则：

- 创建前必须校验发起钱包 free 余额覆盖 `amount + fee + ED`。
- `fee` 使用链上 `onchain_transaction::calculate_onchain_fee` 同口径：
  `max(amount * 0.1%, 0.10 元)`；`ED` 当前为 `1.11 元`。
- `author_submitExtrinsic` 返回的 txHash 只代表交易已提交到节点，不代表创建提案成功。
- `personal_manage_service.dart` 必须使用 `signAndSubmitInBlock()` 等待入块，并从
  `System.Events` 确认 `PersonalAdmins.PersonalAccountProposed(event_index=0)`。
- 如果入块区块包含 `System.ExtrinsicFailed`，必须优先显示真实模块错误，例如
  `PersonalAdmins.InsufficientAmount` 或 `GenesisAdmins.InstitutionAlreadyExists`，
  不能只提示“未找到 PersonalAccountProposed 事件”。
- 本地 `PersonalAccountEntity` 和 `PersonalAccountProposalEntity` 只能在确认事件后写入，
  `proposalId` 必须来自链上事件，不允许预测 `VotingEngine.NextProposalId`。

## 3.1 citizenapp 本地注销显示规则

- 底部 `多签` Tab 标题显示为“多签”，统一展示个人多签与机构多签。
- 已注销个人多签账户继续留在账户列表，状态显示“已注销”，不显示金额。
- 详情页链上明确查不到 `PersonalAdmins::PersonalAccounts` 时，写入本机
  `PersonalAccountLocalState.statusClosed`，页面状态显示“已注销”。
- 已注销详情页不显示余额，不再从创建提案快照显示旧入金金额，也不显示“未找到”。
- Active 详情页右上角三点菜单显示纯文本“关闭个人多签”，不显示删除图标，避免把关闭提案误解为本机删除。
- 已注销详情页右上角三点菜单显示按钮“删除”；确认后删除
  `PersonalAccountEntity`、该账户全部 `PersonalAccountProposalEntity` 和本地状态键。
- 链路异常不把网络失败写成已注销；详情页首屏不得因链上异常显示全屏加载失败。
- 链上注销成功后 runtime 会清空多签账户余额并删除个人多签当前状态；同一钱包地址 +
  同一账户名再次创建仍派生同一地址，但会作为全新的当前账户注册。
- 链上 votingengine 90 天终态提案清理保持不变，citizenapp 不修改链上清理周期。
- 发起创建/注销提案后，runtime 投票引擎会在同一事务自动给发起人记一票赞成；citizenapp 本地提案记录初始 `yesVotes = 1`，不再显示发起人还需要第二次投票。
- 若旧版本已写入“本地 create 提案仍为 voting，但链上 Proposals[id] 不存在”的记录，
  该记录视为未上链幽灵数据，列表同步时删除本地多签和提案快照，不显示为“已注销/未知提案”。
- 个人多签历史、待激活创建提案反查、反向索引发现和本地状态更新全部通过
  `WalletIsar.instance.read()` / `WalletIsar.instance.writeTxn()` 进入统一队列，避免与钱包创建/导入、余额刷新和钱包交易流水同步抢 MDBX 锁。
- 统一多签列表首屏只读本机 `PersonalAccountEntity` 和 `PersonalAccountLocalState`，
  不等待 `PersonalAdmins::PersonalAccounts`、`PersonalAdmins::AdminAccounts` 或 discovery 链上读取。
- `PersonalAccountLocalState` 复用 `AppKvEntity.stringValue` 保存状态，
  `AppKvEntity.intValue` 保存最近一次成功链上状态同步时间。
- 个人多签详情页额外使用 `personal_account_detail:*` 本机持久化快照保存管理员公钥、
  阈值、余额和最近链上刷新时间；进入详情页先显示本地快照，不为了读取
  `PersonalAdmins::PersonalAccounts`、`PersonalAdmins::AdminAccounts` 或 `InternalVote`
  阈值而全屏等待。
- 个人多签详情页 Active 余额使用 `lastBalanceRefreshAtMillis` 单独判断；若
  `balanceYuan` 为空或余额时间过期，只静默读取余额，不重复拉账户状态、管理员和阈值。
  列表页批量状态刷新不得覆盖已有余额快照。
- 详情页不显示“同步中”类 UI；TTL 到期时静默刷新，用户下拉刷新、转账/投票/关闭
  返回时才强制刷新当前个人多签。链上失败保留本机快照，不覆盖为已注销。
- Active 个人多签 60 分钟内不自动重复查链；Pending / Closed 个人多签
  10 分钟内不自动重复查链；用户下拉刷新才强制忽略 TTL。
- 自动 discovery 只在首次进入多签 Tab 或本机钱包 pubkey fingerprint 变化时触发；
  下拉刷新才强制执行全量 discovery。
- 个人多签列表状态刷新使用 `PersonalManageService.fetchPersonalAccountsBatch()`：
  先批量读取 `PersonalAccounts` 与 `PersonalAdmins::AdminAccounts`，再按解码出的
  `institution_code + account_id` 批量读取 `InternalVote.ActiveDynamicThresholds`，缺失时再批量读取
  `PendingDynamicThresholds`。
- 列表页从个人多签详情、创建、关闭、投票返回时只刷新相关账户或本地记录，不重新扫描全部多签。

## 3.2 创建 / 注销阈值 UI

- 新增个人多签页面提供“普通提案阈值”输入框。
- “阈值规则”右侧浅色文案显示“注册须全员同意”。
- 注销个人多签页面“阈值规则”右侧浅色文案显示“注销须全员同意”。
- 扫码添加管理员使用 `assets/icons/scan-line.svg`，不使用二维码图标。
- 账户列表右上角加号弹窗：
  - 新增个人多签：副文案“无需身份ID”。
  - 新增机构多签：副文案“需要身份ID”，图标使用建筑/机构类图标。

## 4. 与 organization-manage 目录关系

`citizenapp/lib/transaction/organization-manage/` 不再承载 `PersonalAdmins` 主业务。当前仅保留：

- 机构多签 OrganizationManage 服务与机构 storage codec。
- `AdminInstitutionCodec` 等跨个人/机构都需要读取的底层 Subject 解码能力。

个人账户详情、反向索引发现、创建、关闭、管理员激活和提案历史均不得回流到 `organization-manage`。
个人多签列表入口只允许通过 `lib/citizen/shared/institution_account_list_page.dart` 统一呈现。
`AdminInstitutionCodec` 只属于底层 Subject 解码能力，不承载 `PersonalAdmins` 主业务。

## 5. 测试

个人多签测试集中在：

- `citizenapp/test/governance/personal-manage/personal_manage_service_test.dart`
- `citizenapp/test/governance/personal-manage/personal_manage_storage_codec_test.dart`
- `citizenapp/test/governance/personal-manage/personal_manage_discovery_service_test.dart`
- `citizenapp/test/governance/personal-manage/personal_pending_create_lookup_test.dart`
- `citizenapp/test/governance/personal-manage/personal_proposal_history_service_test.dart`

本轮拆分验收命令：

```bash
cd citizenapp
flutter analyze
flutter test test/organization-manage test/personal-manage
```

2026-05-11 第 1 步验收已执行：

```bash
cd citizenapp
flutter analyze
flutter test test/governance/personal-manage
```
