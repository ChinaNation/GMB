# wuminapp vs wumin 角色边界

- 创建日期:2026-04-09
- 来源:协议统一任务(`memory/08-tasks/open/20260409-qr-protocol-unify-v1.md`)审计
- 目的:明确两个独立 Flutter app 的职责,防止将来再出现"两份拷贝相互漂移"的字段散乱

## 定位

| | wuminapp | wumin |
|---|---|---|
| 中文名 | 热钱包 | 冷钱包(软件形态的硬件钱包) |
| pubspec name | `wuminapp_mobile` | `wumin` |
| 网络连接 | 连链(smoldot 轻节点) / 连 SFID/CPMS 后端 | **完全离线** |
| 主题 | Light | Dark |
| 依赖关系 | 不依赖 wumin | 不依赖 wuminapp |
| 代码共享 | **无** —— 两个独立 Flutter app |

**关键**:wuminapp 和 wumin **没有任何 Dart 包依赖关系**。两者通过二维码对扫交互,协议一致性**只能**通过 `memory/01-architecture/qr/qr-protocol-spec.md` + `qr-protocol-fixtures/` 强制对齐。

## 职责划分(QR 协议角度)

| kind | wuminapp | wumin |
|---|---|---|
| `login_challenge`(接收) | ❌ 不处理 | ✅ 扫码,展示,签名 |
| `login_receipt`(生成) | ❌ 不处理 | ✅ 签完生成,展示给笔记本摄像头 |
| `sign_request`(生成) | ✅ 热端构造交易,展示给冷钱包扫 | ❌ |
| `sign_request`(接收) | ❌ | ✅ 扫码,展示交易摘要 |
| `sign_response`(生成) | ❌ | ✅ 签完生成,展示给热端扫 |
| `sign_response`(接收) | ✅ 扫回,广播交易 | ❌ |
| `user_contact` | ✅ 生成+扫 | ❌ 不涉及用户码 |
| `user_transfer` | ✅ 生成+扫 | ❌ |
| `user_duoqian` | ✅ 生成+扫 | ❌ |

**核心结论**:
- **登录**是 wumin 公民钱包专属能力(SFID/CPMS 后端只认冷钱包签的登录回执)
- **交易签名**是两端协作(热端发起 → 冷端签名 → 热端广播)
- **用户码/联系人/收款/多签**是 wuminapp 热钱包专属能力

2026-05-11 个人多签创建交易口径：

- wuminapp 生产 `PersonalManage(7).propose_create(0)` 时只使用
  `account_name / duoqian_admins / regular_threshold / amount` 新载荷。
- wumin 公民钱包只解析上述新载荷；缺少 `regular_threshold` 的旧个人多签创建载荷直接拒绝。
- `regular_threshold` 必须在 `floor(admin_count / 2) + 1 ..= admin_count` 范围内。

2026-05-15 管理员更换交易口径：

- wuminapp 生产 `AdminsChange(12).propose_admin_set_change(0)` 时必须使用
  `org / account_id / new_admins / new_threshold` 新载荷。
- wumin 公民钱包只解析上述新载荷；缺少 `new_threshold` 或尾部有多余字节的旧/错载荷直接拒绝。
- 内置治理机构没有创建/注册提交；只有管理员更换提案会携带固定制度阈值，且 UI 不允许用户修改。
- 个人多签和机构账户的 `new_threshold` 必须严格过半且不超过新管理员数量。

## 实现约束

1. **wuminapp 禁止出现任何登录二维码生成代码**(`login_challenge` / `login_receipt`)。如果历史上有,按协议统一任务一并删除。
2. **wumin 禁止出现任何用户码生成代码**(`user_*`)。如果历史上有,按协议统一任务一并删除。
3. **两端的 `QrEnvelope` / `QrKind` / `bodies/*.dart` / `signature_message.dart` 必须逐字节一致**。通过 golden fixture 测试强制对齐:两端测试都从 `memory/01-architecture/qr/qr-protocol-fixtures/` 读取同一批样本。
4. 扫到自己不处理的 kind:显示明确错误("此二维码需用 XX 钱包扫描"),不能静默忽略。

## 后端角色(便于查阅)

| 后端 | 生成 | 接收 |
|---|---|---|
| `sfid/backend/admins/login/mod.rs` | `login_challenge` | `login_receipt` |
| `cpms/backend/login/mod.rs` | `login_challenge` | `login_receipt` |

sfid / cpms 前端只是扫码 UI 宿主:
- 笔记本浏览器显示 `login_challenge` 二维码
- 手机 wumin 扫码
- 手机 wumin 展示 `login_receipt` 二维码
- 笔记本摄像头反扫 `login_receipt` → 前端调后端 API 验证

## 前端其他角色

| 前端 | 消费的 kind | 用途 |
|---|---|---|
| `citizenchain/node/frontend` | `user_contact` / `user_transfer` | 治理转账提案收款地址、手续费收款地址、安全基金提案收款地址 |
| `sfid/frontend` | `user_contact` / `login_receipt` | 管理员账户绑定(扫 wuminapp 用户码)、登录(显示 challenge 给 wumin 扫) |
| `cpms/frontend`(登录部分) | `login_receipt` | 登录(显示 challenge 给 wumin 扫) |

**注意**:CPMS 的 `SFID_CPMS_V1 / INSTALL` 与 `ARCHIVE` 是**另一套完全独立的协议**,与 `WUMIN_QR_V1` 无关,永远不合并。相关代码位于:
- `cpms/backend/initialize/mod.rs`
- `cpms/backend/dangan/mod.rs`
- `cpms/frontend/initialize/`
- `cpms/frontend/super_admin/`
- `cpms/frontend/operator_admin/`

这些目录在协议统一任务的零命中 grep 扫描中**被排除**。
