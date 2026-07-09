# 官网手续费卡片/导航名/会员页扫码与高度调整

## 任务需求（均在 /Users/rhett/GMB）
1. Tokenomics 手续费板块：删掉「链上交易手续费/按8:1:1分配」「链下交易手续费/全部归清算行」两组小标题。
2. 4 张手续费卡片改 2×2：上左 全节点、上右 清算行、下左 国储会费用账户、下右 安全基金账户。
3. 导航名：区块链技术→区块链、公民币经济→公民币、产品体系→产品（Header 导航；Footer 快速链接/产品分区同步）。
4. 会员页扫码弹层缩小（太大），确认是页内弹窗而非另开页面（QrScannerModal，已是页内 overlay，只需缩小）。
5. 会员页顶部 CitizenApp Membership 区块高度缩小。

## 涉及文件
- citizenweb/src/pages/Tokenomics.tsx（1、2）
- citizenweb/src/components/Header.tsx、Footer.tsx（3）
- citizenweb/src/components/QrScannerModal.tsx（4）
- citizenweb/src/pages/Membership.tsx（5）

## 核心边界
- 手续费卡片 desc 沿用现有含链上/链下语境；费率数据不动。
- 导航改名同步 Footer 保持一致；Ecosystem 页内 eyebrow「产品体系」属页面内容，暂不动（报备）。
- 只在 /Users/rhett/GMB 操作。

## 验收
- build/lint 通过；浏览器抽查手续费 2×2、导航名、会员页扫码弹窗尺寸与顶部高度。

## 执行记录
### 阶段 0：任务卡创建
- 已创建。

### 阶段 1：实现 + 验证（2026-07-08，均在 /Users/rhett/GMB）
- Tokenomics.tsx：删两组小标题；onchain/offchain 两数组合并为 feeDistribution，2×2 网格(md:grid-cols-2)，顺序=全节点奖励/清算行/国储会费用账户/安全基金账户。DOM 实测 1280px 下：上左全节点80%、上右清算行100%、下左国储会费用账户10%、下右安全基金账户10%。
- Header.tsx：导航 区块链技术→区块链、公民币经济→公民币、产品体系→产品。
- Footer.tsx：快速链接同步(区块链/公民币)，产品分区标题 产品体系→产品。
- Membership.tsx：顶部 section py-20 md:py-28 → py-10 md:py-12（hero 高 ~482→354）。
- QrScannerModal.tsx：弹层 max-w-md→max-w-xs(448→320px)、p-6→p-5，视频/扫码框加 max-w-[240px] 居中缩小。确认仍是页内 fixed overlay 弹窗(非路由页)。
- build/lint 通过；浏览器 DOM 逐项核对全过、控制台无 error。
- 备注：Ecosystem 页内 eyebrow「产品体系」为页面内容，未动。
### 结论
- 5 项完成并验证。
