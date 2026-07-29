# Cloudflare Worker 身份主键 account_id → cid_number 彻底重构

状态:✅ **DONE**(2026-07-28,R1–R6 全部落地;门禁 tsc EXIT=0 + vitest 30 文件/181 测试全绿;待前端 Flutter 同步)
所属模块:citizenapp/cloudflare(Worker/D1/广场·聊天·会员·通讯录 BFF)

## 背景(用户揪出)
公民 App 身份主键早已切为 **CID 号**([[citizenapp-cid-identity-master-key]]:CID=身份/账户不可改,钱包 account_id=控制该身份的签名凭证/"密码",可换绑),但 **Cloudflare Worker 从没跟上** —— 全库仍以 `account_id`(钱包账户)为身份主键,CID 只在 square_posts 有一个可空快照列。后果:换绑钱包后新账户登录 resolve 到同一 CID,但数据全挂旧 account_id 下且被 rebind/revoke 删掉 → 用户社交资产(动态/文章/粉丝/通讯录/会员/订阅)全丢,正是母卡要根治的背景漏洞在 worker 侧没实现。

## 用户三条硬约束(2026-07-28)
1. **cid_number = 用户唯一身份主键**,worker 所有用户数据必须以它为主键存。
2. **操作签名 = 该 cid_number 当前绑定的钱包账户**;命名严格 `account_id`(Substrate 标准全称,禁 `account` 缩写)[[wallet-account-naming-account-id]]。
3. **彻底重构:不迁移、不残留、不退化、不兼容**(dev 期零用户 [[in-development-zero-users]],直接重建 schema,无 migration)[[no-compatibility]][[no-remnants]]。

## 现状实锤(citizenapp/cloudflare/)
- **schema**(migrations/0001_square_core.sql):15+ 表全 `account_id` 主键/归属(设备子钥/通讯录/会员/创作者档位/订阅/上传/媒体/文章/粉丝/通知/浏览/用量/聊天设备/密钥包);`cid_number` 全库仅 1 处(square_posts:201 可空快照列,无键无索引)。
- **登录/会话**(auth/service.ts):挑战 payload=`account_id‖challenge‖expires`;会话=account_id;设备子钥注册在 account_id 下。
- **鉴权**(security/request_guard.ts):每写=会话(account_id)+P-256 设备子钥签名+nonce;限流 key 也是 account_id。
- **链上解析**(chain/identity.ts:80):worker 已能 account_id→CID 解析(读链 CidByAccount + CidRegistry),但只当派生属性,没当主键。
- **换绑**(rebind/service.ts + account/purge.ts):/v1/square/rebind/revoke = 全表 DELETE by 旧 account_id(错:删掉该 CID 资产)。
- 路由面 ~45 endpoint(chain/chat/constitution/security/square:account/auth/contacts/creator/feed/follows/media/membership/notify/posts/profile/rebind/signals/uploads/users)。

## 修正后的正确模型(用户 2026-07-28 纠正,已核实)
- **cid_number = 唯一身份主键**:worker 所有**用户数据**以它存。
- **account_id(sr25519 钱包账户)= 控制该身份的当前凭证**(可换绑=改密码):①链上绑定到 cid_number;②签**链上交易**并付费;③生成/授权设备子钥。**子钥由钱包账户生成 → 属于 account_id,不属于 cid(cid 只是号,不能生成密钥)**。换绑 → 旧 account_id 及其子钥作废不可用,新 account_id 重新生成注册子钥。
- **两类操作分清(区块链 vs 公民app)**:
  - **链上交易(付费)= 发布动态/文章**:真链上 extrinsic(SquarePost pallet),**钱包 account_id 签名+付费**(子钥不能签链上交易);worker 只 **relay + confirm**(读链上 `SquarePostPublished` 事件)→ 按 cid_number 镜像到 D1。会员/订阅/创作者定价同属链上镜像。
  - **off-chain 私有数据(不付费)= 删除自己的动态/文章、通讯录增删、关注、浏览、通知**:只改 worker D1;**设备子钥(account_id 的委托签名)签名**即可 → 按 cid_number 增删。

