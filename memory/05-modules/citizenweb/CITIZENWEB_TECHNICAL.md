# 官网模块技术文档

## 1. 模块定位

`citizenweb/` 是 GMB 官网前端工程，用于对外展示公民区块链与项目基础信息。

该模块只负责公开官网页面、白皮书展示和 CitizenApp 会员订阅发起页，不承载 CitizenChain、链上中国、CitizenApp 或 CitizenWallet 的信任根逻辑。

白皮书唯一真源位于 `citizenweb/src/whitepaper.md`，官网白皮书页通过 Vite raw import 读取该文件；白皮书图片资源继续通过官网构建流程打包展示。

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

- `/membership` 是 CitizenApp 官网会员订阅入口，展示访客会员、投票公民会员、竞选公民会员三档美元月费和权益说明。
- 官网页面只调用 Cloudflare Worker `POST /v1/square/membership/stripe/checkout` 创建 Stripe Checkout Session，不保存会员状态、不保存本地法币金额、不接触 Stripe secret。
- 会员权益真源在 CitizenApp Cloudflare Worker / D1：Stripe subscription webhook 写入 `square_memberships` 后，iOS / Android / GitHub Android 版 CitizenApp 均通过同一钱包账户读取会员状态。
- `VITE_CITIZENAPP_SQUARE_API_BASE_URL` 可在官网构建时指定 Worker API 根地址；未设置时使用 production Worker 默认地址。

## 3.1. 白皮书结构维护记录

- 2026-07-01：白皮书运行时章节按当前模块边界重排为投票引擎、治理模组、管理员模组、公权业务模组、实体模组、发行模组、交易模组和其他模组。
- 2026-07-01：节点章节拆为节点简介、治理机构、链下清算行；链上中国章节拆为链上中国简介、注册局、链上立法、链上选举。
- 后续更新白皮书时，应继续以 `citizenweb/src/whitepaper.md` 为唯一正文真源，并保持目录锚点与正文标题同步。

## 4. 线上部署口径

当前已确认 `64.181.239.233` 的 HTTP 80 端口由 Nginx 提供官网静态页，但仓库内尚无官网专用部署脚本。

推荐标准流程：

1. 本地执行 `npm run build`。
2. 确认 `citizenweb/dist/` 内生成 `index.html`、`assets/`、`favicon.svg`、`icons.svg`。
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
