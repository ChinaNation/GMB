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

- `citizenapp/lib/8964/`：本人发布内容本地仓库、发布确认和本地查看；Worker B1 仅清理
  已废弃的归档字段解析和归档占位 UI，不提前改动本地副本 schema。
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

## Worker B1 实施记录（2026-07-29）

- 权益到期清理已从旧 `membership/archive.ts` 收口为
  `membership/expiration_cleanup.ts`；删除旧 90 天视频归档、重新订阅回灌、归档状态机、
  S3 预签名和 `aws4fetch` 依赖，不保留兼容入口。
- 每日任务先完成订阅对账，再把本轮 finalized 区块 `Timestamp.Now` 直接传入清理函数；
  `cancelled/terminated + paid_until <= finalized_chain_timestamp` 是唯一删除资格，禁止
  使用设备时间、Worker 墙钟或退订天数。
- 清理按 CID 覆盖已发布帖子和只有上传尚未发布的内容；先删 Images/Stream/R2，再在同一
  D1 batch 中回收存储总量并删除帖子、上传、媒体行。外部失败保留 D1 索引，404/重复删除
  可幂等收敛。
- 创世 D1 基线已删除 `archive_state/archived_at/r2_archive_key` 与归档索引；
  `chain_transaction_confirmations`、`topup_orders` 保留策略未改变。
- CitizenApp 已删除媒体模型/API 解析/UI 中的旧归档状态和“已归档/恢复中”占位；这只清理
  已废弃协议，不涉及本人发布内容本地副本或 Isar schema。
- 门禁：Worker TypeScript 类型检查通过；31 个测试文件、195 项测试通过；全新本地 D1
  基线 55 条语句执行成功；Miniflare 真实 D1/R2 binding 验收确认 finalized 链时间戳
  `10000` 能删除 1 个到期身份的 1 个内容项、D1 帖子/上传均归零且 R2 manifest 不再
  存在；相关 Flutter 静态检查及 22 项广场 UI 测试通过。完整任务仍为 open，因为
  CitizenApp 本人发布内容本地副本尚未实施。

## CitizenApp 第 1 步实施记录（2026-07-29）

- 新增 `SquareLocalPostEntity` / `SquarePostStore`：本人副本归属唯一使用
  `cid_number`，`account_id` 只保留发布时签名账户事实；CID 换绑不迁移。
- 本地内容真源只有参与链上 `content_hash` 的原始 `manifest_bytes`；不拆重复正文字段，
  不保存图片、视频、封面、文件路径、临时 URL、公共 feed 或设备缓存时间。
- 写入与读取都校验 SHA-256、唯一 manifest schema、账户、分类、内容形态和
  `post_state=published`；磁盘行损坏 fail-closed，不静默显示空白。
- `created_at` 只能由 Worker 调用方传入；仓库不读取设备时间。列表同毫秒时使用
  `post_id` 作稳定次级排序。
- Isar schema 与生成代码已登记新 collection；不迁移、不兼容不存在的旧广场缓存。
- `post_id` 作为不可变发布事实：相同字段可幂等重放，任何正文、CID、签名账户或链锚
  冲突都拒绝覆盖。
- 验收：`flutter analyze lib/ test/` 零问题；本地仓库真实 Isar 磁盘往返、关闭重开、
  篡改拒绝和 CID 删除边界 8 项通过；`flutter test test/8964` 共 134 项通过。
- 本步尚未接发布确认、离线主页或删除/注销流程；任务保持 open。

## Worker 第 2 步实施记录（2026-07-29）

- 新增 `GET /v1/square/posts/self`：必须通过 Bearer session、链上当前绑定复查和 P-256
  设备请求证明；查询属主只取 `session.cid_number`，每页最多 5 条，按
  `(created_at DESC,post_id DESC)` 稳定游标分页，不接受客户端自报 CID。
- 回灌只返回本人 `published` 内容的原始 `manifest_bytes_base64`、不可变发布字段和
  Worker `created_at`；不返回 Images/Stream 媒体字节、R2 对象键、临时 URL，不写
  `square_browse_days`，也不读取设备时间。
- R2 manifest 键唯一取自服务端 `square_uploads.object_keys_json`；删除了 feed 读取中的
  账户/post_id 猜路径兜底。缺键、多键、对象缺失、超限、非法 UTF-8/JSON/schema、
  CID/账户/分类/内容形态/存储回执冲突均 fail-closed。
- 发布确认和本人回灌共用同一 manifest 验证边界；R2 原始字节 SHA-256 必须同时匹配
  `square_posts.content_hash`、`square_uploads.content_hash` 与 `manifest_hash`。
  回灌一页任一项损坏即整页拒绝，不下发部分副本。
- 未新增 D1 migration，未修改会员到期清理、iOS、链、runtime 或清算行。
- 验收：Worker TypeScript 类型检查通过；32 个测试文件、203 项测试全部通过；新增 5 项
  使用 Miniflare 真实 D1/R2/KV binding 和完整 Worker HTTP 入口，覆盖 Session CID 隔离、
  稳定分页、原始字节无损、浏览量零写入、对象键不猜测、哈希/归属冲突整页拒绝。
