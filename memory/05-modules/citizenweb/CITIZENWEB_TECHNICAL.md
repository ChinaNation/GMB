# 官网模块技术文档

## 1. 模块定位

`citizenweb/` 是 GMB 官网前端工程，用于对外展示公民区块链与项目基础信息。

该模块只负责公开官网页面、白皮书展示、公民宪法读链展示和 CitizenApp 会员订阅发起页，不承载 CitizenChain、链上中国、CitizenApp 或 CitizenWallet 的信任根逻辑。

白皮书唯一真源位于 `citizenweb/src/whitepaper.md`，官网白皮书页通过 Vite raw import 读取该文件；白皮书图片资源继续通过官网构建流程打包展示。

公民宪法唯一真源在**链上** `LegislationYuan.LawVersions[0][effective_version].chapters`（创世值 = runtime 内置 `constitution.scale`）。官网不打包宪法正文，改由 Cloudflare Worker 读链下发（见 3.3），与 CitizenApp / 区块链节点 / 链上中国四端同源，修宪后自动更新。

## 2. 当前技术栈

- 前端框架：React
- 类型系统：TypeScript
- 构建工具：Vite
- 样式：Tailwind CSS Vite 插件与本地 CSS
- 生产产物目录：`citizenweb/dist/`
- 白皮书正文：`citizenweb/src/whitepaper.md`
- 会员订阅页：`citizenweb/src/pages/Membership.tsx`

## 3. 本地构建

在 `citizenweb/` 目录执行：

```bash
npm run build
```

该命令会先执行 TypeScript 构建检查，再执行 Vite 生产构建。发布前必须确认该命令通过。

白皮书内容、首页发行量、链上中国卖点、技术页、生态页和会员页更新后，必须至少访问首页、技术页、生态页、代币经济页、会员订阅页和白皮书页确认页面可正常渲染。

## 3.2. 会员订阅页

- `/membership` 是 CitizenApp 官网会员订阅入口，共三档会员并列卡（ADR-036，与身份彻底解耦）：**自由会员 `freedom`($2.99) / 民主会员 `democracy`($9.99) / 薪火会员 `spark`($99.99)**。任意身份可订阅任意档，官网无身份门槛、卡内不含身份字段（身份改由 CitizenApp 电子护照展示）。每卡展示会员权益（聊天单文件上限 10MB/100MB/5GB、动态、文章）+ 价格 + 订阅按钮。
- 官网订阅先调用 `POST /v1/square/membership/subscribe/challenge`（响应含 `current` 当前订阅态，供官网判定 新订阅/升档/降档/续订），CitizenApp 钱包扫码签名后调用 `POST /v1/square/membership/subscribe`。**一钱包一订阅（ADR-033）**：无活跃订阅→创建 Stripe Checkout Session（返回 `checkout_url`）；已有活跃订阅→在同一订阅上按 `action` 分派——升档 `upgraded`/`upgrade_pending`(返回 `payment_url` 付差价)、降档 `downgraded`(剩余价值进 Stripe 信用余额)、续订 `resumed`、无操作 `already_subscribed`，绝不新建第二个订阅。官网不保存会员状态、不保存本地法币金额、不接触 Stripe secret。
- 支付方式与 USDC（ADR-034 段4，官网订阅面板）：**卡 / USDC** 二选一。卡走上面的 subscribe 分派；USDC 分「购买/续费」（选季/年，`SigningKind=usdc-purchase` → `/prepaid/challenge`+`/prepaid`，金额=月数×月费无折扣，付成功跳 `checkout_url`）与「换档」（`usdc-change` → `/prepaid/change/challenge`+`/prepaid/change`，challenge 响应 `preview{kind,amount_cents?,new_days?}` 在签名弹窗展示；降/平档即时切档出文案、升档跳 `checkout_url`）。官网只是操作入口、不展示当前会员态（会员态展示在 CitizenApp）。
- 取消入口（ADR-034 段4，一个入口按支付方式识别）：`/cancel/challenge`+`/cancel` 签名后，Worker 按 `subscription_source` 分派并回 `cancel_kind`——卡=`stripe`（`cancel_at_period_end` 到期取消）、USDC=`usdc_prepaid`（无自动续、到期自然失效，不动订阅）；官网据 `cancel_kind` 出对应文案。无活跃订阅→`no_active_subscription`。
- 换档金额预览：challenge 响应带 `preview`（Worker 按当期剩余周期比例本地估算的 `{kind, amount_cents}`），官网在签名弹窗展示「升档需补 $X / 降档 $Y 转权益」（估算，实际以 Stripe proration 为准）。
- 身份绑定冻结已废止（ADR-036 取代 ADR-033 规则5）：会员与身份解耦后不再有「身份≠档位」冻结或暂停收款；权益态即订阅态（`active`=`subscription_active`）。官网/App 不再展示冻结横幅。
- USDC 预付路线（ADR-034，后端 + 官网 UI 段4 已落地）：加密钱包无法自动扣款，故 USDC 走**预付固定时长**（季=3 月 / 年=12 月，无折扣=月数×月费）——`POST /membership/prepaid/challenge` + `/prepaid` 签名(`context=level|duration`)后建一次性 Checkout(`mode=payment`)，付成功由 webhook `checkout.session.completed`(`metadata.route=usdc_prepaid`)授时长(`expires_at` 从当前到期日往后叠)。无自动续、无取消，`expires_at` 单独定生死。USDC **换挡**走 `/prepaid/change`(升档补差价一次性 Checkout、降档补时长本地折算)。`/prepaid` 购买对**已有活跃 USDC 且异档**拒(`prepaid_tier_change_required`)，强制走换档，防旧时长被贴成新档漏收。
- 切换支付（一钱包一路线，零权益损失）：USDC→卡=订卡时 `subscription_data[trial_end]` 到 USDC 到期日(试用满首扣)；卡→USDC=预付付成功的 webhook 里先把卡订阅设到期取消再从卡到期日往后叠。均"从当前到期日起算"，旧的照用到到期。
- 会员权益真源在 CitizenApp Cloudflare Worker / D1：Stripe subscription webhook 写入 `square_memberships` 后，iOS / Android / GitHub Android 版 CitizenApp 均通过同一钱包账户读取会员状态。
- 官网与 API 统一使用 `www.crcfrcn.com`：production 默认同源调用 `/api`，不得恢复 `workers.dev` 或独立 API 子域名；`VITE_API_URL` 仅用于明确的本地联调构建。

