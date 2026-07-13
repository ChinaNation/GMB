# private-manage 技术说明

模块：`private-manage`

职责：私权机构生命周期。只负责私权机构 CID 登记、机构创建、机构关闭和被拒提案清理。

## 链上入口

- `register_cid_private_institution`
- `propose_create_private_institution`
- `propose_close_private_institution`
- `cleanup_rejected_private_proposal`

## 边界

- 只接受私权机构码。
- 不写入创世机构；创世机构本体由 `genesis-pallet/src/institution.rs` 在创世时写入 `public-manage`。
- 管理员生命周期只调用 `private-admins`。
- CID 重复校验通过 `entity-primitives::InstitutionCidQuery` 查询 `public-manage`。
- 不承担多签转账，转账只归 `multisig-transfer`。
- 不承担管理员集合变更，管理员真源只归 `private-admins`。
- 关闭机构账户时通过 `public-manage` 的封存表识别创世机构，创世机构永不可按普通私权机构关闭。
- 机构业务状态只解释为占号中、运行中、永久关闭：主账户登记存在但尚无机构记录即占号中，`Active` 即运行中，`Closed` 即永久关闭；关闭后禁止恢复，只能用新 CID 新建机构，名称允许与历史机构相同。
- 节点 `core/node_guard/cid_lifecycle.rs` 以 RAW storage 再次强制 CID 不删除/不复用、公私权不重复和永久关闭终态，runtime 升级不能绕过。

## 管理员写入边界

- 所有私权机构的 `InstitutionInfo` 必须保存 `legal_representative_name`、`legal_representative_cid_number`、`legal_representative_account` 三个链上公开字段。
- 本模块管理机构岗位定义、岗位权限和机构管理员任职关系；任职关系绑定 `cid_number + admin_account + role_code`。
- 创建机构时只把管理员钱包账户集合 `admins` 传给 `private-admins`；姓名、CID、岗位、任期和来源不再内嵌到管理员集合。
- 通过本模块创建机构时，对应管理员来源由 `private-admins` 统一落为 `Registry`。

## MODULE_TAG

- `b"pri-mgmt"`

## 钱包扫码

- pallet index：`31`
- 创建动作：`propose_create_private_institution`
- 关闭动作：`propose_close_private_institution`
- 清理动作：`cleanup_rejected_private_proposal`
