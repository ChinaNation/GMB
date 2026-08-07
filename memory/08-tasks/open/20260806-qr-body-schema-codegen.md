# 任务卡：QR body 声明层收进 qr-protocol 并三语言生成

状态：open（2026-08-06 立卡，**方案已定、未开工**）
触发点：与 `20260729-qr-three-code-classification` 剩余待办第 3 条（k=4 补 `i`/`e` 校验）**同批执行**。
不单独提前做，理由见「为什么等到 k=4 那次」。

## 来源

`20260806-qr-audit-fixes` 整改完成后复盘：全仓 QR 一致性目前靠**三种不同机制**混合，
只有第一类真正收进了 `citizenchain/crates`。

| 类别 | 真源位置 | 一致性靠什么 |
|---|---|---|
| 动作码 / 中文动作名 / 字段名 / 拒绝原因 | `crates/qr-protocol/registry/*.yaml` | **生成物** → 两端 `qr/generated/qr_action_registry.g.dart` |
| 签名 op_tag + `signing_message` | `runtime/primitives/src/sign.rs`（不在 crates，且**不应**挪：链上 pallet 要用它验签） | **金标向量** `signing_domain_vectors.json` |
| envelope / body 编解码 | **无真源**，五端各写一份 | **守卫测试**（只能保证两份 TS 一样，管不了 Dart×2 + Rust×1） |

本卡只处理第三类。

## 为什么值得做（审计实据，非理论）

上一轮 10 项发现里，**第 5、6、8 项完全落在这一类**，第 1 项是同一病根的另一种形态：

- 第 5 项：`_requireExactKeys` 全仓 **4 份手写副本**，缺口恰好落在权限最高的 k=1/k=2
- 第 6 项：`fromWire` 热端吞 `"1"`/`" 1"`/`"+1"`/`"0x1"`，冷端与 Rust 拒；
  base64url 热端收 `"++++"`，Rust 拒
- 第 8 项：`c`/`bank` 字符集白名单三端有的有、有的没有
- 第 1 项：两份 `citizenQr.ts` 各自演化，桌面矿工端扫账户码链路整条断掉
- 另：`isAccountIdText` 已有 **2 份手写 Dart 副本**
  （`citizenapp/lib/citizen/shared/account_derivation.dart:92`、
  `citizenwallet/lib/util/account_id_text.dart:16`）

全部是「A 端有这道闸、B 端没有」，且全是人工纪律漏掉的、不是设计分歧 ——
**这类缺陷 codegen 能从结构上消灭，review 消灭不了**（证据：它们活到了那次审计才被发现）。

且这些 body **本来就已经是纯声明式**：`citizenapp/lib/qr/bodies/user_transfer_body.dart`
97 行里 90 行是「键集 + 每键类型 + 每键格式」，零业务逻辑。手写五遍是纯浪费。

## 现状体量（2026-08-06 实测）

| 文件 | 行数 |
|---|---:|
| `citizenapp/lib/qr/envelope.dart` | 161 |
| `citizenapp/lib/qr/qr_protocols.dart` | 187 |
| `citizenapp/lib/qr/bodies/*.dart`（5 个） | 565 |
| `citizenwallet/lib/qr/envelope.dart` | 139 |
| `citizenwallet/lib/qr/qr_protocols.dart` | 227 |
| `citizenwallet/lib/qr/bodies/*.dart`（4 个） | 421 |
| `citizenchain/onchina/src/core/qr/mod.rs` | 562 |
| `citizenchain/node/frontend/shared/qr/citizenQr.ts` | 374 |
| `citizenchain/crates/qr-protocol/src/codec.rs` | 127 |
| **合计** | **2763** |

其中可生成的声明层估 **1200–1400 行**；emitter 本身估 **600–900 行 Rust**。
现有 `crates/qr-protocol/src/export.rs` 只有 167 行且只吐 Dart 常量表，emitter 是净新增。

## 执行方案（四步）

### 第一步：schema 落地

新增 `citizenchain/crates/qr-protocol/registry/kinds.yaml`，与现有
`actions.yaml`(1328) / `fields.yaml`(212) / `reject_reasons.yaml`(30) 并列。
描述 5 个码型的 `k` 值、临时/固定、body 键序、每键约束。

