# SCALE 编码原语跨端金标

状态：completed（2026-08-01；跨端金标、解码失败关闭和 CI 门禁全部完成）

## 缺陷

三端各有一份**手写** SCALE 实现，且没有一份被链端真值锁住：

| 端 | 实现 | 方向 | 现有覆盖 |
| --- | --- | --- | --- |
| Worker TS | `scaleCompact` / `scaleString` / `u64Le`（`src/shared/signing_message.ts`） | 编码 | **自证**：`device_subkey.test.ts` 用它们构造期望值 |
| citizenapp Dart | `scaleString` / `u64Le` / `_scaleCompact`（`lib/signer/signing.dart:219-254`） | 编码 | **自证**：`square_action_payload_test.dart` 用它们构造期望值 |
| citizenwallet Dart | `_decodeCompactU32`（`lib/signer/payload_decoder.dart`） | **解码** | **零测试引用** |

自证的含义：实现算错，期望值同步错，测试照样绿。

**这些字节决定被签 payload**。编码错 → 签出链端不认的交易；
解码错 → **冷钱包给用户展示的交易内容与实际要签的不符**，用户在错误信息下按下签名，
这比编码错危险一个量级。

手写 compact 的三档分支（`< 2^6` / `< 2^14` / `< 2^30`）边界是典型易错点。

## 已核实的跨端差异（向量设计必须避开）

`u64Le` 的上界两端不同：

- TS：`Number.isSafeInteger` 守卫 → 上界 `2^53-1`
- Dart：只拒负数，`int` 为 64 位有符号 → 上界 `2^63-1`

故向量上界取 **`2^53-1`**：超过它 TS 必抛错，向量若含更大值会让 TS 测试恒红。
该差异本身记录在此，实际字段（时间戳、revision）远小于 `2^53`，暂不构成问题。

## 方案

### 真源

`citizenchain/runtime/primitives/tests/` 新增 `scale_codec_golden.rs` + `fixtures/scale_codec_vectors.json`。

- 用 `codec::Encode`（parity-scale-codec，`primitives/Cargo.toml:11` 已有依赖）生成
- 复用既有 `sign_golden.rs` 的 `SIGN_GOLDEN_UPDATE=1` 重写机制，环境变量取名
  `SCALE_GOLDEN_UPDATE`；默认只断言不漂移

### 向量覆盖（重点在分支边界）

| 组 | 取值 | 理由 |
| --- | --- | --- |
| `compact_u32` | 0, 1, **63, 64**, 255, **16383, 16384**, 65535, **2^30-1** | 三档分支的两侧边界，每对跨越一次字节数跳变 |
| `scale_string` | 空串、1 字节、63 字节、64 字节、含中文、含 emoji | 长度前缀跨档 + UTF-8 字节数 ≠ 字符数 |
| `u64_le` | 0, 1, 255, 256, 2^32-1, 2^32, **2^53-1** | 小端字节序 + JS 安全整数边界 |

### 各端接入

| 端 | 方式 |
| --- | --- |
| Worker TS | **直读真源**（第 2 步已走通） |
| citizenwallet | **直读真源**；解码方向额外验 `decode(encode(x)) == x`，并覆盖 fail-closed 路径 |
| citizenapp | 直读或镜像 + 门禁，执行时按实际情况定 |

解码方向单独设计断言：不只「解码正确」，还要**拒绝非法输入**——截断的 compact、
声明长度超过剩余字节。这些是解码器的 fail-closed 路径，编码向量覆盖不到。

### CI

真源属 `citizenchain/**`、TS 属 `citizenapp/**`、wallet 属 `citizenwallet/**`，
三条流水线在上一张卡第 1 步都已挂门禁，**无需改 CI**。
仅当 citizenapp 走镜像时，才需在 `check-golden-vectors-sync.mjs` 的 `GROUPS` 加一组。

## 明确不做（已决策的边界）

前端安全边界（header 跨端锁 + 三档头部组装）已于第二轮落地，见上。
**仍然不做**的是余下两类，记录于此以免日后反复纠结覆盖率数字：

