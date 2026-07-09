# 官网品牌 logo + 底部产品体系 + 治理体系文案

## 任务需求
1. Footer「产品体系」列表改为：CitizenApp 公民 / CitizenWallet 公民钱包 / CitizenChain 公民链。
2. 治理体系 tab 的「四种投票体系」标题改为「投票引擎」。
3. 左上角 logo：图标换成 docs/国旗.png 中间花徽剪成圆形；文字上白「中华民族联邦共和国」、下金「公民储备委员会」。

## 所属模块
- citizenweb：Header.tsx / Footer.tsx / pages/Governance.tsx / assets/flag-emblem.png

## 实现（均在 /Users/rhett/GMB）
- 图标：`sips -c 480 480 docs/国旗.png --out citizenweb/src/assets/flag-emblem.png`（图片中心即花徽中心，居中裁正好框住白花红底），前端用 `rounded-full object-cover` 圆形遮罩 + `ring-1 ring-white/15`。
- Header.tsx：G 方块 → 圆形国旗花徽 `<img>`；文字「公民区块链/CITIZENCHAIN」→「中华民族联邦共和国」(text-base 白) /「公民储备委员会」(text-xs 金)。
- Footer.tsx：产品体系列表三项改名；logo 同步为国旗花徽 + 同款两行文字（品牌一致，超出任务3字面范围但属统一 logo）。
- Governance.tsx L117：`四种投票体系` → `投票引擎`（仅此一处字面替换）。

## 未改（超出字面范围，待用户确认）
- Governance 首屏 hero 标题仍为「四种民主投票机制」（用户只点名「四种投票体系」）。

## 验证（2026-07-08）
- `npm run build`、`npm run lint` 通过。
- 浏览器（main 5205）：Header 圆形国旗徽 + 新两行文字截图确认，旧 G/CITIZENCHAIN 消失；Footer 产品体系三项 + logo 同步；Governance 标题「投票引擎」在、「四种投票体系」消失；控制台无 error。

## 结论
- 3 项完成并验证。新增资源：citizenweb/src/assets/flag-emblem.png。