**关键设计约束：约束用有限词汇表，不是通用表达式语言。**
能写进 YAML 的必须在三种语言都生成出**语义完全相同**的代码。当前实际只用到 8 种：

```
account_id          # 0x + 64 位小写 hex
cid                 # ^[A-Za-z0-9-]{1,32}$
b64u_bytes{min,max} # 严格无填充 base64url + 重编码回环
text{min_len}
text_optional
u16_positive
enum_int[…]         # 如 g 只允许 1
action_conditional  # 引用 actions.yaml 谓词
```

`action_conditional` 用于表达 `sign_request_body.dart:67` 的
`QrActions.isSelfAccountDomainAction(action)` 分支（占号/换绑时 `u` 必须留空，
其余动作 `u` 必须解码为 32 字节）—— 该谓词**已在 `actions.yaml` 里**，故此分支同样可生成。

**逃生口**：遇到第 9 种约束，YAML 标 `hand_written: true` 显式豁免，
**禁止**为了塞进词汇表而扭曲语义。这是本方案最大风险的唯一缓解手段。

### 第二步：emitter

`export.rs` 扩三个 emitter：`emit_dart_bodies` / `emit_rust_bodies` / `emit_ts_bodies`。
每种语言配一份**手写 prelude**（`b64url` / `hex` / `exact_keys` 三个原语，各约 50 行），
prelude 只写一次、被全部生成物共用。落点：

```
citizenapp/lib/qr/generated/qr_bodies.g.dart
citizenwallet/lib/qr/generated/qr_bodies.g.dart
citizenchain/onchina/src/core/qr/generated.rs
citizenchain/node/frontend/shared/qr/generated/qrBodies.g.ts   ← onchina/frontend 用同一份
```

### 第三步：切换 + 守卫升级

五份手写 body 改为消费生成物，只保留端特有部分（citizenapp 的 `payloadHex` getter、
钱包账户定位、UI）。`repo_guard.rs` 加两条：

1. 手写文件里不得再出现 `_requireExactKeys` 的**定义**（只能 import 生成物）——
   与现有「禁止恢复手写 actionLabels 表」同范式。
2. CI 跑 `export_registry --check`：YAML 重新生成后必须与仓库内容**逐字节相同**。

### 第四步：三语言等价性测试

同一批 `memory/01-architecture/qr/qr-protocol-fixtures/*.json` 喂三种语言的解析器，
断言**接受集与拒绝集完全相同**。负样例必须含：`=` 填充、`+`/`/` 标准字母表、
零宽字符 CID、多一个键、少一个键、`k` 为字符串。

现状 `golden_fixtures.rs` 只跑 Rust 一侧 —— 这一步才把「跨端一致」变成可回归的断言，
而不是「三个端各自跑自己的测试、恰好都过」。

## 明确不收

- **签名字节计算**：属 `primitives::sign` 金标域，另一套机制，不得混入本卡。
- **`onchina/src/core/qr/mod.rs` 的 562 行**：绝大部分是**构造**二维码 + 业务权限判断，
  不是解析，生成不了也不该生成。本卡只动其中的解析部分。
- **UI、钱包/账户定位、生物识别**：端特有，永远手写。

## 为什么等到 k=4 那次

`20260806-qr-audit-fixes` 刚整改完，五端**此刻是对齐的**；codegen 的收益兑现在
「下一次改 body 字段」那一刻。第 10 项落地时正好要三端同改 `user_transfer_body`
补 `i`/`e` 校验 —— 改动与收益同批发生。现在单独做，等于在刚验证过一致的系统上动大手术，
收益空等到下次改动才兑现。

## 验收

- [ ] `kinds.yaml` 覆盖 5 个码型全部键与约束；未进词汇表的显式标 `hand_written`
- [ ] 三语言生成物落地，五份手写 body 只剩端特有部分
- [ ] `export_registry --check` 进 CI，改 YAML 不重新生成即红
- [ ] 守卫能拦住手写 `_requireExactKeys` 定义复活（**须实证注入一次确认变红**，
      不接受「看起来会拦」——上一轮守卫就是因为扫描根缺两个目录而报了假绿）
- [ ] 三语言等价性测试跑通，负样例六类全覆盖
- [ ] 文档更新（`qr-protocol-spec.md` 标注 body 校验已生成化）、注释完善、残留清理
