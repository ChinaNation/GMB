# CitizenConsole 生物识别与 Cloudflare 最小权限改造

> 状态：原任务范围完成；生产签名与进程隔离由
> `20260726-citizenconsole-production-hardening.md` 接续整改
>
> 用户已确认本任务新增本任务卡，以及本机私有测试文件
> `citizenconsole/test/biometric-security.test.mjs`。CitizenConsole 整目录继续由 Git 忽略。

## 任务目标

1. CitizenConsole 只能通过 Touch ID 打开，禁止 Mac 密码回退。
2. 所有本机 Secret 使用 Apple Data Protection Keychain 与
   `biometryCurrentSet` 保护，删除普通命令行明文读取路径。
3. CitizenConsole「发币控制台」经一次 Touch ID 解锁后可持续发币，不设置时间超时；
   点击锁定、离开页面、连接断开或进程退出时锁定并清除内存密钥。
4. Cloudflare 本机管理权限拆分为 `CF_DEPLOY_TOKEN`、`CF_DATA_TOKEN`、
   `CF_ZT_TOKEN` 三个最小权限令牌；Worker 运行时 `CF_API_TOKEN` 保持独立。
5. 登录挑战改为 D1 条件更新原子消费；设备子密钥只允许 `issued_at` 单调递增，
   拒绝并发重放和旧绑定回滚。

## 硬边界

- 不修改 `citizenchain/runtime/`。
- 不修改平台订阅、创作者订阅、取消订阅、换档、续费或扣款逻辑。
- 不增加任何用户签名。
- 不部署国储会节点。
- 不执行 GitHub push，不触发 GitHub Actions。
- Secret 不进入仓库、浏览器响应、日志、命令参数、剪贴板或明文文件。
- 充值发币是唯一允许在页面生命周期内持续持有内存密钥的功能；其他 Secret
  写入、删除、查看和使用仍按单次敏感动作执行 Touch ID。

## 预计修改目录

- `citizenconsole/`
  - 本机私有代码：统一 Swift 生物识别与 Keychain 安全代理，调整控制台锁屏、
    充值发币持续解锁、三令牌调用隔离和测试。
  - 残留清理：删除 `keychain.sh`、普通 `security` 读取、旧 Wrangler OAuth
    回退和旧令牌命名。
  - Git 边界：整目录继续忽略，不提交、不推送。
- `citizenapp/cloudflare/src/auth/`
  - Worker 代码：登录挑战原子消费与设备子密钥单调更新。
- `citizenapp/cloudflare/test/`
  - 测试代码：并发登录、已消费挑战、失败后烧毁挑战、设备绑定重放和回滚。
- `memory/03-security/`
  - 文档：固化 CitizenConsole 生物识别、Keychain 与 Cloudflare 最小权限规则。
- `memory/01-architecture/citizenapp/`
  - 文档：同步 Worker 认证原子消费和 CitizenConsole 充值发币安全边界。

## 执行清单

- [x] 建立 Apple 原生生物识别安全代理。
- [x] 检查现有 Keychain 项并清理旧 Keychain 读取实现；旧服务已无可迁移项目。
- [x] CitizenConsole 总入口改为 Touch ID 成功后才签发本机会话。
- [x] 充值发币实现无时间限制的单实例持续解锁和事件锁定。
- [x] 三个 Cloudflare 管理令牌完成代码隔离、权限验证和安全存储。
- [x] 新令牌验收后撤销并删除旧 Wrangler OAuth。
- [x] 登录挑战使用 D1 条件更新原子消费。
- [x] 设备子密钥使用时间窗校验和单调条件 UPSERT。
- [x] 补齐中文注释、文档和残留清理。
- [ ] 完成充值发币真实解锁运行验收；当前发币私钥及其它配置不存在，必须先由用户恢复。

## 验收记录

- Xcode：`Xcode 27.0 beta (27A5228h)`；Apple Development Team
  `7QJXLLBA6J` 自动签名成功。签名 entitlements 同时包含独立 application identifier
  与同值 Keychain access group；Data Protection Keychain 临时项目的 Touch ID
  写入、读取、删除均真实通过，临时项目已删除。
- CitizenConsole：`npm run check && npm test` 通过，31/31；真实 HTTP 未解锁时
  `/api/status` 返回 403，Touch ID 解锁后返回 200，真实页面只在解锁后渲染目录。
- Cloudflare Worker：`npm test -- --run` 通过，29 个测试文件、178/178。
- Cloudflare 三令牌均经 `/user/tokens/verify` 返回 `active`。部署令牌访问
  Workers/Pages 为 200、访问 D1 被拒；数据令牌访问 D1/KV/R2/Queues 为 200、访问
  Workers 被拒；Zero Trust 令牌访问 Tunnel/Access/限定 Zone DNS 为 200、访问
  Workers 被拒。旧 Wrangler OAuth 已执行 `wrangler logout` 并确认配置文件不存在。
- 残留：旧无签名 `security-broker`、`touchid-auth` 和临时 Swift 模块已移出运行目录；
  旧令牌长名、普通 Keychain CLI 读取、OAuth 回退和迁移入口扫描均无正式实现残留。
- 后续复查：本任务当时使用的 Apple Development 自动签名仍带有调试授权，且通用原生
  命令行和可修改 Node/脚本不满足最终生产边界；后续任务已禁止把该验收记录作为正式签名
  完成依据。充值发币正式配置后来已在生产密钥统一治理任务中补齐，本段旧阻塞不再有效。
