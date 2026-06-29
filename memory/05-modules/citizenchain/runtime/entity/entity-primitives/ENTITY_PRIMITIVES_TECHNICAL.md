# entity-primitives 技术说明

模块：`entity-primitives`

职责：实体生命周期共用类型与 trait。该 crate 不含 storage、不含 extrinsic、不保存 CID 登记状态。

## 边界

- 定义 `EntityKind`，区分公权机构、私权机构、个人多签。
- 定义 `InstitutionMultisigQuery`，供交易、清算、扫码验签等模块统一查询机构账户状态和管理员快照。
- 定义 `InstitutionCidQuery`，供 `public-manage` 和 `private-manage` 互查 CID 是否已登记，防止同一 CID 在两个生命周期模块重复写入。
- 定义 `CidInstitutionVerifier`，统一 CID 机构登记、注销凭证验签接口。
- 复用 `primitives::multisig` 的账户校验、保留地址、保护地址 trait。

## 禁止事项

- 不允许在本 crate 增加 storage。
- 不允许把公权、私权、个人多签生命周期状态写到本 crate。
- 不允许恢复单独的 entity-registry pallet。