1. **135 个 `.tsx` 组件**：展示层，测它们收益递减
2. **WebAuthn 交互链**：`registerPasskey` / `assertPasskey` / `getPasskeyStatus`
   函数体内的 `navigator.credentials.create/get` 调用。要 mock 整套浏览器 API，
   测的是 mock 行为而非真实行为，收益低于维护成本

判据不变：真正的鉴权判据在后端，`require_admin_security_grant` 的档位判定已由
`operation_auth.rs` 内联测试锁住；前端被绕过后端仍 fail-closed。

## 执行结果（2026-08-01）

### 已完成

**真源**：`primitives/tests/scale_codec_golden.rs` + `fixtures/scale_codec_vectors.json`
（3 个测试）。`SCALE_GOLDEN_UPDATE=1` 生成，默认只比对。除向量表外另有两条
**独立于向量的规范推导锁**：compact 低 2 位模式标记与字节数对应、
字符串前缀必须是 utf8 **字节**数（中文/emoji 用例覆盖）。

**Worker TS**：`test/scale_codec.test.ts`，**26 用例**，直读真源。
含 fail-closed 断言：`scaleCompact` 拒负数/非整数/≥2^30，`u64Le` 拒负数/超安全整数。

**citizenapp Dart**：`test/signer/scale_codec_golden_test.dart`，**25 用例**，直读真源。
`_scaleCompact` 是 private，改用 `scaleString(长度为 N 的串)` 反推长度前缀，
把三档分支边界（0/1/63/64/255/16383/16384/65535）全部锁住。

**破坏性验证**：改 fixture 一个 hex 字节 → 真源测试红并指出 `compact_u32[2]`。

**回归**：primitives 全量绿；Worker **254 passed**；citizenapp **1095 passed / 0 failed**。

### citizenwallet 解码方向 ✅ 已完成（2026-08-01 第二轮）

**先纠正首轮判断**：首轮写的「入口不可达、成本高一个量级」是**错判**。
`payload_decoder_test.dart:199` 早有完整的 transfer 构造模板，`withSigningTail` /
`hexOf` / `u128LeForTest` 全是现成辅助函数，构造成本是**低**的。不该以此为由跳过。

新增 `citizenwallet/test/signer/scale_codec_golden_test.dart`，**11 用例**，直读真源。

**关键设计**：payload 的长度前缀**整段取自真源 hex**，本文件不做任何 compact 编码。
测的是「解码器能否解析链端真值」，而非「解码器与测试的编码实现是否自洽」。

| 组 | 内容 |
| --- | --- |
| 7 条真源向量 | 空串 / 1B / 63B / **64B（跨档）** / 中文 27B / emoji 17B / CID 26B |
| 3 条 fail-closed | 声明长度超剩余字节 → `null`；两字节档前缀被截断 → `null`；长度 0 但尾部非法 → `null` |
| 1 条守卫 | 真源可读且非空 |

fail-closed 三条是编码向量覆盖不到的解码器特有边界。

**破坏性验证**：把真源 64 字节向量前缀改成 1 字节档 → 红。
**回归**：citizenwallet 全量 **282 passed / 0 failed**。

### `compactVec` 隐患 ✅ 已修（2026-08-01 第二轮）

`payload_decoder_test.dart:21` 原实现内联 `[bytes.length << 2, ...bytes]`，**只覆盖 1 字节档**。

**实测错误形态**：64 字节时 `64 << 2 = 256`，产出一个值为 256 的「字节」，
转 `Uint8List` 时截断成 `0x00` —— 解码器读成「长度 0」，remark 整段丢失。

修法：将同文件的 `compactU32`（三档完整）前移到 `compactVec` 之前
（Dart 局部函数须先声明后使用），`compactVec` 改为复用它。

**106 个既有用例零变化全绿**——63 字节内两实现逐字节相同，不可能影响既有用例。