## 3.3. 公民宪法页（读链）

- `/constitution` tab 位于导航「白皮书」与「关于我们」之间（`Header.tsx` navItems），lazy 加载 `pages/Constitution.tsx`，UI 复用白皮书 `whitepaper-*` 样式（左目录树 + 右正文 + 回顶），另加 `constitution-*`（版本标签、不可修改徽章、章标题复位）。
- 数据源：`GET {VITE_API_URL||'/api'}/v1/constitution`（Cloudflare Worker），返回结构化 `citizenapp.constitution.v1`：`{version, content_hash, version_label{cn,en}, immutable_articles[], chapters[章>节>条>款 + 中英]}`。官网用 **JSX 直接渲染**（无 `dangerouslySetInnerHTML`），中英并列、条级「不可修改条款 · Immutable」徽章、顶部版本标签、底部链上内容摘要。
- Worker 侧（`citizenapp/cloudflare/src/chain/constitution.ts`）：经 CF Access 反代用**已放行的 `state_getStorage`** RAW 读 `Laws[0]`→显式 `effective_version`（只展示已生效版，不露待生效修宪版，ADR-027 §6.1）→`LawVersions[0][v]` / `LawVersionLabels[0][v]` / `ConstitutionImmutableManifest`，TS 逐字节 SCALE 解码（字段序对齐 runtime `legislation-yuan`；House 定长 36B），KV 短缓存 `CONSTITUTION_TTL_SECONDS`（缺省 300s，修宪后一个 TTL 内刷新）。安全口径与节点 `constitution_getDocument` 一致（RAW 读，不走可被恶意升级伪造的 runtime API）。
- 该页公开只读，Worker guard 早返回放行、无会话门禁；解码器单测以真 `constitution.scale` 为夹具（`test/constitution.test.ts`）。

## 3.1. 白皮书结构维护记录

- 2026-07-01：白皮书运行时章节按当前模块边界重排为投票引擎、治理模组、管理员模组、公权业务模组、实体模组、发行模组、交易模组和其他模组。
- 2026-07-01：节点章节拆为节点简介、治理机构、链下清算行；链上中国章节拆为链上中国简介、注册局、链上立法、链上选举。
- 后续更新白皮书时，应继续以 `citizenweb/src/whitepaper.md` 为唯一正文真源，并保持目录锚点与正文标题同步。

## 3.4. 产品页下载按钮（GitHub releases 直下）

