# CitizenApp Cloudflare 统一资源限制与聊天中继清理

## 任务需求

- 在 `citizenapp/cloudflare/src/limits/` 建立服务端唯一资源限制真源。
- 所有经 Cloudflare 传输或存储的数据，在读取正文、写 D1、R2、Images、Stream 或推送前统一校验单文件、单请求、单账户和周期额度。
- 头像、背景、广场清单、图片和视频上传按会员权益与成本边界强制限制，客户端声明不得作为最终依据。
- 聊天附件不再使用云端中继；附件只允许设备间直接传输，Cloudflare 不存储、不转发附件字节。
- 用户注销时继续立即硬删除 Cloudflare 中全部聊天必需元数据。
- 完成后更新文档、补齐中文注释并彻底清理旧接口、旧配置、旧密钥引用、旧数据表与旧测试残留。

## 所属模块

- CitizenApp Flutter 客户端
- CitizenApp Cloudflare Worker / D1 / R2 / Images / Stream

## 输入文档

- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md`
- `memory/03-security/security-rules.md`
- `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`
- `memory/07-ai/unified-naming.md`
- `memory/07-ai/unified-protocols.md`
- `memory/07-ai/module-checklists/citizenapp.md`
- `memory/07-ai/module-definition-of-done/citizenapp.md`

## 预计修改目录

- `citizenapp/cloudflare/src/limits/`：新增统一限制目录；承载限制表、请求读取、上传校验、额度预留、存储核销和交付约束代码。
- `citizenapp/cloudflare/src/`：接入统一限制入口；改造上传、资料、聊天、路由和账户注销代码，清除聊天附件中继与分散限制常量。
- `citizenapp/cloudflare/migrations/`：直接重写当前 D1 基线为目标结构；新增资源预留/用量表并删除附件中继旧表，不保留迁移兼容。
- `citizenapp/cloudflare/test/`：新增统一限制测试并改造现有上传、资料、聊天和账户测试，删除附件中继旧断言。
- `citizenapp/lib/8964/`：收口客户端上传调用，删除附件中继获取流程，只保留设备直连附件和离线唤醒。
- `citizenapp/test/8964/`：更新客户端测试，验证统一上传契约和无附件中继目标状态。
- `citizenapp/cloudflare/`：清理 Wrangler 配置、示例变量、脚本和旧密钥名称；更新模块说明。
- `memory/`：更新 CitizenApp 架构、聊天、安全与统一协议文档，清除旧上传和附件中继口径。

## 必须遵守

- 不修改 `citizenchain/runtime/`，不触碰工作区已有 runtime 改动。
- 不保留旧上传接口、旧附件中继分支、旧配置或兼容逻辑。
- 服务端限制为唯一授权真源；客户端限制只用于提前反馈。
- 硬上限只能由代码确定，环境变量最多只能降低限制。
- 大请求必须先检查声明长度并进行有界读取，禁止无界解析请求体。
- 关键安全逻辑必须补中文注释。
- 未获得单独授权不得推送 GitHub 或触发远端 CI。

## 输出物

- 统一资源限制代码与中文注释
- 头像、背景、清单、Images、Stream 上传强制校验
- 原子额度预留、核销与删除回收
- 聊天附件中继代码、接口、配置、数据表和客户端残留清理
- Worker、Flutter 与 D1 测试
- 架构、安全、聊天和接口文档更新

## 验收标准

- 未登记路由在进入 D1 前返回 404。
- 超限请求在读取完整正文和调用存储提供商前被拒绝。
- 头像、背景、清单、图片、视频均按统一限制表校验实际字节；图片校验尺寸，视频由 Stream TUS 绑定上传长度和最长时长。
- 同一账户的活动上传数与订阅周期额度使用 D1 原子预留，不能通过并发请求超额。
- Cloudflare API、D1、R2、Images、Stream、推送和链代理请求都有明确上限。
- 仓库内不存在聊天附件中继接口、配置、密钥、表、客户端调用和文档残留。
- 用户注销真实路径会硬删除聊天必需元数据，不保留恢复副本。
- TypeScript 类型检查、Worker 测试、Flutter 静态检查与相关测试通过。
- 使用真实本地 Worker、D1、R2 和 HTTP 请求验证正常、超限、未授权、删除和未知路由行为。
- 文档已更新、注释已完善、残留已清理。

## 状态

- 已完成

## 2026-07-12 补充清理

- 当前没有用户和需保留的媒体资产；Cloudflare Images 与 Stream 若存在任何对象，一律直接永久删除，不做迁移、兼容或备份。
- 清理后分别复查 Images 与 Stream 账户资产清单为空，不能只依据 D1 媒体引用为 0 判断完成。

## 完成记录

- 新增 `src/limits/` 六个真源文件，统一管理路由白名单、请求有界读取、文件字节/MIME/尺寸校验、D1 原子额度预留与核销、R2/KV 写入和交付上限。
- 资料和 manifest 改为同域 Worker 有界上传；图片由 Worker 校验后写 Cloudflare Images；全部视频使用绑定精确字节与最长时长的 Stream TUS。旧上传接口、开发代理和用户 R2 写入授权已删除。
- Chat 只使用公共 STUN 发现设备直连候选；附件中继接口、服务端代码、客户端分支、D1 表和四项 staging/production Secret 已删除，Cloudflare 控制台中的两个 Realtime 应用已永久删除。
- staging/production D1 均清空旧数据并按唯一 `0001_square_core.sql` 重建；当前只保留 3 张 Chat 最小元数据表及 `resource_reservations`、`resource_usage`、`resource_totals` 三张资源表，不存在旧附件中继表或旧用量表。
- staging Worker 版本为 `f8fbb3e0-b5b3-4055-bf69-d0f305f4a8bb`；production 活动版本为 `54432c7a-3572-4f55-86cc-38a95b25c2d0`。
- 真实本地 Worker 验收覆盖 404、411、413、401、P-256 设备签名、访客会员第二个活动上传 429；同账户连续上传 1×1 与 2×2 头像始终使用同一固定 R2 键，第二次覆盖后 Bearer-only 图片请求回读 2×2，无 session 返回 401。临时 KV、D1、R2 数据已硬删除。
- 真实 production HTTP 验收覆盖 health 200、旧/未知路由 404、未登录上传 401、1.5 MiB+1 字节资料请求 413、伪造 Origin 403；staging Access 未登录返回预期 302。
- Worker 20 个测试文件 124 项、TypeScript 类型检查、Flutter 定向分析和 61 项相关测试全部通过。
- staging R2 实查为空；production 发现的 4 份旧 `profile.json` 已逐项硬删除，R2 对象列表复查为空。`npm audit --omit=dev` 返回 0 漏洞。
- 2026-07-12 使用已登录 Cloudflare 控制台完成账户级复查：Images 显示当前存储图像 `0`，Stream 显示当前视频数 `0`、存储分钟数 `0`；没有残留对象需要删除，媒体账户清理已闭环。
