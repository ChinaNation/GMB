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

- `propose_create_private_institution`（call 5）：注册局管理员以 `actor_cid_number + origin` 创建目标 CID、完整账户集合、岗位、任职与 admins。
- `update_institution_info`（call 6）：注册局管理员更新目标机构名称。
- `add_institution_account`（call 7）：注册局管理员给目标 CID 新增自定义账户。
- `propose_close_private_institution`（call 1）：严格使用 `actor_cid_number + institution_account + origin`，并校验账户属于该 CID 且为自定义账户。
- `cleanup_rejected_private_proposal`（call 4）：第 1 步暂存；第 4 步由 votingengine 统一清理。
- `apply_institution_governance_result` 是内部回调。call 0 的旧重复注册入口永久留洞，不复用、不兼容。

## 边界与 ABI

- 只接受私权法人和非法人机构码；跨 namespace CID 重复校验通过 `entity-primitives::InstitutionCidQuery`。
- 外层 origin 必须属于 `AdminAccounts[actor_cid_number].admins`；注册局凭证只作背书，不构成第二授权。
- 本模块不实现投票或转账；投票归 votingengine，机构账户转账归 `multisig`。
- pallet index：`31`；`MODULE_TAG = b"pri-mgmt"`。
- 不兼容旧 storage、旧 call payload、旧解码或旧创世数据。