## 重构骨架(关联闭包纵切;用户 2026-07-28 选 A 定案)
切法定案:15 张用户数据表经 account_id 强关联(feed 作者/会员徽章/关注流/profile 计数),
`account_id` 在链上相关表是**终态保留列**(=当前绑定签名账户/链上收款账户,非兼容 shim)。
故按「关联闭包」纵切:每步 = 一组自洽表 + 其全部 handler + 测试,身份归属键 account_id→cid_number,
未切换表暂经各表保留的 account_id 列关联,每步门禁绿、零兼容列。原「R2 纯 schema / R3-R7 纯 handler」
横切法会产生 tsc 红半截态,已弃。
- **R1 身份地基/登录** ✅ 完成 2026-07-28:session/identity=cid_number。登录:钱包 account_id 签挑战 → 验签(wallet_signature)+ 链查 CidByAccount 得 cid_number(须当前确绑,否则访客)→ 发 cid_number 会话;设备子钥注册记 (cid_number, 当前 account_id),验签时校该子钥 account_id == 链上 cid 当前绑定账户(换绑后旧子钥自然被拒)。
- **R2 广场社交闭包** ✅ 完成 2026-07-28:posts / uploads / media / follows / notify_reads / signals / browse + profiles(资料·头像资产 key·计数)+ feeds。归属键 account_id→cid_number(链上镜像表 posts/uploads/media 保留 account_id 列);follows 双端 cid;posts/confirm 从链上 `SquarePostPublished` 取 cid 镜像;头像资产 key `profile/{account_id}/…`→`profile/{cid_number}/…`。
- **R3 会员/创作者/订阅/资源镜像** ✅ 完成 2026-07-28:memberships / creator_tiers / creator_subscriptions / resource_reservations / resource_usage 归属键 account_id→cid_number(镜像表保留 account_id);feed 读会员改按 cid。
- **R4 通讯录** ✅ 完成 2026-07-28:square_contacts PK (account_id, contact_id)→(cid_number, contact_id),设备子钥鉴权。
- **R5 chat** ✅ 完成 2026-07-28:chat_devices / chat_keypackages / chat_device_binding_nonces 归属/路由按 cid_number,设备/密钥属 account_id 保留。
- **R6 收尾** ✅ 完成 2026-07-28:`account/purge` + rebind/revoke 语义重构(按 cid 删身份、去掉"删旧账户数据");保留表 `chain_transaction_confirmations` / `topup_orders` / `square_login_challenges` 归属决策落地(默认保留 account_id=链上事实,如需"我的交易/充值"跨换绑聚合再加 cid 冗余列);全套 e2e 门禁:换绑后同一 cid 社交数据不丢。

## 设计原则定案(用户已答)
- 设备子钥**属钱包账户**(账户生成);换绑→旧账户+旧子钥作废,新账户重注册子钥;worker 靠链上 CidByAccount 实时校当前绑定,不搞子钥迁移。
- 发布=链上交易走钱包 account_id 签名付费;删除/通讯录等 off-chain 走设备子钥。

## 门禁
worker `vitest` 全绿 + `tsc` 0;换绑后同一 cid_number 社交数据不丢(端到端);account_id 全称命名零缩写。

## R1 落地记录(2026-07-28)
- `src/types.ts`:`SessionState` 增 `cid_number`(身份主键)+ account_id(当前绑定凭证);`DeviceSubkeyRow` 增 `cid_number`/`device_id`,主键语义 (cid_number, device_id)。
- `src/chain/identity.ts`:`fetchChainIdentityState` 返回 `cid_number`=账户当前绑定的 active CID(双向绑定 + CidRegistry active 校验后),匿名/投票/竞选一视同仁;投票/竞选只作 `identity_level`/`has_*_identity` 属性,不决定"有没有身份"。
- `src/auth/service.ts`:`createSession` 先 resolve cid_number(未绑定 → 403 `cid_not_bound`),按 `WHERE cid_number=? AND account_id=?` 取多设备子钥逐个验签,会话写 `{cid_number, account_id, device_key_hash}`;`registerDeviceSubkey` resolve cid_number + `device_id=sha256(p256)` + `ON CONFLICT(cid_number, device_id)` 单调覆盖(同一身份多设备并存)。
- `src/security/request_guard.ts`:每请求链上绑定复查(`identity.cid_number !== session.cid_number` → 401 `cid_binding_changed`);限流 key `cid_number:${session.cid_number}`;`requireDeviceProof` 按 (cid_number, device_id) 定位子钥并校 `account_id` 一致;nonce hash + 落库按 cid_number。
- `migrations/0001_square_core.sql`:`square_device_subkeys` PK(cid_number, device_id)+ 索引(cid_number, account_id);`square_request_nonces` 键列 account_id→cid_number。
- 测试:`test/{device_subkey,contacts,auth}.test.ts` 重写为多设备/CID 映射;8 处 `SessionState` fixture 补 `cid_number`。门禁 `tsc` EXIT=0、`vitest` 29 文件/179 测试全绿。
- 修正过时注释:`registerDeviceSubkey` 头注释由"一账户一活跃子钥覆盖"改为"(cid_number, device_id) 多设备并存 + 同设备单调覆盖"。
- 边界说明:`session_index.ts`(account→token 注销索引)、`square_posts.cid_number` 快照列、`chat_devices`/`keypackages`(account_id, device_id)分别属 R6/R3/R7,R1 未动,非残留。

