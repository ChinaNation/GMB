# OnChina 静态资源 Cache-Control 策略

状态：open（2026-07-29 代码完成、编译期验证通过；待链上中国重启后做一次运行态 curl 复核）

## 实现（2026-07-29）

- `core/http_security.rs`：新增缓存策略单源
  - `HASHED_ASSET_PREFIX = "/assets/"`（带尾斜杠）、`CACHE_IMMUTABLE`、`CACHE_REVALIDATE` 三常量
  - `static_cache_control(path) -> &'static str` 纯函数
  - `static_cache_headers(request, next)` 中间件，用 `insert` 覆盖写头
  - 4 个单元测试
- `main.rs`：`frontend_service` 改为
  `Router::new().fallback_service(ServeDir…).layer(middleware::from_fn(static_cache_headers))`

实现选型偏离方案一处：原方案写 `tower::ServiceBuilder`，但 `tower` **不是** onchina 的直接依赖
（只有 `tower-http`）。改用 axum 自带的嵌套 `Router` 包装，保持依赖零变更这一硬边界。
内层 Router 无任何 route，全部请求落到它的 `fallback_service`，SPA 回退行为不变。

## 验证结果

| 项 | 结果 |
|---|---|
| `cargo check` | 通过 |
| `cargo test` | **161 passed**（原 157 + 新 4） |
| 新测试 | `hashed_assets_get_immutable_cache` / `index_and_spa_routes_must_revalidate` / `assets_prefix_requires_trailing_slash` / `unhashed_future_resources_default_to_revalidate` 全绿 |
| `cargo clippy --all-targets` | 仅 2 条 pre-existing warning（`admin_entry.rs` 参数数、`genesis_projection.rs` expect），无新增 |
| `cargo fmt --check` | 本次改动的两处漂移已修；其余漂移为 pre-existing（`occupy.rs`、`auth/actions.rs`、`main.rs:2705` 等），按 no-scope-expansion 未动 |
| `cargo build` | 产出 `target/debug/onchina`（17:17:35）：`wallet_code` 命中 2 次、`max-age=31536000, immutable` 命中 1 次、旧 `complete QR_V1 user_contact code` **0 次** —— 问题 2 一并解决 |

## 运行态复核（重启后执行）

链上中国的 onchina 子进程在本次改动期间已停止。桌面端按
[onchina_proc.rs:32](citizenchain/node/src/onchina_proc.rs:32) 的 dev 兜底解析
`current_exe().parent()/onchina` = `target/debug/onchina`，因此在节点设置页点「启动」即会
拉起 17:17:35 的新二进制，无需其他动作。

启动后复核三条：

1. `curl -skD - https://onchina.local:8964/ -o /dev/null` → 应含 `cache-control: no-cache`
2. `curl -skD - https://onchina.local:8964/assets/index-BH-jNQdT.js -o /dev/null` → 应含
   `cache-control: public, max-age=31536000, immutable`
3. `curl -skD - https://onchina.local:8964/api/health -o /dev/null` → **不应**含
   `cache-control`（中间件只包静态服务，不碰 API）

外加一次浏览器/WebView 硬刷新——本修复对已缓存旧 `index.html` 的客户端无效（见「主要风险」）。

## 任务需求

OnChina 同源托管前端的 `ServeDir` 不发任何 `Cache-Control`，只发 `Last-Modified`。浏览器按
RFC 9111 走启发式缓存（新鲜期 ≈ `(Date − Last-Modified) × 10%`），`index.html` 因此被长期
缓存——它是内容哈希的唯一索引，一旦被缓存整套 vite 哈希失效机制就废掉：客户端永远拿不到
新的 `assets/index-<hash>.js` 引用。

2026-07-29 三码分类改造后实测复现：dist 已重建为 `index-BH-jNQdT.js`、服务端返回的
`index.html` 也已引用新哈希，但浏览器仍在跑缓存里的旧 `index-IHEWbXwx.js`，扫钱包码报
「未知 k: 5」（旧 JS 的 `CODE_TO_KIND` 只有 1–4）。这不是偶发，是每次前端改动都会踩。

