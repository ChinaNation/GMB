# 任务卡（草案·待确认）：公民币按月自动扣会员 + 双边订阅市场 + 链上税务

> 状态：**草案，待用户拍板政策点后定稿**。确认前不改链码/产品码。依据 ADR-037 + ADR-038。

任务需求：会员订阅直接用公民币**按月自动扣**（钱包账户即唯一身份）。双边订阅市场：① 平台会员（进技术公司账户）② 创作者会员（自设价/权益，扣所得税后归创作者）。银行卡仍走 Stripe，**清除原虚拟货币订阅，Stripe 只留卡**。

所属模块：citizenchain（新 subscription pallet + tax-registry pallet + primitives 单源 + legislation/internal-vote 回调 + onchina 入口）、citizenapp/cloudflare（会员后端 + 门禁）、citizenapp（会员页 + 创作者 UI + 专属内容门禁）、citizenweb（官网）。跨产品、含重大链改。

输入文档：**memory/01-architecture/gmb/membership-tax.md（工程架构:分层/目录/命名/闭合，落地照此）**、memory/04-decisions/ADR-037、ADR-038、ADR-036/034/030/028/027/025；memory/07-ai/*；记忆 feedback_signing_layer_selection_rule、project_qr_signing_two_color、project_seed_biometric_binding_design、project_chain_fee_model_and_payment_diagnosis、feedback_chain_dev_never_ask_migration、feedback_in_development_zero_users。

必须遵守：不破模块边界/契约（fee_policy 单源、account_derive 派生、ADR-025 机构码单源、签名分层、onchain_gate scope）；链开发期重新创世无 migration/兼容/spec_version/残留；签名分层（公民订阅热钱包标准 extrinsic + 生物识别；onchina 机构写冷签 internal-vote；禁 0x1D 套公民币轨、禁冷钱包盲签）；所有可调数值进链上 storage 单源（禁 primitives 常量做可调值）；不清楚先沟通。

---

## 第 1 期：平台会员·公民币按月自动扣（+ Stripe 收敛只留卡）

- **链**：`subscription` pallet（新 index）：`Subscription:(subscriber,IssuerKey)`、`PlatformPrice:StorageMap<Level,分>`（默认 199900/599900/5999900）、`DueQueue` 时间桶到期索引（O(有界)桶扫，禁遍历全表）、`subscribe`/`cancel`、`on_initialize` 原子扣款（`with_storage_layer`，净额<ED/失败→欠费即停，不烧毁不部分提交）；`set_platform_price`（技术公司 internal-vote 写，能力位绑技术公司唯一 CID）；自动扣走 `fee_policy::Free`。`primitives::membership_price` 只留枚举/单位/硬护栏。`Config::TaxQuery` 关联类型（第 3 期前返回零税）。
- **onchina**：`domains/membership/chain_call.rs` 构造 `set_platform_price`（仿 AddressRegistry 范式）；`operation_auth` 加 `AdminActionType::SetPlatformPrice`（PasskeyColdSign）；`capability` 加 `can_set_platform_price`（唯一技术公司 CID）；workspace action；CitizenWallet 三处登记；`unified-protocols.md` 登记。
- **Worker**：删 USDC/Stripe-Crypto 残桩（见 ADR-037 影响清单）；加 `citizen_coin.ts`（链读 `Subscription` 确认 + 镜像 D1 `citizen_coin`，`tx_hash` 幂等）；`subscriptionIsActive` 加分支；`types.ts`；D1 基线重建；卡轨收敛为三事件。双价源启动/CI 断言档位一致。
- **App/官网**：`membership_page.dart` 平台会员公民币轨（选档→`subscribe`→生物识别→签→提交→同步→显有效/下次扣款/欠费）；删 USDC 入口；官网删加密卡 + 加把手。
- 中文注释、测试（扣款/欠费即停/桶扫有界 + worker + App）、文档、残留清理。

前置/阻塞：技术公司须先建为私权法人 + 补管理员 + `PLATFORM_MEMBERSHIP_ACCOUNT` 地址补入（公司后期注册），补入前平台轨挂起（明确态）。

## 第 2 期：创作者会员·双边订阅市场（未接税=0 税全额）

- **链**：`CreatorPlans:StorageMap<Creator(cid),BoundedVec<Tier>>` + `set_creator_plans`（caller 须闭合 CID 纳税主体，个人本人签/机构 internal-vote）；`subscribe(Creator(cid))` 复用扣款循环。
- **门禁**：专属内容标 `required_subscription`；广场/聊天 BFF `requireCreatorSubscription` 链读订阅（fail-closed）；打通 ADR-028/ADR-020。
- **App**：创作者"开启我的会员/设档定价"、订阅他人、专属帖/群/频道发布与解锁。
- 依赖广场/聊天权限系统就绪；不阻塞于税务（税率未设=0 税全额）。

## 第 3 期：链上税务·申报期结算机制（ADR-038，串行最后）

> 关键：税**不逐笔预扣**，订阅款全额到收款方；税走申报期结算。税率/征收方式/申报期/落账**由税务机构运行期设定，不在架构、不在本期定值**——本期只建机制容器。

- **前置串行**：a. 税务机构（财税部等）补管理员；b. 《税法》经 legislation-yuan 立法→回调写 `TaxAuthorization`（机器可读：授某税务机构对某范围征税权 + 上限区间 + 有效期；或 ADR-039）；c. 税务 pallet：`IncomeLedger:(taxpayer_cid, tax_period)->收入`（收款时记账）、`TaxAuthorization`、税率/征收规则可配置 storage（税务机构 internal-vote 写）、`RuntimeTaxSubjectQuery`（钉稳定 CID 反查，不跑地址 union；CID 定管辖）、申报期结算征收执行（结算时点复核授权区间/有效期 + caller_cid∩scope，能力位绑税务机构唯一 CID）、`InternalVoteResultCallback` 扩 executor、`primitives::tax_policy` 只放硬顶；d. `subscription` 收款成功记 `IncomeLedger`（不扣钱）。
- **onchina**：`domains/fiscal/` 税务机构设规则入口（`internal-vote` + 冷签 + 能力位绑唯一税务机构 CID）+ 申报/结算 workspace + 三处登记。
- fail 语义：未启用/无征税机构/无适用税率=不征（等税务机构设）；主体有 CID=纳管辖台账；无 CID=自行申报不阻塞收款。

验收标准（各期）：全链跑通、扣款/记账/结算原子且权重有界、残桩零残留、重新创世一次成功；征税权结算时点复核（未授权不征、越界/过期不征）、纳税主体钉 CID 不可伪造、能力位绑唯一 CID。

---

## 架构层仅剩的少量拍板点（非税率/征收细节——那些税务机构后期设）
1. 平台价是否设 `primitives` 硬上下限护栏（技术公司"自由调节"的边界）。
2. 财政账户支出通道是否本期建（补管理员 + propose_transfer），还是另立卡。

影响范围：跨 4 产品、含两个新 pallet + 立法/治理集成 + 重新创世。链改须用户显式确认后动手。
