# 账户级硬删除 + 官网签名订阅/取消订阅

## 收尾微调（2026-07-10）
- 官网地址框 placeholder `输入 公民 钱包地址`（公民两侧留空格）。
- **卡片高度统一（根因=响应式断点）**：原来只有 `lg`(≥1024) 才两列等高，<1024 堆叠→4 卡不等高（用户在窄视口看到）。改成**单一网格 `grid auto-rows-fr sm:grid-cols-2 lg:grid-cols-4`**（3 平价卡+动作卡为同级 cell，`auto-rows-fr` 令所有行等高）→ **每个宽度都 4 卡全等高**（preview 实测 1280=478 / 900=416 / 600=373，订阅/取消两 tab 均全等）。QR 签名移入弹层不撑卡。
- **订阅链上身份校验确认存在**：`checkout.ts assertCheckoutEligibility` 挑战与确认两处都实时读链（`CitizenIdentity.Voting/CandidateIdentityByAccount`，active 校验），投票会员需投票公民、竞选会员需竞选公民；`identitySatisfies` 用 ≥ 层级（竞选可买投票会员）。
- **徽章勾色规则更新**（[identity_badge.dart](../../citizenapp/lib/ui/identity_badge.dart)，用户定稿）：扇贝底色仍=身份档；**勾默认白**，仅当「降档买会员」（会员档<身份档，如竞选身份买投票会员）时勾染成会员档颜色（红扇贝+蓝勾），同档保持白勾——降档必异色一定清晰。6 例单测绿。

## 任务需求（用户定稿）

1. **官网会员订阅页**：右侧订阅面板"当前选择"上方加两段切换——**订阅会员**（默认=现卡片）/ **取消订阅**（切到取消卡）。
2. **订阅与取消订阅都必须钱包签名**（不再只凭地址）。官网无私钥环境 → 走扫码签名：官网出挑战 QR → 钱包扫码签名 → 回签名 QR → 官网扫回（复用已加的 QrScannerModal）→ 连签名发 Worker 验签 → 再走 Stripe（订阅=建 checkout；取消=退订）。签名走 ADR-026 「链下 challenge」层（op_tag）。
3. **CitizenApp 用户主页 ⋮ 菜单加「注销用户」**（仅本人 isSelf 可见）→ 二次确认弹窗（明确：无冷静期、签名完成即刻删全部数据）→ 确认后钱包签名 → Worker 验签 → **硬删除该用户在 Cloudflare 的全部数据**。公民 App 无登录登出：无删除签名即存在，有删除签名即删除。

## 删除原则（钉死）

- **硬删、零残留**：R2 + D1 + KV + Images/Stream + Stripe，owner=A 的数据一个不留（含换头像旧图、uploads 悬挂行、signals、media 等）。
- **只删 A 自己的**：B 的数据属于 B，绝不碰。A 发给 B、已进 B 信箱的密文=B 的（保留）；别人关注 A（owner=B）=B 的（保留）。只删「以 A 为 owner/recipient」的行。
- **删帖=彻底删内容**：deletePost 从软删（留行）改硬删。

### A 的清扫清单（owner=A / recipient=A）
- R2：`profile/{A}/**`、`square/{A}/posts/**`、`chat/{A}/**`（附件：跳过仍被 B 未 ack 信封引用的，避免误删 B 未收到的）。
- D1：square_memberships、square_uploads、square_posts、square_follows(owner=A)、square_user_signals(owner=A)、chat_devices、chat_keypackages、chat_envelopes(recipient=A)、square_media_assets(owner=A)、square_login_challenges。
- Images/Stream：先按 owner 取 provider_asset_id 调 provider DELETE，再删 D1 行（顺序不可反，防孤儿）。
- KV：A 的会话（token 键→需账户反查或删当前）。
- Stripe：以 stripe_subscription_id 退订（否则删了仍扣费）。

## 链上（用户已明确：不动）
竞选公民姓名/性别/出生地/钱包/CID、投票公民钱包/CID/居住地——本就必须链上公开不可篡改（参选公职要求），是设计目的，非隐患。链上一律不改、不涉删除。

