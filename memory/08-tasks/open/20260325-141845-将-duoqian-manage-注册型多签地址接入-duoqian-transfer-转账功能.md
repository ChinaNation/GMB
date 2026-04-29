# 任务卡：将 duoqian-manage 注册型多签地址接入 duoqian-transfer 转账功能

- 任务编号：20260325-141845
- 状态：open
- 负责人：Codex

## 1. 任务目标

让 `duoqian-manage` 注册出来的多签地址也能复用现有
`duoqian-transfer` 的提案、投票和执行转账流程。

## 2. 范围

- `citizenchain/runtime/transaction/duoqian-transfer/`
- `citizenchain/runtime/transaction/duoqian-manage/`
- `citizenchain/runtime/src/configs/mod.rs`
- `wuminapp/lib/governance/`
- `wumin/lib/signer/`
- 对应模块文档与任务记录

## 3. 关键约束

- 不新增新的转账 pallet，直接扩展现有 `duoqian-transfer`
- 不改现有 `DuoqianTransfer` pallet index / call index
- 运行时改动后，需要同步更新在线端与冷钱包兼容信息

## 4. 执行计划

1. 扩展链上 `duoqian-transfer` 支持 `ORG_DUOQIAN`
2. 让注册型多签地址通过 `InstitutionPalletId(48)` 接入现有内部投票
3. 更新 `wuminapp` 构造注册型机构转账 payload 的逻辑
4. 更新 `wumin` 冷钱包的扫码摘要显示兼容
5. 补测试、补文档、回写任务卡

## 5. 验证要求

- `cargo test -p duoqian-transfer`
- `cargo check -p citizenchain`
- `dart analyze` 相关文件
- `git diff --check`

## 6. 执行结果

- 已扩展 `duoqian-transfer` 支持 `ORG_DUOQIAN`，注册型 Active 多签地址现在可直接复用 `propose_transfer / vote_transfer / execute_transfer`
- 已补链上单测，覆盖注册型多签地址达到阈值后自动执行转账
- 已同步更新 `wuminapp`
  - 统一 institution id 编码：治理机构用 `shenfen_id`，注册型机构用 `duoqian_address(32)+16字节0`
  - 注册型机构管理员与阈值改为从 `DuoqianManage.DuoqianAccounts` 动态读取
- 已同步更新 `wumin` 冷钱包
  - `org = 3` 摘要显示为“注册多签机构”
  - `supportedSpecVersions` 增加到 `{2, 3}`
- 已将 runtime `spec_version` 升级为 `3`

## 7. 验证结果

- `cargo test -p duoqian-transfer`：通过
- `cargo check -p citizenchain`：通过
- `dart analyze wuminapp/lib/governance/institution_data.dart wuminapp/lib/governance/institution_admin_service.dart wuminapp/lib/governance/proposal_context.dart wuminapp/lib/governance/transfer_proposal_service.dart wumin/lib/signer/payload_decoder.dart wumin/lib/signer/pallet_registry.dart`：通过
