# 任务卡:修复 PoW 创世期难度过低导致的分叉风暴 + 交易池 nonce 视图错乱

> 2026-07-12 状态校正：当前代码已删除 `MILLISECS_PER_BLOCK`、GenesisPallet 动态出块时间和
> CPU/GPU 六分钟提交等待，统一使用 `POW_TARGET_BLOCK_TIME_MS=360_000` 作为固定平均难度目标。
> 下文关于旧 30 秒占位和阶段时间切换的内容只保留为当时证据，不再代表当前实现。本任务仍因
> `POW_INITIAL_DIFFICULTY=100`、调整周期与多节点分叉真实收敛尚未处理而保持 open。

## 任务需求

全新创世后,本机桌面端转账"显示成功"但**区块高度不增、余额不变**,只能靠**重启本机节点**临时恢复。经排查为 PoW 难度参数失调引发的全网分叉风暴,进而导致接收交易那台节点的交易池(txpool)nonce 视图错乱。需根治(不靠重启)。

所属模块:citizenchain/runtime/otherpallet/pow-difficulty + primitives(Blockchain Agent)

## 现象(2026-06-08 实测)

- 账户"国1"前 9 笔转账成功上链(nonce 0→9、余额变动);第 10 笔起"成功"但不上链。
- 链停在 #9 数小时;本机 + 服务器 mempool 均为 0;`author_pendingExtrinsics`(ready)=0。
- dry-run(`system_dryRun`)返回 `0x0000`(有效),`author_submitExtrinsic` 也接受(返回 hash → App 显示成功)。
- 重启本机节点后,重发即出块(#10)、余额变动。

## 根因(已验证,统一一个)

**创世初始难度过低 → 全网分叉风暴 → 接收交易的节点交易池 nonce 视图错乱 → 新交易被分到 future 队列、永不就绪、永不被挖。**

证据链:
- `primitives/pow_const.rs`:`POW_INITIAL_DIFFICULTY=100`(过低)、`DIFFICULTY_ADJUSTMENT_INTERVAL=600`、`DIFFICULTY_MAX_ADJUST_FACTOR=4`、`MILLISECS_PER_BLOCK=30_000`(兜底);难度真实目标取 `genesis-pallet::target_block_time_ms()`,默认 **6 分钟/块**(`genesis/src/lib.rs:75`)。
- 难度 100 + 7 节点同时挖"同一笔交易"的块(空块禁止规则)→ 实测**每高度 6~7 个竞争块**(reorg 风暴)。
- `CurrentDifficulty` 存储**未写入**(第 9 块 < 第 600 块,从未调整)→ 全程难度 100;从 100 爬到 6 分钟目标难度需多次 ×4×600 块,风暴持续几千块,链在风暴里走不动。
- fork-aware 交易池每次 reorg 要回灌/重验交易,风暴下 nonce 视图跟不上规范最优链 → nonce=9 被当 future。future 交易**不向 peer 广播** → 卡在本机、传不到服务器(故服务器只是空闲待机、并未损坏;重启本机即恢复)。
- GRANDPA 未停,finalized 7→8,落后 best 2 块,同样是分叉风暴所致,风暴平息即追上。

## 必须遵守

- **走 runtime 升级(`system.setCode`),不重新创世**([feedback_chainspec_frozen]):这两个是 runtime 逻辑常量,setCode 即生效;`POW_INITIAL_DIFFICULTY` 是 `CurrentDifficulty` 的 ValueQuery 默认值,存储未写入时改默认即生效。
- 升级里**带 migration 显式 `CurrentDifficulty::put(新值)`**,避免依赖"存储恰好未写入"的脆弱前提(且兼容已过 600 块的情况)。
- 不破坏"创世期低难度快引导 → 运行期 6 分钟出块"的设计意图。
- 不动其他模块边界。

## 待 user 拍板的三个数

1. `POW_INITIAL_DIFFICULTY`:抬到多少(接近这 7 台真实算力下 6 分钟目标的量级,避免 7 路分叉)。
2. `DIFFICULTY_ADJUSTMENT_INTERVAL`:600 → 多少(建议 10~20,让难度几十块内爬到目标、快速结束风暴)。
3. `DIFFICULTY_MAX_ADJUST_FACTOR`:4 → 是否调大(爬得更快)。

## 落地方案(待数定后)

- 改 `primitives/pow_const.rs` 三个常量。
- pow-difficulty 加 migration:setCode 时 `CurrentDifficulty::put(POW_INITIAL_DIFFICULTY)` + 复位 `WindowStartMs`(让新难度+新窗口立即生效)。
- spec_version bump,补单测(难度调整在新 interval 下正确收敛)。
- 走 wasm CI → setCode 提案上链(联合投票升级路径)。
- 现实障碍:setCode 也是交易,需挤进当前卡顿的链,可能要配合一次重启把升级交易打包进去;一旦上链永久生效。

## 验收标准

- runtime 升级后:连续转账无需重启即稳定上链,块高、余额正常变动。
- 难度在数十块内自动爬升到 6 分钟目标附近,每高度竞争块数显著下降(不再 6~7 路分叉)。
- GRANDPA finalized 追上 best(滞后回落到正常 1~2 块内)。
- `cargo test`(pow-difficulty + runtime)通过;无需重新创世。
- 文档/记忆更新(难度调参 + 分叉风暴根因)。

## 2026-06-12 降级备注(ADR-017)

出块即固化 + 全端 finalized 单一口径落地后,分叉只在链尖秒级窗口出生即被 GRANDPA 裁决,且全端只读 finalized 不可观察其影响——本卡从"阻塞项"降级为**运行期前优化项**(矿工奖励公平性/防刷块/后期难度调整时再处理)。