## 建议模块 / 改动面
- Worker：新增签名验证的 membership challenge/sign 接口 + Stripe 取消 + 账户级 purge 接口（DELETE /v1/square/users/{account} 或 POST account/delete，验签授权）。复用 auth 现有挑战+验签。
- 官网：Membership.tsx 加切换 + 取消卡 + 扫码签名 QR 往返（复用 QrScannerModal）。
- CitizenApp：user_profile_page ⋮ 加注销入口(isSelf) + 确认弹窗 + 钱包签名 + 调 purge；membership 扫码签名 handler（订阅/取消）。

## 主要风险点
- Stripe 退订必须真调 Stripe API（现仅改本地 D1）；否则删号/取消后继续扣费。
- Images/Stream 删除顺序（先 provider 再 D1）。
- IM 附件边界：只删 A 的、保留 B 未 ack 引用的附件。
- 官网扫码签名的 payload/验签必须与钱包端签名协议（ADR-026 op_tag 链下challenge）逐字节一致。
- 硬删不可逆 + 无冷静期：二次确认文案必须明确。

## 验收
- App：注销→签名→Cloudflare 全清，无残留（各表/各前缀/Images/Stream/Stripe）；B 的数据完好。
- 官网：订阅/取消都需扫码签名验签通过才生效；未签不动 Stripe。
- Worker typecheck + test 绿。

---

## 阶段拆分

| 阶段 | 范围 | 状态 |
|---|---|---|
| 阶段 1 | Worker 后端：签名挑战/验签 + Stripe 退订 + 账户级硬删 purge + 删帖改硬删 | ✅ 完成（typecheck + 80 test 绿） |
| 阶段 2 | CitizenApp：⋮ 注销入口(isSelf) + 二次确认 + 钱包签名 + 调 delete；取消订阅底座 | ✅ 完成（analyze 净 + 6 例绿；取消订阅按用户决定留官网） |
| 阶段 3 | 官网订阅/取消 + CitizenApp「扫一扫」签名往返 | ✅ 完成（3A✅ · 3B✅ · 3C✅） |

> **全任务完成（2026-07-09）**：签名统一 op_tag + 阶段1(注销后端) + 阶段2(App注销) + 阶段3(官网订阅/取消扫码签名闭环)。链上零改动。

