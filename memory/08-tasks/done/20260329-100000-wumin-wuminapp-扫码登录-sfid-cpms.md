# 任务卡：wumin + wuminapp 独立支持扫码登录 SFID/CPMS

## 任务信息
- 创建时间：2026-03-29
- 状态：已完成（代码编写完成，待真机测试）
- 优先级：高
- 涉及系统：wumin（冷钱包）、wuminapp（热钱包）、SFID 前端、CPMS 前端

## 需求背景
SFID 和 CPMS 管理员登录使用 `WUMIN_QR_V1` QR 码协议。当前 wuminapp 已有登录能力但需验证完整性，wumin 冷钱包完全不支持登录协议。两端需独立支持扫码登录，不依赖对方中转。

## 协议规格

### 登录 QR（SFID/CPMS 后端生成）
```json
{
  "proto": "WUMIN_QR_V1",
  "type": "challenge",
  "system": "sfid" | "cpms",
  "challenge": "<UUID>",
  "issued_at": <unix_ts>,
  "expires_at": <unix_ts>,
  "sys_pubkey": "0x<64hex>",
  "sys_sig": "0x<128hex>"
}
```

### 系统签名消息
```
WUMIN_QR_V1|<system>|<challenge>|<issued_at>|<expires_at>|<sys_pubkey>
```

### 用户签名消息
```
WUMIN_QR_V1|<system>|<challenge>|<expires_at>
```

### Receipt（签名结果）
```json
{
  "proto": "WUMIN_QR_V1",
  "type": "login_receipt",
  "system": "<system>",
  "challenge": "<challenge_id>",
  "pubkey": "0x<64hex>",
  "sig_alg": "sr25519",
  "signature": "0x<128hex>",
  "payload_hash": "<sha256_of_sign_message>",
  "signed_at": <unix_ts>
}
```

## 任务拆分

### T1: wumin 冷钱包 — 新增登录协议支持
- [ ] 新增 `LoginQrHandler`：解析 `WUMIN_QR_V1` challenge QR
- [ ] 验证系统签名（sr25519 验签 sys_sig）
- [ ] 显示登录确认界面（系统名、过期时间）
- [ ] 用户确认后用选定钱包签名
- [ ] 生成 receipt QR 显示在屏幕上
- [ ] QR 路由器增加 `WUMIN_QR_V1` 分发

### T2: wuminapp 热钱包 — 验证已有登录能力
- [ ] 确认 LoginService + QR 路由完整可用
- [ ] 确认热钱包直接签名 + 提交 receipt 到后端的链路
- [ ] 如有缺失或 bug 修复

### T3: SFID/CPMS 前端 — 支持扫描冷钱包 receipt QR
- [ ] 登录页增加"扫描签名回执"按钮/摄像头扫码
- [ ] 解析 receipt QR JSON
- [ ] 提交 receipt 到 `/api/v1/admin/auth/qr/complete` 完成登录

## 验收标准
1. wumin 冷钱包扫描 SFID 登录 QR → 签名 → 显示 receipt QR → SFID 前端扫描完成登录
2. wumin 冷钱包扫描 CPMS 登录 QR → 同上
3. wuminapp 热钱包扫描 SFID 登录 QR → 直接完成登录
4. wuminapp 热钱包扫描 CPMS 登录 QR → 同上
5. 过期 QR 码被拒绝
6. 非法系统签名被拒绝

## 文件索引
- 协议定义：`wumin/lib/qr/qr_protocols.dart`
- wumin 签名器：`wumin/lib/signer/qr_signer.dart`
- wuminapp 登录服务：`wuminapp/lib/qr/login/login_service.dart`
- wuminapp QR 路由：`wuminapp/lib/qr/qr_router.dart`
- SFID 登录后端：`sfid/backend/src/login/mod.rs`
- CPMS 登录后端：`cpms/backend/src/login/mod.rs`
- SFID 前端：`sfid/frontend/src/`
- CPMS 前端：`cpms/frontend/src/`（如果存在）
