# 任务卡：未注册身份引导补齐(通讯录 + 会员/订阅)

状态：进行中(2026-08-06)

## 任务需求(用户逐字)

1. 未注册用户点通讯录**还能进去**——不允许;要和其它页面一样显示注册引导。
2. 未注册用户进「会员｜订阅」,顶部显示「会员数据加载失败……重试」——
   **删掉**。"都没有注册用户加载个屁啊！重试个毛啊！"三张平台会员卡正常显示,
   卡片订阅按钮改为 **「注册用户」**,点击弹**同一个**注册弹窗。
   没注册就该引导注册,而不是加个重试。

## 定性

未注册是**合法状态,不是错误**。两处都在「把权限态当故障报」:

- 通讯录:`_load()` → `getContacts()` → `_requireIdentityOwner()` 对未注册**必抛**
  `WalletAuthException('请先注册 CID 身份')`([contact_service.dart:912]);页面 catch
  后 `_contacts` 保持空 → 渲染落到 `_contacts.isEmpty` → 显示 **假的「空通讯录」**。
- 会员页:`_loadFailure` 由 `catch (error)` 兜底置为「会员数据加载失败,请点右上刷新
  重试」([membership_page.dart:208]);未注册用户 `ensureSession()` 抛的 401 正好落进
  这个 catch。**该缺陷由 2026-08-06 上一轮「未注册统一引导」改动引入**,属自造问题。
  卡片按钮的 `unavailableLabel` 同病:未注册(有钱包)会显示「会员状态同步中」。

## 实施范围

### 一、通讯录页(`lib/my/user/contact_book_page.dart`)

- `_load()` 开头判身份缓存,未注册 → 置 `_unregistered` 并 **直接 return**
  (不调 `getContacts()`,避免注定抛异常的调用;与 `chat_tab._reload` 短路同款)。
- 渲染层最前分支:`_unregistered` → `IdentityRegisterGuide`
  (description「注册后即可使用通讯录。」,onRegistered 回刷)。
- 链读失败**不算未注册**,沿用现有错误路径 fail-closed。

### 二、会员页(`lib/my/membership/membership_page.dart`)

1. 区分未注册 vs 真故障:照搬广场已验证模式 —— 本地身份缓存命中未注册直接判定;
   缓存未命中由 Worker 真源 `errorCode == 'cid_not_bound'` 判定。不新造轮子。
2. 未注册时**不置 `_loadFailure`**,顶部无任何失败/重试横幅。
3. 三张卡照常完整显示**且带真实价格**:`fetchAllPlatformPrices` 是**纯链上读、
   不需要会话**,故把价格拉取**移到会话之前**(现排在会话之后,会话一挂价格就没了)。
4. 卡片按钮文案 →「注册用户」,点击弹 `startCidRegistrationFlow`(全 App 同一弹窗);
   占号成功回刷本页即进正常订阅流程。
5. **保留**:已注册用户遇真故障时失败横幅 + 重试照旧;无热钱包提示不动。

### 三、漏网页面普查结论(不重复加门)

| 页面 | 判定 | 处置 |
|---|---|---|
| 通讯录 | 首屏必须有 CID | 本次加引导 |
| 会员/订阅 | 首屏部分依赖 | 本次特殊处理 |
| 广场 / 聊天 / 创作者 | 已有引导 | 不动 |
| 个人资料页、资料编辑页、帖子/文章详情 | 首屏依赖会话,但入口全在已拦页面内(广场 feed / 聊天 / 通讯录) | 通讯录一拦即不可达,不重复加门(避免层层弹窗) |
| 聊天搜索、群管理、机构详情、公权页 | 浏览开放,动作才要身份 | 不动(符合既有设计) |

## 验收

- [x] 通讯录:未注册 → 引导 + `getContactsCalls == 0`(短路铁证);已注册 → 正常列表
      且计数 > 0;链读失败 → 走原故障路径,不冒充未注册
- [x] 会员页:未注册 → **无失败横幅**、三卡在、价格在、按钮「注册用户」、
      **不发登录挑战**(`sessionProvider.calls == 0`)
- [x] 会员页:已注册 + 真故障 → 失败横幅 + 重试仍在(既有用例守住)
- [x] 测试:通讯录 12/12、会员 24/24;受影响全目录 **306/306 全绿**
- [x] analyze 零问题
- [x] 文档:`USER_TECHNICAL.md §8`、`PROFILE_TECHNICAL.md`「会员页未注册呈现」
- [ ] 真机验收(装机已完成,待用户在手机上确认)

## 实施记录(与原计划的偏差)

1. **价格前置一度写成无条件** —— 绕过了 `_pricesCacheTtl`(30 分钟),已注册用户每次
   进页都打一次链;被既有三个缓存用例(「有效缓存直接展示且不重复读取」等)当场逮住。
   已改为**只在未注册分支** `_enterUnregistered()` 内补取,正常路径仍走 TTL 缓存。
2. **通讯录既有测试的身份 fake 是 `_NullIdentityCache`**(resolve 返回 null),在新逻辑下
   等价于「未注册」,会让全部既有用例测成引导态。已换成 `_RegisteredIdentityCache`,
   另加 `_UnregisteredIdentityCache` 供新用例使用。
3. 按钮改造走 `registerInsteadOfSubscribe` 参数贯穿 `_buildStackedCard` →
   `_MembershipTierCard`:未注册时文案恒为「注册用户」且**按钮必可点**,
   不受 `subscriptionReady` / `priceFen` 就绪限制(否则又变成点不动的死按钮)。
