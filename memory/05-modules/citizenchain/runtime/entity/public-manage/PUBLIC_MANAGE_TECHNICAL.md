# public-manage 技术说明

模块：`public-manage`

职责：公权机构生命周期、岗位目录和管理员任职真源。

## 链上入口

- `register_cid_public_institution`
- `propose_create_public_institution`
- `propose_close_public_institution`
- `cleanup_rejected_public_proposal`

内部结果入口（不是外部 extrinsic）：

- `apply_institution_governance_result`：消费已经完成业务流程的通用机构治理结果。

## 边界

- 只接受公权机构码。
- 不通过运行期 extrinsic 写入创世机构；国家储委会、省储委会、省储行、联邦注册局、国家司法院的机构本体、岗位和创世任职由 `runtime/genesis/src/institution/seeder.rs` 直接写入本模块 storage。
- 管理员生命周期只调用 `public-admins`。
- CID 重复校验通过 `entity-primitives::InstitutionCidQuery` 查询 `private-manage`。
- 不承担多签转账，转账只归 `multisig-transfer`。
- 岗位定义和任职变更归本模块；`public-admins` 只保存由全部有效任职派生的钱包账户集合。
- 关闭机构账户时读取本模块 `ProtectedGenesisAccounts` 封存表，创世机构永不可按普通公权机构关闭。
- 机构业务状态只解释为占号中、运行中、永久关闭：主账户登记存在但尚无机构记录即占号中，`Active` 即运行中，`Closed` 即永久关闭；关闭后禁止恢复，只能用新 CID 新建机构，名称允许与历史机构相同。
- 节点 `core/node_guard/cid_lifecycle.rs` 以 RAW storage 再次强制 CID 不删除/不复用、公私权不重复、固定机构保持 Active、封存创世账户索引不变，runtime 升级不能绕过。
- 关闭执行器必须处于投票引擎 callback scope，并绑定本模块 owner、内部投票 kind/stage、机构码、管理员根账户、CID、目标账户作用域和 `PendingCloseProposal`；执行时重新检查 active 状态、创世保护/永久单例和受益人。

## 法定代表人与管理员边界

- `InstitutionInfo` 已直接保存 `legal_representative_name`、`legal_representative_cid_number`、`legal_representative_account` 三个链上公开字段；运行期注册创建三项必填且全部进入注册局签名域。
- 创世没有真实任免资料时三项统一写 `None`，不得伪造，也不得从首位管理员回退。
- 法定代表人查询唯一读取本模块 `InstitutionInfo`；`public-admins` 不再保存法定代表人副本或 setter。
- 机构岗位和任职关系的唯一真源是本模块 `InstitutionRoles` 与 `InstitutionRoleAssignments`；`public-admins` 只保存由有效任职去重得到的钱包账户集合。
- runtime 按机构码把 `InstitutionGovernanceResult` 路由到本模块；单个结果可原子包含动态岗位定义变化、多个岗位的完整目标任职集合和法定代表人三字段整体更新。
- 每条任职独立携带任期、制度来源和追溯引用；未出现在结果中的岗位保持不变，动态岗位允许暂时空缺，停用岗位必须清空任职。
- 只有机构码、创世 CID 和主账户同时命中保护清单的 89 个创世核心机构禁止修改岗位定义，并逐岗位强制治理骨架席位；依法轮换只改变任职账户及其任期、来源、引用。一般机构只执行动态治理规则。
- 完整校验后，从机构全部有效岗位任职重新派生 admins；岗位、任职、法定代表人和 admins 同事务提交，失败整体回滚。普通动态机构保持既有 Active 阈值，固定五类治理机构保持代码级固定阈值，六个国家单例不建立账户级阈值。
- 89 个受保护创世机构的岗位代码、名称、所属 CID、席位和有效任职集合由 Node Guard 独立保护；一般机构管理员人数、岗位和组织结构不进入 Node Guard。法定代表人不属于该守卫。
- PRS、NLG、NSN、NRP、NSP、NED 的机构码只能由约定 block#0 CID 和主账户占用，运行期注册与注销入口均拒绝这六类单例。
- 六个国家单例创世均不预设岗位或 admins，保持“尚未组成”。首次有效治理结果在同一事务写入岗位、任职和 admins，不写动态阈值。
- NSN/NRP/NED 首次组成还必须满足指定成员岗位、法定人数区间及 admins 完全相等；组成后停用指定岗位、低于下限、高于上限或通过辅助岗位引入非成员 admin 均拒绝并整体回滚。
- NLG 由 NSN 与 NRP 组成，但 NLG 自身以及 NSP、PRS 不设置岗位或管理员人数永久限制；这些运行期结构继续由业务治理增删改。六个单例的一般内部事项门槛由 `internal-vote` 按每次提案 admins 快照计算严格过半。

## MODULE_TAG

- `b"pub-mgmt"`

## 钱包扫码

- pallet index：`30`
- 创建动作：`propose_create_public_institution`
- 关闭动作：`propose_close_public_institution`
- 清理动作：`cleanup_rejected_public_proposal`