### onchina 前端 ✅ 已完成（2026-08-01 第二轮，推翻「明确不做」）

首轮结论是「明确不做」，用户第二轮要求执行，遂做。**范围仍严格限定在安全边界**，
135 个 `.tsx` 组件与 WebAuthn 交互链依旧不测（理由见下方「明确不做」）。

- 装 `vitest@^3.2.7`（与 Worker 同版本，仅此一个包，不装 jsdom / testing-library）
- 新增 `frontend/admins/securityHeaders.test.ts`，**5 用例**：
  - **跨端 header 锁**：直读 `src/auth/actions.rs`、`auth/passkey/mod.rs` 的 Rust 常量
    严格相等比对（照搬 `cloudflare/test/cross_end_contract.test.ts` 做法）
  - **三档头部组装**：Passkey 档不得带 grant 头（带了等于把本地写伪装成链上写）；
    ColdSign 档必须两头俱全；基础头不被覆盖
  - `assertPasskey` mock 掉（走 WebAuthn），但**常量取真实值**，否则跨端锁成自证
- `package.json` 加 `"test": "vitest run"`
- `citizenchain-ci.yml` 在「构建 OnChina 前端」后追加「测试 OnChina 前端」，复用已装依赖

**破坏性验证**：改后端 `PASSKEY_ASSERTION_HEADER` 常量 → 前端红。

**过程中打破又修好的回归**：`tsc -b` 会编译测试文件，而它用 `node:fs` /
`import.meta.dirname`，浏览器侧 tsconfig 无 Node 类型 → `npm run build` 报 3 个
TS2307/TS2339。修法：`tsconfig.json` 加 `exclude: ["**/*.test.ts", "**/*.test.tsx"]`
（测试由 vitest 自行转译）。**不跑 build 验证的话这个回归会直接进 CI。**

### header 大小写统一 ✅ 已完成（2026-08-01 第二轮）

发现前端 `PASSKEY_ASSERTION_HEADER = "X-Passkey-Assertion"`，后端为
`"x-passkey-assertion"`。HTTP header 名大小写不敏感（RFC 7230 §3.2），功能无碍，
但两端书写不一致。

**统一为小写，依据是硬的**：`onchina/src/core/http_security.rs:221` 用
`HeaderName::from_static("x-passkey-assertion")` 注册该头，**该 API 传入非小写直接 panic**。
小写是全仓唯一合法写法，不是风格偏好。

改前端一处常量后，两条跨端断言同步从「归一到小写比对」收紧为**严格相等**。
全仓搜索 `"X-Passkey-Assertion"` 零命中。

| header | 前端 | 后端 | 状态 |
| --- | --- | --- | --- |
| security grant | `'x-cid-security-grant'` | `"x-cid-security-grant"` | 严格相等锁住 |
| passkey assertion | `"x-passkey-assertion"` | `"x-passkey-assertion"` | 严格相等锁住 |

## 验收

- [x] 任一端手写 SCALE 编码实现被单方面改动，其金标测试必红
- [x] 破坏性验证：改真源边界值 → 真源测试红 / 各端测试红
- [x] 解码方向的 fail-closed 路径独立断言（3 条）
- [x] `compactVec` 隐患已修，106 个既有用例零变化
- [x] 前端安全边界测试接入 CI
- [x] header 名全仓统一并由严格相等断言锁住

## 最终测试规模

| 端 | 文件 | 用例 |
| --- | --- | --- |
| Rust 真源 | `primitives/tests/scale_codec_golden.rs` | 3 |
| Worker TS | `cloudflare/test/scale_codec.test.ts` | 26 |
| citizenapp | `test/signer/scale_codec_golden_test.dart` | 25 |
| citizenwallet | `test/signer/scale_codec_golden_test.dart` | 11 |
| onchina 前端 | `frontend/admins/securityHeaders.test.ts` | 5 |

全量回归：primitives 全绿 · Worker 254 · citizenapp 1095 · citizenwallet 282 ·
onchina 189 + 前端 5，均 0 failed。
