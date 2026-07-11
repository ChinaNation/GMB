# CitizenApp 聊天前台实时收消息修复（免手动刷新）

## 任务需求

- 双方在聊天窗口时，新消息应**自动即时显示**，不需要手动点同步/刷新。
- 现象根因（已定位）：
  1. **服务端推送静默失败**：`ChatRealtimeObject` 只 `implements DurableObject`（接口），未 `extends` `cloudflare:workers` 的 DurableObject RPC 基类；WS 挂载走 `getByName(owner).fetch()`（可用），但推送走 `stub.notify()` 自定义方法 RPC（不可用，抛错），且被 `notifyChatRealtime(...).catch(() => 0)`（`src/chat/service.ts`）静默吞掉 → 收件方"有新消息"通知从未发出。
  2. **客户端兜底被抑制**：`connectRealtime` 只要 WS 握手成功就返回"已连接"，聊天页据此 `_stopPolling()` 关掉 8s 兜底轮询（`lib/chat/chat_page.dart`）→ 推送不来时既无推送也无轮询。

## 建议模块

- Worker 实时：`citizenapp/cloudflare/src/chat/realtime.ts`、`src/chat/service.ts`
- 客户端聊天页：`citizenapp/lib/chat/chat_page.dart`

## 影响范围

- Worker：`notify` 由自定义方法 RPC 改为经 `stub.fetch()` 内部请求投递（与 WS 挂载同一条 `.fetch()` 路，最兼容）；`.catch` 保留但加可观测日志，不再纯静默。
- 客户端：实时连上后保留低频心跳兜底轮询（~20s），推送偶发丢失也能兜底；WS 只当通知触发器。
- 需重新部署 production Worker + 客户端小改；**链端 0 改**。
- 修 Worker 推送对聊天页和聊天 Tab 列表**同时生效**（两者共用同一 notify）。

## 主要风险点

- `getByName(...).fetch()` 已被 WS 挂载证明可用，方案风险低；不采用 `extends RPC 基类` 方案以避免 runtime/wrangler 版本不确定性。
- 心跳轮询频率需平衡实时性与电量/流量：聊天页前台 ~20s，后台由生命周期观察者暂停。
- 静默 `.catch(()=>0)` 是此类问题温床（对齐仓库 silent-failure 教训），修后失败要能观测。

## 是否需要先沟通

- 否。用户已确认先做 A。

## 预计修改目录

- `citizenapp/cloudflare/src/chat/`：修 DO 推送投递路径 + 收窄静默吞错；代码。
- `citizenapp/lib/chat/`：聊天页心跳兜底轮询；代码。
- `memory/05-modules/citizenapp/chat/`、`memory/01-architecture/citizenapp/`：补实时投递说明；文档。

## 分步骤技术方案

### 步骤 1：Worker DO 推送改经 fetch
- `ChatRealtimeObject.fetch()` 增加内部路径 `POST /__notify`：解析 `ChatNoticePayload`，调用抽出的 `deliver()`（原 `notify` 逻辑），返回 `{ ok, sent }`。
- `notifyChatRealtime` 由 `stub.notify(payload)` 改为 `stub.fetch("https://chat-realtime.internal/__notify", { method:"POST", body: JSON })`，读回 `sent`。
- `service.ts` envelope POST 的 `.catch(() => 0)` 改为记录 `console.warn` 后返回 0（不阻断发送，但可观测）。

### 步骤 2：客户端心跳兜底
- `chat_page.dart` 新增 `_heartbeatPollInterval = 20s`。
- 实时连上后不再 `_stopPolling()`，改为调度心跳轮询；`_runPoll` 移除"实时连上就 return"的抑制，改为实时在线时按心跳间隔续跑、离线时常规/退避并尝试重连。

### 步骤 3：部署与验收
- Worker：`npm --prefix citizenapp/cloudflare run typecheck` + `test`；`wrangler deploy --env production`。
- 客户端：`dart analyze` + `flutter test` 覆盖 im。
- 真机：两台都在聊天窗口，一方发送 → 另一方**不刷新自动显示**；离开再进也同步。

## 当前执行状态

- [x] 步骤 1：`ChatRealtimeObject.fetch()` 增 `POST /__notify` 分支，`notify` 逻辑抽成私有 `deliver()`；`notifyChatRealtime` 改为构造 `Request` 经 `stub.fetch()` 投递并读回 `sent`；删除 `ChatRealtimeStub` 类型。`service.ts` 静默 `.catch(()=>0)` 改为 `console.warn` 后返回 0。
- [x] 步骤 2：`chat_page.dart` 新增 `_heartbeatPollInterval=20s`；实时连上后改为调度心跳而非 `_stopPolling()`；`_runPoll` 移除"实时连上即 return"抑制，改为实时在线按心跳续跑、离线常规/退避并重连。
- [x] 步骤 3：Worker `typecheck` + 15 测试全过；`wrangler deploy --env production` 已部署（版本 `8986eb53`）。客户端 `dart analyze` 干净；`flutter test test/chat` 38 过 4 skip（native 库 CI 跳过）。
- [ ] 待用户真机验收：两台都在聊天窗口，一方发送 → 另一方不刷新自动显示。
