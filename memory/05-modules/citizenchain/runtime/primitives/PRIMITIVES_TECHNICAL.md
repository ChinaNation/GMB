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
| `src/core_const.rs` | 货币基础参数、手续费模型、利率模型、安全参数 |
| `src/pow_const.rs` | PoW 难度、全节点发行、区块时间参数 |
| `src/citizen_const.rs` | 公民轻节点认证发行常量 |
| `src/count_const.rs` | 投票治理常量（机构数量、投票阈值、期限） |
| `src/genesis.rs` | 创世宣言、创世人口、创世发行总量 |
| `china/mod.rs` | 机构常量模块声明 |
| `china/china_ch.rs` | 43 个省储行（人口、质押、多签地址） |
| `china/china_cb.rs` | 44 个储委会（1 国储会 + 43 省储会） |
| `china/china_jc.rs` | 48 个联邦监察院 |
| `china/china_jy.rs` | 公民教育委员会 |
| `china/china_lf.rs` | 45 个立法院 |
| `china/china_sf.rs` | 45 个审计院 |
| `china/china_zb.rs` | 277 个制度保留地址（防抢注） |
| `china/china_zf.rs` | 55 个行政保留地址 |

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
| SHENGBANK_INITIAL_INTEREST_BP | 100 | 首年 1.00% |
| SHENGBANK_INTEREST_DECREASE_BP | 1 | 逐年递减 0.01% |
| SHENGBANK_INTEREST_DURATION_YEARS | 100 | 100 年后归零 |
| 利息总量（计算值） | 72,896,617,589 元 | 白皮书 §5.2.1 一致 |
| BLOCKS_PER_YEAR | 87,600 | 白皮书定义每 87,600 块结算一次 |

### 2.4 全节点铸块发行
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| FULLNODE_BLOCK_REWARD | 999,900 分 | 每块 9,999.00 元 |
| FULLNODE_REWARD_END_BLOCK | 9,999,999 | 发行 9,999,999 块 |
| FULLNODE_TOTAL_ISSUANCE（计算值） | 9,998,999,000,100 分 | 99,989,990,001 元 |

### 2.5 公民轻节点发行
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| CITIZEN_LIGHTNODE_HIGH_REWARD_COUNT | 14,436,417 | 高额阶段节点数 |
| CITIZEN_LIGHTNODE_HIGH_REWARD | 999,900 分 | 前期 9,999 元/节点 |
| CITIZEN_LIGHTNODE_NORMAL_REWARD | 99,900 分 | 后期 999 元/节点 |

### 2.6 交易手续费
| 常量 | 值 | 白皮书对应 |
|------|-----|-----------|
| ONCHAIN_FEE_RATE | 0.1% | 链上交易费率 |
| ONCHAIN_MIN_FEE | 10 分 | 最低 0.1 元 |
| 全节点分成 | 80% | — |
| 国储会分成 | 10% | — |
| 安全���金 | 10% | NRC_ANQUAN_ADDRESS |

### 2.7 投票治理
| 常量 | 值 | 说明 |
|------|-----|------|
| NRC_ADMIN_COUNT | 19 | 国储会管理员 |
| PRC_ADMIN_COUNT / PRB_ADMIN_COUNT | 9 | 省储会/省储行管理员 |
| JOINT_VOTE_TOTAL | 105 | 19 + 43 + 43 |
| JOINT_VOTE_PASS_THRESHOLD | 105 | 全票通过立即执行 |
| VOTING_DURATION_BLOCKS | 7,200 | 投票到期区块数（240 × 30 天） |
| ACCOUNT_EXISTENTIAL_DEPOSIT | 111 分 | 账户最低余额 |

---

## 3. china/ 目录规则
- 每个机构文件必须在 `china/mod.rs` 中显式声明，避免目录中存在未编译的残留文件。
- 多签管理员字段统一命名 `duoqian_admins`，不允许 `admins` 变体。
- `china_zb.rs` 中的 277 个保留地址由 `duoqian-manage-pow` 模块在转账时校验，防止抢注机构地址。

---

## 4. 区块计数口径说明
| 常量 | 值 | 来源 | 用途 |
|------|-----|------|------|
| SECONDS_PER_BLOCK | 360 | 运行期 6 分钟出块 | 派生基础 |
| BLOCKS_PER_HOUR | 10 | 3,600 / 360 | 链下交易打包阈值、密钥轮转延迟 |
| BLOCKS_PER_DAY | 240 | 10 × 24 | 投票到期计算 |
| BLOCKS_PER_YEAR | 87,600 | 白皮书固定值 | 省储行利息结算周期 |
| VOTING_DURATION_BLOCKS | 7,200 | 240 × 30 | 投票到期门槛（30 天） |
| MILLISECS_PER_BLOCK | 30,000 | 创世期占位 | 仅用于 benchmark/test 和 node 层 fallback |

注意：SECONDS_PER_BLOCK / BLOCKS_PER_HOUR / BLOCKS_PER_DAY 按运行期 6 分钟出块计算。MILLISECS_PER_BLOCK 保留为创世期 30 秒占位值，仅 benchmark/test 和难度调整窗口常量引用。

---

## 5. 测试覆盖
执行命令：
- `cargo test -p primitives`

当前覆盖：
- `citizens_sum_matches_genesis_total` — 43 省人口汇总 = GENESIS_CITIZEN_MAX
- `stake_sum_matches_population_basis` — 43 省质押 = 人口 × 10,000
- `fee_percents_sum_to_100` — 链上手续费分成比例总和 = 100
- `joint_vote_total_matches_threshold` — 联合投票总票数 = 通过阈值
- `fullnode_total_issuance_is_consistent` — 全节点发行总量 = 区块奖励 × 区块数
- `genesis_issuance_matches_population` — 创世发行 = 人口 × 10,000
- `all_china_ch_duoqian_addresses_are_unique` — 43 省 duoqian_address 全部唯一
