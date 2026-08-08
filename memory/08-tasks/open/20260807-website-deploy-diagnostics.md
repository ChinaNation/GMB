# 官网部署：连通性诊断 + 失败原因分流 + 成败结论简明化

任务需求：官网部署失败时要简明扼要说清**为什么**失败，成功时给简明结论；
并在部署前加连通性诊断，避免真实原因被工具吞掉。

所属模块：citizenconsole（不在 Git 版本库内）

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 起因：一个误导性文案连累两轮排查

`actions/cloudflare.sh` 原写法：

```bash
npm exec -- wrangler whoami >/dev/null || { echo 'CF_DEPLOY_TOKEN 验证失败' >&2; exit 1; }
```

两个问题叠加：

1. **任何非零退出都被贴成令牌问题**。断网、DNS 失败、Cloudflare 5xx、wrangler 崩溃，
   全报「CF_DEPLOY_TOKEN 验证失败」。
2. **`>/dev/null` 把真实报错吞了一半**。wrangler 的 `✘ ERROR fetch failed` 走 stdout，
   控制台上只剩一句日志文件路径 + 那句误导的中文。

实际后果：真因是 TCP 连不上（wrangler 日志里 `TypeError: fetch failed`，
请求发出到失败恰好 10.5 秒 = undici 默认 connect 超时），而排查时连查两轮令牌。

## 改动

| 位置 | 内容 |
|---|---|
| 新增 `probe_cloudflare_api()` | 用与 wrangler 相同的 `GMB_NODE_BIN` 先直连一次 Cloudflare API，DNS 与连接分别报告，退出码 2=DNS、3=连接 |
| 新增 `website_fail()` | 统一失败出口：第一行一句话说清原因，第二行起才是细节 |
| 新增 `website_step_failed()` + ERR trap | 未被显式分类拦下的失败也要报出**死在哪一步**，不只留退出码 |
| 令牌检查 | 不再 `>/dev/null`，输出原样打出；失败按 `fetch failed` / `6111·6003·10000·Invalid API Token` / 其它三类分流 |
| 健康检查 | 从「curl 是否报错」改为**看状态码**，非 200 同样判失败并说明 |
| 成功结论 | 三行：项目与分支、本次部署地址（从 wrangler 输出提取）、正式域名 + HTTP 状态码 |
| `WRANGLER_SEND_METRICS=false CI=1` | 与 `deploy_wrangler` 同口径。遥测是与部署无关的额外出站请求，失败时只会多一行噪声 |

## 诊断探针的关键实现点

**必须递归剥 cause 并展开 `AggregateError`。** undici 走 Happy Eyeballs 会同时试多个地址，
把每个地址的真实错误裹进 `cause.errors`，而 `AggregateError` 自身既没有 `code` 也没有 `syscall`。
只看 `error.cause.code` 的写法在最常见的 ECONNREFUSED 下只会打印一句光秃秃的 `fetch failed`
——加了等于没加。另外只有 `message` 没有 `code` 的 cause（`bad port`、证书错误）也要收。

四种形态实测（用包内 node 真跑）：

```text
正常        连通正常，HTTP 400（413ms）
连接被拒    错误码 ECONNREFUSED，系统调用 connect，目标 127.0.0.1:49999
连接超时    错误码 UND_ERR_CONNECT_TIMEOUT（10537ms）   ← 与线上故障签名完全吻合
TLS 证书    错误码 DEPTH_ZERO_SELF_SIGNED_CERT
DNS 失败    DNS 解析失败：ENOTFOUND
```

ERR trap 会不会触发也实测过：脚本是 `set -euo pipefail`（**无 `-E`**），
但 trap 在函数内设置、失败命令也直接在该函数内，实测确认触发。

## 尚未定位

**控制台跑就 TCP 超时、我从终端跑同一条命令每次都通**，这个差异没查出来。
已排除（均实测）：令牌本身、包内 node、entitlements（无 app-sandbox）、PATH 顺序、
令牌尾随空白、Surfshark 隧道（路由未被劫持）、`npm ci` 与无 TTY。
一度归因到互联网共享，**是错的**——路由表显示到 Cloudflare 走 en0，内核不会选 bridge100 源地址，
该结论已撤回。剩下唯一无法从终端复刻的是「launchd 拉起的签名应用的子进程」这个上下文。

下次失败时，新增的诊断会直接给出 errno、系统调用与目标地址，届时再定位。

## 测试

99 个用例（本次新增 2 组）。钉住：旧的一刀切文案与 `>/dev/null` 不得回归、
失败按网络/鉴权分流、每步失败报出步骤名、成功给三行结论、健康检查看状态码、
诊断必须排在令牌检查之前、`AggregateError` 展开与 message 兜底都在。

写测试时自己踩了一个坑并修掉：切片用 `script.indexOf('\ncase "$mode" in')` 取结束位置，
而该字符串在 `deploy_website` **之前**就出现过一次，切出空串会让整组断言恒真。
已改为从函数起点之后再找。

## 与本次无关的既有红灯

`test/production-security.test.mjs:588` 钉 `frozen_run_id='30724462739'`，
而 `actions/citizenchain.sh:106` 现为 `31164684013`（另一线程于 08-07 07:16 改动）。
两者不一致，且这个值决定 44 个节点装哪份二进制，未擅自改动，需用户裁决。