## R2 落地记录(2026-07-28)
- **schema**(`migrations/0001`):square_posts(cid_number 升 NOT NULL 主归属 + account_id 保留 + 索引 cid_number)、square_uploads/square_media_assets(新增 cid_number + 保留 account_id)、square_follows(双端 (follower_cid_number, followed_cid_number))、square_notify_reads/square_user_signals/square_browse_days(account_id→cid_number)。
- **归属解析**:profile/关注/发帖/删除/上传归属校验全按 cid_number;路由内部用 `fetchChainIdentityStateCached` 把"目标钱包账户"resolve 成 cid(profiles/service `resolveTargetCid`、feeds/follows `resolveFollowedCid`);feed 徽章 `resolveAuthorSignals` 改按 (cid, account) 对——资料按 cid 读、身份/会员按当前绑定 account_id 读。
- **发布确认**(posts/confirm):从链上 `SquarePostPublished.cid_number` 取发布者 cid 写入(NOT NULL,null 抛 409 `square_event_cid_missing`);删除/上传归属按 cid(`post_owner_mismatch`/`upload_owner_mismatch`)。
- **profile R2 key**:`profile/{cid_number}/…`(换绑头像不丢);新增 `assertCidNumber`(ids.ts,纵深防御路径穿越)。
- **通知扇出**(notify_fanout):follows 按 cid;设备经 R1 的 `square_device_subkeys(cid_number→当前 account_id)` JOIN `chat_devices` 桥接(chat_devices 零改,留 R5)。
- **purge**(account/purge,R6 目标):仅修列有效性——resolve cid 后按 cid 删 user_signals/follows/browse/notify_reads/request_nonces/rate_windows(顺带修 R1 遗留的 request_nonces/rate_windows 失效列);语义 reshape(换绑不删迁移数据)仍留 R6。
- **测试**:notify/notify_fanout/feed/profiles/chain_confirm/account/profile_assets/uploads_quota 全改 cid 模型 + chain identity mock。门禁 `tsc` EXIT=0、`vitest` 29 文件 / 179 测试全绿。
- **未改(边界)**:limits/usage(resource_* 属 R3、uploads 仅按 upload_id 更新状态);post manifest/媒体 provider R2 key 仍按 account_id(发布签名者路径,post 行保留 account_id);membership/creator/resource_* 归属仍 account_id(R3);chat_* 表 account_id(R5)。

## ⚠️ 前端跟进(R2 改了对外 API 契约,Flutter 须同步)
- 关注/粉丝列表响应字段 `accounts` → `entries`,项由 `{account_id}` 变 `{cid_number, created_at}`(前端据 cid 展示/跳转)。
- feed/profile 响应作者身份主键 = `cid_number`(account_id 仍在,作当前绑定);前端标识用户应逐步以 cid 为主键。
- `GET /users/:account`、关注/取关/通知 API **入参仍收目标钱包 account_id**(worker 内部 resolve),前端 URL 契约暂不变;但若前端手上只有对端 cid(如从关注列表 entries),需要能按 cid 定位用户 → 后续可加 cid 变体路由(记为前端联调项)。
- 依 [[dto-field-rename-bump-cache-version]]:前端接入时须 bump 相关缓存版本 + 形状校验。

