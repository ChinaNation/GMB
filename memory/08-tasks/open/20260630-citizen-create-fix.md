# 修复公民档案创建功能

## 任务目标

- 新增公民时钱包账户不再必填,允许未成年人和无钱包公民先建立本地电子护照档案。
- 公民列表默认显示当前市全部公民,支持搜索和高效分页。
- 公民详情从弹窗改为详情页面,在详情中承接钱包绑定与推送链上身份流程。
- 未满 16 周岁不得推送链上公民身份,前端、后端和 runtime 均要校验。
- 推送链上身份时必须录入钱包账户,并要求该钱包对链上身份载荷签名。
- 完成后同步更新文档、完善中文注释、清理旧流程残留,并本地提交。

## 涉及范围

- `citizenchain/onchina/frontend/citizens/`
- `citizenchain/onchina/src/domains/citizens/`
- `citizenchain/onchina/src/core/`
- `citizenchain/runtime/otherpallet/citizen-identity/`
- `citizenwallet/`
- `memory/05-modules/citizenchain/onchina/`
- `memory/07-ai/unified-protocols.md`

## 边界

- 不触碰其它线程的 legislation 相关改动。
- 不执行 `git push` 或创建 PR。
- runtime 改动已由用户在当前任务中二次确认。

## 验收

- 新增公民不要求钱包账户。
- 未成年公民可以建档,但无法推送上链。
- 成年公民详情页可录入钱包账户并发起链上身份推送。
- 公民列表默认显示当前市公民,分页不使用 offset。
- 文档与协议登记同步更新。

## 实现记录

- OnChina 本地创建公民移除 `wallet_account` 必填项,钱包字段改为可空,数据库迁移清理旧空字符串。
- 公民列表空搜索改为当前市全量 cursor 分页,点击行进入详情页。
- 公民详情页新增链上身份推送流程:录入/扫描钱包、生成公民钱包签名二维码、验签后展示注册局管理员链上交易二维码。
- runtime `citizen-identity` 的 `VotingIdentityPayload` 增加 `citizen_age_years`,注册/更新/候选升级统一拒绝未满 16 周岁。
- CitizenWallet 增加 `citizen_identity` 哈希域签名和 `register_voting_identity` 解码。
- QR 协议、OnChina 前后端文档、数据安全文档和公民链总览已同步更新。

## 真实验收记录

- `npm run build` 通过,OnChina 前端构建产物已更新。
- `flutter test test/signer/payload_decoder_test.dart test/signer/qr_signer_test.dart` 通过。
- `cargo test --manifest-path citizenchain/Cargo.toml -p citizen-identity under_sixteen_cannot_register_onchain_identity --locked` 通过。
- `cargo check --manifest-path citizenchain/Cargo.toml -p onchina --locked` 通过。
- 使用 `/tmp/gmb-onchina-pg-citizen-fix` 临时内嵌 PostgreSQL 启动 OnChina,自动生成公权目录 401364 条,`GET /api/v1/health` 返回 200。
