# SFID 机构备案上链字段口径收口

- 日期:2026-05-02
- 状态:done
- 完成日期:2026-05-02
- 归属:SFID Agent

## 目标

先固定第 1 步"SFID 机构备案上链"的最小字段口径,避免把 SFID 系统内部档案、机构生成流程、链上正式多签注册混在一起。

## 需求结论

第 1 步只确认备案推链字段:

- `sfid_id`:机构 SFID 号。
- `institution_name`:机构名称。
- `account_name`:机构账户名称。

## 明确不做

- 不梳理 SFID 系统登录流程,后续由 SFID 系统任务补齐。
- 不梳理 SFID 号生成和机构详情生成流程,后续由 SFID 系统任务补齐。
- 不把照片、章程、许可证、股东会决议、法人授权书等 SFID 内部档案推到区块链。
- 不把第 1 步备案等同于第 2 步链上多签机构注册。
- 不修改代码实现。

## 文档范围

- `memory/05-modules/sfid/backend/institutions/INSTITUTIONS_TECHNICAL.md`
- `memory/05-modules/sfid/backend/chain/CHAIN_TECHNICAL.md`
- `memory/05-modules/sfid/clearing-bank-eligibility.md`

## 验收

- 文档明确市管理员备案时只发送 3 个字段。
- 文档明确市管理员是操作人,省管理员签名密钥是链上授权签名。
- 文档明确备案成功后链上只是有机构备案记录,不是正式链上多签机构。
- 文档明确机构照片、章程等材料属于 SFID 系统内部资料,不进入备案上链 payload。

## 完成记录

- 已在 `INSTITUTIONS_TECHNICAL.md` 固定第 1 步机构备案上链字段口径。
- 已在 `CHAIN_TECHNICAL.md` 把第 1 步备案推链标记为常规链 pull 架构之外的明确例外。
- 已在 `clearing-bank-eligibility.md` 说明清算行资格字段不属于备案上链最小 payload。
- 本任务只更新文档,未修改代码。
