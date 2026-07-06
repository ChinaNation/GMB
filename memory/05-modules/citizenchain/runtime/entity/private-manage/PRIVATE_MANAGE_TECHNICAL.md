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

## 管理员写入边界

- 本模块创建私权机构时把机构 `cid_number`、机构码、主账户和 `AdminProfile` 列表传给 `private-admins`。
- 具体管理员本人、姓名、个人 CID、岗位快照、任期和来源只落在 `private-admins::AdminAccounts`。
- 本模块不保存管理员真源，也不判断岗位产生方式；后续岗位制度只能作为 entity 侧规则供 admins 模块校验。
- 通过本模块创建机构时，对应管理员来源由 `private-admins` 统一落为 `Registry`。

## MODULE_TAG

- `b"pri-mgmt"`

## 钱包扫码

- pallet index：`33`
- 创建动作：`propose_create_private_institution`
- 关闭动作：`propose_close_private_institution`
- 清理动作：`cleanup_rejected_private_proposal`