附带隐患：vite 默认 `emptyOutDir: true` 会删掉旧哈希文件，所以被缓存的旧 `index.html`
一旦过期，它引用的旧 JS 已不存在 → 404 → 白屏。

## 已确认边界

- 两档策略，按路径分流：
  - `/assets/**`（vite 内容哈希产物）→ `public, max-age=31536000, immutable`
  - **其他一切**（`index.html`、`/`、所有 SPA 回退路由）→ `no-cache`
- 兜底档定为 `no-cache` 是刻意 fail-safe：将来出现 `favicon.ico` 等非哈希资源时默认保守，
  只有明确带内容哈希的路径才升级 immutable。反向配置漏一个就永久钉死客户端，代价不对称。
- 用 `no-cache` 而非 `no-store`：允许缓存但强制校验，配已有的 `Last-Modified` 走 304，
  响应体 0 字节。`no-store` 每次全传无收益。
- 不补 ETag：tower-http 0.6 的 `ServeDir` 不发 ETag，`Last-Modified` + `no-cache` 已等效。
- 实现走 axum `middleware::from_fn`，**不加 `tower-http` 的 `set-header` feature**，
  依赖零变更。`map_response` 不可用（拿不到请求路径）。
- 中间件只包静态服务（`fallback_service`），不碰 API 路由——静态与 API 的缓存策略分开管。
- 策略归口 `core/http_security.rs`（已是 CORS/限流/HTTP 边界策略单源），不另开文件。
- 头用 `insert` 而非 `append`，避免出现两个 `Cache-Control`。
- 不动 CSP、HSTS、`X-Content-Type-Options` 等其他响应头。
- 不改 node 桌面端（`tauri.conf.json` `frontendDist` 走 Tauri asset protocol，不经 ServeDir）。
- 不改 vite dev server（`npm run dev` 走 vite preview:5179，有自己的头）。
- 不修改 `citizenchain/runtime/`、`citizenchain/pallets/`。

## 预计修改目录

- `citizenchain/onchina/src/core/http_security.rs`：新增 `static_cache_control` 纯函数 +
  `static_cache_headers` 中间件 + 单元测试。
- `citizenchain/onchina/src/main.rs`：`frontend_service` 外包一层中间件。

## 主要风险

- **本修复对已中毒客户端无效**：已缓存旧 `index.html` 的浏览器/WebView 在其启发式新鲜期内
  根本不发请求，拿不到新头。当前这次仍需手动硬刷新一次；本修复的价值是此后不再复现。
- `/assets` 前缀判断必须用 `/assets/`（带尾斜杠）。写成 `/assets` 会让 `/assetsfoo`
  之类路径误命中 immutable 档。
- 仓库无 HTTP 层集成测试基建（无 `tests/` 目录、全仓无 `oneshot`/`ServiceExt`），因此
  验证靠纯函数单测 + 一次运行态 curl，不新建集成基建。
- 改的是 Rust 代码，必须 `cargo build` 重建二进制并重启 onchina 才生效。顺带解决同批发现
  的问题：正在运行的 `target/debug/onchina`（14:48:33 编译）不含 `wallet_code`，仍要求
  `k=3` 用户码。

## 完成标准

- `/assets/**` 返回 `public, max-age=31536000, immutable`；`index.html` 与 SPA 回退路由
  返回 `no-cache`，且第二次请求返回 304。
- `static_cache_control` 单测覆盖 immutable 档、no-cache 档、`/assets`（无尾斜杠）与
  未来非哈希资源四类边界。
- `cargo test` 全绿、`cargo clippy --all-targets` 不新增 warning（基线 2 条 pre-existing）。
- `cargo build` 产出的二进制含 `wallet_code`（问题 2 一并解决）。
