# ADR-011 onchain-issuance Plain FT 目标态

- 状态:Accepted target-state
- 决议日期:2026-06-07
- 关联任务卡:`memory/08-tasks/open/20260607-duoqian-accountid-governance.md`

## 决议

`onchain-issuance` 作为链上发行资产的外壳 pallet，内核仍使用 `pallet_assets`。发行人、治理主体、管理员主体统一为机构多签 `AccountId`。

`asset_id` 保留为资产编号，用来标识某个已经发行的资产；它不承担治理身份、投票身份或管理员账户身份。

## 发行资格

- 允许：注册机构或内置机构对应的机构多签 `AccountId`。
- 不允许：裸单签账户。
- 不允许：把资产编号当作发行人、治理主体或管理员主体。

## 资产编号

- `asset_id: u32` 是 pallet 内部资产编号。
- `asset_id` 可用于查询资产、绑定 `pallet_assets` 内核资产、定位被治理对象。
- 所有投票、审批、冻结、强制操作的发起资格仍由机构多签 `AccountId` 判断。

## 监管边界

NRC 监管能力仍由 NRC 机构多签 `AccountId` 承担。监管操作治理的是某个 `asset_id` 对应的资产，投票身份仍来自 NRC 机构多签账户。

## 模块契约

- `Assets` storage 以 `asset_id` 记录资产元数据。
- 资产元数据直接记录 `issuer: AccountId`。
- 提案数据记录 `asset_id` 作为被治理对象。
- 投票引擎和管理员模块只接收 `AccountId` 主体。
