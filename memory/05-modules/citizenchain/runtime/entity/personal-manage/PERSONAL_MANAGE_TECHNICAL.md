# personal-manage 技术说明

模块：`personal-manage`

职责：个人多签账户生命周期。只负责个人多签创建、关闭、被拒提案清理和账户生命周期状态。

## 边界

- 个人多签管理员真源归 `personal-admins`。
- 个人多签转账归 `multisig-transfer`。
- 个人多签机构码为 `PMUL`。
- 不承担公权机构、私权机构生命周期。

## MODULE_TAG

- `b"per-mgmt"`

## 钱包扫码

- pallet index：`7`
- 创建动作：`propose_create_personal`
- 关闭动作：`propose_close_personal`
- 清理动作：`cleanup_rejected_personal_proposal`
