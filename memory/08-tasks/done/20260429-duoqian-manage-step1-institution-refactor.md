# 任务卡：duoqian-manage 第1步机构级多签改造

## 状态

- done

## 背景

当前 `duoqian-manage` 同时承载机构多签和个人多签，但机构侧仍偏向“单账户注册/单账户创建”模型。根据最新需求，链上注册应以“机构”为单位：一个 SFID 机构下包含主账户、费用账户和可新增账户，管理员与阈值绑定到机构，账户资金初始化按账户填写，创建提案发起时先冻结创建者资金，管理员投票通过后再激活机构及其账户。

## 目标

- 只改造 `citizenchain/runtime/transaction/duoqian-manage` 模块本身。
- 在模块内部拆分机构多签与个人多签目录结构。
- 新增机构级创建模型：`sfid_id + 机构名称 + 多账户初始余额 + 管理员 + 阈值`。
- 创建提案发起时冻结创建者资金；通过后划转到机构账户；拒绝/清理时释放冻结。
- 默认主账户、费用账户必须存在，所有初始余额必须满足链上最低金额。
- 保留个人多签独立路径，避免与机构多签继续混在同一业务模型里。
- 执行完成后更新文档、完善中文注释、清理残留。

## 不在本步范围

- 不改 node/offchain。
- 不改 wuminapp/wumin。
- 不改 SFID 后端接口。
- 不做 runtime 顶层、前端、移动端对新 extrinsic 的适配；这些放到第2步。

## 涉及模块

- `citizenchain/runtime/transaction/duoqian-manage`
- `memory/05-modules/citizenchain/runtime/transaction/duoqian-manage`

## 完成记录

- 已新增 `address`、`institution`、`personal` 业务分区目录。
- 已新增机构级 storage：`Institutions`、`InstitutionAccounts`、`PendingInstitutionCreate`。
- 已新增 `propose_create_institution`，支持机构级多账户初始余额。
- 已实现创建提案发起时 reserve、通过后 unreserve + 扣费 + 入账、拒绝/执行失败后 unreserve + 清理索引。
- 已新增机构创建通过、拒绝退款、缺少主账户、初始余额不足测试。
- 已更新 `DUOQIAN_TECHNICAL.md`。

## 验证

- `cargo fmt --package duoqian-manage`
- `cargo test -p duoqian-manage`：21 passed
