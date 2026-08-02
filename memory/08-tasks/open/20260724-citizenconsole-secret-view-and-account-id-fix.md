# 20260724 · 公民控制台:密钥「查看」按钮 + 充值发币 account_id 对齐

状态:已完成(2026-07-24)· account_id 对齐 + 密钥查看 + Project ID 注入 · `npm test` 26/26 绿、`npm run check` 通过、`dart analyze` 无问题
关联:[20260718-recharge-e2e-audit](20260718-recharge-e2e-audit.md)(充值发币 E2E 审计)

## 背景
排查 `citizenconsole/test/settle.test.mjs` 「H1 并发锁」失败(实为 6/6 全挂)时,顺带核出充值发币链路两处问题:

1. **测试夹具过期**:settle 单测 `CONFIG` 用了弃用键 `TOKEN_CONTRACTS`,而代码已按币种分键 `TOKEN_USDC/TOKEN_USDT`(`topup/routes.mjs:14`)。`cfg.tokenContracts=[]` → 白名单 fail-closed 守卫返回 400,胜出调用拿 400 而非 200。已修:夹具键改 `TOKEN_USDC`。
2. **🔴 致命契约不匹配**:Worker `/square/topup/settlement/pending` 吐目标地址字段 `account_id`(`citizenapp/cloudflare/src/topup/settlement.ts:56`),控制台却读 `order.gmb_address`(`routes.mjs:292` / `ledger.mjs:42`)。仓库已统一账户字段为 `account_id`(commit `44d29b4c`),控制台因整目录 gitignore 漏掉这次扫描。运行时 `destAccountId=undefined` → @polkadot 编码 AccountId32 抛错 → 每笔真单发币必被强制转异常,**币永远发不出**。单测 stub 伪造了 `gmb_address` 掩盖了它。

## 决策
- **account_id 对齐(方案 A)**:改控制台向全仓 `account_id` 看齐,不动已部署 Worker。`destSs58` 形参一并更名 `destAccountId`(消除误导性命名残桩——值本是 hex AccountId,非 SS58 串)。
- **密钥查看**:密钥表操作列「更换/配置」左侧加「查看」,Touch ID 通过后行内回显明文;范围 = 配置/令牌类(含 `SETTLE_TOKEN`)。**发币私钥 `DISBURSE_KEY` 不可查看**,守「私钥永不回显网页」既有不变量(见 memory `topup-disburse-security-invariants`)。

## 改动清单
account_id 对齐:
- `topup/routes.mjs:292` `destSs58: order.gmb_address` → `destAccountId: order.account_id`
- `topup/chain_transfer.mjs` `destSs58` → `destAccountId`(形参 + 调用)
- `topup/ledger.mjs:42` `gmb_address: order.gmb_address` → `account_id: order.account_id`
- `test/settle.test.mjs`、`test/ledger.test.mjs` stub `gmb_address` → `account_id`(回放真实 Worker 字段名,今后能守住此契约)

密钥查看:
- `topup/routes.mjs` 新增 `POST /api/topup/config/reveal {name}`:`validOrigin` → 白名单 `CONFIG_ITEMS`(不含 `DISBURSE_KEY`)→ `authorizeProduction`(Touch ID)→ 返回 `keychainGet` 值;**值绝不写日志**。
- `web/citizenconsole.js` 每已配置的可查看行加「查看」按钮 + 行内明文展示(只读 / 复制 / 20 秒自动隐藏);发币私钥行不加。
- `web/styles.css` 加 `.secret-ops .view` 次级(ghost)按钮样式。
- `test/reveal.test.mjs` 锁定:令牌可查看、私钥拒查看、未配置拒、来源拒。

## 验证
- `npm test` 全绿(含新 reveal 用例)。
- `npm run check`(node --check)通过。

## Project ID 注入(citizenapp · 2026-07-24 追加)
WalletConnect(Reown)Project ID `8830074307d80484b839db4eb10b1f2c`(公开值)已烘焙进源码默认值,不依赖任何构建脚本:
- `citizenapp/lib/transaction/onchain-topup/topup_webview_page.dart`:`projectId` 由 `static const` 改为「`_prodProjectId` 默认 + `--dart-define=WALLETCONNECT_PROJECT_ID` 覆盖」的 getter(与 `SQUARE_API_URL`/`prodBaseUrl` 同一约定)。
- 效果:release / CI / iOS(Xcode)/ `citizenapp-run.sh` 任意构建路径都带正式 Project ID,付款步不再短路「WalletConnect 未配置」;开发换项目时显式传 dart-define 覆盖。
- 验证:`dart analyze lib/transaction/onchain-topup/` 无问题。
- 由此解决关联审计卡 [20260718-recharge-e2e-audit](20260718-recharge-e2e-audit.md) 的「Project ID 全构建未注入」阻断项。

## 遗留(不在本卡)
- 充值发币真实 testnet E2E 仍待 Worker 侧 `TOPUP_*` 变量 / D1 迁移 / `TOPUP_SETTLE_TOKEN` secret 落地(见关联审计卡)。
