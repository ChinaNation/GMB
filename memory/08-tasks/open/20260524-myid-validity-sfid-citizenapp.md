# 电子护照有效期字段接入 SFID 与 citizenapp

## 任务需求

SFID 保存档案码中的电子护照有效期和公民状态，按“公民状态 + 有效期”计算身份ID状态；citizenapp 电子护照页面展示有效期。

## 建议模块

- SFID 公民身份绑定与 citizenapp 状态接口
- SFID 公民详情展示
- citizenapp 电子护照页面

## 影响范围

- `sfid/backend/cpms/`
- `sfid/backend/citizens/`
- `sfid/frontend/citizens/`
- `cpms/backend/src/dangan/`
- `cpms/backend/dangan/`
- `citizenapp/lib/my/myid/`
- `citizenapp/test/`
- `memory/01-architecture/`
- `memory/05-modules/`

## 主要风险点

- 绑定状态 `status` 与身份ID状态 `identity_status` 必须继续分离。
- citizenapp 不返回、不保存 `status_updated_at`；该字段只归 SFID 内部防旧档案码回放。
- 公民列表不能新增有效期列；有效期只在详情和 citizenapp 电子护照页展示。
- 当前任务不做投票拦截、人口快照过滤或链上投票引擎快照拦截。
- 有效期字段源自 CPMS 签发的档案码；为保证 SFID 能验真新字段，本任务包含 CPMS 后端档案码签发的最小闭环改造。

## 验收标准

- SFID 记录保存公民状态、生效日期、截止日期和内部状态更新时间。
- CPMS 档案码载荷和签名原文包含生效日期、截止日期和状态更新时间。
- SFID 返回给 citizenapp 的状态接口包含 `valid_from / valid_until`，不包含内部状态更新时间。
- SFID 计算身份ID状态时，只有公民状态为 `NORMAL` 且当前日期在有效期内才返回正常。
- SFID 公民详情中显示公民状态、身份ID状态和有效期。
- citizenapp 电子护照页面展示 `有效期：yyyy年MM月dd日-yyyy年MM月dd日`。
- 文档同步更新，残留旧口径清理完成。
