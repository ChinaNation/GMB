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
- 用户注销必须按 `cid_number` 删除 Cloudflare 内该身份的全部记录，包括
  `chain_transaction_confirmations` 最小证明和 `topup_orders` 充值订单；链上/EVM 原始
  交易事实仍由公链保存。权益到期清理只删除广场内容，不扩大到身份注销范围。

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
- 创世 D1 基线已删除 `archive_state/archived_at/r2_archive_key` 与归档索引；后续注销
  收口又为 `chain_transaction_confirmations` 和 `topup_orders` 增加 CID 归属并纳入删除。
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
- 本人主页由上层用永久 CID 判定本人边界，先显示本地正文，再按 `post_id` 合并 Worker
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
  已发起两次：首次与另一线程 Flutter 测试争用同一 `build/` 后主动停止；第二次在
  Gradle 持续下载缺失 Maven/Google 依赖 20 分钟后仍未产出新 APK，旧 APK 时间戳未变，
  因此没有把本步记为真机已验收，也未在设备执行发布、删除或注销。
- 当前步骤没有修改 Worker、链、runtime、清算行或媒体到期规则；任务保持 open，待最后
  一步完成真实账户数据下的端到端页面/删除生命周期验收与全仓残留复核。

## CitizenApp 第 5 步实施记录（2026-07-29）

- 在 Pixel 8a（`3C071JEKB09000`）完成新 APK 构建、安装和真实启动；Isar 正常打开，
  Smoldot 从已验证缓存恢复并完成 finalized `#6`。系统弹出硬件金库生物识别时仅执行
  “取消”，没有绕过验证，也没有执行发布、单帖删除或注销。
- 真机在生产 Worker 返回“广场服务暂时不可用”时发现首页冷启动未处理异步异常。根因是
  后台通知和前台信息流共享快速失败的会话 Future：后台 `unawaited` 边界原先只捕获
  `Exception`，且首页 `_feedFuture` 在 `FutureBuilder` 首帧监听前存在错误时间窗。
- `SquareHomePage` 现将会话提供器显式注入测试；后台通知、清读和会员软门禁捕获完整
  `Object` 边界；信息流 Future 创建后立即挂只读错误观察器，同时保留原始错误交给页面
  展示，既不再产生未处理异常，也不把失败伪装成成功或空数据。
- 验收：`flutter analyze lib/ test/` 零问题；`test/8964` 共 152 项全部通过，其中新增
  “信息流与后台通知会话快速抛出 `StateError`”回归。热重启后再次真实触发相同 Worker
  失败，日志只出现受控的 `[SquareHomePage] feed load failed`，不存在
  `Unhandled Exception` 或 `FATAL EXCEPTION`。
- 当前代码准备和匿名启动验收已完成；由于没有用户生物识别授权，本线程没有读取真实身份
  私有态，故不能把本人主页本地副本、真实发布、删除和注销记为端到端通过。任务保持 open，
  后续只需在用户现场授权下完成这些真实账户交互验收；不得以自动化方式绕过生物识别。
- 本步只修改 CitizenApp 广场首页、既有测试与文档；没有修改 Worker、链、runtime、
  清算行、扫码 UI 或媒体到期规则。

## 关闭前复查修复第 1 步（2026-07-29）

- 修复本人主页对在线会话的错误依赖：`ProfilePostsTab.session` 改为可空，本人身份仍由
  页面入口使用永久 `cid_number` 判定；断网、Worker 不可用或无法建立 session 时，五个
  内容 Tab 不再停留于永久加载状态，仍可读取本机已校验的正文和 manifest。
- session 只用于远端作者内容请求和受保护媒体 Bearer header，不再充当本人本地副本读取
  凭证；他人主页和公共 feed 继续禁止读取本机副本，没有扩大任何写操作或授权权限。
- 已补无会话本人读取与无会话他人隔离回归。按用户要求，1—7 项全部修复完成前不再执行
  测试、构建、安装或真机验收；最终统一测试与真实运行态结果将在第 7 步一次回写。

## 关闭前复查修复第 2 步（2026-07-29）

