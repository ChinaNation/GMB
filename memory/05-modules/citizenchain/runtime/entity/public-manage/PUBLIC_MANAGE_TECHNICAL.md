# public-manage 技术说明

模块：`public-manage`。职责：公权机构、CID 下账户集合、岗位目录与管理员任职真源。

## 唯一身份与存储

- 机构唯一主键是 `cid_number`；机构码只从 CID 解析，主账户不得作为身份或管理员 key。
- `Institutions[cid_number]` 保存机构信息；`InstitutionAccounts[(cid_number, account_name)]` 是账户正向真源；`AccountRegisteredCid[address]` 仅作反向索引。
- 不保存机构/账户生命周期状态、默认账户标志、CID 到账户的重复正向表或创世保护旁路。
- 普通机构强制主账户和费用账户；特殊机构由 `primitives::institution_constraints::required_protocol_account_kinds` 返回完整协议账户集合。每种协议账户恰好一个且永远不可关闭，只有 `InstitutionNamed` 自定义账户可关闭。
- 逻辑账户允许零余额；第 6 步新机构创建业务必须把全部协议账户以零余额原子建立，后续入金另走账户交易。

## 管理员、岗位与授权

- `PublicAdmins::AdminAccounts[cid_number].admins` 是机构可任职人员名册，每项为统一的 `Admin { account_id, cid_number, family_name, given_name }`。非空 CID 必须引用 `citizen-identity` 的 CTZN CID↔账户绑定。该名册不是机构业务授权真源，主账户、费用账户和管理员账户均不能单独授权。
- ADR-039 目标授权主体是 `RoleSubject(cid_number, role_code)`。`InstitutionRoles`、岗位权限、`InstitutionRoleAssignments`、`InstitutionRoleNonce` 与永久 `UsedRoleCodes` 归本模块；任职只能引用既有管理员。
- CID 顶层能力封顶岗位可授予的 `RoleBusinessPermission`；业务动作权限至少区分 `Propose` 与 `Vote`。岗位权限不可原地修改，变更权限必须删除旧动态岗位并生成新岗位码。
- 动态岗位码固定为 `R_<32 位大写十六进制>`，由 runtime 使用本 pallet `MODULE_TAG`(`b"pub-mgmt"`) 作哈希域生成；调用方不得提供，删除后永不复用。动态岗位只允许依法改 `role_name`。
- 全部机构永久存在唯一可空缺 `LR`，任职只能为 0 或 1；法定代表人原子结构必须与 LR 任职一致。机构内岗位码和岗位名分别唯一，同名多人属于同一岗位的多个席位；管理员可兼任不同岗位。创世固定岗位码、名和权限不可修改或删除，但创世机构仍可增加普通动态岗位。
- `InstitutionGovernanceThresholds[cid_number]` 是公权机构治理阈值真源，与 admins 钱包数、岗位数分别独立。投票引擎只在建案时读取并冻结提案阈值快照。
- ADR-039 目标外层标准 extrinsic 必须同时满足 origin 属于 admins、对指定 `RoleSubject` 有有效任职且岗位拥有目标业务权限。注册局凭证只表达业务背书，不得成为第二授权真源。
- 本机构治理、管理员更换、岗位维护和法定代表人任免分别由业务模块登记权限并静态指定投票引擎；不能因为 `actor_cid_number == cid_number` 或属于 admins 就自动取得发起权。
- 注册局登记管理员同样按注册局岗位主体授权；仅属于注册局 admins 必须拒绝。
- 法定代表人只读取 `InstitutionInfo` 三字段；创世没有真实资料时统一为 `None`，不得从管理员或主账户推导。

## 链上入口

- call 5 已永久关闭并从 metadata/QR/钱包解码移除。普通机构创建由第 6 步的新业务模块原子提交 admins、完整零余额协议账户、强制 LR、至少一个初始治理岗位及固定权限、初始任职和初始投票规则；不得恢复旧直接创建载荷。
- `update_institution_info`（call 6）：注册局管理员更新目标机构名称。
- `add_institution_account`（call 7）：注册局管理员给目标 CID 批量新增自定义账户。
- `propose_institution_governance`（call 8）：本机构指定岗位任职人发起内部治理提案；SCALE 在 `actor_cid_number` 后固定编码独立 `proposer_role_code`。入口校验完整 `RoleSubject + pub-mgmt/3 + Propose`，再按同一 CID 拥有 `Vote` 权限的岗位构造内部 `VotePlan`。通过后可原子替换 `admins`、变更动态岗位/任职、整体设置或清空法定代表人结构；岗位任职来源必须是 `InstitutionGovernance`，不得伪装成普选、互选或任命结果。
- `register_institution_admins`（call 9）：注册局管理员按注册局授权直接完整替换目标机构 `admins`，用于注册局管理路径，不改岗位任职。
- `propose_close_public_institution`（call 1）：账户型交易，严格使用 `actor_cid_number + proposer_role_code + institution_account + origin`；只有拥有 `pub-mgmt/2 + Propose` 的有效岗位任职人可发起，投票主体来自拥有对应 `Vote` 权限的岗位，只允许关闭该 CID 下自定义账户。
- `apply_institution_governance_result` 是内部回调，不是 extrinsic。
- call 0、call 4 与 call 5 永久留洞，不复用、不兼容；关闭提案否决、超时或执行失败后的
  `InstitutionPendingClose` 只由 votingengine 终态回调清除，不存在人工清理交易。

## 费用与 ED

- 资料更新、新增账户、本机构治理、注册局直接登记管理员和关闭提案的外层链上操作费只从 `actor_cid_number` 的费用账户收取，管理员钱包只签名。
- 自定义账户关闭通过后，执行手续费按被关闭账户余额计算并从 actor CID 费用账户收取；随后仅被关闭账户以 `AllowDeath` 把余额转给受益人。收费、转账和账户索引删除处于同一事务。
- 普通支出和费用账户扣款都必须保留 ED；只有显式关闭的 `InstitutionNamed` 账户允许死亡。

## 模块边界

- 只接受公权机构码；跨公私权 CID 重复校验通过 `entity-primitives::InstitutionCidQuery`。
- 创世机构本体、协议账户、岗位和初始任职由 `runtime/genesis/src/institution/seeder.rs` 写入相同真源并校验完整账户集合。
- 本模块不实现投票或转账；投票统一归 votingengine，机构账户转账归 `multisig`。
- 本模块作为 entity 只提供岗位、权限、任职和机构生命周期真源；具体业务模块决定动作权限、静态选择投票引擎并执行通过后的业务。
- 关闭执行器必须处于 votingengine callback scope，并重新校验提案 owner、CID、账户归属、已绑定 `RoleSubject` 授权、协议账户不可关闭与受益人。

## ABI

- pallet index：`30`
- `MODULE_TAG = b"pub-mgmt"`
- 不保留旧 storage、旧 call payload 或旧解码兼容；开发期重新创世。
