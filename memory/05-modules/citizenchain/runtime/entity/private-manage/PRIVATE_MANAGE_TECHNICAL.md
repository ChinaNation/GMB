# private-manage 技术说明

模块：`private-manage`。职责：私权法人、非法人机构、CID 下账户集合、岗位和任职真源。

## 唯一模型

- 机构唯一主键是 `cid_number`；`Institutions[cid_number]` 保存机构信息。
- `InstitutionAccounts[(cid_number, account_name)]` 是账户正向真源，`AccountRegisteredCid[address]` 只是反向索引；不存在 CID 到账户的重复正向表、生命周期状态或默认账户标志。
- 普通机构必须具有主账户、费用账户；特殊制度账户统一由 `primitives::institution_constraints` 决定。协议账户恰好一个且永久不可关闭，只有 `InstitutionNamed` 可关闭。
- 初始余额允许为零；非零金额必须大于等于 ED。

## 管理员与岗位

- `PrivateAdmins::AdminAccounts[cid_number].admins` 是执行授权唯一真源，主账户不参与授权 key。
- 岗位和任职按 `(cid_number, role_code)` 保存；更新后从有效任职原子重算同一 CID 的 `admins`。
- 普通机构阈值只保存于 `InternalVote::ActiveInstitutionThresholds[cid_number]`。
- 法定代表人只读取 `InstitutionInfo` 三字段，不在 admins 中保存副本。

## 链上入口

- `propose_create_private_institution`（call 5）：注册局管理员以 `actor_cid_number + origin` 创建目标 CID、完整账户集合、岗位、任职与 admins；零初始入金必须不携带 `funding_account`，非零入金必须由同一 actor CID 下明确且可支出的机构账户承担本金。
- `update_institution_info`（call 6）：注册局管理员更新目标机构名称。
- `add_institution_account`（call 7）：注册局管理员给目标 CID 新增自定义账户。
- `propose_close_private_institution`（call 1）：严格使用 `actor_cid_number + institution_account + origin`，并校验账户属于该 CID 且为自定义账户。
- `apply_institution_governance_result` 是内部回调。call 0 与 call 4 永久留洞，不复用、
  不兼容；关闭提案否决、超时或执行失败后的 `InstitutionPendingClose` 只由
  votingengine 终态回调清除，不存在人工清理交易。

## 费用与 ED

- 机构创建、资料更新、新增账户和关闭提案的外层链上操作费只从 `actor_cid_number` 的费用账户收取，管理员钱包只签名。
- 创建时的费用按初始入金合计计算；本金只从显式 `funding_account` 转入新机构各账户，不允许改扣管理员。
- 自定义账户关闭通过后，执行手续费按被关闭账户余额计算并从 actor CID 费用账户收取；被关闭账户以 `AllowDeath` 转出余额。收费、转账和索引删除原子执行。
- 普通支出与费用扣款必须保留 ED；只有明确关闭的 `InstitutionNamed` 账户允许死亡。

## 边界与 ABI

- 只接受私权法人和非法人机构码；跨 namespace CID 重复校验通过 `entity-primitives::InstitutionCidQuery`。
- 外层 origin 必须属于 `AdminAccounts[actor_cid_number].admins`；注册局凭证只作背书，不构成第二授权。
- 本模块不实现投票或转账；投票归 votingengine，机构账户转账归 `multisig`。
- pallet index：`31`；`MODULE_TAG = b"pri-mgmt"`。
- 不兼容旧 storage、旧 call payload、旧解码或旧创世数据。
