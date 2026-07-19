# 公民信息上链单次签名流程

## 任务需求

- OnChina 公民身份签名弹窗统一使用“公民签名确认”。
- 公民身份签名同时支持 CitizenApp 交易页扫一扫、钱包详情右上角扫一扫和 CitizenWallet 扫码签名。
- 同一公民信息上链操作中，管理员钱包只签最终链交易一次，Passkey 只验证一次，目标公民钱包只授权签名一次。
- 删除 `prepare/complete` 各自重复生成管理员安全 grant 和重复消费 Passkey 的旧流程。
- 完成后同步更新文档、完善中文注释并清理旧代码、旧测试、旧文案和旧流程描述。

## 所属模块

- `citizenchain/onchina`
- `citizenchain/crates/qr-protocol`
- `citizenapp`
- `citizenwallet`
- `memory`

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/03-security/security-rules.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/unified-naming.md`
- `memory/01-architecture/citizenchain/CITIZEN_IDENTITY_FLOW.md`
- `memory/01-architecture/qr/qr-action-registry.md`
- `memory/05-modules/citizenchain/onchina/BACKEND_TECHNICAL.md`
- `memory/05-modules/citizenchain/onchina/FRONTEND_TECHNICAL.md`

## 必须遵守

- 扫码协议仍只有 `QR_V1`，action registry 仍只有 `citizenchain/crates/qr-protocol/registry/*` 一个代码真源。
- 一个公民信息上链 operation 固定只允许：Passkey 一次、公民钱包签名一次、管理员最终链交易签名一次。
- 公民签名负责本人身份授权；管理员签名负责最终完整 extrinsic，二者不得合并，也不得额外叠加管理员安全 grant 签名。
- CitizenApp 交易页扫一扫按请求公钥定位本地热钱包；钱包详情扫一扫固定使用当前钱包，公钥不一致必须拒绝。
- 链确认成功前不得写正式钱包绑定和上链投影。
- 不修改 `citizenchain/runtime/`。
- 不保留旧接口分支、重复 grant、旧文案或双轨兼容。

## 预计修改目录

- `citizenchain/crates/qr-protocol/registry/`：统一“公民签名确认”中文动作名，属于协议登记和生成物输入。
- `citizenchain/onchina/frontend/citizens/`：收敛单次操作前端流程和统一弹窗标题，清理重复安全授权。
- `citizenchain/onchina/src/domains/citizens/`：实现一次性 operation 状态流和链确认后落库。
- `citizenchain/onchina/src/auth/`：公民上链只消费一次 Passkey，不再生成额外管理员安全 grant。
- `citizenapp/lib/qr/`、`citizenapp/lib/signer/`、`citizenapp/lib/wallet/pages/`、`citizenapp/lib/my/myid/`：统一扫一扫、公民身份签名服务和钱包详情入口。
- `citizenwallet/lib/`：复用现有通用离线签名，只对齐统一标题和一次签名边界。
- `memory/`：更新协议、架构、模块文档并清理旧流程描述。

## 输出物

- OnChina 单次授权和单次签名实现
- CitizenApp 两个扫一扫签名入口
- CitizenWallet 对齐
- 中文注释
- 跨端测试
- 文档更新
- 旧流程残留清理

## 验收标准

- 一次公民信息上链只出现一次 Passkey、一次公民签名、一次管理员最终链签名。
- OnChina、公民和公民钱包统一显示“公民签名确认”，身份详情仍明确显示投票身份或参选身份。
- CitizenApp 两个扫一扫入口与 CitizenWallet 都能签 `citizen_identity`，钱包、公钥、action、字段或有效期不合法时红色拒绝。
- OnChina 后端、前端和移动端基础测试通过。
- 使用真实本地 OnChina 服务、真实 PostgreSQL、真实页面或真实 HTTP 完成运行态验收。
- 文档、中文注释和残留清理完成。

## 执行记录

- 2026-07-18：用户确认执行，并明确要求完成后更新文档、完善注释、清理残留。
- 2026-07-18：OnChina 删除 prepare/complete 双 grant；prepare 一次消费 Passkey 并创建短期操作，complete 原子消费公民回执，最终链确认后才绑定钱包。
- 2026-07-18：QR action 统一改为“公民签名确认”并重新生成 CitizenApp/CitizenWallet 注册表；CitizenApp 交易扫一扫及钱包详情扫一扫接入统一服务，删除孤立 `MyIdSignPage`。
- 2026-07-18：验证通过：OnChina 135 项测试、QR registry 6 项测试、CitizenApp 9 项签名测试与定向 analyze、CitizenWallet 118 项签名/解码测试、OnChina 前端 TypeScript + Vite build、`git diff --check`。
- 2026-07-18：本机 `https://127.0.0.1:8964/` 与 `/api/v1/health` 均真实返回 200，页面已加载新构建 `index-CvPREDqC.js`；健康状态为 `DEGRADED`。当前运行中的后端进程早于本次编译启动，完整 Passkey + 两台钱包扫码 + 链 finalized 验收仍需在用户 Touch ID 重启/部署后执行，未伪造为已通过。
