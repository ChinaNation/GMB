# 任务卡：为 SFID 机构生成流程新增独立机构名称字段并用于 wuminapp 与 wumin 显示

- 任务编号：20260325-150802
- 状态：open
- 负责人：Codex

## 1. 任务目标

在 `sfid` 系统生成机构 `sfid_id` 时，新增一个与 `sfid_id` 分开的独立字段
`institution_name`，由 `sfid` 系统管理员手工输入，用于后续在 `wuminapp`
发起注册型多签机构转账时展示具体机构名称，并在 `wumin` 冷钱包扫码签名页中显示。

## 2. 范围

- `sfid/frontend/src/components/App.tsx`
- `sfid/frontend/src/api/client.ts`
- `sfid/backend/src/super-admins/institutions.rs`
- `sfid/backend/src/models/mod.rs`
- `sfid/backend/src/chain/app_api.rs`
- `wuminapp/lib/governance/`
- `wumin/lib/signer/`
- 对应模块文档与任务记录

## 3. 关键约束

- 现有 `sfid_id` 生成规则不变
- 现有 `institution` 字段继续表示机构类型码，不改为机构名称
- 新增的 `institution_name` 不参与 `sfid_id` 生成、不参与链上注册签名载荷
- 区块链 runtime 不改
- `wumin` 冷钱包本次只负责显示机构名称，不新增名称合法性验签要求

## 4. 需求拆分

1. `sfid` 机构 SFID 生成表单新增独立输入项 `institution_name`
2. `sfid` 后台在生成机构 `sfid_id` 时持久化保存 `sfid_id -> institution_name`
3. `sfid` 后台新增给 App 使用的机构名称查询接口
4. `wuminapp` 在处理注册型多签机构时，查询机构名称并写入二维码显示摘要
5. `wumin` 冷钱包在扫码签名页面展示该机构名称，但不把名称纳入强校验

## 5. 实现说明

- 机构 `sfid_id` 仍按当前规则生成：`A3-R5-T2P1C1-N9-D8`
- 其中 `T2` 继续表示机构类型码，如 `ZF / LF / SF / JC / JY / CB`
- `institution_name` 是新增的业务显示字段，语义上与 `T2` 分离
- 推荐查询方式优先考虑：
  - `duoqian_address -> institution_name`
- 如后端先只提供：
  - `sfid_id -> institution_name`
  则 `wuminapp` 需要先从链上反查 `sfid_id`

## 6. 验证要求

- `sfid` 机构 SFID 生成后，可查到对应 `institution_name`
- `wuminapp` 发起注册型多签机构转账时，二维码摘要中包含具体机构名称
- `wumin` 冷钱包扫码后能显示该机构名称
- 不改 runtime、不改现有链上 payload 编码结构

## 7. 备注

- 当前机构 `sfid_id` 本身不包含具体机构名称，不能从 `sfid_id` 直接反推出“广州市公安局”这类名称
- 本任务解决的是“显示具体机构名称”，不是“把机构名称上链”
