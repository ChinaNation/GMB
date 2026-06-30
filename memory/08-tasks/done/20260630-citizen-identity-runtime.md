# 链上公民身份模块终态改造

## 目标

- 使用 `citizen-identity` 链上公民身份模块,保存投票身份和参选身份。
- 投票引擎只读取 runtime 链上公民身份和链上人口快照,不得再依赖 OnChina 本地库或 OnChina 签名的虚假人口数。
- OnChina 只作为注册局操作入口,创建本地档案后按公民选择提交投票身份或参选身份上链。
- CitizenApp 电子护照区分未上链、投票身份上链、参选身份上链。
- CitizenApp/CitizenWallet/文档统一使用公民身份确认命名。

## 修改范围

- `citizenchain/runtime/otherpallet/citizen-identity/`:新增链上公民身份 pallet。
- `citizenchain/runtime/primitives/`:统一公民身份、签名和 QR action 常量。
- `citizenchain/runtime/votingengine/`:改为读取链上公民身份和链上人口统计。
- `citizenchain/runtime/src/`:注册新 pallet,接入费用分支、benchmark 和测试。
- `citizenchain/onchina/`:接入链上身份级别、状态和调用生成。
- `citizenapp/`、`citizenwallet/`:同步二维码 action 和电子护照展示。
- `memory/`:更新架构、模块、QR 协议和统一命名文档。

## 验收

- 当前源码、当前技术文档和生成物无旧身份 pallet 名、旧链下身份流程、链下投票凭证和链下人口证明残留;归档 `docs/citizenpassport/` 保留历史 CPMS 字段,不作为当前实现真源。
- `cargo fmt --all` 通过。
- `cargo test -p primitives --test sign_golden` 通过。
- `cargo test -p citizen-identity -p citizen-issuance -p votingengine -p joint-vote -p legislation-vote -p internal-vote` 通过。
- `cargo test -p citizenchain` 通过。
- `cargo check -p onchina` 通过。
- `flutter test test/signer/signing_golden_test.dart` 通过。
- `flutter test test/signer/payload_decoder_test.dart` 通过。
- `flutter analyze` 检查 CitizenApp/CitizenWallet 相关文件通过。
- `npm --prefix citizenchain/onchina/frontend run build` 通过。
- `npm --prefix citizenchain/node/frontend run build` 通过。
- 真实 OnChina 服务健康检查通过;首次行政区自动校准约 150 秒后 `/api/v1/health` 返回 `UP`。
- `git diff --check -- citizenchain citizenapp citizenwallet docs memory` 通过。

## 状态

- 2026-06-30:完成。
