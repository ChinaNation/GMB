# ADR-037 公民币原生会员订阅：链上直付 + 原生订阅 pallet（接 ADR-036 / 取代 ADR-034 加密路线）

- 状态：**Proposed（草案，待用户确认；本轮不改链码）**
- 决议日期：2026-07-16（草案）
- 关联：ADR-036（会员身份解耦，三档 freedom/democracy/spark）、ADR-034（USDC 预付——本决策**取代其加密路线**）、ADR-033（生命周期规则 1–4 保留）、ADR-011（onchain-issuance Plain FT）、`primitives::fee_policy`（费率单源范式）、`primitives::account_derive`（机构派生账户）。

## 标题

把「加密货币预付」从 Stripe-Crypto(USDC) 迁到 GMB 自有链上代币【公民币＝原生 Balances】直付：用户 CitizenWallet 即账户，直接扣公民币开通/续订会员，绕过 Stripe 与一切外链。银行卡仍留 Stripe（卡收单无法自托管，且照顾非加密用户）。形成「**Stripe 管卡 + 公民币管原生**」双轨。

## 背景

- ADR-034 的 USDC 路线依赖 Stripe「Stablecoins and Crypto」能力；LIVE 账户 `acct_1Trr2fHSzSYWD2rF` 被判 `crypto_payments=inactive`（商户被描述为去中心化/区块链业务触发 Stripe 风控），**生产环境加密预付当前不可用**。
- ADR-034 已坐实根本难题：**加密钱包无法被 off-session 静默扣款**（链上转账须本人签发），故不能沿用卡的 subscription 自动续。
- 独特优势：**GMB 自有 runtime**，可做外链做不到的**原生订阅 pallet**（链上按期自动扣托管额度）。这是本决策相对任何外链方案的根本差异。
- 事实核对：公民币＝原生 GMB 代币（`core_const`：`TOKEN_SYMBOL="GMB"`、`TOKEN_DECIMALS=2` 元/分制、`ED=111` 分）；链上**暂无**会员/订阅 pallet；worker 已能链读（`chain/rpc.ts` 的 `state_getStorage`）并有签名交易广播兜底（`chain/extrinsic_relay.ts`，`RELAY_ENABLED`）。
- 纠错：现「订阅签名」`subscribe_membership → op_tag 0x1D(OP_SIGN_SQUARE_ACTION)` 是**链下 BFF 哈希域授权证明**，真金走 Stripe。公民币方案要**真金上链**，不能复用 0x1D。

## 决策

**1. 双轨模型（互不折算）**
- `stripe`：银行卡自动续订（mode=subscription），D1 为真源，非加密用户与官网首选。**不变**。
- `citizen_coin`：公民币链上直付，原生 pallet，主权闭环，无 Stripe。**新增**，取代 ADR-034 加密路线。
- **退役 USDC/Stripe-Crypto 路线**：生产已死（crypto_payments=inactive），开发期零用户，直接删 `prepaid.ts` 与 `subscription_source='usdc_prepaid'`，无迁移无兼容（`feedback_in_development_zero_users`）。

**2. 定价口径：直接公民币定价，不引预言机**
- 公民币是本闭环主权币，无外部浮动市价；「USD↔公民币 预言机」本质仍是治理设一个数，只多出预言机攻击面/陈旧风险/治理负担，**否**。
- 会员价对公民币档**直接以固定分币量表达**（每档每月固定分），单源 `primitives::membership_price`（镜像 `fee_policy` 范式），治理按标准路径调整。
- USD 价（299/999/9999 分）**仅留给卡档**（法币）。两轨各自独立计价、不做跨轨折算（延续 ADR-034「无跨路线折算」原则）。

**3. 资金去向：指定运营机构派生账户**
- 收入进**指定运营机构的主账户（OP_MAIN）或费用账户（OP_FEE）**，单源常量 `MEMBERSHIP_REVENUE_ACCOUNT`（记 `cid_number`，派生入口走 `account_derive`）。
- **不进两和基金（OP_HE）**——OP_HE 有专属对账用途，非平台运营收入。
- pallet 只向该账户**入账**，从不代其支出；该账户按机构多签常规治理花费。

**4. 签名层：热钱包 + Substrate 标准 extrinsic + 生物识别**
- 会员支付是小额高频消费（$2.99–$99.99/月等值），按签名分层规则（`feedback_signing_layer_selection_rule`：默认标准 extrinsic）走 **CitizenWallet 热钱包标准 extrinsic**，`local_auth` 每次动钱弹一次（`project_seed_biometric_binding_design`）。
- **不走 op_tag 0x1D**（那是链下 BFF 授权，不动真金）；**不走冷钱包两色签**（冷钱包留给机构/治理高风险 call，需三处登记，`project_citizenwallet_call_registration_three_points`）。
- CitizenWallet 交易预览须能解码显示新 pallet call（「订阅薪火会员，支付 X 公民币 + 链上手续费 Y」），两色识别显绿名（`project_qr_signing_two_color`）。

**5. 确认闭环：链读 pallet 存储，非事件订阅**
- pallet 把会员窗口写成自述链上态 `MembershipOf[account] = {level, expires_at}`。worker `state_getStorage` **按需读 + 短缓存**（懒判定，无常驻监听服务）。
- 流程：App 签 `prepay` extrinsic（热钱包+生物识别）→ 提交（直连 RPC 或 worker `extrinsic_relay` 兜底）→ 落块 → App 调 worker「同步会员」→ worker 链读 `MembershipOf` → 镜像 D1（`subscription_source='citizen_coin'`，按 `tx_hash` 幂等去重）→ 权益仍从 D1 出（限额/BFF 路径不变）。
- Workers 无状态，**不用持久 WS 订阅**。

