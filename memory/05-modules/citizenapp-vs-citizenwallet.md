# citizenapp vs citizenwallet 角色边界

- 创建日期:2026-04-09
- 来源:协议统一任务(`memory/08-tasks/open/20260409-qr-protocol-unify-v1.md`)审计
- 目的:明确两个独立 Flutter app 的职责,防止将来再出现"两份拷贝相互漂移"的字段散乱

## 定位

| | citizenapp | citizenwallet |
|---|---|---|
| 中文名 | 热钱包 | 冷钱包(软件形态的硬件钱包) |
| pubspec name | `citizenapp` | `citizenwallet` |
| 网络连接 | 连链(smoldot 轻节点) / 连 CID/CPMS 后端 | **完全离线** |
| 主题 | Light | Dark |
| 依赖关系 | 不依赖 citizenwallet | 不依赖 citizenapp |
| 代码共享 | **无** —— 两个独立 Flutter app |

**关键**:citizenapp 和 citizenwallet **没有任何 Dart 包依赖关系**。两者通过二维码对扫交互,协议一致性**只能**通过 `memory/01-architecture/qr/qr-protocol-spec.md` + `qr-protocol-fixtures/` 强制对齐。

## 职责划分(QR 协议角度)

| kind | citizenapp | citizenwallet |
|---|---|---|
| `sign_request`(接收) | ❌ 不处理 | ✅ 扫码,展示,签名 |
| `sign_response`(生成) | ❌ 不处理 | ✅ 签完生成,展示给笔记本摄像头 |
| `sign_request`(生成) | ✅ 热端构造交易,展示给冷钱包扫 | ❌ |
| `sign_request`(接收) | ❌ | ✅ 扫码,展示交易摘要 |
| `sign_response`(生成) | ❌ | ✅ 签完生成,展示给热端扫 |
| `sign_response`(接收) | ✅ 扫回,广播交易 | ❌ |
| `user_contact` | ✅ 生成+扫 | ❌ 不涉及用户码 |
| `user_transfer` | ✅ 生成+扫 | ❌ |
| `user_multisig` | ✅ 生成+扫 | ❌ |

**核心结论**:
- **登录**是 citizenwallet 公民钱包专属能力(CID/CPMS 后端只认冷钱包签的登录签名响应)
- **交易签名**是两端协作(热端发起 → 冷端签名 → 热端广播)
- **用户码/联系人/收款/多签**是 citizenapp 热钱包专属能力

2026-05-11 个人多签创建交易口径：

- citizenapp 生产 `PersonalManage(7).propose_create(0)` 时只使用
  `account_name / admins / regular_threshold / amount` 新载荷。
- citizenwallet 公民钱包只解析上述新载荷；缺少 `regular_threshold` 的旧个人多签创建载荷直接拒绝。
- `regular_threshold` 必须在 `floor(admins_len / 2) + 1 ..= admins_len` 范围内。

2026-05-15 管理员更换交易口径：

- citizenapp 生产 `AdminsChange(12).propose_admin_set_change(0)` 时必须使用
  `org / account_id / admins / new_threshold` 新载荷。
- citizenwallet 公民钱包只解析上述新载荷；缺少 `new_threshold` 或尾部有多余字节的旧/错载荷直接拒绝。
- 内置治理机构没有创建/注册提交；只有管理员更换提案会携带固定制度阈值，且 UI 不允许用户修改。
- 个人多签和机构账户的 `new_threshold` 必须严格过半且不超过新管理员数量。

## 实现约束

1. **citizenapp 禁止出现任何登录二维码生成代码**(`sign_request` / `sign_response`)。如果历史上有,按协议统一任务一并删除。
2. **citizenwallet 禁止出现任何用户码生成代码**(`user_*`)。如果历史上有,按协议统一任务一并删除。
3. **两端的 `QrEnvelope` / `QrKind` / `bodies/*.dart` / `signature_message.dart` 必须逐字节一致**。通过 golden fixture 测试强制对齐:两端测试都从 `memory/01-architecture/qr/qr-protocol-fixtures/` 读取同一批样本。
4. 扫到自己不处理的 kind:显示明确错误("此二维码需用 XX 钱包扫描"),不能静默忽略。

## 后端角色(便于查阅)

| 后端 | 生成 | 接收 |
|---|---|---|
| `citizencode/backend/admins/login/mod.rs` | `sign_request` | `sign_response` |
| `citizenpassport/backend/login/mod.rs` | `sign_request` | `sign_response` |

cid / cpms 前端只是扫码 UI 宿主:
- 笔记本浏览器显示 `sign_request` 二维码
- 手机 citizenwallet 扫码
- 手机 citizenwallet 展示 `sign_response` 二维码
- 笔记本摄像头反扫 `sign_response` → 前端调后端 API 验证

## 前端其他角色

| 前端 | 消费的 kind | 用途 |
|---|---|---|
| `citizenchain/node/frontend` | `user_contact` / `user_transfer` | 治理转账提案收款地址、手续费收款地址、安全基金提案收款地址 |
| `citizencode/frontend` | `user_contact` / `sign_response` | 管理员账户绑定(扫 citizenapp 用户码)、登录(显示签名请求给 citizenwallet 扫) |
| `citizenpassport/frontend`(登录部分) | `sign_response` | 登录(显示签名请求给 citizenwallet 扫) |

**注意**:CPMS 的 `CID_CPMS_V1 / INSTALL` 与 `ARCHIVE` 是**另一套完全独立的协议**,与 `QR_V1` 无关,永远不合并。相关代码位于:
- `citizenpassport/backend/initialize/mod.rs`
- `citizenpassport/backend/archive/mod.rs`
- `citizenpassport/frontend/initialize/`
- `citizenpassport/frontend/admins/`
- `citizenpassport/frontend/archive/`

这些目录在协议统一任务的零命中 grep 扫描中**被排除**。
