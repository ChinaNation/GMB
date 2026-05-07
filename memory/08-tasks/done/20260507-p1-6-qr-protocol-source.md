# 任务卡:P1-6 QR 协议真源归位与过期内容清理

## 任务需求

执行重新创世前总审计 P1-6：把 QR 扫码协议、扫码签名识别方案、action registry 与 golden fixture 从非标准 `memory/05-architecture/` 归位到架构目录，并同步清理已经过期的 pallet / call / action 登记。

## 背景

- 统一协议入口已经固定为 `memory/07-ai/unified-protocols.md`。
- 扫码协议只有一个：`WUMIN_QR_V1`。
- 当前旧非标准 QR 架构目录下的文档自称唯一事实源，但该目录不在 `repo-map.md` 的当前主结构中。
- `qr-action-registry.md` 仍包含旧 `VotingEngine(9)` 投票入口、旧 `DuoqianManage` 名称和已删除的 execute wrapper。
- `qr-signing-recognition.md` 仍要求 `supportedSpecVersions`，但冷钱包已改为 strict decoder 两色识别，不再依赖 spec_version 集合门控。

## 预计修改目录

| 目录 | 用途、边界和修改类型 |
|---|---|
| `memory/01-architecture/qr/` | QR 协议架构真源目录；承接 spec、识别方案、action registry 和 fixture；涉及文档移动与协议内容修正。 |
| `memory/05-architecture/` | 仅移出 QR 相关文档与 fixture；不处理目录内其他历史架构文档；涉及残留清理。 |
| `memory/07-ai/` | 更新统一协议文件、统一命名文件中的 QR 真源路径和目录登记；涉及 AI 系统规则文档。 |
| `memory/08-tasks/open/` | 更新本任务卡和前置审计记录；涉及任务记录，不涉及代码。 |
| `wumin/lib/qr/` | 删除冷钱包侧已下线 `user_duoqian` kind 残留；涉及 QR 协议代码。 |
| `wumin/lib/signer/` | 对齐冷钱包 action label 与 decoder 输出；涉及签名识别代码。 |
| `wumin/test/signer/` | 补关闭机构/个人多签 action 的 decoder 回归测试；涉及测试代码。 |
| `wuminapp/lib/duoqian/` | 同步关闭机构/个人多签 QR action 注释，避免继续引用旧 `propose_close` Registry 表述；涉及代码注释。 |
| `citizenchain/node/frontend/shared/qr/` | 同步 QR TS 类型、解析器和收款码解析边界；涉及前端协议代码。 |
| `sfid/` | 同步后端/前端 QR 协议类型中的当前 6 kind 与新真源路径；涉及协议代码。 |
| `cpms/` | 同步后端/前端 QR 协议类型中的当前 6 kind 与新真源路径；涉及协议代码。 |

## 执行清单

- [x] 新建 `memory/01-architecture/qr/`。
- [x] 移动 QR spec、签名识别方案、action registry 和 fixture 到新目录。
- [x] 更新统一协议文件中的 QR 真源路径。
- [x] 更新统一命名文件中的 QR 目录登记。
- [x] 清理 action registry 中旧 `VotingEngine(9)` 投票入口、旧 `DuoqianManage` 和已删除 execute wrapper。
- [x] 删除签名识别方案中的 `supportedSpecVersions` 要求。
- [x] 删除 wumin 冷钱包侧 `user_duoqian` kind 残留。
- [x] 删除 node / sfid / cpms QR 协议类型中的 `user_duoqian` kind 残留。
- [x] 对齐 `propose_close_institution` / `propose_close_personal` 与冷钱包 decoder 输出。
- [x] 扫描旧路径、旧 action、旧 pallet 名和旧识别规则残留。
- [x] 更新总审计记录。

## 执行结果

- QR 文档与 fixture 已迁到 `memory/01-architecture/qr/`。
- `memory/07-ai/unified-protocols.md` 与 `memory/07-ai/unified-naming.md` 已登记新目录和新路径。
- `qr-action-registry.md` 已改为当前 pallet/call/action 表，旧 wrapper action 只保留在“不得恢复”清单。
- `qr-signing-recognition.md` 已改为 strict decoder 两色识别，不再要求 `supportedSpecVersions`。
- `qr-protocol-spec.md` 已把当前 kind 固定为 6 个，并删除 sign_request fixture 的 `spec_version` 字段。
- wumin 已删除 `user_duoqian` kind/body 残留。
- node / sfid / cpms 的 QR TS/Rust 类型已同步删除 `user_duoqian`。
- wumin decoder 已区分 `propose_close_institution` 与 `propose_close_personal`，并补回归测试。

## 验证记录

- `flutter test test/signer/payload_decoder_test.dart`（wumin）：通过。
- `flutter test test/qr/qr_router_test.dart`（wuminapp）：通过。
- `npx tsc --noEmit`（citizenchain/node/frontend）：通过。
- `npx tsc -b --pretty false`（sfid/frontend）：通过。
- `npx tsc -b --pretty false`（cpms/frontend/web）：通过。
- `cargo check`（sfid/backend）：通过，保留既有 warning。
- `cargo check`（cpms/backend）：通过，保留既有 warning。
- `git diff --check` / `git diff --cached --check`：通过。
- `rg 'memory/05-architecture/qr' ...`：无命中。
- `rg 'user_duoqian|UserDuoqian|userDuoqian|\"spec_version\"|spec_version: number|spec_version: requireInt' ...`：当前代码无命中；仅 QR 文档保留 `user_duoqian` 已下线说明。

## 验收标准

- 旧非标准 QR 架构目录下不再存在 QR 文档或 fixture。
- `memory/07-ai/unified-protocols.md` 所有 QR 路径指向 `memory/01-architecture/qr/`。
- QR action registry 登记的 pallet / call 与 `wumin/lib/signer/pallet_registry.dart` 当前事实一致。
- 当前 QR 文档不再要求 `supportedSpecVersions`。
- 旧 `DuoqianManage(pallet_index = 17)`、旧 `VotingEngine(9)` 投票 action、已删除 execute wrapper 不再出现在当前 QR 真源文档中。
