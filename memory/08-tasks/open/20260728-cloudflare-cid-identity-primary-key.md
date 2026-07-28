# Cloudflare Worker 身份主键 account_id → cid_number 彻底重构

状态:open(2026-07-28,用户拍板;需求分析完成,待技术方案确认后开工)
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

## 重构骨架(子步,每步先出聚焦方案待确认)
- **R1 身份地基/登录**:session/identity=cid_number。登录:钱包 account_id 签挑战 → 验签(wallet_signature)+ 链查 CidByAccount 得 cid_number(须当前确绑,否则访客)→ 发 cid_number 会话;设备子钥注册记 (cid_number, 当前 account_id),验签时校该子钥 account_id == 链上 cid 当前绑定账户(换绑后旧子钥自然被拒)。
- **R2 D1 schema 重建**:所有用户数据表主键 account_id→cid_number(直接改 0001 migration,零迁移);account_id 仅作"当前绑定/验签"属性,不作身份键。
- **R3 链上发布路径**:posts/confirm/relay 保持"钱包 account_id 签 extrinsic + 链确认"不变,只把 D1 镜像的归属键 account_id→cid_number。
- **R4 off-chain 数据路径**:contacts/follows/notify/browse/uploads/media/profile 等 by cid_number,设备子钥鉴权。
- **R5 会员/订阅镜像**:membership/creator/subscription 链上镜像按 cid_number。
- **R6 换绑变数据无操作**:删 rebind/revoke + purge 的"删旧账户数据"逻辑(数据按 cid 存不动;旧 account_id+子钥失效由验签拒);真正注销(删身份)按 cid_number 另做。
- **R7 chat**:chat_devices/keypackages 记 (cid_number, account_id),子钥属 account_id、数据/路由按 cid_number。
- **R8 测试**:worker vitest 全套改 cid_number。

## 设计原则定案(用户已答)
- 设备子钥**属钱包账户**(账户生成);换绑→旧账户+旧子钥作废,新账户重注册子钥;worker 靠链上 CidByAccount 实时校当前绑定,不搞子钥迁移。
- 发布=链上交易走钱包 account_id 签名付费;删除/通讯录等 off-chain 走设备子钥。

## 门禁
worker `vitest` 全绿 + `tsc` 0;换绑后同一 cid_number 社交数据不丢(端到端);account_id 全称命名零缩写。
