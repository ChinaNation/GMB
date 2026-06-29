# 拆分机构生命周期模块

## 任务目标

- 删除旧私有实体目录。
- 将个人多签生命周期迁移到 `runtime/entity/personal-manage`。
- 将机构生命周期按公权/私权拆分为 `runtime/entity/public-manage` 和 `runtime/entity/private-manage`。
- 将实体共享 trait、查询接口和公共类型沉淀到 `runtime/entity/entity-primitives`。
- 更新 runtime、CitizenWallet、CitizenApp 解码与错误映射，清理旧单机构模块残留。

## 边界

- 不做旧 pallet 兼容。
- 不做链上数据迁移。
- 不改其他线程已有的工作区改动。
- `multisig-transfer` 只通过实体查询 trait 接入注册机构账户转账，不接管生命周期。

## 完成情况

- [x] runtime workspace 依赖和 pallet wiring 已改为 entity 模块。
- [x] 公权机构生命周期只接 `public-admins`。
- [x] 私权机构生命周期只接 `private-admins`。
- [x] 个人多签生命周期保留在 `personal-manage`，管理员真源保留在 `personal-admins`。
- [x] 钱包扫码签名解码改为 pallet 32/33。
- [x] 当前 runtime 模块技术文档已同步。
