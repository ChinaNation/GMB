# 官网公民币经济 + 产品体系文案与卡片调整

## 任务需求

### 公民币经济 tab（Tokenomics.tsx）
1. 合并「基本单位 元(Yuan)」「最小单位 分(Fen)」两张卡为一张「货币单位」卡，卡内含元和分两行。
2. 「交易费率」卡改名「链上交易费」，其右侧新增「链下交易费」卡。
3. 「手续费分配」板块区分链上/链下：链上交易手续费仍 8:1:1；链下交易手续费全部归清算行。

### 产品体系 tab（Ecosystem.tsx）
4. 字面替换：
   - `公民` → `CitizenApp 公民`
   - `公民链` → `CitizenChain 公民链`
   - `CitizenChain · runtime / node / onchina 三大版本` → `CitizenChain · runtime / 运行时协议、node / 节点程序、onchina / 链上中国`
   - `公民链软件由三大版本组成：` → `公民链软件由统一运行时协议、节点程序、链上中国平台组成：`
   - `onchina 机构统一控制台面向公权机构提供 CID 颁发、公民与机构档案和立法表决操作台。` → `onchina 链上中国为所有机构提供链上操作平台，注册局 CID 颁发、立法提案表决、公权机构选举、企业内部治理等。`

## 所属模块
- citizenweb：`src/pages/Tokenomics.tsx`、`src/pages/Ecosystem.tsx`

## 费率真源（citizenchain/runtime/primitives/src/fee_policy.rs）
- 链上交易费 0.1%，最低 0.1 元；分账 80/10/10。
- 链下清算行 L2：单笔最低 0.01 元，个体费率 0.01%–0.1%，全部归清算行。

## 核心边界
- 只做字面替换 + 卡片结构调整，不扩展未要求的文案（features/tech 不动）。
- 金额/费率以 fee_policy.rs 真源为准，不臆造。
- 只在 /Users/rhett/GMB 主检出操作。

## 验收标准
- `npm run build`、`npm run lint` 通过。
- 浏览器抽查两页渲染正确、无回归。
- 回写本卡。

## 执行记录
### 阶段 0：任务卡创建
- 已按当前任务授权创建本任务卡。

### 阶段 1：实现（2026-07-08，均在 /Users/rhett/GMB）
- Tokenomics.tsx：
  - 任务1：`economics` 引入 `Economic` 类型（value | units 二选一），合并「基本单位/最小单位」为「货币单位」卡（含元/分两行）。
  - 任务2：「交易费率」→「链上交易费」，其后新增「链下交易费」= `0.01%–0.1% (最低 0.01 元)`（费率取自 fee_policy.rs 真源）。
  - 任务3：手续费分配拆两组——链上交易手续费三卡 80/10/10（8:1:1），链下交易手续费一卡 100% 清算行；SectionTitle 描述同步。
- Ecosystem.tsx（任务4，纯字面替换，features/tech 不动）：
  - 公民→CitizenApp 公民；公民链→CitizenChain 公民链；
  - 副标题→`CitizenChain · runtime / 运行时协议、node / 节点程序、onchina / 链上中国`；
  - desc 开头→「公民链软件由统一运行时协议、节点程序、链上中国平台组成：」；
  - onchina 句→「onchina 链上中国为所有机构提供链上操作平台，注册局 CID 颁发、立法提案表决、公权机构选举、企业内部治理等。」

### 阶段 2：验证（2026-07-08）
- `npm run build`、`npm run lint` 通过。
- 浏览器（main 5205）DOM 逐项核对：
  - 货币单位合并卡（元/分）+ 链上/链下交易费相邻，截图确认。
  - 手续费分配两组标题 + 卡片 [80%全节点/10%手续费账户/10%安全基金] + [100%清算行]，描述已更新。
  - 产品体系 7 项检查全过（新文案在、旧「三大版本」「机构统一控制台面向公权机构」已消失）。
  - 控制台无 error。

### 结论
- 4 项全部完成并验证，无回归。
