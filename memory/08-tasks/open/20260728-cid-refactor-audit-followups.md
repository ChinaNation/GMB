# CID 重构审计遗留项(部署 + 跨层洞 + 文档回写)

状态:open(2026-07-28 全面审计产出;6 项实缺陷已当场修复并入门禁,本卡只收**需决策或独立执行**的遗留)
来源:`done/20260728-cloudflare-cid-identity-primary-key.md` 的「全面审计 + 修复」一节
所属模块:citizenapp(cloudflare worker + Flutter)/ citizenchain(square-post)/ 文档

## 处理进度(2026-07-28)
- **#5 unified-protocols.md 回写 ✅ 已完成**:广场/资料/关注端点(cid 路由 + entries)、D1 全表字段(15+ 表 cid 归属 + 保留表说明)、chat HTTP/DO/D1/proto/验签规则(recipient_cid_number、(cid,device_id) 验签、删 requester、profile R2 key→cid)全部改到位;`:498`/`:1075`/`:1079` 把已修缺陷写成规范的三处已订正。**其余 05-modules 下 CHAT_TECHNICAL/USER_TECHNICAL/PROFILE_TECHNICAL、subscription-part1-tech.md 仍待回写**(下同,未做)。
- **#1 换绑吊销时序、#2 链上订阅账户键**:已出推荐方案(见下),**待用户拍板后实施**。
- **#4 生产 D1 重建**:并入创世部署流程处理(见下),本卡记录。

## P0 — 发版前必须执行(否则线上必炸)

### 1. 生产 D1 重建
`citizenapp/cloudflare/migrations/0001_square_core.sql` 是**原地重写的唯一基线**(25 张 `CREATE TABLE`,无 `DROP`、无 `IF NOT EXISTS`),而部署流程按 `20260713-citizenapp-cloudflare-deploy-scripts` 的规则**刻意不执行**它。本次重构把 15+ 张表主键从 `account_id` 改为 `cid_number`,不重建则新代码查 `cid_number` 列会打到旧表 → 全线 500。
- 工具已存在且正确:`citizenconsole/actions/cloudflare.sh` 的 `reset_d1()`(DROP 全部 25 表 + `d1_migrations` → 执行 0001 → 断言表数=25)与 `clear_kv()`(清空 SQUARE_CACHE)。
- 挂在**独立手动动作** `reset-formal-data`(`server.mjs`),不在 `production` 部署流程里。
- **动作**:发版前先跑 `reset-formal-data`;并把「本次发版需先重建 D1」写进部署流程或 deploy 脚本任务卡,避免下次再靠人记。
- 顺带解决:KV 身份缓存旧形状(`square_identity:` 的 value 新增 `cid_number`、读侧无形状校验,TTL 45s)、R2 陈旧对象。

## P1 — 需产品/架构决策

### 2. 换绑吊销链路时序死锁(止损函数永不执行)
`revokeRebindOldAccount` 设计正确(只删账户级鉴权材料、不碰随 cid 迁移的数据),但**触发时机错**:`myid_service` 在链上 `self_rebind_cid_account_id` **finalize 之后**才用**旧账户**建会话调用它,而此时:
- `auth/service.ts` 登录读链 → 旧账户已 `cid_not_bound` → 403;
- 设备子钥行的 `account_id` 已被重绑改成新账户 → `device_not_registered` → 401;
- `request_guard` 每请求链上绑定复查 → `cid_binding_changed` → 401。
三重拒,且客户端 `catch` 静默吞掉。**真实敞口有限**(guard 的实时复查本身已 fail-closed 挡住旧会话),但旧 `chat_keypackages` 行等残留不被清,且一个明确声明的安全控制从不执行。
- **候选修法**:(a) 把吊销移到提交换绑 extrinsic **之前**;(b) 改为**新账户**会话发起,body 带 `old_account_id`,服务端校验该账户曾绑同一 cid。
- 附带:`purge.ts` 的 `closeChatRealtime(env, cidNumber)` 在换绑场景会踢掉**新账户**刚建的实时连接(DO 按 cid 命名,新旧共享),换绑路径下应去掉或改按设备定向断开。

### 3. 链上订阅是账户键,换绑不跟随(「换绑不丢」在会员项不成立)
citizenchain `runtime/misc/square-post` 的 `Subscriptions` 键 = `(AccountId, Issuer)`,`RenewalSchedule`/`RenewalIndex`/`CreatorPlans` 同为账户键;而 `self_rebind_cid_account_id` 只调 `rebind_account_id()` 改 CitizenIdentity 双向绑定,**不动 SquarePost 任何 storage**。
- 后果:worker 镜像按 cid 存活(e2e 断言成立),但链上真源留在旧账户,`RenewalSchedule` 继续从**旧账户**扣款——而换绑的典型动机正是旧私钥泄漏。
- **候选修法**:链上加订阅迁移 extrinsic / 订阅改 cid 键 / 明确接受并文档化。需产品决策。

### 4. 通讯录孤儿密文累积
`contactId = HMAC-SHA256(peer_account_id, key=indexKey)`,`indexKey` 由账户派生 → 换绑后同一联系人的 contact_id 变 → 客户端重加密写**新行**,旧行既不被吊销删(R4 定案不删 contacts)也无「按 cid 清理不在列表内的行」接口。每次换绑该 cid 下行数翻倍,且旧行用**已泄漏的旧密钥**加密仍留在服务端。
- **候选修法**:contact_id 改用与账户无关的派生(如 peer cid + 身份级 indexKey);或加一个「按 cid 全量替换通讯录」的写接口。

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
- `cloudflare/src/account/service.ts` 的 `deleteAccountChallengeRoute` 未校验 `body.account_id === session.account_id`(与 `request_guard.ts:113-115` 自己写的不变量矛盾);真正销号仍需目标账户签名,故**无法真删他人**,但可对任意账户堆积 `square_login_challenges` 行(该表无定时清理)。
- `cloudflare/src/routes.ts:211` 的 `POST /v1/square/signals` 全仓零调用方 → `square_user_signals` 表永不写入,推荐流行为信号维度实际为空。
