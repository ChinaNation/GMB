# 官网五项更新（公民币经济/订阅/治理/产品体系/PQC）

## 任务需求

1. **公民币经济页**：删除「公民币代币经济模型」板块（页面 Hero），并把其描述句「基于《公民宪法》发行的法定数字货币，通过多渠道发行机制确保公平分配。」移到「发行分配方案」板块，替换现描述「公民币固定发行合计为 2,229,386,218,778 GMB，不含后续治理决议新增发行。」
2. **会员订阅页**：展示文案 `VotingIdentityByAccount` → `认证投票公民`、`CandidateIdentityByAccount` → `认证选举公民`；钱包账户地址输入框右侧新增区块链常用扫码图标（取景框中间一条横线），点击弹出摄像头识别用户钱包地址二维码并回填输入框。
3. **治理体系页**：「四种投票体系」4 卡改 2×2 布局——上排左内部投票/右联合投票，下排左立法投票/右选举投票，全部长方形卡片。
4. **产品体系改版**：导航「生态系统」tab 移到「公民币经济」与「会员订阅」之间并改名「产品体系」；「四大核心系统」改为三大——上排左「公民」（结合 citizenapp 现有功能更新内容）、上排右「公民钱包」（内容基本保持，卡名「CitizenWallet 冷钱包」改为「CitizenWallet 公民钱包」）、下排整行长方形「公民链」（介绍 runtime / node / onchina 三大软件交付物）。
5. **PQC 介绍**：「端到端密码学安全」板块（实际位于 Ecosystem.tsx，非 Technology.tsx）新增一张 PQC 卡，内容取材 ADR-022（ML-DSA-65 / ML-KEM-768，不换助记词/地址在位升级），口径必须是「已定稿升级路线」而非「已上线」。

## 建议模块

- 全部在 `citizenweb/`（React 19 + Vite + Tailwind v4），无链端改动：
  - `citizenweb/src/pages/Tokenomics.tsx`（需求 1）
  - `citizenweb/src/pages/Membership.tsx` + 新增扫码组件（需求 2）
  - `citizenweb/src/pages/Governance.tsx`（需求 3）
  - `citizenweb/src/pages/Ecosystem.tsx`、`citizenweb/src/components/Header.tsx`、`Footer.tsx`、`citizenweb/src/pages/Home.tsx`（需求 4）
  - `citizenweb/src/pages/Ecosystem.tsx` 安全板块（需求 5）

## 影响范围

- 需求 1：Tokenomics.tsx 删 69–83 行（Hero + 金色分隔线），第 105 行 description 换句。全页无 id 锚点，无路由级断裂。固定发行总额数字仍保留在核心指标（第 55 行 2.23万亿）与 Home.tsx。
- 需求 2：Membership.tsx 第 29/37 行是纯展示文案（与链上 storage 同名但不参与查询，checkout 只发 owner_account/membership_level），改名安全；链端/Worker/Dart 端零改动。扫码需新增依赖（jsQR）+ getUserMedia + 弹层组件，站点无任何现存扫码能力。
- 需求 3：Governance.tsx 第 118 行 `md:grid-cols-3` → `md:grid-cols-2` 单点改动；数组顺序 [内部,联合,立法,选举] 天然满足目标排布。
- 需求 4：Header.tsx navItems 顺序+label；Footer.tsx「生态系统」栏标题同步；Ecosystem.tsx systems 数组重写（4 卡→3 卡，「链上中国平台」+「NodeUI 桌面端」并入「公民链」整行卡）；SectionTitle「四大核心系统」→「三大核心系统」；Home.tsx:173「探索生态系统」CTA 同步；路由 `/ecosystem` 不改。
- 需求 5：Ecosystem.tsx 164–186 行安全板块 3 卡→4 卡（网格改 `md:grid-cols-2 lg:grid-cols-4`），新增「ML-DSA-65 / 抗量子升级」卡。

## 主要风险点

- 需求 1 删 Hero 后页面无开场标题，直接以「公民币核心指标」开头——按字面执行，不擅自补标题。
- 需求 2 摄像头扫码需 HTTPS 环境与用户授权；需处理拒绝授权/无摄像头的错误提示；识别结果按纯文本地址回填（若为 URI 格式需剥离前缀）。
- 需求 4 内容口径：公民链三交付物、公民App 功能、冷钱包描述均已从仓库实证（citizenchain/Cargo.toml、citizenapp/lib 模块、citizenwallet/lib/signer），文案不得声称未实现能力。
- 需求 5 PQC 未上线（ADR-022 §16 创世前零改动），文案必须写「升级路线已定稿」；现有 AES-256 卡「保护省级签名密钥」描述已过时（省级 wrap key 已删），本次一并修正为现状口径。
- Chat 加密文案用「加密聊天」，不写「端到端加密」（未核实达标）。

## 是否需要先沟通

- 否。五条边界清晰，逐条字面执行；上述风险按报告中给出的默认口径处理。

## 输出物

- 代码（citizenweb/src 各页面 + 扫码组件）
- 中文注释
- `npm run build` 通过 + 页面视觉核验（320/768/1280 断点）
- 文档回写（本卡 + memory 相关）
- 残留清理（无旧文案残留：全站不留「四大核心系统」「CitizenWallet 冷钱包」「生态系统」旧标签）

## 验收标准

- 五条需求逐条可见生效，构建通过
- 扫码弹层在支持摄像头的浏览器可用，拒绝授权有明确提示
- 全站 grep 无旧文案残留
- Review 问题已处理

## 完成记录（2026-07-07）

- 五条需求全部落地并在本地预览逐项核验（DOM 坐标级验证布局、点击验证弹层）：
  1. Tokenomics Hero 板块删除（含金色分隔线），宪法句替换发行分配方案描述。
  2. 会员卡文案改「认证投票公民 / 认证选举公民」；地址输入框右侧新增取景框+横线扫码图标；新建 `citizenweb/src/components/QrScannerModal.tsx`（jsqr + getUserMedia），识别后提取 base58 地址回填，非地址二维码给出明确提示。
  3. 治理页四卡 `md:grid-cols-3`→`md:grid-cols-2`，2×2 排布（上：内部|联合，下：立法|选举）已实测坐标确认。
  4. 导航「产品体系」移至公民币经济与会员订阅之间；三大核心系统=公民（按 citizenapp 现状重写）/ CitizenWallet 公民钱包（内容保持，改名）/ 公民链（整行卡，介绍 runtime/node/onchina）；Footer、首页 CTA、页面 Hero 副标题同步，全站无旧标签残留。
  5. 安全板块 4 卡（新增 ML-DSA-65 抗量子升级卡，口径为「升级路线已定稿」）；顺手修正 AES-256 卡过时的「省级签名密钥」表述。
- 对抗式评审（3 lens + 独立验证）：确认 1 个 HIGH（弹层无 Esc/焦点管理）已修复（Esc 关闭、焦点移入、Tab 圈闭、焦点归还，预览实测通过）；low 项一并处理：错误提示 role="alert"、play() AbortError 兜底、解码节流 150ms+640px 降采样、地址提取零匹配不回填、Footer 命名与产品页对齐。
- `npm run build`、`npm run lint` 全绿；320px 无横向溢出。
- 新增依赖：jsqr@1.4.0。npm audit 的 7 个既有漏洞与本卡无关，已另立后台任务处理。
- 未提交 git（等待用户指示）。
