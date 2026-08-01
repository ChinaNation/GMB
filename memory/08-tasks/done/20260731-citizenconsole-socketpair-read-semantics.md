# CitizenConsole 私有安全通道读取语义修复

状态：done（2026-07-31 修复已验证；临时诊断日志已删除并重建）

## 缺陷

点「使用 Touch ID 打开」永远转圈、指纹框不出现，随后整个控制台不响应。

`AuthenticatedChannel.serve` 用 `FileHandle.read(upToCount: 64 * 1024)` 读 socketpair。
该 API 在 socket 上**不是**「最多读 N 字节」，而是**阻塞到读满 N 字节或 EOF**
（栈实证内部转调 `-[NSConcreteFileHandle readDataOfLength:]`）。安全请求只有几十字节，
永远凑不满 65536，原生侧永久停在读，Node 侧发完请求后停在等回包，双向僵死。

Node 的 `securityRequest` 是同步通道（`writeSync`+`readSync`），一旦僵死就冻结 Node
事件循环，端口不再 accept、已有连接无响应，表现为「控制台打不开」。

## 定位过程（可复用）

1. `sample` 两端进程：原生侧 100% 停在 `serve → read`，Node 侧停在 `node::fs::Read`。
2. `lsof` 确认 socketpair 两端正确配对（原生 fd 3 ↔ Node fd 1），排除通道断裂。
3. 在 `securityRequest` 加 **stderr**（fd 2）诊断日志，得到决定性数据：

```
[chan] send op=authorize.open fd=1 bytes=75
[chan] wrote=75/75
```

写入完整（排除部分写入），且**完全没有 `read=` 行**——把问题从 Node 侧逼到原生侧收不到。

4. 独立小程序实证（不碰项目代码）：socketpair 写 75 字节，
   `FileHandle.read(upToCount: 64*1024)` 4 秒不返回，语义确认。

## 修复

`serve` 改用 POSIX `Darwin.read(descriptor, ...)`：有多少读多少，
`count == 0` 视为 EOF，`count < 0` 且 `errno == EINTR` 重试，其余抛错。

## 缺陷来源

2026-07-26 `20260726-citizenconsole-production-hardening.md` 把「通用密钥 CLI」改为
「常驻签名 app + 匿名 socketpair 私有通道」时写错 API。该卡执行清单最后一项
`[ ] 完成 Developer ID 正式签名、公证、真实 Touch ID 和篡改拒绝验收` 从未打勾——
`.runtime/` 一直没有构建产物，**这条通道从建成起就没被真正执行过**，
直到 2026-07-31 首次构建成功才暴露。

## 已知遗留隐患（未在本卡修）

`server.mjs:securityRequest` 的 `writeSync(securityFd, request)` **不检查返回值**。
本次实测 `wrote=75/75` 未触发，但请求体增大（如 `secret.write` 长值）时部分写入会
造成同类僵死。建议补循环写满。

## 同批修复的三个连带缺陷（2026-07-31）

**① 写入侧未检查返回值。** `securityRequest` 的 `writeSync(securityFd, request)` 不检查
返回值，状态页 9356 字节的批量请求必然部分写入。已改为循环写满。

**② 批量查询超过原生侧 128 项上限。** `OperationCatalog.checkedItems` 限定单次 ≤128 项，
而 `knownKeychainRefs()` 一次提交 198 项（44 节点 × 4 + 22）。整批被拒 → `/api/status` 500
→ 前端 `statusById` 为空 → **全部模块显示「此操作不需要部署密钥」**（充值发币走
`/api/topup/*` 专属页故不受影响）。已改为按 100 项分批，且每批独立 try/catch——
任一批失败只丢该批，不再一个模块拖垮整页。**节点数超过 26 个此页就必然打不开**，
同样是从 7/26 起就没跑通过。

**③ 前端初始化无容错。** `loadStatus()` 的 `status.modules.map` 在接口 500 时抛
TypeError，整个初始化从该行断掉，后续渲染与事件绑定全不执行——表现为**卡片点了没反应**。
已改为 `status.modules || []`。

同批 UI 调整（用户指定）：`密钥状态` → `密钥列表`；密钥名与 44 个节点名的颜色改为按
配置状态着色（齐全 `#35d07f` 绿 / 缺配置白），删除原「生产红 / GitHub 灰」的来源着色
（`env-production`、`env-github` 已全库清零，来源信息保留在悬浮 title）。

临时诊断日志 `channelTrace` 已整段删除，仅保留 `consoleWarn` 用于分批失败告警（写 stderr）。

## 待执行

- [ ] 删除 `server.mjs` 中的 `channelTrace` 临时诊断日志并重建（[[no-remnants]]）。
      当前刻意保留，用于 Cloudflare 生产部署过程中的通道观测。
