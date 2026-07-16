# Primitives Technical Notes

## 0. 模块定位
`primitives` 是 CitizenChain 全链统一常量模块，被 runtime 内 24 个 pallet、node 层和工具脚本依赖。
本模块只包含常量定义、数据结构和辅助函数，不含状态机逻辑、存储操作或 extrinsic。

代码位置：
- `/Users/rhett/GMB/citizenchain/runtime/primitives/`

---

## 1. 文件结构

| 文件 | 职责 |
|------|------|
| `src/lib.rs` | 模块声明入口 |
| `src/core_const.rs` | 货币基础参数、利率模型、安全参数、统一签名/派生域 |
| `src/code.rs` | 国家码、省级行政区码、CID 机构码及其谓词的全仓唯一常量真源 |
| `src/fee_policy.rs` | **费率规则常量单一权威源**：链上/链下/投票统一价 + 三方分账比例 |
| `src/pow_const.rs` | PoW 难度、全节点发行、区块时间参数 |
| `src/citizen_const.rs` | 公民认证发行常量 |
| `src/count_const.rs` | 投票治理常量（机构数量、投票阈值、期限、不可修改条款清单） |
| `src/constitution.rs` | 修宪「章→档位」分类判定单源（第十九条 `classify`/`AmendmentScope`，runtime 与节点守卫共用） |
| `src/governance_skeleton.rs` | 创世治理保护清单：精确枚举 89 个机构及其机构码、CID、主账户、管理员人数、岗位代码/名称和席位；完整身份查询禁止只按机构码扩大保护范围。实际 storage 写入唯一在 genesis seeder，Node Guard 使用共享管理员/entity 类型解码 |
| `src/institution_constraints.rs` | 国家级单例与成员组成永久约束：精确枚举 PRS/NLG/NSN/NRP/NSP/NED 六个创世身份，固定 NSN `SENATOR` 105–155、NRP `REPRESENTATIVE` 305–355、NED `COMMITTEE_MEMBER` 105–155，并声明 NLG 由 NSN、NRP 组成；不冻结 NLG/NSP/PRS 的岗位或 admins，六个单例的内部投票规则由投票引擎按提案类型处理 |
| `src/genesis.rs` | 创世宣言、创世人口、创世发行总量 |
| `china/mod.rs` | 机构常量模块声明 |
| `china/china_ch.rs` | 43 个省储行（人口、质押、多签账户） |
| `china/china_cb.rs` | 44 个储委会（1 国家储委会 + 43 省储委会） |
| `china/china_jc.rs` | 47 个监察机构（国家监察院 + 3 联邦署 + 43 省级联邦监察院） |
| `china/china_jy.rs` | 公民教育委员会 |
| `china/china_lf.rs` | 46 个立法机构（国家立法院 + 国家参议会 + 国家众议会 + 43 省级联邦立法院） |
| `china/china_sf.rs` | 44 个司法院（国家司法院 + 43 省级联邦司法院） |
| `china/china_zb.rs` | 637 个制度保留地址（防抢注） |
| `china/china_zf.rs` | 71 个政府机构（总统府 + 联邦局 + 部委 + 宪法国家级机构 + 43 省级联邦政府） |

---

## 2. 经济常量白皮书对照表

### 2.1 货币基础
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| TOKEN_SYMBOL | "GMB" | 公民币 |
| TOKEN_DECIMALS | 2 | 元/分制，1 GMB = 100 FEN |
| SS58_FORMAT | 2027 | 地址前缀 |

### 2.2 创世发行
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| GENESIS_CITIZEN_MAX | 1,443,497,378 | 第 7 次人口普查总人口 |
| GENESIS_ISSUANCE | 14,434,973,780,000 分 | 每人 100 元 = 144,349,737,800 元 |

### 2.3 省储行质押利息
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| PROVINCIALBANK_INITIAL_INTEREST_BP | 100 | 首年 1.00% |
| PROVINCIALBANK_INTEREST_DECREASE_BP | 1 | 逐年递减 0.01% |
| PROVINCIALBANK_INTEREST_DURATION_YEARS | 100 | 100 年后归零 |
| 利息总量（计算值） | 72,896,617,589 元 | 白皮书 §5.2.1 一致 |
| BLOCKS_PER_YEAR | 87,600 | 白皮书定义每 87,600 块结算一次 |

### 2.4 全节点铸块发行
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| FULLNODE_BLOCK_REWARD | 999,900 分 | 每块 9,999.00 元 |
| FULLNODE_REWARD_END_BLOCK | 9,999,999 | 发行 9,999,999 块 |
| FULLNODE_TOTAL_ISSUANCE（计算值） | 9,998,999,000,100 分 | 99,989,990,001 元 |

