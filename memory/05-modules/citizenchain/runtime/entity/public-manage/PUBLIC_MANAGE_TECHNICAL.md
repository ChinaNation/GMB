# public-manage 技术说明

模块：`public-manage`。职责：公权机构、CID 下账户集合、岗位目录与管理员任职真源。

## 唯一身份与存储

- 机构唯一主键是 `cid_number`；机构码只从 CID 解析，主账户不得作为身份或管理员 key。
- `Institutions[cid_number]` 保存机构信息；`InstitutionAccounts[(cid_number, account_name)]` 是账户正向真源；`AccountRegisteredCid[address]` 仅作反向索引。
- 不保存机构/账户生命周期状态、默认账户标志、CID 到账户的重复正向表或创世保护旁路。
- 普通机构强制主账户和费用账户；特殊机构由 `primitives::institution_constraints::required_protocol_account_kinds` 返回完整协议账户集合。每种协议账户恰好一个且永远不可关闭，只有 `InstitutionNamed` 自定义账户可关闭。
- 逻辑账户允许零余额；非零初始金额必须大于等于 ED。

## 管理员、岗位与授权

- `PublicAdmins::AdminAccounts[cid_number].admins` 是机构执行授权唯一真源。
- `InstitutionRoles[(cid_number, role_code)]` 与 `InstitutionRoleAssignments[(cid_number, role_code)]` 是岗位和任职真源；有效任职变化原子刷新同一 CID 的 `admins`。
- 普通机构动态阈值写 `InternalVote::ActiveInstitutionThresholds[cid_number]`；固定治理机构使用制度阈值。
- 外层标准 extrinsic `origin` 必须属于 `actor_cid_number` 的 `admins`。注册局凭证只表达业务背书，不得成为第二授权真源。
- 法定代表人只读取 `InstitutionInfo` 三字段；创世没有真实资料时统一为 `None`，不得从管理员或主账户推导。

## 链上入口

- `propose_create_public_institution`（call 5）：注册局管理员以 `actor_cid_number + origin` 创建目标 CID、完整协议账户、岗位、任职与 admins；零初始入金必须不携带 `funding_account`，非零入金必须由同一 actor CID 下明确且可支出的机构账户承担本金。
- `update_institution_info`（call 6）：注册局管理员更新目标机构名称。
- `add_institution_account`（call 7）：注册局管理员给目标 CID 批量新增自定义账户。
- `propose_close_public_institution`（call 1）：账户型交易，严格使用 `actor_cid_number + institution_account + origin`，只允许关闭该 CID 下自定义账户。
- `apply_institution_governance_result` 是内部回调，不是 extrinsic。
- call 0 与 call 4 永久留洞，不复用、不兼容；关闭提案否决、超时或执行失败后的
  `InstitutionPendingClose` 只由 votingengine 终态回调清除，不存在人工清理交易。

## 费用与 ED

- 机构创建、资料更新、新增账户和关闭提案的外层链上操作费只从 `actor_cid_number` 的费用账户收取，管理员钱包只签名。
- 创建时的费用按初始入金合计计算；本金只从显式 `funding_account` 转入新机构各账户，收费与本金都不允许回落管理员。
- 自定义账户关闭通过后，执行手续费按被关闭账户余额计算并从 actor CID 费用账户收取；随后仅被关闭账户以 `AllowDeath` 把余额转给受益人。收费、转账和账户索引删除处于同一事务。
- 普通支出和费用账户扣款都必须保留 ED；只有显式关闭的 `InstitutionNamed` 账户允许死亡。

## 模块边界

- 只接受公权机构码；跨公私权 CID 重复校验通过 `entity-primitives::InstitutionCidQuery`。
- 创世机构本体、协议账户、岗位和初始任职由 `runtime/genesis/src/institution/seeder.rs` 写入相同真源并校验完整账户集合。
- 本模块不实现投票或转账；投票统一归 votingengine，机构账户转账归 `multisig`。
- 关闭执行器必须处于 votingengine callback scope，并重新校验提案 owner、CID、账户归属、管理员授权、协议账户不可关闭与受益人。

## ABI

- pallet index：`30`
- `MODULE_TAG = b"pub-mgmt"`
- 不保留旧 storage、旧 call payload 或旧解码兼容；开发期重新创世。