### 阶段 3 进度（2026-07-09）
- **3A Worker ✅（typecheck净 + 93例绿）**：`SignedAction+='subscribe_membership'`（payload 绑 level，`buildActionScalePayload` 加可选 context 字段）；`checkout.ts` 退役无签名 `stripeCheckoutRoute`，新增 `subscribeChallengeRoute`+`subscribeConfirmRoute`（验签+level一致才建 Stripe checkout）；challenge 响应加 `owner_pubkey_hex`（`shared/ids.ownerPubkeyHex`=decodeAddress）；删死码 `assertSessionOwner`；routes 换 `/membership/subscribe{,/challenge}`；`membership_checkout.test.ts` 重写为签名流。
- **3B 核心 ✅（golden 绿）**：`QrActions.squareAccountAction=9` + `sign.rs QR_ACTION_SQUARE_ACCOUNT=9`（单源，off-chain）；`qr_signer.signingBytesForHex` 加 `squareAccountAction→signingMessage(0x1D)`；`qr_signer_test` 加 0x1D 金标。
- **3B-UI ✅（analyze 净 + 18例绿）**：交易 tab「扫码支付→扫一扫」（[transaction_tab_page](../../citizenapp/lib/transaction/transaction_tab_page.dart)）+ 新 [scan_dispatch_flow.dart](../../citizenapp/lib/qr/scan_dispatch_flow.dart)（QrRouter 分类：支付→`proceedOffchainPayment`／signRequest→签名响应方／未来）；[qr_scan_page](../../citizenapp/lib/qr/pages/qr_scan_page.dart) 加 `QrScanMode.dispatch`；签名响应方 [square_action_sign_service.dart](../../citizenapp/lib/signer/square_action_sign_service.dart)（`parseRequest`→按 `u` 解析 owner 钱包/拒非本机/拒冷钱包→两色解码→`signWithWallet` 生物识别→`buildResponse`；**复用 CitizenApp 现成 qr_signer，未移植 CitizenWallet**）；两色解码器 [square_action_payload.dart](../../citizenapp/lib/signer/square_action_payload.dart)（逐字节对齐 Worker，未知动作/截断→null 禁盲签）；signResponse 二维码页 [qr_sign_response_page.dart](../../citizenapp/lib/qr/pages/qr_sign_response_page.dart)；`QrActions.squareAccountAction=9`+`sign.rs QR_ACTION_SQUARE_ACCOUNT=9`+`signingBytesForHex→0x1D`+金标；删死码 `openOffchainScanPaymentFlow`（扫一扫替代）；QR action 单源 `qr-action-registry.md` 已登记 9。**签名钱包=QR u 对应钱包(owner)，≠付款钱包**。
- **3C ✅（tsc -b + eslint + vite build 全绿）**：[Membership.tsx](../../citizenweb/src/pages/Membership.tsx) 订阅/取消分段切换 + 取消卡；加 `qrcode.react` 生成 signRequest 二维码（`QRCodeSVG`，SVG 内联、CSP 安全）；[QrScannerModal](../../citizenweb/src/components/QrScannerModal.tsx) 加 title/hint 复用于扫回 signResponse；新 [qrV1.ts](../../citizenweb/src/lib/qrV1.ts)（`base64url↔bytes`/`hex` + `QR_V1` build/parse，逐字节对齐 app 信封）；订阅 `/membership/subscribe{,/challenge}`→checkout_url 转 Stripe、取消 `/membership/cancel{,/challenge}`→提示成功；**未签一律不动 Stripe**（先出挑战→扫码签→confirm 才走 Stripe）。官网无测试框架，验证=tsc/lint/build。
- **端到端闭环打通**：官网(桌面)出 signRequest QR → CitizenApp 交易 tab「扫一扫」按 u 定位 owner 钱包、两色核对、主钥签 0x1D → 出 signResponse QR → 官网扫回取 64B 签名 → Worker consumeActionSignature 验签 → Stripe。三仓签名逐字节对齐（金标锁 chain↔worker↔app；官网信封对齐 app）。

### 阶段 3 定稿（含核查结论 + 扫一扫决策）
- **核查**：CitizenApp **无**「扫 signRequest→签→出 signResponse」能力（[qr_scan_page](../../citizenapp/lib/qr/pages/qr_scan_page.dart) 仅 transfer/contact/raw；[qr_sign_session_page](../../citizenapp/lib/qr/pages/qr_sign_session_page.dart) 是请求生成方）；签名响应方只在 CitizenWallet [offline_sign_service](../../citizenwallet/lib/signer/offline_sign_service.dart)（用 signWithWallet 主钥+生物识别，拒冷钱包，两色核对）。
- **决策（用户定案）**：手机签名方 = CitizenApp 交易 tab「扫码支付」升级为**「扫一扫」分发器**（按 QrRouter 分类：链下支付→现有支付流程；signRequest→签名响应方；未来类型再加）。**签名钱包 = QR `u` 对应钱包(owner)，≠ 付款钱包**；「请先选付款钱包」检查后移到支付分支。
- **3A Worker**：`SignedAction+='subscribe_membership'`（payload 绑 membership_level）；新 `/membership/subscribe{,/challenge}`，退役无签名 `/stripe/checkout`；challenge 响应加 `owner_pubkey_hex`(decodeAddress)。取消 challenge/confirm 复用阶段1。
- **3B CitizenApp**：交易 tab 扫码支付→扫一扫分发器 + 签名响应分支（移植 offline_sign，签名钱包按 u 解析）；`QrActions.squareAccountAction=9`+sign.rs QR_ACTION_SQUARE_ACCOUNT=9；`signingBytesForHex`→signingMessage(0x1D)；PayloadDecoder 加广场动作两色解码。
- **3C 官网**：Membership.tsx 订阅/取消切换 + 取消卡；加 `qrcode` 生成 signRequest QR；QrScannerModal 扫回 signResponse；TS base64url/hex + QR_V1 build/parse（对齐 app）；未签不动 Stripe。
- **信封契约**：signRequest `{p:"QR_V1",k:1,i,e,b:{a:9,g:1,u:32Bpubkey,d:SCALE payload}}`；signResponse `{p:"QR_V1",k:2,i,b:{u,s:64B sig}}`；被签=signingMessage(0x1D,decode(d))。