- 产品页（`pages/Ecosystem.tsx`）三卡右上角各有醒目「下载」按钮（金色 `text-gold-400`、`text-xl`、加粗），点击弹出平台下拉（`components/DownloadButton.tsx`，自带点击外部关闭）。
- 直下走 **`https://github.com/ChinaNation/GMB/releases/latest/download/<固定资产名>`**（基址常量在 `DownloadButton.tsx`）：只要每次发版都用相同固定名，链接永远指向最新版对应资产。
- 各卡选项与资产：
  - 公民 CitizenApp：iOS（弹提示去 App Store，无直链）/ Android → `citizenapp-android.apk`
  - 公民钱包 CitizenWallet：iOS（弹提示）/ Android → `citizenwallet-android.apk`
  - 公民链 CitizenChain：macOS → `citizenchain-macos-arm64.dmg`、Windows → `citizenchain-windows-x64.msi`、Linux-arm → `citizenchain-linux-arm64.deb`、Linux-amd → `citizenchain-linux-amd64.deb`
- **发布约定**：上述固定资产名是官网下载与发布流程的契约，发版须把同名资产挂到 GitHub 最新 release，否则直下 404。当前 release 命名尚未固定（`macos-arm64.dmg` 已固定，`.deb/.msi` 带版本号、缺 Android/Linux-arm/钱包资产），需按此对齐。
- iOS 为纯提示文案（`window.alert`），暂无 App Store 直链。

## 3.5. 标签页标题与图标

- `index.html`：`<title>公民链｜中华联邦公民储备委员会</title>`；`<link rel="icon" type="image/png" href="/favicon.png">` = 官网国旗图（`public/favicon.png`，由 `src/assets/flag-emblem.png` `sips -Z 128` 生成）。原紫色闪电 `favicon.svg` 已删。

## 4. 线上部署口径

当前官网由 Cloudflare Pages 项目 `citizenweb` 承载，正式域名为 `https://www.crcfrcn.com`；`/membership` 与同源 `/api` 共同组成官网订阅入口。发布使用 `npx wrangler pages deploy dist --project-name citizenweb --branch main`，发布后必须真实访问 `/membership` 验证页面与同源 API。

以下 Nginx 流程是 2026-04-30 的历史部署记录，不是当前 production 发布入口：

当前已确认 `64.181.239.233` 的 HTTP 80 端口由 Nginx 提供官网静态页，但仓库内尚无官网专用部署脚本。

推荐标准流程：

1. 本地执行 `npm run build`。
2. 确认 `citizenweb/dist/` 内生成 `index.html`、`assets/`、`favicon.png`。
3. 通过 SSH 登录官网服务器。
4. 备份当前 Nginx 静态根目录。
5. 上传并替换为 `citizenweb/dist/` 产物。
6. 执行 `nginx -t` 与 `systemctl reload nginx`。
7. 使用 `curl -I http://64.181.239.233` 和浏览器访问验证。

## 5. 2026-04-30 发布记录

- 本地 `npm run build` 已通过。
- 产物入口为 `citizenweb/dist/index.html`。
- 生成的主要资源为：
  - `citizenweb/dist/assets/index-CyL55-iR.js`
  - `citizenweb/dist/assets/index-C4uOHj7J.css`
- 服务器使用 releases/current 部署结构：
  - 新 release：`/var/www/crcfrcn/releases/20260430-122742`
  - 当前指针：`/var/www/crcfrcn/current`
- Nginx 站点配置：`/etc/nginx/sites-available/crcfrcn`。
- Nginx root：`/var/www/crcfrcn/current`。
- `nginx -t` 通过，`systemctl reload nginx` 已执行。
- `http://64.181.239.233` 返回 Nginx `200 OK`，上线后 `Last-Modified` 为 `2026-04-30 19:27:54 GMT`。
- 本次部署过程中发现 `/Users/rhett/.ssh/ed25519` 是加密私钥；如果 SSH agent 未加载该 key，服务器会先接受公钥但本机无法完成签名。处理方式是在本机 Terminal 执行 `ssh-add --apple-use-keychain /Users/rhett/.ssh/ed25519` 并输入私钥密码。

## 6. 后续要求

- 官网部署前需确认本机 SSH agent 已加载 `/Users/rhett/.ssh/ed25519`。
- 建议补充官网专用部署脚本，固化远程 Web 根目录、备份目录、上传目录与 Nginx reload 流程。
- 若官网服务器同时承载 CitizenChain 节点，不得在官网部署过程中重启或清理 `citizenchain-node` 服务。