### 2.5 公民发行
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| CITIZEN_ISSUANCE_HIGH_REWARD_COUNT | 14,436,417 | 高额阶段节点数 |
| CITIZEN_ISSUANCE_HIGH_REWARD | 999,900 分 | 前期 9,999 元/节点 |
| CITIZEN_ISSUANCE_NORMAL_REWARD | 99,900 分 | 后期 999 元/节点 |

### 2.6 交易手续费(2026-05-03 后单一权威源 = `primitives::fee_policy`)
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| ONCHAIN_FEE_RATE | 0.1% | 链上交易费率 |
| ONCHAIN_MIN_FEE | 10 分 | 最低 0.1 元 |
| TRANSACTION_TIP | 0 | tip 不属于交易费，任何非零 tip 拒绝 |
| VOTE_FLAT_FEE | 100 分 | 实际投票统一价 1 元/票 |
| OFFCHAIN_FEE_RATE_MIN/MAX | 0.01% / 0.1% | 清算行个体费率上下限 |
| OFFCHAIN_MIN_FEE | 1 分 | 链下最低 0.01 元 |
| OPERATIONAL_FEE_MULTIPLIER | 1 | 运营类不额外加价 |
| ONCHAIN_FEE_FULLNODE_PERCENT | 80% | 铸块全节点分成 |
| ONCHAIN_FEE_NRC_PERCENT | 10% | 国家储委会分成 |
| ONCHAIN_FEE_SAFETY_FUND_PERCENT | 10% | 安全基金 SAFETY_FUND_ACCOUNT |

**全链唯一五类费用路由**由 `fee_policy::FeeRoute<AccountId, Balance>` 定义，
`runtime/src/configs.rs::RuntimeFeeRouter` 负责把 `RuntimeCall` 穷尽映射到该类型：

- `Free`：框架固有、Root 回调或内部维护调用；外层不扣费。
- `Onchain { transaction_amount, payer }`：`max(round(amount × 0.1%), 0.1 元)`。机构提案和机构操作的 `payer` 必须是 actor CID 的唯一费用账户；普通用户和 Fullnode 操作由签名者支付。
- `Offchain { fee_amount, payer: OffchainFeePayer::BatchItemPayers }`：链下清算批次可有多个付款公民；每个 item 的付款公民从其 L2 存款支付对应费用，外层不得重复扣款，也不得把收款方机构费用账户误当付款人。
- `Vote { payer }`：只允许实际 `cast_*` / 表决动作，固定 1 元，由投票签名者支付；发起提案不是投票。
- `Reject`：未分类、未开放、管理员授权失败、CID/账户不匹配或费用账户解析失败的调用一律拒绝。

收费分支的 `payer` 是必填字段，不存在缺省付款人或回落到签名者。`WeightToFee`、`LengthToFee` 均为零，不产生五类之外的框架费用。

投票回调中的资金执行没有第二笔外层 extrinsic，由
`fee_policy::OnchainFeeCharger` 接收业务已经核验的确切付款账户和交易金额。
该 trait 不是第二套分类或付款路由；生产实现
`onchain::OnchainExecutionFeeCharger` 仍只调用本文件的
`calculate_onchain_fee`，完整扣款、保留 ED、进入同一 80/10/10 分账并发出
`FeePaid`。机构资金执行必须显式传入该 CID 的费用账户；个人账户执行传入
个人账户本身，扣款失败即整笔执行回滚。

### 2.7 投票治理
| 常量 | 值 | 说明 |
|------|-----|------|
| NRC_ADMIN_COUNT | 19 | 国家储委会管理员 |
| PRC_ADMIN_COUNT / PRB_ADMIN_COUNT | 9 | 省储委会/省储行管理员 |
| NJD_ADMIN_COUNT | 15 | 国家司法院创世公职人员：7 名护宪大法官 + 1 名首席大法官 + 2 名次席大法官 + 5 名大法官 |
| JOINT_VOTE_TOTAL | 105 | 19 + 43 + 43 |
| JOINT_VOTE_PASS_THRESHOLD | 105 | 全票通过立即执行 |
| VOTING_DURATION_BLOCKS | 7,200 | 投票到期区块数（240 × 30 天） |
| ACCOUNT_EXISTENTIAL_DEPOSIT | 111 分 | 账户最低余额 |

投票能力与业务权限必须分层：内部投票程序面向所有有效机构；转账、销毁、密钥变更等具体业务由各业务 pallet 按机构权限独立限制。`primitives` 只保存确需跨模块复用的制度常量，不建立把机构和全部业务绑定在一起的全局能力表。

---

## 3. china/ 目录规则
- 每个机构文件必须在 `china/mod.rs` 中显式声明，避免目录中存在未编译的残留文件。
- 多签管理员字段统一命名 `admins`，不允许 `admins` 变体。
- 内置机构名称统一使用 `cid_full_name / cid_short_name / cid_full_name_en / cid_short_name_en` 四字段。
- `builtin_institution_name_digest()` 覆盖全部内置机构名称四字段；修改任一名称字段都必须通过 runtime 升级生效。
- 具体机构命名规范见 `memory/07-ai/institution-naming.md`。
- `china_zb.rs` 中的 637 个保留地址由 `RuntimeReservedAccountGuard` 注入 `public-manage`、`private-manage`、`personal-manage`，在注册和创建账户时统一校验，防止抢注制度地址。