### 阶段 2 落地记录（2026-07-09，CitizenApp 客户端，链端零改）
- **签名走统一 op_tag 0x1D**（阶段签名统一后）。客户端**钉死 op_tag**，不采信服务端下发。
- **SquareApiClient**（[square_api_client.dart](../../citizenapp/lib/8964/services/square_api_client.dart)）：`SquareActionSigner` typedef + `deleteAccount`/`cancelMembership`/`_consumeAccountAction`(challenge→钉死0x1D重算摘要签→confirm) + `clearSession`。
- **编排服务**（新 [square_account_deletion_service.dart](../../citizenapp/lib/8964/services/square_account_deletion_service.dart)）：先服务端硬删(失败即抛、绝不清本地)→成功后清本地(资料缓存/会话/IM私信历史/原生设备子钥)。
- **IM 本地清**：[im_isar_store.dart](../../citizenapp/lib/im/storage/im_isar_store.dart) 加 `clearAllForOwner`(清会话/消息/出入站队列;路由缓存非owner归属不清)。
- **UI**：[profile_kebab_menu.dart](../../citizenapp/lib/8964/profile/widgets/profile_kebab_menu.dart) 加 `deleteAccount`(仅 isSelf,红色末位) + `_MenuRow` 加 color；[user_profile_page.dart](../../citizenapp/lib/8964/profile/user_profile_page.dart) `_openDeleteAccount`(resolve walletIndex→二次确认弹窗[文案定稿]→signWithWallet主钥签生物识别→服务→snack+popUntil root;冷钱包/无热钱包拦截;失败/取消不清本地)。
- **测试**：deleteAccount op_tag钉死0x1D+round-trip+缺字段异常；服务成功清齐/失败不动；kebab isSelf 双向门禁。6 例绿，`dart analyze lib test` 仅 2 处 pre-existing info。
- **本地清扫遗留（小follow-up，非阻断）**：IM 附件缓存目录 + IM 设备绑定 prefs 缓存键(`_deviceBindingCacheKey`,私有于 im_runtime)未清——均为可再生本地缓存,服务端 A 的 IM 数据已由 Worker purge 删除;私信明文历史(Isar)已清。

