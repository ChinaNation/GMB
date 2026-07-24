# 全仓审计整改 · 第 3 轮：控制台(TC4)

任务需求：落地审计 TC4 控制台项。承接 Round 1/2。
所属模块：citizenconsole(本机私有运维工具,已整目录移出 Git;改动本机生效、不推 GitHub,见 [[citizenconsole-out-of-git]])。

## 本轮条目与处置(用户确认)
| # | 处置 | 落点 |
|---|---|---|
| 19 | 对账开关写 production KV 也过 Touch ID | server.mjs reconcile-flags POST |
| 20 | 从根上不让子进程回显密钥(治本,非只靠 chunk 脱敏) | actions/*.sh + server.mjs 流处理 |
| 21 | keychain 密钥值改 stdin 传,不走命令行 argv(防 ps 窥密) | keychain.sh |
| 22 | 状态接口批量(一次 dump)+缓存+异步,不再同步 spawn 200+ | server.mjs nodeStatus / /api/status |
| 23 | hasSession 常量时间比较;删 citizenconsole.js 残留「▶三角」注释;启动 grandpa 字段缺失守卫 | server.mjs / web/citizenconsole.js |

## 必须遵守
- 只在 /Users/rhett/GMB 主检出的 citizenconsole 目录;不推 GitHub(gitignore)
- 不改部署动作的 Touch ID 既有门禁,只补齐缺口
- 死规则:展开指示器禁实心三角(item 23 正是清残留注释)

## 输出物 / 验收
- server.mjs / keychain.sh / citizenconsole.js 改动 + 中文注释
- Node 语法自检(node --check);现有 test/ 若有相关用例跑通
- 残留清理干净

## 执行进度
| Step | 状态 | 说明 |
|---|---|---|
| 19 对账开关 Touch ID | ✅ | reconcile-flags POST 写生产 KV 前加 `authorizeProduction`(与密钥写入同级) |
| 21 keychain stdin | ✅ | put/put-multiline 改 `printf '%s\n%s\n' \| security ... -w`(无值,从 stdin 连读密码+重输),值不入 argv;multiline 加 `tr -d '\n'` 防换行截断;实测 put/get/put-multiline/get-multiline 全 round-trip 通过 |
| 20 密钥回显根治 | ✅ | 先核实 actions/*.sh 本就不 `set -x`/不 echo 密钥(根干净);再把流脱敏由「按 chunk」改「按完整行」(单行密钥不再被 chunk 边界劈开漏出)+ 未闭合 PEM 头整体 hold 到 END(多行私钥不逐行漏);close/error 兜底 flush |
| 22 状态批量+缓存+异步 | ✅ | 一次 `security dump-keychain` 解析本服务全部账户(实测 147 项,15ms)+ 2s TTL 缓存,替代每轮 147+ 个 `exists` spawn;写/删失效缓存;gh secret list 加 10s TTL 缓存(避免每轮 15s 超时网络调用);仅状态展示走批量,写/删/动作路径仍精确单项 |
| 23 常量时间+残注释+grandpa 守卫 | ✅ | hasSession 改 `timingSafeEqual` 常量时间;catalog 缺 grandpa_public_key 启动即给可读报错(指明第几项/标签),不再 `undefined.toLowerCase()` 崩;citizenconsole.js 残留「▶三角」注释改「chevron 折线」 |

## 本轮验证记录
- `npm run check`:node --check server.mjs/routes.mjs + `bash -n` 全脚本(含 keychain.sh)全绿
- node --check:server.mjs / topup/routes.mjs / web/citizenconsole.js 全 OK
- keychain.sh:put/get/put-multiline/get-multiline/exists/delete round-trip 全通过;`security` 以裸 `-w` 调用,值不在 argv
- item 22:批量解析实测(tc4smoke:AAA/BBB=true,ZZZ=false,本服务 147 账户)正确
- `npm test`:唯一失败 `settle.test.mjs`「H1 并发锁」= **既有失败**,该测试只 import topup/routes.mjs+ledger.mjs、零引用本轮改动(server.mjs 未被其触及),属 topup 结算并发测试的既有问题,与 TC4 无关,建议单列排查
- citizenconsole 整目录 gitignore:本轮改动本机生效、不推 GitHub