## R3 落地记录(2026-07-28)
- **schema**(`migrations/0001`):square_memberships(PK→cid_number + account_id 保留)、square_creator_tiers(PK (creator_cid_number, tier_id) + creator_account_id 保留)、square_creator_subscriptions(PK (subscriber_cid_number, creator_cid_number) + 双 account 保留)、resource_reservations(account_id→cid_number)、resource_usage(PK (cid_number, resource_key, period_start))。
- **会员读取翻转**:`getMembership(env, cidNumber)`、`batchMemberships(env, cidNumbers)`(map 键 cid)、`requireActiveMembership(env, cidNumber)` 全按身份主键;清除 R2 遗留的"会员暂按 account_id 读"临时桥接——`resolveAuthorSignals`(feed 徽章)、`browse`(getBrowseState 删 accountId 参)、`profiles`(buildProfileResponse 按 targetCid)、uploads/posts 发布闸门全改按 cid。
- **镜像写入**:citizen_coin `mirrorPlatformState`(cid PK + account 列,ON CONFLICT(cid_number))、creator `replaceCreatorTiers`/`mirrorCreatorSubscription`(cid 主键 + account 列);confirm 路由从 session/body 账户 resolve cid(creator 新增 `resolveBoundCid`)。
- **charge_due keeper**(reconcile):平台/创作者对账**按 cid 迭代镜像、回链读订阅用当前绑定 account、更新按 cid**(用户定案);creator 复合主键 (subscriber_cid, creator_cid)。
- **资源用量**:`reserveUploadUsage` 入参 `cid_number`(原 account_id)、`consumeUploadUsage` RETURNING/INSERT 按 cid;uploads/service 调用改传 session.cid_number。
- **视频冷归档**(archive):`selectLapsedCidNumbers`(JOIN a.cid_number=m.cid_number)、`selectVideoAssets(cid)`、`restoreAccountVideos(cid)`;R2 归档对象 key 仍按每行 account_id(发布签名者路径)。
- **purge**:memberships/resource_*/creator_* 移入 cid 删除分支(并修 R2 遗留的 resource_* account_id 失效列),补 creator 表删除防孤儿;account_id 分支仅剩 uploads/posts/media/device_subkeys/login_challenges。
- **链交易验证不变**:verifyTransaction/readSubscriptionAtBlock/readCreatorPlansAtBlock/bindFinalizedTransactionConfirmation 仍按 account_id(链查入口 + chain_transaction_confirmations 保留表)。
- **测试**:feed/membership/membership_reconcile/creator_reconcile/archive/account/profiles 全改 cid 模型。门禁 `tsc` EXIT=0、`vitest` 29 文件 / 179 测试全绿。残留自审:memberships/creator/resource 零 account_id 归属残留。
- **⚠️前端**:creator plan 响应字段 `creator_account_id` → `creator_cid_number`(前端须同步)。

## R4 落地记录(2026-07-28)
- **schema**(`migrations/0001`):square_contacts PK (account_id, contact_id)→(cid_number, contact_id),索引 cid_number;去掉 account_id 的 hex CHECK。
- **contacts/service.ts**:list/put/delete 三路由 SQL 与 bind 全按 `session.cid_number` 属主隔离;`publicContactRow` 省略 cid_number(响应仍不下发属主键);`ContactCiphertextRow.account_id`→`cid_number`。设备子钥鉴权由 guardRequest 已按 cid,不变。
- **purge**:通讯录密文按 cid 归属——`purgeAccount` 从 step1 移入 step4 cid 分支;**`revokeRebindOldAccount` 不再删 square_contacts**(密文按 cid 随身份保留给新账户,AEAD 密文对旧账户已无解密价值),吊销只删 chat 端到端材料/登录挑战/设备子钥(账户级)。
- **顺带修 R2 残留 bug**:`purgeAccount` step3 的 R2 资料前缀从 `profile/{account_hex}/` 修正为 `profile/{cid_number}/`(R2 起 profile key 已迁 cid 路径,原按账户前缀会漏删头像/资料→注销隐私 bug);cid resolve 前移到 step3 前。
- **测试**:contacts.test.ts 按 cid 属主 + 存储行键断言 cid_number;account.test.ts purge/revoke 断言更新(contacts 入 cid 分支、revoke 不删 contacts、profile R2 路径按 cid)。门禁 `tsc` EXIT=0、`vitest` 29 文件 / 179 测试全绿。残留自审:square_contacts 零 account_id 归属;profile R2 前缀生成+删除全 cid 一致。