### 待办：签名协议归位（用户提出，阻塞前必须定）
- 用户质疑 `GMB_SQUARE_ACTION_V1`。事实：Worker 的广场 BFF 会话鉴权本来就是一套**独立于链上 signing_message/op_tag 的 raw-string 家族**——已存在 `GMB_SQUARE_LOGIN_V1`(登录) + `GMB_SQUARE_DEVICE_BIND_V1`(设备绑定)，`verifyWalletSignature` 直接 `signatureVerify(原文)`。我加的 `GMB_SQUARE_ACTION_V1`(注销/取消) 与这俩同族，非凭空造。
- 但 `lib/signer/signing.dart` 的单源纪律写死「禁止写 `GMB_*_V1` 字符串域，一律走 `signingMessage(op_tag)=blake2_256(GMB‖op_tag‖scale)`」(canonical=citizenchain primitives::sign)。官网无私钥→需 App 扫码签名回传，走的是 `CITIZEN_QR_V1` 冷签信封；冷签路径签的是 signing_message/op_tag，raw 字符串未必能搭现成 QR 通道。
- **workflow 综合（signing-landscape-map，四面已核实）**：全仓签名共三层——① `signing_message(op_tag)=blake2_256(GMB‖op_tag‖SCALE)`（citizenchain primitives::sign 单源，op_tag 0x10-0x1A 占，**空闲 0x11/0x12/0x1B-0x1F**）② `QR_V1` 冷签**信封**（传输壳，非被签内容）③ Square-BFF 原始字符串族 `GMB_SQUARE_LOGIN_V1`/`DEVICE_BIND_V1`/`ACTION_V1`（Worker 验，永不上链）。
- **关键先例**：op_tag 表里**已有一个"Worker 验签、永不上链"的 op_tag = IM 钱包绑定 0x1A**（cloudflare/src/chat/binding.ts 里就是 `blake2_256(GMB‖0x1A‖SCALE)`，进了 golden vectors）。→ "广场后端动作用统一 op_tag"有现成范式，非语义污染。
- **官网硬约束**：citizenweb 现为 address-only（Membership.tsx 只 POST 明文，jsqr 解码+base58 正则取地址，无签名库）。官网无私钥→必须 App 扫码签；App QR 冷签只认 op_tag（0x10/0x1A）与链交易，**不该签 raw 字符串**（signing.dart:15-16 禁手拼 GMB_*_V1）。→ 只有 op_tag 路线能让官网扫码往返成立。
- **用户已拍板 B + 加码：统一全部走 op_tag，删除所有 GMB_*_V1 字符串域，规则钉死**（见 [[feedback_unified_signing_optag_only]]）。

### 签名统一 op_tag 方案（✅ 已完成 2026-07-09）
三条 BFF 字符串域全删，各配一个 HASH 域 op_tag（紧挨 IM 绑定 0x1A，走 `signing_message=blake2_256(GMB‖op_tag‖SCALE)`）：
| op_tag | 名称 | SCALE payload | 签名密钥（不变） |
|---|---|---|---|
| `0x1B` | OP_SIGN_SQUARE_LOGIN | owner, challenge_id, expires_at | P-256 设备子钥 ES256（静默握手，seed 生物绑定定案，不改） |
| `0x1C` | OP_SIGN_SQUARE_DEVICE_BIND | owner, p256_pubkey, issued_at | sr25519 主钥 |
| `0x1D` | OP_SIGN_SQUARE_ACTION | action, owner, challenge_id, expires_at | sr25519 主钥 |

payload 编码照 0x1A：SCALE_string(文本) ++ u64_le(时间戳)。挑战/绑定流由 worker 下发 payload bytes(hex)，客户端只 `signing_message(op_tag, payloadBytes)` 再签，**杜绝跨语言 SCALE 漂移**（worker 单侧编码）。device-bind 双侧编码 → 严格照 0x1A 的 TS/Dart 编码器对齐。

**定性(钉死)**：这 3 个 op_tag 只被**链下**验签(Worker + App)，链上 runtime pallet 不引用 → **不创世、不 setCode、不重启节点、无 migration，运行链不动**。改的只是 citizenchain `primitives::sign` 源常量(签名单源)+ 重生金标 JSON(纯 dev 测试夹具)，让 Worker/App 镜像逐字节对齐。

改动面：① citizenchain(纯源+dev 测试，不部署)：sign.rs +3 常量、`SIGN_OP_TAGS`→`[u8;10]`、fixture +3 向量、`SIGN_GOLDEN_UPDATE=1 cargo test -p primitives --test sign_golden` 回填 message_hex；**不碰 runtime pallet、不 build-spec、不出 deb、不重启**。② worker：抽 `signing_message` 共享 helper(现仅 chat/binding.ts 手写)、登录/绑定/动作三处验签改 op_tag digest、挑战下发存 op_tag+payload(hex)。③ app：signing.dart +3 op_tag 常量、三 signer 改 signingMessage、同步金标。验证：rust golden 绿 + worker typecheck/test 绿 + app analyze 绿；运行链零动作。
- **统一的是"被签消息"，非密钥算法**：登录仍 ES256 设备子钥(静默)，强改主钥会让开广场必弹生物识别(UX 倒退)，故保留密钥分层。

