# public-manage 技术说明

模块：`public-manage`

职责：公权机构生命周期。只负责公权机构 CID 登记、机构创建、机构关闭和被拒提案清理。

## 链上入口

- `register_cid_public_institution`
- `propose_create_public_institution`
- `propose_close_public_institution`
- `cleanup_rejected_public_proposal`

## 边界

- 只接受公权机构码。
- 不通过运行期 extrinsic 写入创世机构；国家储委会、省储委会、省储行、联邦注册局、国家司法院的机构本体由 `genesis-pallet/src/institution.rs` 在创世时直接写入本模块 storage。
- 管理员生命周期只调用 `public-admins`。
- CID 重复校验通过 `entity-primitives::InstitutionCidQuery` 查询 `private-manage`。
- 不承担多签转账，转账只归 `multisig-transfer`。
- 不承担管理员集合变更，管理员真源只归 `public-admins`。
- 关闭机构账户时读取本模块 `ProtectedGenesisAccounts` 封存表，创世机构永不可按普通公权机构关闭。
- 机构业务状态只解释为占号中、运行中、永久关闭：主账户登记存在但尚无机构记录即占号中，`Active` 即运行中，`Closed` 即永久关闭；关闭后禁止恢复，只能用新 CID 新建机构，名称允许与历史机构相同。
- 节点 `core/node_guard/cid_lifecycle.rs` 以 RAW storage 再次强制 CID 不删除/不复用、公私权不重复、固定机构保持 Active、封存创世账户索引不变，runtime 升级不能绕过。

## 法定代表人与管理员边界

- `InstitutionInfo` 已直接保存 `legal_representative_name`、`legal_representative_cid_number`、`legal_representative_account` 三个链上公开字段；运行期注册创建三项必填且全部进入注册局签名域。
- 创世没有真实任免资料时三项统一写 `None`，不得伪造，也不得从首位管理员回退。
- 法定代表人查询唯一读取本模块 `InstitutionInfo`；`public-admins` 不再保存法定代表人副本或 setter。
- 本步骤不改变当前管理员资料布局；机构岗位、任职关系和 `admins` 账户集合收口属于下一步，未执行前不得把目标模型写成当前实现。

## MODULE_TAG

- `b"pub-mgmt"`

## 钱包扫码

- pallet index：`30`
- 创建动作：`propose_create_public_institution`
- 关闭动作：`propose_close_public_institution`
- 清理动作：`cleanup_rejected_public_proposal`