## R5 落地记录(2026-07-28)
- **schema**(`migrations/0001`):chat_devices PK (cid_number, device_id)+account_id 保留(设备所有者)+索引 cid_number;chat_keypackages +cid_number(JOIN/索引 cid)+account_id 保留;chat_device_binding_nonces PK (cid_number, nonce_hash)(**去 account_id 列**)。
- **收件寻址(A 定案)**:会话层全按身份主键 cid_number 寻址——`requireActiveDevice(cid, device)`、KeyPackage 发布/拉取/领取按 cid、`submitChatEnvelope/Signal` 收件人 `recipient_cid_number`、DO 命名空间 `getByName(cid_number)`(每身份一信箱,换绑后同一 cid)、relay/wake 按收发件人 cid。
- **codec/binding**:新增 `assertChatCidNumber`(严格全称,复用 assertCidNumber);设备绑定 `ChatDeviceBindingInput.account_id` **保留不动**(设备属账户,由账户 P-256 子钥签,跨端 SCALE golden);注册仍用 `square_device_subkeys WHERE account_id`(账户子钥)验签。
- **realtime**:`ChatRelayPayload` sender_cid_number/recipient_cid_number;WS attachment `{cid_number, device_id}`,连接头 `x-chat-cid-number`;`closeChatRealtime(env, cidNumber)`。
- **push**:`sendChatWake(env, recipientCidNumber, senderCidNumber)` 按 cid 查 chat_devices;WakePayload `{kind:'chat_wake', sender_cid_number}`。
- **notify_fanout 简化**:删 R2 的 `square_device_subkeys` 桥接,直接 `chat_devices WHERE cid_number IN (...)` 取粉丝设备(回收 R2 临时 workaround)。
- **purge**:chat DO 按 cid 关闭(先 resolve cid,cid 存在才 close);chat_devices/chat_keypackages 按 account_id 删(设备/密钥属账户,注销/换绑吊销就该断旧设备);chat_device_binding_nonces 按 cid_number 删(表已无 account_id 列)。
- **测试**:chat.test.ts / notify_fanout.test.ts / account.test.ts 全改 cid;门禁 `tsc` EXIT=0、`vitest` 29 文件 / 179 测试全绿。残留+严格命名自审:chat 读侧/路由/归属全 cid(account_id 命中仅 purge 按设备所有者删);无裸 `account` 缩写;`recipient/sender_cid_number` 全切。
- **⚠️前端(chat 契约变更,Flutter 须同步)**:会话发信 `recipient_account_id`→`recipient_cid_number`;KeyPackage 发布体 `account_id`→`cid_number`、拉取路由末段用目标 `cid_number`、领取体 `cid_number`(删 requester 字段);WS 握手头 `x-chat-account-id`→`x-chat-cid-number`;推送唤醒 payload `sender_account_id`→`sender_cid_number`。

## R6 落地记录(2026-07-28,收尾结题)
- **注销语义(purgeAccount)**:注销=删身份——身份内容(square_posts/uploads/media_assets)与全部 off-chain/镜像表按 **cid_number** 删(删该身份跨换绑账户的全部内容);媒体 provider 本体按 cid 载入删除;仅账户级鉴权凭证(square_device_subkeys/square_login_challenges)按当前 account_id 删。R2 对象(帖子 manifest/归档原片)按发布 account 段删(dev 无换绑历史故完整,跨账户历史前缀待生产迁移工具)。
- **换绑吊销(revokeRebindOldAccount)**:已确认接线 `rebind/service.ts`;只删旧账户账户级鉴权材料(chat 端到端/登录挑战/设备子钥/会话),**不删** posts/media/memberships/follows/通讯录(按 cid 随身份留存);注释订正对齐实现。
- **保留表决策(无 schema 改动)**:`chain_transaction_confirmations`(仅按 tx_hash 查,account_id=签名者事实)、`topup_orders`(按 tx/intent/order_id 查 + account_id 订单归属校验,账户级财务记录)、`square_login_challenges`(挑战阶段 cid 未 resolve,主体=待验证账户)——三表保留 account_id,不加 cid。
- **e2e 门禁**:新增 `test/rebind_data_survives.test.ts`——账户 A 写入通讯录密文,"同一 cid + 账户 B"会话按 cid 取回同一密文;会员镜像按 cid 读取跨账户不丢。
- **全仓最终审计**:剩余 account_id 归属谓词全部合法(chat 设备/密钥属账户、square_device_subkeys 登录凭证、square_login_challenges 主体);社交/身份数据零 account_id 残留;严格命名核验无裸 `account` 缩写(routes.ts 路由段局部 `account`→`accountId`;chain/subscription.ts SCALE 解码器 32B 局部非归属字段不动)。
- **门禁**:`tsc` EXIT=0、`vitest` 30 文件 / 181 测试全绿。

## 结题汇总(R1–R6)
Worker 全库用户数据身份主键 account_id→cid_number 彻底重构完成:
- R1 会话/登录/设备子钥;R2 广场社交闭包(posts/uploads/media/follows/notify/signals/browse/profiles/feeds);R3 会员/创作者/订阅/资源镜像;R4 通讯录;R5 chat(设备/密钥/nonce/路由/DO/推送);R6 注销/换绑语义 + 保留表 + e2e。
- 15+ 张用户数据表全按 cid_number 归属;account_id 仅作链上镜像/凭证表的"当前绑定/签名者"保留列。
- 换绑不丢:社交/身份数据随 cid 存续,旧账户鉴权材料吊销、新账户重注册子钥/设备。
- **待前端 Flutter 同步**(汇总):关注列表 `accounts`→`entries`(cid_number 项);feed/profile 作者身份主键=cid;creator plan 响应 `creator_cid_number`;chat 全契约(见 R5 前端条)。前端接入按 [[dto-field-rename-bump-cache-version]] bump 缓存版本+形状校验。