#### 落地记录（三仓改动，运行链零动作）
- **citizenchain（源+dev 金标，未部署）**：`sign.rs` +`OP_SIGN_SQUARE_LOGIN/DEVICE_BIND/ACTION`(0x1B/1C/1D)、`SIGN_OP_TAGS`→`[u8;10]`；`signing_domain_vectors.json` +3 向量、`SIGN_GOLDEN_UPDATE=1` 回填；确认 `SIGN_OP_TAGS` 仅金标测试消费、无 pallet 引用 → 不动 runtime。`cargo test --test sign_golden` 绿。
- **worker**：新建 `src/shared/signing_message.ts`(signingMessage+SCALE 编码,从 chat/binding.ts 抽出共用)；`wallet_signature.ts`/`device_subkey.ts`/`account/action_challenge.ts`/`auth/service.ts` 验签全改 op_tag digest；挑战下发 `signing_payload_hex`+`op_tag`；**删** buildLoginPayload/buildDeviceBindingPayload/buildActionPayload 三字符串域。新增 `test/signing_message.test.ts` 金标(worker↔链 10 tag)。`typecheck` 净 + `npm test` 92 例绿。
- **app**：`signer/signing.dart` +3 op_tag 常量 + `scaleString`/`u64Le`(im_binding_payload 去重复用)；`device_subkey.dart` buildDeviceBindingPayload→`buildDeviceBindingSigningMessage`(digest)；`SquareLoginSigner`/`DeviceBindingSigner`/`ImSquareLoginPayloadSigner` 改收 `Uint8List`，op_tag+hex 解码集中在 `_establishSession`/registrar(客户端钉死 op_tag,不采信服务端)；signer 闭包只签字节；**删** `signUtf8WithWallet`/`WalletSignResult` 死码。金标 fixture 同步 + 新增 `device_binding_golden_test.dart`(0x1C 字段编码 App↔Worker 同 hex `e9e25d…`)。`analyze` 净(2 处 pre-existing info) + signer26/im+publish11/wallet+api24 例绿。
- **跨语言字节锁**：signing_message 原语 链↔worker↔app 三处金标(10 tag)全绿；device-bind 0x1C 双侧编码同 hex 锁死。登录/动作流由 worker 单侧编码下发 hex，客户端只 hash+sign，零漂移。

### 注销二次确认弹窗文案（用户定稿·精简）
> 注销将立即硬删除你在公民广场/私信的全部数据，无冷静期、不可恢复，链上数据不受注销影响。

## 阶段 1 落地记录（2026-07-09）

### 签名动作基座（防跨动作重放）
- `cloudflare/src/account/action_challenge.ts`（新）
  - `SignedAction = 'delete_account' | 'cancel_membership'`。
  - `buildActionPayload` = `GMB_SQUARE_ACTION_V1\naction:<x>\nowner_account:<ss58>\nchallenge_id:<id>\nexpires_at:<ms>`；**action 行入 payload** → A 动作的签名不能用于 B 动作。
  - `issueActionChallenge` 复用 `square_login_challenges` 表落挑战；`consumeActionSignature` 校验：存在 / owner 一致 / 未用 / 未过期 / **payload 内 action 行匹配** → 再 `verifyWalletSignature`（sr25519）→ 标记 used_at。

### Stripe 真退订（此前只有 webhook，无主动退订）
- `cloudflare/src/membership/stripe_api.ts`（新）
  - `cancelStripeSubscriptionNow` = `DELETE /v1/subscriptions/{id}`（注销用，当场终止）。
  - `cancelStripeSubscriptionAtPeriodEnd` = `POST cancel_at_period_end=true`（官网取消用，当期用完再终止）。
  - `STRIPE_DEV_CHECKOUT_PROXY==='1'` dev 短路；缺 `STRIPE_SECRET_KEY` 抛 503；非 2xx 抛 502。

