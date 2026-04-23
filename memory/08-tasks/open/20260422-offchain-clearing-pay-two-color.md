# 任务卡:offchain_clearing_pay 两色识别接入

- 时间:2026-04-22(跟进项)
- 状态:open
- 归属:Mobile Agent(wumin + wuminapp)
- 承接:`20260422-cold-wallet-two-color-recognition.md`(主任务卡 PR-C)

## 背景

PR-C 删除 `allowedHashedActions` 白名单后,`wuminapp/lib/trade/offchain/
offchain_clearing_pay_page.dart` 的 `action: 'offchain_clearing_pay'`
sign_request 路径失去黄色兜底,当前状态:

- `payload_hex` 是 32 字节 `signing_hash`(NodePaymentIntent 的 blake2_256)
- decoder 无对应分支 → 解码失败 → 冷钱包 🔴 红色拒签
- 原注释明言"冷钱包对 payload_hex **盲签**(只把 32 字节当字节流处理)"

## 设计分歧点

**本质问题**:冷钱包收到 32 字节哈希时无法独立还原交易内容。两色识别
模型下没有"盲签绿色"空间 — 要么走 🟢,要么 🔴 拒。

## 三条可选路

### A. 扩展 payload,让冷钱包能独立解析 NodePaymentIntent SCALE
- 热钱包改为把 `NodePaymentIntent` 完整 SCALE 编码填 `payload_hex`
- decoder 加分支解出 `to / amount / fee / bank_shenfen`
- display.fields key 对齐 Registry → 🟢
- 代价:QR 变大(NodePaymentIntent ≈ 150 字节 vs 当前 32 字节)

### B. envelope 层增加 `offchain_clearing_pay` body
- 类似 `login_challenge`,在 envelope 层用专有 body 承载结构化字段
  (amount/fee/to/bank)
- 签名原文仍然是 signing_hash(保持与链下账本的 hash 绑定一致)
- 冷钱包按 body 字段展示 + 按 signing_hash 签名
- Registry 在 envelope 层加 `offchain_clearing_pay` kind(非 sign_request)
- 代价:扩展 WUMIN_QR_V1 协议,需要更新 `qr-protocol-spec.md`

### C. 改走 sfid 后端 ShengSigningPubkey 直签
- 清算行收到 QR 后把 signing_hash 送给 sfid 后端
- sfid 后端用省级密钥签名
- 冷钱包不参与 offchain_clearing_pay
- 代价:要求用户手机先登录 sfid 系统

## 推荐

先走 **方案 B**(envelope 层新 kind):最符合 WUMIN_QR_V1 的现有分层,
用户体验最好(冷钱包能完整展示金额/收款方),代价是协议扩展。

## 当前态暂存

PR-C 合并后,`offchain_clearing_pay_page.dart` 的 SignDisplayField 仍是
无 key 形态(PR-C 范围不含此文件)。冷钱包扫这个 QR 会 🔴 红。
清算行相关业务当前停用,直到本任务落地。

## 验收

- wumin decoder 有 `offchain_clearing_pay` 合法分支(按选定方案)
- wumin 扫 offchain_clearing_pay QR → 🟢 绿色,用户看清金额/收款方
- wuminapp `offchain_clearing_pay_page.dart` SignDisplayField 补 key 或改
  envelope body 按选定方案
- 手工构造垃圾 32 字节 payload(非本 runtime)→ 🔴 红
- Grep 无残留

## 关联

- 主任务卡:`20260422-cold-wallet-two-color-recognition.md`
- 方案文档:`memory/05-architecture/qr-signing-recognition.md`
- Registry:`memory/05-architecture/qr-action-registry.md`
- 协议:`memory/05-architecture/qr-protocol-spec.md`