- 注销挑战与确认都从 Bearer session 取得唯一目标 `cid_number`，并各自读取 finalized
  链身份复核该 CID 当前绑定账户；请求 `account_id` 必须等于 session 当前账户，且 CID
  被编入动作挑战签名上下文。换绑后旧会话、其它账户或跨 CID 挑战均不能触发注销。
- Worker 删除入口从账户语义收口为 `purgeIdentity(cid_number,
  authorization_account_id)`：后者只清本次临时挑战与当前账户短缓存；Chat、设备子钥、
  通讯录、会员、创作者关系、上传、帖子、媒体、关注、信号、浏览、通知、防重放、额度和
  用量全部按 CID 删除，跨换绑账户不构成第二删除分支。
- Session 明文 token 只返回客户端；KV 键和 D1 `square_sessions.session_token_hash` 均只
  保存 `SHA-256(token)`。D1 以 `cid_number` 建立强一致注销索引，注销先按 CID 查出全部
  哈希并逐个删除 KV，再在最终 D1 batch 删除索引行；不依赖 KV `list(prefix)` 的最终一致
  结果。换绑吊销只查询同一 CID + 旧账户的哈希，新账户会话保持在线；过期索引随既有
  Worker 定时清理删除。
- `square_login_challenges` 在创世基线增加必填 `cid_number`：登录挑战创建和 Session
  签发分别读取最新 finalized 双向绑定，换绑竞态直接拒绝；注销按 CID 删除历次换绑账户
  的全部挑战，换绑吊销只删同一 CID 下旧账户挑战，不再只清当前授权账户。
- 本机私有 `citizenconsole/actions/cloudflare.sh` 的 D1 重建清单同步加入
  `square_sessions`，创世后表数断言由 25 更新为 26；只改现有本机运维脚本，不纳入 Git、
  不部署、不读取或输出任何 Secret。
- 注销先按 CID 读取全部上传记录并严格校验 `object_keys_json` 与记录中的发布账户和
  `post_id`；R2 帖子对象按精确清单覆盖历次发布账户，资料对象按
  `profile/{cid_number}/` 删除。清单损坏或已发布帖缺上传索引时 fail-closed，禁止删除
  内容 D1 行后丢失清理依据，不再保留“生产期迁移工具补齐”。
- 已补写 CID 跨换绑清理、CID 会话清除、旧账户会话隔离、对象清单损坏和注销账户不匹配
  用例；按用户要求本步骤未执行任何测试、构建、安装或真机验收，统一留到第 7 步。

## 关闭前复查修复第 3 步（2026-07-29）

- 修复注销后的本地资料缓存漏删：`CitizenProfileCache` v3 的键是 `cid_number`，注销编排
  现改为 `clear(cid_number)`，不再错误传入当前授权 `account_id`。
- API Session 与 Chat 本地数据仍按 `account_id` 清理，广场本人副本按 CID 清理，原生
  设备子钥按 `walletIndex` 清理；只修正资料缓存参数，没有扩大其它模块边界。
- 现有测试 Fake 改为记录实际清理 CID，并断言服务端成功后传入永久 CID、不得传入账户；
  服务端失败时仍不得触碰本地缓存。按用户要求未执行测试，统一留到第 7 步。

## 关闭前复查修复第 4 步（2026-07-29）

- 删除 `uploads/service.ts` 与 `posts/confirm.ts` 两个把损坏 JSON 退化为空数组的宽松
  `parseObjectKeys`；manifest 上传、完成、feed/本人回灌、单帖删除、权益到期、未完成
  上传清理和注销统一复用 `uploadObjectKeys`。
- 唯一合法清单必须逐项等于由记录中 `account_id + post_id` 生成的规范 manifest 路径；
  非法 JSON、空数组、额外键、错误账户路径和错误帖子路径统一
  `upload_object_keys_invalid`，不保留任意对象键或路径猜测分支。
- 删除路径在 provider、R2、D1 副作用前完成验证；已发布帖子缺上传索引时
  `post_upload_index_missing` 并保留帖子。到期上传逐项记录失败并继续同批其它项，失败项的
  provider/R2/D1 保持完整供修复后重试。
