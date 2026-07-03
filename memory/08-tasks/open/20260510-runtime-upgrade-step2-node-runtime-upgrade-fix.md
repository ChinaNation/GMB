# 修复 node runtime-upgrade 模块边界

## 任务目标

只修复 node 前后端 runtime-upgrade 模块，清理模块内不该承担的人口快照、投票引擎、投票状态职责。

## 修改边界

- 后端仅限 `citizenchain/node/src/governance/runtime_upgrade`、`citizenchain/node/src/governance/mod.rs` 及必要的 runtime-upgrade 类型残留。
- 前端仅限 `citizenchain/node/frontend/governance/runtime-upgrade` 及必要的 runtime-upgrade 类型残留。
- 不新增、不迁移、不修改投票引擎目录。
- 不修改 runtime 链上 pallet。
- 不修改 citizenapp。
- 不修改 citizenwallet 公民钱包。

## 修复原则

- runtime-upgrade 只负责协议升级业务调用的构建、签名请求生成和提交。
- 人口快照、联合投票、投票状态属于投票引擎，不属于 runtime-upgrade。
- 用户可见文案统一使用“协议升级”。
- 清理过时注释和残留字段。

## 验证要求

- 检查 runtime-upgrade 模块内不再直接获取人口快照。
- 检查前端 runtime-upgrade 不再展示“获取人口快照”。
- 运行可行的后端和前端检查。

## 完成记录

- 已删除 node 后端 `runtime_upgrade` 内部的 CID 人口快照获取。
- 已删除 `governance/cid_api.rs`，并从治理聚合入口移除。
- 已删除 node 后端 `runtime_upgrade` 的 `eligible_total / joint_nonce / joint_signature / province / signer_pubkey` 入参。
- 已删除 node 前端 `runtime-upgrade` 的联合提案上下文 DTO 和缺上下文报错。
- 已接入省储委会协议升级入口，国家储委会和省储委会管理员均可进入协议升级业务提案页。
- 已删除前端 runtime-upgrade 页面里的快照返回保存和“获取人口快照”文案。
- 已更新模块技术文档，补充 node 侧边界。
- 已更新统一协议登记和 CID 人口快照 handler 注释，明确人口快照只服务投票引擎流程。
- 2026-05-10 追加修复：node 提案聚合层已删除协议升级摘要 `status` 解码，协议升级真实状态只读取 `VotingEngine::Proposals.status`。

## 阻塞记录

- 历史阻塞已解除：链上 `runtime-upgrade` 已收口为只接收 `reason + code`。
- 当前剩余的“发起联合提案前准备人口快照”属于投票引擎客户端流程，不属于 node runtime-upgrade 业务模块。
