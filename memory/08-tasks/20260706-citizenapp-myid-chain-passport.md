# CitizenApp 电子护照链上唯一身份展示

## 任务需求

- 删除公民 App「我的 → 电子护照」中与链上唯一身份展示无关的旧功能和字段。
- 电子护照页只展示链上中国已上链后的唯一公民身份信息：投票账户、身份 CID 号、状态、有效期。
- 链上中国公民身份上链流程区分投票公民和参选公民；参选公民走更完整的链上身份载荷。
- 公民 App 用户主页头像右下角显示现有认证图标，但认证只代表当前唯一链上公民身份，不能按多个钱包分别认证。
- 完成后更新文档、完善必要中文注释、清理旧逻辑残留。

## 所属模块

- citizenapp：电子护照页、唯一身份状态读取、用户主页头像认证、我的钱包唯一身份钱包标记。
- onchina：注册局端公民身份上链流程，区分投票身份和参选身份。
- citizenwallet：补充参选身份链上交易确认解析。
- memory：同步更新模块技术文档。

## 核心边界

- 一个公民只有一个身份 CID。
- 一个身份 CID 只能绑定一个投票账户钱包。
- 公民 App 不允许自行选择、更换身份钱包；身份钱包只能由链上已确认数据决定。
- 电子护照页不再展示本机档案状态、钱包二维码、选择钱包、换钱包、扫码签名等旧功能。
- 本任务不修改 `citizenchain/runtime/`；runtime 已支持 `register_voting_identity` 和 `upgrade_to_candidate_identity`。

## 目标字段

```text
identity_wallet_account
identity_cid_number
identity_status
passport_valid_from
passport_valid_until
```

## 执行记录

### 阶段 0：任务卡创建

- 已按当前任务授权创建本任务卡。

### 阶段 1：电子护照链上唯一身份展示

- 重写 `citizenapp/lib/my/myid/myid_service.dart`：删除 `SharedPreferences myid.*` 本机档案状态和 OnChina 本地状态接口依赖，改为扫描本机钱包列表并读取 finalized `CitizenIdentity::VotingIdentityByAccount`。
- 重写 `citizenapp/lib/my/myid/myid_page.dart`：页面只展示投票账户、身份 CID 号、状态、有效期；删除护照号、选择钱包、更换钱包、钱包二维码、扫码签名入口。
- 删除 `citizenapp/lib/my/myid/myid_api.dart` 和 OnChina 旧 `/api/v1/app/myid/status` handler/route/model DTO，避免本地库状态冒充链上真源。

### 阶段 2：唯一身份钱包与认证图标

- `citizenapp/lib/my/user/user.dart`：用户主页头像右下角复用现有 `Icons.verified` 认证图标；只有默认用户钱包等于链上唯一身份钱包且身份状态正常时显示。
- `citizenapp/lib/wallet/pages/wallet_page.dart`：钱包列表只把 `wallet.address == identity_wallet_account` 的唯一钱包标为“身份钱包”；删除旧电子护照钱包选择模式残留。

### 阶段 3：投票/参选身份上链分流

- `citizenchain/onchina/src/domains/citizens/chain_identity.rs`：`prepare/complete` 请求必须带 `identity_level=voting/candidate`；投票身份编码 `VotingIdentityPayload` 并生成 `0x0a00 register_voting_identity`；参选身份编码 `CandidateIdentityPayload` 并生成 `0x0a01 upgrade_to_candidate_identity`。
- `citizenchain/onchina/frontend/citizens/`：公民详情页新增“投票身份 / 参选身份”选择，并将同一 `identity_level` 绑定到安全 grant、签名二维码和 complete 请求。
- `citizenwallet/lib/signer/` 与 `citizenapp/lib/my/myid/voting_identity_payload.dart`：补参选身份 payload/call 解码和中文确认字段。

### 阶段 4：文档、注释、验收

- 已更新 CitizenApp、OnChina、钱包、签名与统一协议文档。
- 验收已通过：
  - `cargo check`（`citizenchain/onchina`）
  - `npm run build`（`citizenchain/onchina/frontend`）
  - `flutter analyze`（CitizenApp touched files）
  - `flutter analyze`（CitizenWallet signer files）
  - `flutter test test/myid_page_test.dart test/my/myid/voting_identity_payload_test.dart`
  - `flutter test test/signer/payload_decoder_test.dart test/signer/field_labels_test.dart`