### 账户级硬删编排
- `cloudflare/src/account/purge.ts`（新）`purgeAccount(env, owner)` 顺序：
  1. `getMembership` 取 `stripe_subscription_id`；
  2. `cancelStripeSubscriptionNow`——**失败即抛、整单中止**（绝不「删了库还在扣费」）；
  3. `SELECT square_media_assets(provider,provider_asset_id) WHERE owner=A` → 逐个 `deleteProviderAsset`（**先 provider 再 D1**，防孤儿）；
  4. R2 前缀清扫 `profile/{A}/`、`square/{A}/posts/`、`chat/{A}/`（`chat` 下跳过仍被 B 未 ack 信封引用的附件目录=B 的数据）；
  5. D1 批删（只删 owner=A / recipient=A）：memberships/uploads/posts/user_signals/media_assets/follows(owner=A)/chat_devices/chat_keypackages/chat_envelopes(recipient=A)/device_subkeys/login_challenges；
  6. KV 删 `square_identity:{A}` + `clearOwnerSessions`（定向失效该账户全部会话）。
- `cloudflare/src/auth/session_index.ts`（新）：登录时 `indexSessionToken` 维护 `square_sessions_by_owner:{A}` token 列表；注销 `clearOwnerSessions` 删全部 `square_session:{token}` + 索引。`auth/service.ts` 登录成功处已接入。

### 路由 / 服务
- `cloudflare/src/account/service.ts`（新）4 handler：delete challenge/confirm、cancel challenge/confirm。
- `cloudflare/src/routes.ts`（改）注册：`POST /v1/square/membership/cancel/challenge`、`/membership/cancel`、`/account/delete/challenge`、`/account/delete`。

### 删帖改硬删（原则 6）
- `cloudflare/src/posts/confirm.ts`（改）`deletePostCloudflareData`：`UPDATE post_state='deleted'` → `DELETE FROM square_posts`；并加 `DELETE FROM square_uploads`（清悬挂上传行）。链上仅存 content_hash 不受影响。

### 测试
- `cloudflare/test/account.test.ts`（新）：consumeActionSignature 6 例 + purgeAccount 2 例。
  - purge 测试**不 mock stripe_api**，改用 env 驱动真函数（dev 短路=退订成功；缺密钥=退订抛 503 中止），既避开 vitest spy 抛错串扰，又直测真实退订中止路径。
- `cloudflare/test/chain_confirm.test.ts`（改）删帖用例改断言硬删（行移除 + 再删 404），FakeDb 处理 `DELETE FROM square_posts/square_uploads`。
- 结果：`npm run typecheck` 干净；`npm test` 16 文件 80 例全绿。

## 官网会员页 UI 收尾（2026-07-10）

### 卡片等高（用户三次报「切换 tab 卡片会变」）
- 根因：`citizenweb/src/pages/Membership.tsx` 操作卡（第 4 张）的可变内容按 tab 不同——订阅=「当前选择/档位/价格」3 行，取消=说明 2 行。用 `grid auto-rows-fr` 全行等高后，在**操作卡为当前行最高卡**的宽度区间，切 tab 会把高度差传导到全部 4 张卡（1024/1280 恰好档卡最高、掩盖了问题；1440 附近操作卡最高、暴露）。行内错误 `message` 也会撑高卡片。
- 定案：**让操作卡内容与 tab 无关**——
  1. 可变块包一层 `min-h-[104px]`，订阅/取消两态渲染同高；
  2. 提示 `message` 从卡内移出为 `fixed inset-x-0 bottom-6` 浮层 toast，永不占卡片高度。
- 验证（preview_resize 逐宽实测，订阅 vs 取消操作卡高度）：1024=513/513、1280=494/494、1440=494/494，四卡全等且切 tab 零变化；tsc -b / eslint / vite build 全绿。

### 其它两处（同批）
- 地址输入 placeholder 改 `输入 公民 钱包地址`（「公民」前后带空格，替换原 CitizenApp）。
- 徽章勾色规则（`citizenapp/lib/ui/identity_badge.dart`）：默认白勾；**高档身份买低档会员**（会员档 < 身份档）时勾染成所买会员档颜色（竞选身份+投票会员=红扇贝+蓝勾）；同档保持白勾。含单测 `test/ui/identity_badge_test.dart`。
