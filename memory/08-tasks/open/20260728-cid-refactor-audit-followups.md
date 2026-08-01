# CID 重构审计遗留项(部署 + 跨层洞 + 文档回写)

状态:open(2026-07-29 按用户最终 CID 身份契约重新打开全面审计；旧“已完成”结论须逐项复验，当前执行第 1 步绑定协议改造)
来源:`done/20260728-cloudflare-cid-identity-primary-key.md` 的「全面审计 + 修复」一节
所属模块:citizenapp(cloudflare worker + Flutter)/ citizenchain(square-post)/ 文档

## 2026-07-29 复审裁定

- 旧换绑签名只覆盖 `(cid_number, new_account_id)`，op-tag 只提供域隔离，不能防重放；改为 CID 单调绑定版本并绑定创世哈希、预期旧/新账户和过期时间。
- 旧“新账户会话携旧账户换绑授权再清理”的方案被用户明确废弃。链上 finalized 换绑本身就是 CID 控制权转移的唯一授权事实；旧账户级材料必须按链上绑定版本自动失效和异步清理。
- 订阅关系归 CID，换绑后由新绑定账户继续付款属于正确产品规则，不得改成暂停或重新授权。
- 个人多签仍以登记的管理员 `account_id` 为签名人，不得把个人多签管理员、管理员快照或个人投票票据改成 CID。
- 当前冻结创世资产与最新 runtime 不一致；本卡所有问题修复并完成真实验收前禁止创世、推送或部署。

## 处理进度(2026-07-28)
- **#5 unified-protocols.md 回写 ✅ 已完成**:广场/资料/关注端点(cid 路由 + entries)、D1 全表字段(15+ 表 cid 归属 + 保留表说明)、chat HTTP/DO/D1/proto/验签规则(recipient_cid_number、(cid,device_id) 验签、删 requester、profile R2 key→cid)全部改到位;`:498`/`:1075`/`:1079` 把已修缺陷写成规范的三处已订正。**其余 05-modules 下 CHAT_TECHNICAL/USER_TECHNICAL/PROFILE_TECHNICAL、subscription-part1-tech.md 仍待回写**(下同,未做)。
- **#1 换绑后旧凭证失效协议 ♻️ 已按 2026-07-29 定稿重做**:旧账户签名 outbox、
  新账户代调用吊销 endpoint 和第二授权协议全部废弃；链上 finalized 绑定版本是唯一
  控制权转移事实。客户端不得保存旧签名清理任务，Worker HTTP 路由已删除，只保留由
  finalized 绑定版本消费者调用的内部幂等账户凭证清理 helper。
- **#2 链上订阅账户键 ✅ 已完成**:SquarePost 帖子、订阅、续费索引、创作者套餐和
  跨端读写均已直接使用 `cid_number`；无迁移、无旧键、无兼容。P0 续费时序修复和
  双节点 +40 天换绑续费验收已完成，详见
  `memory/08-tasks/20260728-square-post-cid-primary-key.md`。
- **#4 生产数据重建准备 ✅ 已完成**:现有 `reset-formal-data` 已逐项核对唯一 D1
  基线、KV、两个 R2 桶和通知队列；本窗口不执行生产清理、不修改正式节点部署。
  真正重创世及同批生产执行交由创世线程按下方清单完成。

## P0 — 发版前必须执行(否则线上必炸)

### 1. 生产数据重建（创世前准备完成，待创世线程执行）
`citizenapp/cloudflare/migrations/0001_square_core.sql` 是**原地重写的唯一基线**(25 张 `CREATE TABLE`,无 `DROP`、无 `IF NOT EXISTS`),而部署流程按 `20260713-citizenapp-cloudflare-deploy-scripts` 的规则**刻意不执行**它。本次重构把 15+ 张表主键从 `account_id` 改为 `cid_number`,不重建则新代码查 `cid_number` 列会打到旧表 → 全线 500。
- 工具已存在且正确:`citizenconsole/actions/cloudflare.sh` 的 `reset_d1()`(DROP 全部 25 表 + `d1_migrations` → 执行 0001 → 断言表数=25)与 `clear_kv()`(清空 SQUARE_CACHE)。
- 挂在**独立手动动作** `reset-formal-data`(`server.mjs`),不在 `production` 部署流程里。
- **安全边界**:`reset-formal-data` 标记为 production，必须经 Touch ID；D1/KV/Queue
  只使用 `CF_DATA_TOKEN`，R2 另使用账户级 R2 凭据。普通 `production` 部署不得自动调用
  该破坏性动作。
- 顺带解决:KV 身份缓存旧形状(`square_identity:` 的 value 新增 `cid_number`、读侧无形状校验,TTL 45s)、R2 陈旧对象。

#### 创世线程执行顺序

1. 完成新链创世资产冻结和全部正式节点重创世，确认所有节点的 block #0 哈希一致；
   本窗口不承担该步骤。
