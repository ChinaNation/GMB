# CitizenConsole 密钥存储迁移与开发者账号解耦

状态：done（2026-07-31 迁移完成并验证，旧访问组已清零）

## 起因

CitizenConsole 全部生产密钥存放在 Data Protection Keychain 的访问组
`7QJXLLBA6J.com.gmb.citizenconsole.security` 下。该存储要求
`keychain-access-groups` entitlement → 要求描述文件授权 → 要求付费开发者账号。

本机唯一可用的 Mac Development 描述文件 `ExpirationDate = 2026-08-03 00:05:07 UTC`
（北京时间 8/3 08:05），且用户明确今后不再使用该 Apple 账号。到期后旧访问组将
**永久无法打开**，51 条生产密钥（含发币私钥）全部够不到。

## 实测确定的三条硬约束

| 方案 | 实测结果 |
| --- | --- |
| Data Protection Keychain（原方案） | `errSecMissingEntitlement -34018`，必须描述文件 |
| 传统钥匙串 + `biometryCurrentSet` | `-34018`，生物识别锁定同样要 entitlement |
| 传统钥匙串普通条目 | **写入 ✅ 读取 ✅**，不需要任何 Apple 账号资源 |

结论：只要继续用 Data Protection Keychain，就永远绑死在开发者账号上。必须迁出。

## 改动

**存储层** `KeychainVault.swift`
- `baseQuery` 去掉 `kSecUseDataProtectionKeychain`，写入落传统钥匙串；
  `dataProtection: true` 仅保留给迁移读取旧数据。
- 删除 `accessControl()`（`biometryCurrentSet`），改用 `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`。
- **Touch ID 未降级**：`authenticate()` 原样保留，每次读写前照旧 `evaluatePolicy`，
  只认生物识别、不允许密码回退。强制点由「存储层锁定」改为「程序层强制」。

**迁移** `migrateAllFromDataProtection`
- **枚举式**，不接受调用方名单：以 `kSecAttrService = "GMB CitizenConsole"` 为界，
  `kSecMatchLimitAll` 列出旧存储全部条目。
- 每条严格「读旧 → 写新 → 回读校验 → 删旧」，任一步失败即抛出且**不删原件**，
  可重复执行、可中断续跑。
- 明文只在原生进程内存周转，回传 Node 的只有 `migrated` / `skipped` 计数
  （与 `secret.read` 的发币私钥守卫互补）。

**入口**：`secret.migrate` 操作 + `/api/secret/migrate` 端点 + 控制台顶部
「迁移密钥」按钮（位于「空闲」与「刷新状态」之间）。

**解耦收尾**
- `entitlements` 摘除 `keychain-access-groups`，只剩两项 `cs.*`。
- `start.sh` 不再嵌入 `embedded.provisionprofile`。
- 启动守卫由「正向要求访问组 + 描述文件」翻转为**反向拦截**：一旦有人加回
  `keychain-access-groups` 或嵌入描述文件，构建直接失败并说明原因。

## 为什么按名单迁移会漏（三轮才迁完）

| 轮次 | 结果 | 漏因 |
| --- | --- | --- |
| 第 1 轮 | 迁移 41 / 跳过 157 | `knownKeychainRefs()` 未覆盖 `topup` 环境 |
| 第 2 轮 | 迁移 9 / 跳过 198 | 补上 topup 9 项后迁走 |
| 第 3 轮 | 迁移 0 / 跳过 431 | 按 `isAllowed` 白名单全集推算（含 node-00..99）仍猜不到最后 1 条 |
| 第 4 轮 | **迁移 1 / 跳过 0** | 改枚举式，直接向 Keychain 要清单 |

教训：**存储迁移必须枚举真实存储，不能靠名单推算**。历史上写入过、后来从名单
或节点清单中消失的条目，按名单扫永远找不到。

另有一次误判：读 `keychain-2.db` 的 `cp` 副本未带 WAL，得出「还剩 10 条」的过期结论。
查活动数据库必须直读实时库（`file:...?mode=ro`）。

## 验收

- 旧访问组 `7QJXLLBA6J.com.gmb.citizenconsole.security` 查实时库 **已清零**（51 → 0）。
- 签名实际 entitlements 只剩 `cs.allow-jit` + `cs.allow-unsigned-executable-memory`；
  `embedded.provisionprofile` 不存在。
- `npm run check` 通过、`npm test` 37 passed / 0 failed、`build-production` 验收通过、
  服务 200。

## 换开发者账户时的操作

1. 同步改 `start.sh` 的 `TEAM_ID` 与 `AppMain.swift` 的 `subject.OU`，用新证书
   `./start.sh build-production`。
2. 控制台顶部点「迁移密钥」。
3. 传统钥匙串 ACL 会弹一次系统授权，允许即可——**密钥不会丢**。

迁移按钮与整套迁移能力**常驻保留**，不是一次性脚本。

## 同批完成的控制台改动

- 密钥列表每行操作列新增「查看」按钮（配置/更换左侧）：Touch ID 后弹窗显示明文供复制，
  关闭即清空 DOM 与变量并重新上锁；仅已配置的 Keychain 项可见，GitHub 密钥不可回显，
  发币私钥由原生侧守卫拒绝返回。
- 充值发币页「← 返回控制台」改走 `history.back()`（同源历史存在时），
  让主页从 bfcache 恢复，消除整页重载造成的「强制刷新」观感。
