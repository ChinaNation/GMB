# private-manage 技术说明

模块：`private-manage`

职责：私权机构生命周期、岗位目录和管理员任职真源。

## 链上入口

- `register_cid_private_institution`
- `propose_create_private_institution`
- `propose_close_private_institution`
- `cleanup_rejected_private_proposal`

内部结果入口（不是外部 extrinsic）：

- `apply_institution_assignment_result`：消费普选、互选或合法任免终态结果。

## 边界

- 只接受私权机构码。
- 不写入创世机构；创世机构本体、岗位和创世任职由 `runtime/genesis/src/institution/seeder.rs` 写入 `public-manage`。
- 管理员生命周期只调用 `private-admins`。
- CID 重复校验通过 `entity-primitives::InstitutionCidQuery` 查询 `public-manage`。
- 不承担多签转账，转账只归 `multisig-transfer`。
- 岗位定义和任职变更归本模块；`private-admins` 只保存由全部有效任职派生的钱包账户集合。
- 关闭机构账户时通过 `public-manage` 的封存表识别创世机构，创世机构永不可按普通私权机构关闭。
- 机构业务状态只解释为占号中、运行中、永久关闭：主账户登记存在但尚无机构记录即占号中，`Active` 即运行中，`Closed` 即永久关闭；关闭后禁止恢复，只能用新 CID 新建机构，名称允许与历史机构相同。
- 节点 `core/node_guard/cid_lifecycle.rs` 以 RAW storage 再次强制 CID 不删除/不复用、公私权不重复和永久关闭终态，runtime 升级不能绕过。

## 法定代表人与管理员边界

- `InstitutionInfo` 已直接保存 `legal_representative_name`、`legal_representative_cid_number`、`legal_representative_account` 三个链上公开字段；运行期注册创建三项必填且全部进入注册局签名域。
- 法定代表人查询唯一读取本模块 `InstitutionInfo`；`private-admins` 不再保存法定代表人副本或 setter。
- 机构岗位和任职关系的唯一真源是本模块 `InstitutionRoles` 与 `InstitutionRoleAssignments`；`private-admins` 只保存由有效任职去重得到的钱包账户集合。
- 普选/互选终态结果由 runtime 路由到本模块；本模块校验机构主账户、岗位状态、任期和账户唯一性，再整体替换目标岗位任职。
- 写入目标岗位后，从机构全部有效岗位任职重新派生 admins；任职与 admins 同事务提交，失败整体回滚，并保持机构既有 Active 动态阈值。

## MODULE_TAG

- `b"pri-mgmt"`

## 钱包扫码

- pallet index：`31`
- 创建动作：`propose_create_private_institution`
- 关闭动作：`propose_close_private_institution`
- 清理动作：`cleanup_rejected_private_proposal`
