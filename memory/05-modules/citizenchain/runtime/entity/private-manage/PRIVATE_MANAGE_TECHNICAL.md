# private-manage 技术说明

模块：`private-manage`。职责：私权法人、非法人机构、CID 下账户集合、岗位和任职真源。

## 唯一模型

- 机构唯一主键是 `cid_number`；`Institutions[cid_number]` 保存机构信息。
- `InstitutionAccounts[(cid_number, account_name)]` 是账户正向真源，`AccountRegisteredCid[address]` 只是反向索引；不存在 CID 到账户的重复正向表、生命周期状态或默认账户标志。
- 普通机构必须具有主账户、费用账户；特殊制度账户统一由 `primitives::institution_constraints` 决定。协议账户恰好一个且永久不可关闭，只有 `InstitutionNamed` 可关闭。
- 第 6 步新机构创建业务必须把全部协议账户以零余额原子建立；后续入金另走账户交易。

## 管理员与岗位

- `PrivateAdmins::AdminAccounts[cid_number].admins` 是机构可任职人员名册，不是执行授权真源；主账户、费用账户和管理员账户均不能单独授权。
- ADR-039 目标授权主体是 `RoleSubject(cid_number, role_code)`。岗位、岗位权限、任职、`InstitutionRoleNonce` 和永久 `UsedRoleCodes` 归本模块；任职只能引用既有管理员。
- CID 顶层能力封顶岗位可授予的 `RoleBusinessPermission`；权限至少区分 `Propose` 与 `Vote`。岗位权限不可修改，变更权限必须删除旧动态岗位并生成新岗位码。
- 动态岗位码固定为 `R_<32 位大写十六进制>`，由 runtime 使用 `GMB_ROLE_V1` 域分隔符生成；调用方不得提供，删除后永不复用。动态岗位只允许依法改 `role_name`。
- 全部机构永久存在唯一可空缺 `LR`，任职只能为 0 或 1；法定代表人三字段必须与 LR 任职原子一致。机构内岗位码和岗位名分别唯一，同名多人属于同一岗位的多个席位；管理员可兼任不同岗位。创世固定岗位码、名和权限不可修改或删除，但创世机构仍可增加普通动态岗位。
- 普通机构阈值只保存于 `InternalVote::ActiveInstitutionThresholds[cid_number]`。
- ADR-039 目标本机构治理、管理员更换、岗位维护和法定代表人任免分别由业务模块登记岗位权限并静态指定投票引擎；不能因为 `actor_cid_number == cid_number` 或属于 admins 就自动取得发起权。
- 注册局登记管理员同样按注册局 `RoleSubject` 授权；仅属于注册局 admins 必须拒绝。
- 法定代表人只读取 `InstitutionInfo` 三字段，不在 admins 中保存副本。
- 中国公民链技术股份有限公司 `GZ018-SFGQ1-201206100-2026` 是受保护私权创世机构：`LR`、`GENESIS_PRODUCT_MANAGER`、`GENESIS_PROGRAMMER` 三岗位的码、名和固定权限不可修改；产品经理、程序员各固定一席，LR 为 0..=1 任职；公司仍可增加普通动态岗位。
- 技术公司依法换人时必须在同一治理结果中更新对应岗位任职；新任人员尚不在 admins 时再原子更新人员名册，已在 admins 时不得为了换岗伪造无关名册变化。执行使用显式 storage transaction 保证全成或全退；法定代表人账户变化还必须与 `InstitutionInfo` 三字段同步。

## 链上入口

- call 5 已永久关闭并从 metadata/QR/钱包解码移除。普通机构创建由第 6 步的新业务模块原子提交 admins、完整零余额协议账户、强制 LR、至少一个初始治理岗位及固定权限、初始任职和初始投票规则；不得恢复旧直接创建载荷。
- `update_institution_info`（call 6）：注册局管理员更新目标机构名称。
- `add_institution_account`（call 7）：注册局管理员给目标 CID 新增自定义账户。
- `propose_institution_governance`（call 8）：本机构管理员发起内部治理提案，可原子替换 `admins`、变更动态岗位/任职、整体设置或清空法定代表人三字段；岗位任职来源必须是 `InstitutionGovernance`，不得伪装成普选、互选或任命结果。
- `register_institution_admins`（call 9）：注册局管理员按注册局授权直接完整替换目标机构 `admins`，用于注册局管理路径，不改岗位任职。
- `propose_close_private_institution`（call 1）：严格使用 `actor_cid_number + institution_account + origin`，并校验账户属于该 CID 且为自定义账户。
- `apply_institution_governance_result` 是内部回调。call 0、call 4 与 call 5 永久留洞，不复用、
  不兼容；关闭提案否决、超时或执行失败后的 `InstitutionPendingClose` 只由
  votingengine 终态回调清除，不存在人工清理交易。

## 费用与 ED

- 资料更新、新增账户、本机构治理、注册局直接登记管理员和关闭提案的外层链上操作费只从 `actor_cid_number` 的费用账户收取，管理员钱包只签名。
- 自定义账户关闭通过后，执行手续费按被关闭账户余额计算并从 actor CID 费用账户收取；被关闭账户以 `AllowDeath` 转出余额。收费、转账和索引删除原子执行。
- 普通支出与费用扣款必须保留 ED；只有明确关闭的 `InstitutionNamed` 账户允许死亡。

## 边界与 ABI

- 只接受私权法人和非法人机构码；跨 namespace CID 重复校验通过 `entity-primitives::InstitutionCidQuery`。
- ADR-039 目标外层 origin 必须属于 admins，并对目标 `RoleSubject` 有有效任职和业务权限；注册局凭证只作背书，不构成第二授权。
- 本模块不实现投票或转账；投票归 votingengine，机构账户转账归 `multisig`。
- 本模块作为 entity 不决定具体业务使用哪个投票引擎；该决定必须静态写在对应业务模块。
- pallet index：`31`；`MODULE_TAG = b"pri-mgmt"`。
- 不兼容旧 storage、旧 call payload、旧解码或旧创世数据。
