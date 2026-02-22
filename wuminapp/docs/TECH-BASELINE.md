# wuminapp 技术方案基线（Flutter 版）

## 1. 文档目标
- 明确 `wuminapp` 轻节点软件的统一技术路线。
- 满足 iOS/Android 一套代码、长期迭代、区块链能力和端到端通信能力。
- 与 `citizenchain`、`fullnode`、`fcrcnode` 保持工程边界清晰。

## 2. 产品定位与边界
- 产品名称：`wuminapp`
- 目标用户：访客轻节点、公民轻节点
- 核心业务域：钱包、转账、SFID 绑定、公民投票、隐私通信、动态发布
- 非目标范围：全节点运维、委员会后台审批

## 3. 技术栈
- 移动端：`Flutter + Dart`（一套代码覆盖 iOS/Android）
- 链接入：Substrate API（推荐 `PAPI` / `Dedot` 的服务端适配层）
- 业务后端：`Rust`（HTTP/WebSocket）
- 通信：`Matrix`（E2EE）+ `WebRTC`（语音/视频）
- 存储：
  - 本地：`SQLite`/`Hive`（按模块选型）
  - 服务端：`PostgreSQL`
- 推送：APNs（iOS）+ FCM（Android）

## 4. 推荐目录规范
```text
wuminapp/
├── backend/
│   ├── src/
│   ├── tests/
│   └── migrations/
├── mobile/
│   ├── pubspec.yaml
│   ├── lib/
│   │   ├── main.dart
│   │   ├── pages/
│   │   ├── widgets/
│   │   ├── services/
│   │   └── utils/
│   ├── assets/
│   ├── test/
│   ├── ios/
│   └── android/
└── docs/
    ├── TECH-BASELINE.md
    ├── API-SPEC.md
    └── RELEASE.md
```

## 5. 架构分层
- UI 层：Flutter 页面与组件
- 应用层：状态管理、路由、用例编排
- 服务层：链交互、认证、消息、推送、文件
- 领域层：账户、交易、投票、社交模型
- 基础设施层：本地数据库、缓存、日志、网络

## 6. 区块链集成原则
- 链上：资产、认证结果、投票、关键审计锚点
- 链下：聊天消息、动态正文、媒体文件、推荐流
- 私钥不出端：签名默认在移动端完成
- 交易流程：本地构造与签名 -> 广播 -> 事件订阅 -> 状态回执

## 7. 通信与社交原则
- 私聊/群聊：Matrix E2EE（Olm/Megolm）
- 语音/视频：WebRTC，采用端到端媒体加密
- 动态发布：正文与媒体走链下存储，哈希与签名可选锚定上链
- 内容审核：服务端做策略过滤，不触碰用户私钥

## 8. 安全基线
- 助记词/私钥仅存系统安全区（iOS Keychain / Android Keystore）
- 敏感操作启用二次确认（生物识别或本地 PIN）
- SFID 绑定/投票凭证必须带 nonce，防重放
- 统一审计日志：时间、账户、公钥、操作、结果、trace_id

## 9. API 与错误规范
- API 前缀：`/api/v1`
- 响应结构：
  - 成功：`{ code: 0, message: "ok", data: ... }`
  - 失败：`{ code: <non-zero>, message: "...", trace_id: "..." }`
- 错误码建议：
  - `1xxx` 参数/校验
  - `2xxx` 身份/权限
  - `3xxx` 业务规则
  - `5xxx` 系统依赖

## 10. 工程质量基线
- Flutter：`dart format`、`flutter analyze`、`flutter test`
- Rust：`cargo fmt`、`cargo clippy`、`cargo test`
- 提交门禁：格式化 + 静态检查 + 单元测试

## 11. 发布与运维
- 平台：iOS、Android
- 制品：IPA、AAB/APK
- 环境：`dev` / `staging` / `prod`
- 最小配置：
  - `CHAIN_RPC_URL`
  - `MATRIX_HOMESERVER_URL`
  - `BACKEND_BASE_URL`
  - `PUSH_PROVIDER_CONFIG`
  - `LOG_LEVEL`

## 12. 分期落地建议
- M1：钱包、链上转账、SFID 绑定、交易记录
- M2：公民投票、消息通知、多账户
- M3：E2EE 聊天、语音视频、动态发布

## 13. 实施检查清单
- [ ] Flutter 工程骨架已初始化
- [ ] 链交互服务已完成抽象
- [ ] 私钥与签名安全策略已落地
- [ ] Matrix + WebRTC 链路已联调
- [ ] API 规范与错误码已统一
- [ ] iOS/Android CI 构建可用
