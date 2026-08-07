# citizenapp vs citizenwallet 角色边界

- 创建日期:2026-04-09
- 来源:历史协议统一任务审计
- 目的:明确两个独立 Flutter app 的职责,防止将来再出现"两份拷贝相互漂移"的字段散乱

## 定位

| | citizenapp | citizenwallet |
|---|---|---|
| 中文名 | 热钱包 | 冷钱包(软件形态的硬件钱包) |
| pubspec name | `citizenapp` | `citizenwallet` |
| 网络连接 | 连链(smoldot 轻节点) / 连 OnChina 后端 | **完全离线** |
| iOS 最低版本 | 16.0 | 16.0 |
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
| `user_contact`(用户码) | ✅ 身份账户生成(用户主页)+扫描并核验 CID | ✅ 只解析，不生成 |
| `user_transfer`(收款码) | ✅ 生成+扫(生成方待实现) | ❌ 既不生成也不解析 |
| `account_id_code`(账户码) | ✅ 账户详情生成+扫 | ✅ 账户详情生成 |
| `user_multisig` | ✅ 生成+扫 | ❌ |

**核心结论**:
- **登录**由两端分工，不是任一端的专属能力：第 1 步由 citizenwallet 出示 `k=5` 账户码提供
  目标账户，第 2 步由同一 citizenwallet 冷签 `k=2` 登录响应。OnChina 后端只认冷钱包签的
  登录签名响应，且登录全程不需要 citizenapp 参与
  （2026-07-29 更正：旧表述「登录是 citizenwallet 专属能力」与当时「第 1 步必须扫
  citizenapp 出的 `k=3`」自相矛盾，实际导致管理员拿冷钱包的收款码去扫而报错）
- **交易签名**是两端协作(热端发起 → 冷端签名 → 热端广播)
- **用户码/联系人关系/多签业务**由 CitizenApp 持有链上与业务真源；CitizenWallet 只解析
  用户码，并为自己的每个账户生成固定账户码（无身份声明、无时效）

2026-06-26 个人多签创建交易口径：

- citizenapp 生产 `PersonalManage(7).propose_create(0)` 时只使用
  `account_name / admins / regular_threshold / amount` 新载荷。
- citizenwallet 公民钱包只解析上述新载荷；缺少 `regular_threshold` 的旧个人多签创建载荷直接拒绝。
- `regular_threshold` 必须在 `floor(admins_len / 2) + 1 ..= admins_len` 范围内。

2026-06-26 管理员更换交易口径：

- citizenapp 只为个人多签生产 `PersonalAdmins(29.0)` 管理员更换交易；公权/私权机构管理员由机构治理和注册局登记流程维护，不得复用个人多签入口。
- 个人多签管理员更换载荷固定为 `institution_code / account_id / admins / new_threshold`，其中每个管理员按 `account_id + family_name + given_name` 编码。
- citizenwallet 公民钱包只解析上述新载荷；缺少 `new_threshold` 或尾部有多余字节的旧/错载荷直接拒绝。
- 个人多签人数至少为 2，动态阈值必须严格过半；冷钱包必须校验 `institution_code=PMUL` 与 `PersonalAdmins(29.0)` 完全匹配。
- 同一次更换只允许一次最终交易签名；姓名确认不形成第二次签名。

## 实现约束

1. **citizenapp 禁止出现任何登录二维码生成代码**(`sign_request` / `sign_response`)。如果历史上有,按协议统一任务一并删除。
2. **CitizenWallet 禁止生成 `k=3 user_contact`**。离线账户没有 CID 真源，只允许解析
   `k=3`，账户详情生成固定 `k=5 account_id_code` 账户码。**禁止在离线端生成任何带 `i/e`
   的码**：冷钱包完全离线、无 NTP，本机时钟漂移或被改写都无从纠正，签发带绝对时间戳的
   凭证本身不成立。CitizenWallet 也不解析 `k=4` 收款码（离线发不了交易，扫它无用途）。
3. **两端的 `QrEnvelope` / `QrKind` / `bodies/*.dart` / `signature_message.dart` 必须逐字节一致**。通过 golden fixture 测试强制对齐:两端测试都从 `memory/01-architecture/qr/qr-protocol-fixtures/` 读取同一批样本。
4. 扫到自己不处理的 kind:显示明确错误("此二维码需用 XX 钱包扫描"),不能静默忽略。

## 后端角色(便于查阅)

| 后端 | 生成 | 接收 |
|---|---|---|
| `citizenchain/onchina/src/admins/login/mod.rs` | `sign_request` | `sign_response` |

OnChina 前端只是扫码 UI 宿主:
- 笔记本浏览器显示 `sign_request` 二维码
- 手机 citizenwallet 扫码
- 手机 citizenwallet 展示 `sign_response` 二维码
- 笔记本摄像头反扫 `sign_response` → 前端调后端 API 验证

## 前端其他角色

| 前端 | 消费的 kind | 用途 |
|---|---|---|
| `citizenchain/node/frontend` | `user_contact` / `user_transfer` | 治理转账提案收款地址、手续费收款地址、安全基金提案收款地址 |
| `citizenchain/onchina/frontend` | `user_contact` / `sign_response` | 管理员账户绑定(扫 citizenapp 用户码)、登录(显示签名请求给 citizenwallet 扫) |