2. 把同一个新创世哈希写入 Cloudflare `CHAIN_GENESIS_HASH` 真源并完成跨端冻结资产核对；
   禁止继续使用旧哈希或手工第二真源。
3. 在 CitizenConsole 单独执行 `CitizenApp Cloudflare → 清空并重建全部数据`，通过该次
   Touch ID 后依次重建 production D1、清空 production KV、清空
   `citizenapp-square-media` / `citizenapp-chat-relay` 两个 R2 桶并 purge
   `square-notify-fanout-production`。该动作不可恢复，执行时必须由创世线程再次取得用户
   对明确生产目标的确认。
4. 数据重建成功后立即执行 Cloudflare `production` 部署，使新 Worker、Secret、
   `CHAIN_GENESIS_HASH` 和新链同时生效；任一步失败立即停止，不得跳步或报部分成功。
5. 真实验收必须确认：D1 业务表数为 25、KV 键数为 0、两个 R2 桶对象数为 0、
   通知队列已 purge、Worker `/api/health` 返回成功，并从 Worker 使用的链 RPC 读取到
   与冻结资产相同的 block #0 哈希。

#### 本窗口准备验收（2026-07-29）

- `0001_square_core.sql` 实际解析为 25 张业务表；`reset_d1()` 白名单与其逐表排序对比
  完全一致，数量同为 25。
- `reset-formal-data` 的资源范围已核对为 D1、`SQUARE_CACHE` KV、两个固定 R2 桶和
  `square-notify-fanout-production`，没有清算行资源。
- `bash -n citizenconsole/actions/cloudflare.sh`、`node --check citizenconsole/server.mjs`
  通过；CitizenConsole 37 项安全与业务自动测试全部通过。
- 未连接 Cloudflare 生产数据面，未调用 Wrangler 远端写命令，未执行 D1/KV/R2/Queue
  删除，未修改 CitizenChain 正式节点部署脚本。

## P1 — 需产品/架构决策

### 2. 换绑后旧账户凭证失效（2026-07-29 现行裁定）

- 链上 finalized 的 `AccountIdByCid + CidByAccountId + BindingRevisionByCid` 是 CID
  控制权转移的唯一授权事实。旧账户不再拥有 CID 后，登录与每请求绑定复查立即
  fail-closed；客户端不得再提交第二份旧账户签名证明。
- CitizenApp 已删除旧账户换绑授权 outbox、启动重试和客户端换绑后吊销调用；
  Worker 不提供对应 HTTP endpoint、路由资源登记或外部请求模型，不得恢复。
- App 先按 finalized 当前绑定完成稳定数据根恢复、新账户包装读回、用途子钥安装和新
  钱包设备子钥登记；Worker 只有确认新子钥成功落库后，才按 CID + 当前
  revision/account 清理旧 `chat_keypackages`、`chat_devices`、
  `square_device_subkeys`、旧登录挑战、旧会话和实时连接。该清理不接收旧账户签名，
  也不构成第二授权。
- `ChatRealtimeObject`、`chat_device_binding_nonces`、通讯录、动态、文章、粉丝、
  关注、媒体和会员均按 CID 归属，换绑时不得整体关闭、删除或迁移。
- 连续 A→B→C 换绑由链上单调 revision 串行化；每笔授权都绑定创世哈希、预期旧/新账户、
  预期 revision 和过期时间，旧授权不能跨链、跨版本或跨目标重放。

### 3. 链上订阅账户键问题（已按 CID 终态解决）

SquarePost 的 `Subscriptions`、`RenewalSchedule`、`RenewalIndex`、`CreatorPlans` 已全部直接
使用 `cid_number` 用户键；`account_id` 仅保留为当前签名、实际付款/收款或不可变交易审计。
换绑不迁移 storage、不读取旧键、不保留兼容分支；自动续费每次由 CID 解析并双向复核当前
绑定账户，旧账户不得再扣。正式链未创世，使用新 storage 重新创世。

P0 真实链验收发现续费若发生在 finalize 后阶段会被 NodeGuard 误判为未经登记发行；
终态改为 `on_initialize` 使用上一块已确认时间处理，最多延后一个区块，所有付款与收款
余额变化均进入 finalize 前状态。该修复只涉及 SquarePost 和 NodeGuard 隔离测试，未修改
清算行 storage、RPC 或支付流程。

### 4. 通讯录孤儿密文累积（2026-07-29 已按 CID 终态解决）

通讯录关系、稳定联系人 ID、备注与云端密文归属已经改为 CID；换绑时 finalized 当前新账户
通过独立恢复层取得**同一把 CID 稳定数据根**，只用自己的 child 重包这 32 字节，不派生新
数据根、不重加密业务密文，也不要求旧账户、旧私钥或旧设备参与。联系人当前账户由 finalized
链上正向映射、绑定版本与反向映射双重闭包刷新，不再用账户派生联系人身份，也不会因联系人
换绑新增另一条关系。

## P2 — 文档回写(CLAUDE.md 要求「必须回写 memory/、ADR 或相关文档」)