- 现有测试代码补充五类损坏清单、缺上传索引、到期清理零副作用和正确清单删除；旧回灌
  错误码同步收口。按用户要求未执行测试，统一留到第 7 步。

## 关闭前复查修复第 5 步（2026-07-29）

- 删除可脱离内容删除而单独执行的媒体用量释放入口；保留的 D1 语句构造器只负责生成
  释放语句，并明确要求调用方将其与对应内容行删除放入同一个原子 batch。
- 用户注销不再先扣减全局 `resource_totals`。Images/Stream、R2、KV 等跨存储清理全部
  完成后，媒体总量释放与该 CID 的上传、帖子、媒体及其它 D1 行删除一次原子提交；提交前
  失败时总量与内容索引均保留，提交成功后媒体行消失，重试不会再次释放同一用量。
- 现有注销测试 Fake 记录 D1 batch 边界，并补充“KV 中途失败不释放、重试后释放语句与
  媒体删除同批且只出现一次”回归；同步更新安全规则、统一协议、CitizenApp 架构与广场
  技术文档。按用户要求未执行任何测试、构建、安装或真机验收，统一留到第 7 步。

## 关闭前复查修复第 6 步（2026-07-29）

- `WalletManager` 接入可测试的 `DeviceSubkey` 清理器。普通删除热钱包、删除末账户级联
  钱包删除和 `clearWallet()` 均销毁对应 `walletIndex` 的 Android Keystore / iOS Secure
  Enclave P-256 子钥；冷钱包与删除非末账户不触碰整钱包共享子钥。
- 钱包删除同步清除全部账户 child、通讯录当前/旧命名密钥、本地数据密钥信封和缓存、
  钱包 KEK。每项独立尝试，首个失败不再跳过后续密钥，全部完成后统一抛
  `WalletLocalCleanupException`；事实数据提交后先广播钱包版本，UI 不停留在已删除身份。
- 现有钱包测试补充整钱包删除、非末账户、全量清空、冷热钱包隔离及账户 child 与设备
  子钥同时失败仍继续清理的回归代码；同步订正“建钱包即绑定设备子钥”的旧协议文案为
  CID 功能入口懒绑定。按用户要求未执行测试、构建、安装或真机验收，统一留到第 7 步。

## 关闭前复查修复第 7 步（2026-07-29）

- 最终复查发现注销仍保留 `chain_transaction_confirmations` 与 `topup_orders`，已按用户
  确认的永久身份主键彻底收口：两表均记录 `cid_number`，注销最终 D1 原子批次按 CID
  删除全部记录；`account_id` 只保留签名账户或交易目标事实，不作为注销范围。
- 充值意图创建必须从 finalized 链身份读取目标账户当前绑定 CID，并把 CID 编入服务端
  签名载荷和订单行；链 RPC 失败直接拒绝，不允许把身份解析失败退化成无 CID 订单。非
  CitizenApp 身份目标仍可为空 CID，但不属于任何身份注销范围。
- 硬件金库账户 child 密文键已唯一收口为 `account_child_key_<account_id>`，旧命名及
  读取分支均已删除，不保留双轨兼容。
- Worker 类型检查通过；全量 32 个测试文件、220 项测试通过。Flutter 全量静态检查零
  问题；全量 1030 项测试通过、5 项因宿主原生库能力按既有条件跳过、0 项失败。
- 全新隔离本地 D1 执行 60 条创世语句成功，共 27 张表，外键检查零错误；两张交易表的
  CID 列及约束符合目标契约。真实本地 Worker `/health` 返回 200，未授权注销入口返回
  401；未连接生产 D1，也未部署。
- Pixel 8a（Android 16）隔离原生设备子钥测试 1/1 通过，Android debug 构建成功；
  没有清除 App 数据。Xcode 27 beta 完成 CitizenApp arm64 模拟器构建，产物最低 iOS
  版本为 16.0；按用户明确要求未执行 iPhone 真机验收，也不把 Apple 开发者账户列为
  当前任务依赖。
- 最终检查未改动 `citizenchain/runtime/`、清算行或扫码签名 UI；未创建分支、worktree，
  未暂存、提交、推送或部署。测试临时 D1、Worker 状态和 Xcode 包装目录均已删除。
