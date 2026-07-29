# CitizenApp 广场本地副本与会员失效云端清理

状态：open

## 任务需求

- 本地只保存本人发布的动态/文章正文、规范 manifest、链上哈希和发布状态，不缓存整个公共 feed。
- 照片和视频只保存于 Cloudflare，不在 App 内建立永久本地媒体副本。
- 取消续订后，在已付款权益 `paid_until` 真正到期时清除该身份的云端广场正文、manifest、Images、Stream 和归档对象。
- 注销继续硬删除身份云端数据。
- 重新订阅不得自动恢复已删除内容。

## 已确认后果

- 权益到期清理后，云端独有的照片和视频永久删除；本地只剩正文和 manifest。
- 现有“退订 90 天后仅归档视频”及重新订阅恢复路径由新规则取代，不保留双轨。
- `chain_transaction_confirmations` 和 `topup_orders` 是链交易/财务事实，继续按 2026-07-28 已定策略保留，不作为本任务的社交内容清理对象。

## 预计修改目录

- `citizenapp/lib/8964/`：本人发布内容本地仓库、发布确认和本地查看。
- `citizenapp/lib/isar/`：本人广场内容持久化结构。
- `citizenapp/cloudflare/src/membership/`：权益到期触发与旧视频归档逻辑清理。
- `citizenapp/cloudflare/src/posts/`、`media/`、`storage/`：按 CID 删除正文和媒体提供商资源。
- `citizenapp/cloudflare/migrations/`：清理任务状态或幂等游标所需数据库契约。
- `citizenapp/cloudflare/test/`、`citizenapp/test/`：到期、失败重试、幂等、发布和本地副本测试。
- `memory/01-architecture/gmb/`、`memory/05-modules/citizenapp/`、`memory/07-ai/`：订阅和广场文档回写。

## 主要风险

- Images/Stream/R2 删除不可逆，必须以 finalized 链时钟和 `paid_until` 为触发真源。
- 多提供商删除不能依赖单次 Worker 请求完成，必须支持幂等重试和部分失败恢复。
- D1 行不能先于媒体提供商本体删除，否则会失去后续清理索引。
- 当前 D1 基线刚完成 CID 主键重构，正式部署前必须执行既有生产 D1 重建流程。

## 完成标准

- 本人已发布正文和 manifest 在离线状态可从本机读取。
- 点击取消但权益未到期时内容仍在线；到期后云端正文和全部媒体不可读取且无残留。
- 重复清理安全幂等，部分失败能重试收敛。
- 真实本地 Worker/D1/R2 替身或授权环境、Android App 和文档验收完成。
