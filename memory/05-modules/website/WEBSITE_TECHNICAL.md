# 官网模块技术文档

## 1. 模块定位

`website/` 是 GMB 官网前端工程，用于对外展示公民币区块链与项目基础信息。

该模块只负责公开官网页面，不承载 CPMS、SFID、CitizenChain 或 WuMinApp 的信任根逻辑。

## 2. 当前技术栈

- 前端框架：React
- 类型系统：TypeScript
- 构建工具：Vite
- 样式：Tailwind CSS Vite 插件与本地 CSS
- 生产产物目录：`website/dist/`

## 3. 本地构建

在 `website/` 目录执行：

```bash
npm run build
```

该命令会先执行 TypeScript 构建检查，再执行 Vite 生产构建。发布前必须确认该命令通过。

## 4. 线上部署口径

当前已确认 `64.181.239.233` 的 HTTP 80 端口由 Nginx 提供官网静态页，但仓库内尚无官网专用部署脚本。

推荐标准流程：

1. 本地执行 `npm run build`。
2. 确认 `website/dist/` 内生成 `index.html`、`assets/`、`favicon.svg`、`icons.svg`。
3. 通过 SSH 登录官网服务器。
4. 备份当前 Nginx 静态根目录。
5. 上传并替换为 `website/dist/` 产物。
6. 执行 `nginx -t` 与 `systemctl reload nginx`。
7. 使用 `curl -I http://64.181.239.233` 和浏览器访问验证。

## 5. 2026-04-30 发布记录

- 本地 `npm run build` 已通过。
- 产物入口为 `website/dist/index.html`。
- 生成的主要资源为：
  - `website/dist/assets/index-CyL55-iR.js`
  - `website/dist/assets/index-C4uOHj7J.css`
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