- 下一步仍需接 CitizenApp 发布确认/启动回灌到 `SquarePostStore`；完成客户端接线前，
  端到端双存仍未完成，任务保持 open。

## CitizenApp 第 3 步实施记录（2026-07-29）

- `SquareApiClient.fetchSelfPublishedPostCopies()` 已接入本人回灌接口：GET 继续走 Bearer
  Session 与 P-256 设备请求证明；客户端严格校验每页数量、CID、账户、分类、内容形态、
  小写 SHA-256、标准 base64、链块、Worker `created_at` 和 `post_state=published`，任一
  条目漂移时拒绝整页。
- `SquarePostStore.saveAll()` 先校验全页再在同一 Isar 事务落盘；同页重复 `post_id`、
  磁盘既有不可变事实冲突或任一 manifest 损坏都会回滚整页。同步检查点复用
  `AppKvEntity`，只保存远端最新 `(created_at,post_id)`，不保存设备同步时间。
- `SquarePostSyncService` 首次完整分页回灌；后续从远端最新位置扫描到上次检查点即停止。
  分页中断时不推进检查点，下次从顶部幂等重放；远端因会员到期或删除变空时只更新空
  检查点，绝不反向删除本地历史副本。
- `SquarePublishService` 在 Worker 确认成功后直接使用发布阶段保留的原始 manifest 与
  Worker 返回的发布锚写本地副本。若磁盘写入失败，远端成功结果仍按“已发布”返回，
  仅显示完成告警并立即调度本人回灌，禁止让用户重试造成重复扣费和重复发帖。
- `SquareHomePage` 获得有效会话后后台启动回灌，不阻塞公共 feed 首屏；同一同步服务内
  同一 CID single-flight。注销本地清理会同时删除该 CID 副本与检查点。
- 已删除客户端 `cover_url/cover_object_key` 归档别名和“保持旧 manifest”注释，不保留
  旧字段解析或旧形态兼容。
- 本步隔离验收：全量 `lib/ + test/` 静态分析零问题；API、发布、本地仓库和同步服务
  27 项通过，覆盖
  原始字节无损、设备证明、分页/增量检查点、整页原子拒绝、远端空集不删本地、注销清理、
  同 CID single-flight、发布后磁盘失败不转发布失败及发布后真实 Isar 原始字节落盘；
  除 `square_home_page_test.dart` 外的广场 139 项全绿。
  共享工作树同期存在另一任务的链上动态费用改造，已按其目标补齐
  `SquarePublishBalanceReader` 合并接口。`square_home_page_test.dart` 当前 4 项因既有
  持续链状态动画导致 `pumpAndSettle` 超时，目标逻辑不进入真实同步分支；需由对应并发
  任务收口动画测试后复跑，不能把该套件记为全绿。
- 本步完成发布确认与启动回灌接线；离线本人内容 UI、单帖删除和完整注销生命周期仍在
  后续步骤实施，因此任务继续保持 open。

## CitizenApp 第 4 步实施记录（2026-07-29）

- 新增 `SquareLocalPostPresenter`：重新复核 manifest SHA-256、schema、账户、分类和内容
  形态后，只恢复正文、标题和文章图文块；媒体只形成“不可用类型”声明，不生成媒体对象、
  文件路径或 URL。
- 本人主页以 session CID 二次确认本人边界，先显示本地正文，再按 `post_id` 合并 Worker
  分页结果；Worker 作者资料与媒体元数据优先，远端为空或失败不抹除本地历史。他人主页与
  公共 feed 不访问 `SquarePostStore`。云端媒体缺失时明确提示“媒体已从云端清理，本机
  仅保留正文”。
- 新增统一 `SquarePostDeletionCoordinator`，动态详情、文章详情和编辑后旧帖清理不再用
  可换绑 `account_id` 判断属主，统一校验 session `cid_number`。Worker 成功后才删本地；
  只有精确 `404/post_not_found` 可清理同 CID 本地残留，403、会话和网络失败都保留本地。
- 注销服务新增必填 `cid_number`，服务端硬删除成功后删除该 CID 全部本人副本和同步检查点；
  每项本地清理独立尝试，单项失败不阻断资料缓存、会话、私信或设备子钥清理，UI 也不会把
  已完成的服务端注销误报为可重复提交的失败。
- 验收：`flutter analyze lib/ test/` 零问题；新增和相关存储、发布、注销、本人主页共
  32 项定向测试通过；除共享并发任务已知持续动画测试
  `square_home_page_test.dart` 外，`test/8964` 共 147 项全部通过。Pixel 8a 真机启动
  验收结果在本步构建结束后补记。
- 当前步骤没有修改 Worker、链、runtime、清算行或媒体到期规则；任务保持 open，待最后
  一步完成真实账户数据下的端到端页面/删除生命周期验收与全仓残留复核。