**6. 续订机制：分期落地**
- **第 1 期（MVP，接 ADR-034 语义）＝预付多期**：`membership` pallet 加 `prepay(level, periods)` extrinsic，转 `periods × 月价` 公民币入运营账户 + 写/延 `MembershipOf` 窗口 + 抛事件。最小新链面，即刻解锁生产。到期由 App 读链提醒（机制 c 的轻量叠加）。
- **第 2 期（旗舰自动续订）＝托管计量 escrow-and-meter**：用户存押 M 公民币入 pallet 托管 + 签 `subscribe(level)`；调度钩子（`on_initialize` 到期扫）按期自动从托管扣一期、延窗口、转运营账户；托管有钱则自动续，取消＝停扣 + 提未计量余额。上限＝存押额，**有界安全**。这是外链做不到的能力，独立 ADR 分节 + 任务卡。
- **拒绝**常驻拉扣授权（standing pull）：pallet 持有对用户自由余额的常驻权限、余额不足即失效，风险过大。

**7. 退款/降档/取消/不可逆**
- 第 1 期预付：取消＝到期自然失效，**已付期不退**（钱已不可逆入链，＝无退单反欺诈特性，延续 ADR-034 降档不退现金）。升档＝补差价新 `prepay`；降档＝剩余价值折算成低档更多天数（本地/链上重算，不退现金，公式同 ADR-034）。
- 第 2 期托管：取消可提**未计量**托管余额（真链上退未消费押金）；已计量期终局。
- 会员窗口无需墓碑（非占号类唯一性资源），续订直接覆盖旧窗口。已消费公民币退款须走治理/人工链上回转，**无自动退**。

**8. 链上手续费**
- `prepay`/`subscribe` 是链上 call，按 `fee_policy` 单源收标准交易费（最低 0.1 元）**叠加**在会员价之上；UX 须明示「会员价 + 链上手续费」。连 `project_chain_fee_model_and_payment_diagnosis`（5 费种按 call 类型）。

## 边界

- **公民币轨天生 App-only**：官网（citizenweb）桌面无 CitizenWallet 私钥（种子在 App `flutter_secure_storage`），无法签链上 extrinsic。官网只出**卡档** + 「用 App 公民币订阅」QR/深链把手交给 CitizenApp；公民币签名/提交全在 App。
- 本决策**不含**公民币获取途径（转账/发行/交易）——是公民币轨可用的**前置依赖**，用户须先持有公民币；无公民币者留卡轨。列为风险，不在本卡实现。
- 第 2 期 escrow-and-meter 的调度扫单必须 O(有界)/块（`BoundedVec` 到期集或滚动游标），防区块权重 DoS——设计约束，第 2 期落地时定死。

## 影响

- **链（重大，须重新创世）**：新 `citizenchain/runtime/membership/` pallet（新 `pallet_index`）；`primitives::membership_price`（公民币价单源）+ `MEMBERSHIP_REVENUE_ACCOUNT`（运营账户单源）；接入 `fee_policy` 收手续费。重新创世，无 migration/兼容/spec_version（`feedback_chain_dev_never_ask_migration`）。
- **Worker（`citizenapp/cloudflare/membership`）**：删 `prepaid.ts`（Stripe-Crypto）；加 `citizen_coin.ts`（链读确认 + 镜像 D1）；`service.ts` `subscriptionIsActive` 加 `citizen_coin` 分支（仅看 `expires_at`，同预付语义）；`types.ts` source 联合改 `'stripe' | 'citizen_coin'`；D1 基线 `0001_square_core.sql` 直接重建（`usdc_prepaid`→`citizen_coin`、去重表键 `tx_hash`）；复用 `chain/rpc.ts`、`chain/extrinsic_relay.ts`。
- **App（`citizenapp/lib/my/membership/membership_page.dart`）**：加公民币轨（选档+期数 → 构 `prepay` extrinsic → 生物识别 → 热钱包签 → 提交 → 轮询 worker 同步 → 显有效）；删 USDC 入口；到期提醒读链。
- **官网（`citizenweb/src/pages/Membership.tsx`）**：删加密预付卡，卡档保留，加「用 App 公民币订阅」把手。
- **Stripe**：卡三价不变；加密能力不再依赖。

## 备选方案

- USD↔公民币 预言机定价：否（主权币无外部市价，徒增攻击面）。
- 收入进两和基金/国库：否（OP_HE 专用；运营收入进运营机构账户）。
- 纯普通转账收款（无专用 call）：否（转账无「事由」字段，worker 无法把某笔转账确定映射到某用户会员意图）。故用专用 `prepay` extrinsic，链上态自述、确定可核。
- 常驻拉扣授权（standing pull from free balance）：否（pallet 常驻权限过大 + 余额不足即失效）。取托管计量。
- 公民币轨自动续用 Stripe subscription：否（加密不能 off-session 扣，ADR-034 冒烟已证 400）。

## 后续动作

- 任务卡草案：`memory/08-tasks/open/20260716-citizen-coin-native-membership.md`（第 1 期 MVP / 第 2 期自动续订分卡）。
- **待用户确认后**方进入实现；确认前状态保持 Proposed，不改链码。
- 落地后同步：本 ADR 转 Accepted、`memory/05-modules` 会员模块文档、`memory/01-architecture/citizenchain` runtime 索引。
