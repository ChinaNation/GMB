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

## MODULE_TAG

- `b"pri-mgmt"`

## 钱包扫码

- pallet index：`33`
- 创建动作：`propose_create_private_institution`
- 关闭动作：`propose_close_private_institution`
- 清理动作：`cleanup_rejected_private_proposal`
