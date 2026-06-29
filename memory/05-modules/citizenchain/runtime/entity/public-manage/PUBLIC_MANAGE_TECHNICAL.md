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
- 管理员生命周期只调用 `public-admins`。
- CID 重复校验通过 `entity-primitives::InstitutionCidQuery` 查询 `private-manage`。
- 不承担多签转账，转账只归 `multisig-transfer`。
- 不承担管理员集合变更，管理员真源只归 `public-admins`。

## MODULE_TAG

- `b"pub-mgmt"`

## 钱包扫码

- pallet index：`32`
- 创建动作：`propose_create_public_institution`
- 关闭动作：`propose_close_public_institution`
- 清理动作：`cleanup_rejected_public_proposal`