### 5. 协议/模块文档仍是旧 account 契约
- `memory/07-ai/unified-protocols.md`:`:299-304`(followed_account_id / accounts / `/users/{account_id}`)、`:369-376`(会员·创作者·follows 表主键)、`:478-498`(chat keypackage/envelope 字段与路由)。**`:498` 尤其危险**——把「Worker 只用 session AccountId 查 `square_device_subkeys` 验签」写成了规范,而这正是本次审计修掉的 HIGH 缺陷。
- `memory/05-modules/citizenapp/chat/CHAT_TECHNICAL.md`(14 处 account_id / 0 处 cid_number)、`user/USER_TECHNICAL.md`、`8964/PROFILE_TECHNICAL.md:18-20`、`01-architecture/gmb/subscription-part1-tech.md:175`(仍写「平台订阅主键为 account_id」,而 `project_membership_tax_engineering_architecture` 记忆指它为真源)。

### 6. 开放任务卡闭环
- 母卡 `open/20260727-citizenapp-cid-identity-rootless-wallet.md`:S1/S3/S4/S6/S7/S8.1-8.3d 均已完成但状态仍 open;卡内两条记录与本次落地**相反**(C1 称 revoke 会删 square_contacts —— R4 已反转为不删;残余 note 称「posts/followers 服务端随 CID 迁移未做」—— R2/R6 正是做了)。
- 因 D1 基线重写而失真的 5 张卡:`20260713-citizenapp-cloudflare-deploy-scripts`(其「不重复执行 0001」规则现在是脚枪)、`20260716-membership-decouple-rollout-alignment`、`20260716-citizen-coin-subscription` §14、`20260711-chat-square-step1`、`20260709-citizenapp-hardware-cryptoobject-seed-vault`(其「一账户一活跃子钥」不变量已被 R1 废除)。
- `20260705-citizenapp-default-wallet-identity.md`:仅剩待办引用的字段已不存在,应关闭。

### 7. 记忆条目补充
- `feedback_cid_rebind_subkeys_must_auto_migrate.md`:需补 as-built —— worker 侧**不迁移**子钥(靠链上绑定实时校验判失效),且通讯录重加密是**写新行不覆盖旧行**(见第 4 项)。
- `project_citizenapp_cid_identity_master_key.md`:只描述了客户端,需补 worker 侧(D1 15+ 表按 cid 归属,account_id 仅作镜像/凭证保留列)。
- `feedback-wallet-account-naming-account-id.md`:其「已执行」清单只有 onchina,需补 citizenapp + cloudflare。
- `project_square_chat_onchain_wallet_gate.md`:结论仍成立,但可补「会话现带 cid_number,且 guardRequest 每请求做链上绑定复查」。
- 建议**新建**一条记忆固化不变量:worker 全库用户数据身份主键 = cid_number(目前只存在于一张 done 任务卡里,下个会话极易按旧模型写代码)。

## P3 — 代码质量(独立于本次重构的既有 debt,顺手可改)
- `lib/signer/{square_action_sign_service,citizen_identity_sign_service}.dart`:冷钱包前置拒绝分支 + 专属错误码 + 负向测试被整体删除(入参从 `WalletProfile` 改 `Account` 时漏迁);底层 `WalletManager.signForAccountId` 仍兜底拒绝,**无签名绕过风险**,但注释仍写「拒冷钱包」与实现矛盾,且该防线零测试覆盖。
- `cloudflare/test/creator_reconcile.test.ts`:本次新增的 `interface Party { account: string }` 违反 account_id 全称死规则(同文件 `interface Row` 却是全称)。
- `lib/citizen/public/data/public_institution_dto.dart:71`:读 `legalRepresentative?['account']`,后端序列化实为 `account_id` → 该字段恒 null;三处测试夹具用同样错误的 key 掩盖(当前无消费方)。
- `lib/chat/group/group_flow.dart` 6 处 `.where((account) => ...)` lambda 形参裸命名(同文件 `_deriveMemberRoles` 已改对)。
- `lib/wallet/pages/wallet_page.dart` 的「身份钱包」徽标零测试覆盖(替换掉的旧「默认用户」徽标原有 6 个测试,一并删除未补)。
- `cloudflare/src/shared/ids.ts` 的 `CID_NUMBER_PATTERN` 允许 1–64 字符,链端 `encodeBoundedBytes` 只接受 ≤32 字节 → 33–64 长度会在链读时抛错并软降级为访客。建议收紧到 32。
- `cloudflare/src/account/service.ts` 注销挑战越权与堆积问题已于 2026-07-29 修复：
  challenge/confirm 都要求请求账户等于 session 当前账户并读取最新 finalized CID 双向绑定；
  `square_login_challenges` 增加必填 CID，注销按 CID 全删，过期挑战进入现有定时清理。
- `cloudflare/src/routes.ts:211` 的 `POST /v1/square/signals` 全仓零调用方 → `square_user_signals` 表永不写入,推荐流行为信号维度实际为空。