---

## 4. cid/code.rs 代码常量规则

- `cid/code.rs` 是国家码、省级行政区码和 CID 机构码的全仓唯一真源。
- 国家码使用 `CountryCode = [u8; 2]`,当前 `COUNTRY_CN = CN`,并携带 `country_full_name / country_short_name`。
- 省级行政区码使用 `ProvinceCode = [u8; 2]`,当前 43 省来自 `citizenchain/onchina/src/cid/china/china.sqlite` 的抽离结果;OnChina 加载 `china.sqlite` 时必须断言 SQLite 省表与 `PROVINCE_CODE_INFOS` 一致。
- 市、镇和镇下完整地址代码不进入 runtime primitives;它们继续由 OnChina `china.sqlite` 按省管理。
- 机构码使用 `InstitutionCode = [u8; 4]`,3 字符码右补 `0`;全部 104 个机构码在 `INSTITUTION_CODE_INFOS` 中维护,分组只用 A-I 注释表达,不得增加第二套 group/kind/admin_level 数据字段。
- 2026-07-04 国家级 A 组从 26 码补齐为 38 码,新增 `FDA/NGB/ARM/NAV/AIR/SPF/JOS/ARC/NVC/AFC/SFC/NGC`。
- 机构码对应中文标签统一使用 `institution_code_label`;真实机构实体中文名仍只使用 `cid_full_name / cid_short_name`,不得恢复旧标签字段或第二份标签表。
- OnChina 只能通过 `crate::cid::code` / `primitives::cid::code` 引用机构码,不得恢复旧 number 码表或第二份机构码数组。

---

## 5. 区块计数口径说明
| 常量 | 值 | 来源 | 用途 |
|------|-----|------|------|
| POW_TARGET_BLOCK_TIME_MS | 360,000 | 创世默认平均 6 分钟 | PoW 难度目标窗口，运行期以 `pow-difficulty::ActiveParams` 为真源 |
| SECONDS_PER_BLOCK | 360 | 目标平均 6 分钟 | 制度期限换算基础 |
| BLOCKS_PER_HOUR | 10 | 3,600 / 360 | 链下交易打包阈值、密钥轮转延迟 |
| BLOCKS_PER_DAY | 240 | 10 × 24 | 投票到期计算 |
| BLOCKS_PER_YEAR | 87,600 | 白皮书固定值 | 省储行利息结算周期 |
| VOTING_DURATION_BLOCKS | 7,200 | 240 × 30 | 投票到期门槛（30 天） |

注意：六分钟是难度调整的长期平均目标，不是最短间隔或最晚期限。有效 PoW 找到后立即出块。
`POW_TARGET_BLOCK_TIME_MS / DIFFICULTY_ADJUSTMENT_INTERVAL / DIFFICULTY_MAX_ADJUST_FACTOR /
DIFFICULTY_MIN_ADJUST_FACTOR` 是创世默认和节点守卫基线，运行期可随 runtime 升级原子写入
`PowDifficultyParams`；`CurrentDifficulty` 不允许直接治理修改，只能由算法推进。
`SECONDS_PER_BLOCK / BLOCKS_PER_HOUR / BLOCKS_PER_DAY` 是制度日历常量，不随 PoW 参数调整而变化。

---

## 6. 测试覆盖
执行命令：
- `cargo test -p primitives`

当前覆盖：
- `citizens_sum_matches_genesis_total` — 43 省人口汇总 = GENESIS_CITIZEN_MAX
- `stake_sum_matches_population_basis` — 43 省质押 = 人口 × 10,000
- `joint_vote_total_matches_threshold` — 联合投票总票数 = 通过阈值
- `fullnode_total_issuance_is_consistent` — 全节点发行总量 = 区块奖励 × 区块数
- `genesis_issuance_matches_population` — 创世发行 = 人口 × 10,000
- `all_china_ch_main_accounts_are_unique` — 43 省 main_account 全部唯一
- **`fee_policy::onchain_fee_percents_sum_to_100`** — 链上手续费分成比例总和 = 100
- **`fee_policy::vote_flat_fee_equals_one_yuan`** — 投票统一价 = 1 元 = 100 FEN
- **`fee_policy::offchain_rate_bounds_consistent`** — 链下费率上下限合法
- **`fee_policy::min_fees_positive`** — 最低费用 > 0,防零费用攻击
- **`fee_policy::onchain_rate_positive`** — 链上费率 > 0,防零费率绕过
- **`code::tests::*`** — 国家/省/机构码格式、唯一性、分类谓词和 CID 号机构码解析一致性
