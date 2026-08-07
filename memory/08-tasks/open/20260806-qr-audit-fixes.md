# 任务卡：二维码/扫码/签名 全仓审计整改

状态：第 1–9 项已完成(2026-08-06),四端回归全绿;第 10 项已合流到
`20260729-qr-three-code-classification`,本卡随该卡一并关闭。

## 来源

四个专项审计(协议解析安全、签名链路安全、扫码入口一致性、跨端协议对齐)的发现,
全部经逐条读码复核属实。**零 CRITICAL**;问题集中在一致性与边界校验。

审计确认全仓有 **7 个端**参与 QR/签名协议,而非此前认为的 5 个:
citizenapp、citizenwallet、onchina Rust、qr-protocol crate、Worker TS、
**node/frontend TS**、**onchina/frontend TS**。

## 整改清单(按优先级)

1. **修两份 TS 并合并为单一共享实现** —— `20260806-qr-single-letter-keys` 遗漏,
   桌面矿工端「扫码识别账户」链路已断(读 `b.account_id` 得 undefined)。
2. **守卫扫描根补 `node/frontend`、`onchina/frontend`** —— 根因:守卫看不见这两处,
   给出"全绿"假象。
3. **golden fixtures 补跨端测试** —— 全仓无任何测试加载 fixtures,金标从未跑过。
4. **修「扫码添加管理员」** —— `personal_account_create_page.dart:133` 访问
   `(env.body as dynamic).address`,该字段**从来不存在**(改动前是 `ss58Address`),
   `as dynamic` 绕过类型检查;合法码 → NoSuchMethodError → 被 catch 吞成
   "请扫描有效的用户二维码"。功能 100% 失效,非本次协议改动引入。
5. **k=1/k=2 补精确键校验** —— 热端 Dart 两个 body 无 `_requireExactKeys`,
   onchina `CompactResponseBody` 无 `deny_unknown_fields`;缺口精确落在权限最高的
   两种码型,且 Dart/Rust 两套实现各自重现(系统性遗漏)。校验函数现有 4 份手写副本。
6. **`fromWire` 改严格 int**(热端接受 `"1"`/`" 1"`/`"+1"`/`"0x1"`,冷端与 Rust 拒绝);
   **base64url 统一到冷端严格版**(热端 `base64Url.decode("++++")` 成功,Rust 拒绝)。
7. **冷端补 `squareAccountAction` 显式拒绝** —— 热端有,冷端落到 `return payload`
   原始字节直签;现由"解码器恰好不认识"隐式挡住,非设计。
8. **`c`/`bank` 补字符集白名单** —— 零宽字符可绕过 trim 校验(实测
   `"​CN...​".trim()` 与原串相同);`account_id` 因锚定正则天然免疫。
9. **onchina 治理签名迁进 `signing_message`** —— 现走 JSON 文本直签,无 `GMB` 前缀、
   无 `op_tag`、不进 `SIGN_OP_TAGS`;且验签同时接受裸文本与 `<Bytes>` 包裹两种编码。
10. k=4 生成方落地时同批补 `e`/`i` 校验(与 `20260729-qr-three-code-classification` 合流)。

## 验收

- [x] 七端协议字段一致;桌面端扫账户码链路恢复
      —— 两份 `citizenQr.ts` 收敛为同一份规范实现(逐字节相同),七端单字母键统一。
- [x] 守卫能扫到全部七端,故意注入旧字段可被守卫捕获
      —— 扫描根补 `node/frontend`+`onchina/frontend`;新增
      `web_frontend_qr_modules_are_byte_identical`。**已实证**:向一份注入一行注释 → 守卫 FAILED,
      还原 → 2/2 绿(不是"看起来会拦")。
- [x] fixtures 有测试真实加载并跨端校验
      —— 新增 `golden_fixtures.rs` 6 个测试:五码齐备、信封形状、精确单字母键集、
      旧长字段不得复活、`account_id` 规范小写 hex、`sign_request` 动作码必须在册。
- [x] 五种码 body 全部有精确键校验(含 k=1/k=2、含 Rust 侧)
- [x] 三端 `k` 解析与 base64url 严格度一致
      —— 热端 `fromWire` 改严格 `int`(原先吞 `"1"`/`" 1"`/`"+1"`/`"0x1"`);
      base64url 统一到冷端严格版(拒 `=` 填充、拒 `+`/`/`、回环校验)。
- [x] 文档更新、注释完善、测试补齐、残留清理

## 回归结果(2026-08-06)

| 端 | 结果 |
|---|---|
| `cargo test -p primitives` | 5 个测试目标全绿 |
| `cargo test -p onchina` | 194 passed / 0 failed |
| `cargo test -p qr-protocol` | 9 passed(含 repo_guard 2 + golden_fixtures 6) |
| `citizenwallet flutter test` | 291 passed / 0 failed |
| `citizenapp flutter test` | 1152 passed / 0 failed(修好下述既有红测试后) |

**顺手修掉的既有红测试**(主分支已提交状态就是红的,与本次审计无关):
`citizenapp/test/signer/citizen_identity_sign_service_test.dart` 的 `_validVotingPayload()`
只造内层 `VotingIdentityPayload`,而解码器早已要求防重放三件套
`genesis_hash(32) ++ payload ++ version(8) ++ expires_at(8)`。已按写入端
`onchina/src/domains/citizens/chain_identity.rs::build_citizen_identity_authorization_bytes`
逐字节重写夹具,并补一条「裸载荷必须拒签」用例 + 三件套字段断言 —— 否则夹具再漂移一次
仍然测不出来。

## 剩余

- 第 10 项(k=4 收款码 `e`/`i` 校验)按原定与 `20260729-qr-three-code-classification` 合流,
  待生成方落地时同批补。
