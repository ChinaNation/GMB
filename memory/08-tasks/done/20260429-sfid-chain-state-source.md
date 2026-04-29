# 任务卡：SFID 链上状态以链为真源改造

## 任务需求

按用户确认的 SFID 设计执行彻底改造：

- SFID 系统只负责机构身份和账户名称。
- `account_name` 保持 DUOQIAN_V1 协议字段，不拆分、不改名。
- 机构创建后默认生成 `主账户`、`费用账户`。
- SFID 后台不再手动激活账户，账户激活状态只能来自链上注册/注销同步。
- 已上链账户不能删除、停用、归档；新增账户只有未上链或链上已注销后才能删除。
- 开发期不做历史兼容、过渡兼容或影子旧流程。

## 建议模块

- `sfid/backend`
- `memory/05-modules/sfid/backend/institutions`
- `memory/07-ai`

## 影响范围

- SFID 机构/账户模型
- SFID 机构/账户服务规则
- SFID 机构 API 返回结构
- SFID 链上状态同步入口
- AI 编程系统强制规则
- SFID 机构技术文档

## 技术方案

1. AI 编程系统规则
   - 增加强制规则：开发期按彻底改造设计，不保留兼容。
   - 增加强制规则：技术方案必须包含更新文档、完善注释、清理残留。
   - 增加强制规则：执行后必须更新文档、完善注释、清理残留。

2. SFID 状态模型
   - 机构增加链上状态：`NotRegistered`、`PendingRegister`、`Registered`、`RevokedOnChain`。
   - 账户增加链上状态：`NotOnChain`、`PendingOnChain`、`ActiveOnChain`、`RevokedOnChain`。
   - 创建 SFID 机构时，机构为 `NotRegistered`，默认账户为 `NotOnChain`。

3. SFID 服务规则
   - 删除 SFID 后台手动激活账户入口。
   - 账户状态只能由链上同步接口变更。
   - 默认账户永远不能单独删除。
   - 新增账户 `NotOnChain` 或 `RevokedOnChain` 才能删除。
   - `ActiveOnChain`、`PendingOnChain` 均不能删除、停用、归档。

4. API 规则
   - 提供机构搜索、机构详情、账户列表给区块链软件查询。
   - 提供受信任的链上状态同步接口。
   - 返回 `can_delete`、`is_default` 等前端判定字段，避免前端自行猜规则。

5. 文档、注释、清理残留
   - 更新 SFID 机构技术文档。
   - 对状态转换和删除规则补中文注释。
   - 清理旧的手动激活流程、废弃入口和文档残留。

## 验收标准

- 创建 SFID 机构后默认账户均为未上链状态。
- SFID 后台/API 不再暴露手动激活账户动作。
- 链上同步成功后，机构和账户状态按同步结果更新。
- 链上注销后，机构和账户状态显示已注销/未激活。
- 默认账户不能单独删除。
- 新增账户未上链或链上已注销后可以删除。
- 已上链或上链中的账户不能删除、停用、归档。
- 技术文档已同步更新。
- 关键状态规则已有中文注释。
- 没有兼容分支、废弃接口、临时调试残留。

## 当前状态

- 状态：已完成
- 创建时间：2026-04-29

## 执行结果

- 已更新 AI 编程系统强制规则。
- 已移除 SFID 后台账户手动激活路由与前端激活按钮。
- 已将机构链上状态和账户链上状态改为链上同步真源模型。
- 已新增区块链软件机构搜索、详情、账户列表与链上状态同步接口。
- 已按 `account_name` 保持 DUOQIAN_V1 协议语义,没有拆分字段。
- 已更新 SFID 机构技术文档。
- 已补充状态同步、删除规则与链上真源相关中文注释。
- 已清理旧 `Inactive/Pending/Registered/Failed` 账户激活状态机、后端直接推链文件和对应省级签名构造残留。

## 验证结果

- `cargo test institutions`：通过,15 个测试通过；存在既有 `province.rs` dead_code warning。
- `npm run build`：通过；存在既有 Vite chunk size / dynamic import warning。
