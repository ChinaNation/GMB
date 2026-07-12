# 广场/聊天增加「链上钱包」门禁:签名钱包必须是链上活账户(free ≥ ED 111 分)

任务需求：公民 App 用钱包签名换 Cloudflare 会话才能用广场+聊天;新增校验——签名钱包必须是链上钱包
(链上 `System.Account` 存在,即 free ≥ ExistentialDeposit 111 分),否则拒发会话,彻底拦住无链上钱包的用户。
所属模块：citizenapp/cloudflare（auth 会话签发 + chain 读余额）、citizenapp（客户端错误文案）

## 已核实前提（全部成立）

- 广场与聊天共用同一套 `ensureSession`(钱包签名→Cloudflare session token),无会话用不了。
- 111 分 = 链上 ExistentialDeposit 真源 `citizenchain/runtime/primitives/src/core_const.rs:22`,
  余额 < 111 分账户被 reap 销毁 → "链上钱包"= `System.Account` 活账户 = free ≥ 111 分。
- `Nonce=u32`、`AccountData=pallet_balances::AccountData<u128>`(free/reserved/frozen/flags),
  `free` 在 `AccountInfo` 字节偏移 **16** 起 u128 LE。
- Worker 已具备读链:`chain/rpc.ts` `fetchChainStorage`(state_getStorage)、`chain/identity.ts` 已读链上护照。

## 用户拍板的边界

1. ED=111 **永不变** → Worker 侧命名常量硬编码 111 + 注释指真源,不走 env。
2. 登录态 24h(`SESSION_TTL_SECONDS`),**仅签发时校验一次**,过期重校验;期间余额掉到 111 以下不管
   (发帖/发文章会另发交易,交易自带校验兜底)。
3. **完全拦截会话**:非链上钱包连 session 都拿不到 → 广场读/写、聊天全用不了。就要这效果。
4. 链 RPC 读不到余额(宕机/超时)→ **fail-closed 拒发**(沿用 rpc.ts 现有 502/504 抛错向上传即拒发)。

## 落点(单点卡在会话签发,不并入 limits)

三层门禁模型:Layer0 链上钱包存在(本卡新增·签发时)→ Layer1 护照身份档(已有)→ Layer2 会员配额 limits(已有)。
新 gate 在 limits **上游**,`limits/` 一行不动(职责=资源量额度,与身份准入不同;生命周期逐请求 vs 一次性签发)。

## 输出物

- `cloudflare/src/chain/storage_key.ts`（新）：抽取 `decodeOwnerAccount` + `storageMapKey`(+`concat`),
  identity 与 wallet 共用(DRY,消 identity.ts 本地副本,无残留)。
- `cloudflare/src/chain/identity.ts`（改）：改 import 上述共用件,删本地副本(纯机械)。
- `cloudflare/src/chain/wallet.ts`（新）：`ACCOUNT_EXISTENTIAL_DEPOSIT_FEN = 111n`（注释指 core_const.rs）;
  `fetchAccountFreeBalance(env,owner)` 读 `System.Account`、解 free(偏移16 u128 LE);
  `assertOnchainWallet(env,owner)` free 为空或 < ED → 抛 `HttpError(403,'not_onchain_wallet',...)`。
- `cloudflare/src/auth/service.ts`（改）：`createSession` 验签通过后、铸 token 前 `await assertOnchainWallet(...)`。
- 客户端（改）：`square_api_client.dart` / 广场·聊天错误面把 `not_onchain_wallet` 映射为明确文案
  (如"需链上钱包·余额≥1.11元才能使用广场和聊天,请先充值"),**不走只读兜底**(与上次 ANR 修复的
  session-fail→只读区分开)。
- 测试：`cloudflare/test/auth.test.ts` 补——链上钱包放行 / 低于111拒 / 账户不存在拒 / RPC失败拒(fail-closed);
  stub `fetch` 返回 state_getStorage 的 AccountInfo hex。

## 验收标准

- Worker：新老 vitest 全绿(尤其 auth 4 新例 + identity/profiles 不回归);`tsc --noEmit` 无错。
- 客户端：`flutter analyze` 无新增;非链上钱包广场/聊天被拦并显示明确文案,不降级只读。
- 语义：链上钱包(free≥111)正常放行;无链上钱包彻底拿不到会话。

## 验收结果（2026-07-12 已通过）

- Worker `npx tsc --noEmit`：无错。`npx vitest run` 全量 **122 passed / 20 files**;
  auth.test.ts 5 例（原成功例补链上余额≥ED stub + 3 新例:低于 ED 拒 / 账户不存在拒 / RPC 故障 fail-closed 拒）;
  identity 抽取后 profiles 13 例无回归。
- 客户端 `flutter analyze`（square_home_page + square_session_provider）：No issues;
  `square_feed_service_test` 通过。
- 语义闭环:门禁在 `createSession` 验签后铸 token 前,单点管住广场+聊天;`not_onchain_wallet` 会被
  `ensureSession` rethrow(不触发懒注册);客户端广场透传"需链上钱包·余额≥1.11元"文案。
- 状态：DONE（代码在主检出未提交）。

## 未做/边界外（有意）

- 聊天侧 `not_onchain_wallet` 精确文案未穿 MLS 复杂初始化(块已服务端强拦,属 UX 打磨,不扩本卡)。
- 端到端真机验证需先部署 Worker 到 crcfrcn.com(生产,需显式授权),本卡以 Worker 单测 + 客户端 analyze 为准。
- 另发现聊天 `chat_runtime.dart` 仍有同款懒注册 Turnstile 隐患(违背 feedback_square_session_never_lazy_register),
  已登记独立后台任务 task_992d5e05,不混入本卡。

## 无遗留

开发期零用户(见 feedback_in_development_zero_users),不涉存量/迁移;干净新增。
