# QR_V1 三码分类：用户码 / 钱包码 / 收款码

状态：open（2026-07-29 分类改造七步全部完成并验证；待收款码生成方落地后闭卡）

## 验证结果（2026-07-29）

| 端 | 结果 |
|---|---|
| citizenapp | `flutter analyze` 0 问题 / `flutter test` 1035 passed |
| citizenwallet | `flutter analyze` 0 问题 / `flutter test` 248 passed |
| onchina 后端 | `cargo test` 157 passed / `cargo clippy` 仅 2 条 pre-existing warning（`admin_entry.rs` 参数数、`genesis_projection.rs` expect，均不在本次改动文件内） |
| onchina 前端 | `tsc -b` 通过；`npm run build` 重建 `dist/`（旧 `index-IHEWbXwx.js` 已被清除，新 `index-BH-jNQdT.js` 已含钱包码判据） |
| node 前端 | `tsc -b` 通过 |

残留扫描：`openAccountQrPage` / `UserQrPage.userTransfer` / `buildOfflineReceiveQr` /
`paymentDisplayName` / `parse_user_contact_identity` / `UserContactIdentity` /
`ONCHINA_LOGIN_USER_CONTACT_INVALID` 全仓零命中。

## 剩余待办（收款码生成方落地时执行）

1. 聊天-加号-收付款入口从钱包码切回收款码（只改 `chat_tab.dart` 一处）。
2. 补金额/备注输入 UI —— 不补则 `amount` 恒为空串，收款码与钱包码无差别。
3. 补扫码侧 `i`/`e` 校验 —— `k=4` 按 spec 是临时码、**必须**带 `i/e`，但当前
   `_validateExpiry` 只覆盖 `k=1/k=2`，`k=4` 路径既不校验过期也不校验 `i` 存在性。
   （= `20260806-qr-audit-fixes` 第 10 项，合流至此，那边不再单独跟踪。）
4. 届时把 spec 第 7 节的「预留：生成方待实现」标注去掉。
5. **同批执行 `20260806-qr-body-schema-codegen`** —— 本条第 3 点要三端同改
   `user_transfer_body`，正是把 body 声明层收进 `crates/qr-protocol` 并三语言生成的
   最佳时点：改动与收益同批发生，不必在已对齐的系统上单独动大手术。

## 任务需求

把展示型二维码按「谁生成 + 表达什么」拆成三种，入口即语义，取消任何运行时分流：

| 码 | 表达 | `k` | 类型 | 生成端 | 入口 |
|---|---|---:|---|---|---|
| 用户码 | 人（永久 CID） | 3 | 固定，无 `i/e` | 仅 CitizenApp，且 CID↔AccountId 闭环命中 | 用户主页 |
| 钱包码 | 账户 | **5**（新增） | 固定，无 `i/e` | CitizenApp + CitizenWallet，任意账户无条件 | 钱包-账户详情 |
| 收款码 | 一笔收款请求 | 4 | 临时，必须 `i/e` | 仅 CitizenApp | 聊天-加号-收付款 |

本轮范围：**只做分类**。收款码功能（金额/备注输入 UI、过期校验）推迟，`k=4` 在协议中标「预留：生成方待实现」，本轮零生成方。

## 已确认边界

- 钱包码 `k = 5`。撤销 `qr-protocol-spec.md` §8 对 `k=5` 的永久拉黑，旧 `chat_node_pairing` 码值回收给钱包码。不用 `k=6`，不留码值空洞。
- 钱包码 body **只有 `account_id` 一个字段**（小写 `0x` + 64 hex）。不带钱包标签、不带昵称、不带 CID、不带 ss58。展示用 ss58 由扫码端自行派生。
- 用户码 `k=3` 结构**不动**（继续 `cid_number` + `ss58_address` + `display_name`）。钱包码用 `account_id` 与 `k=3` 用 ss58 的口径差异本轮**不收敛**，记为已知项。
- 聊天-加号-收付款入口本轮**暂出钱包码**；收款码落地时只切这一个入口。
- CitizenWallet 永远不生成用户码，永远不解析收款码。
- 通讯录只接受用户码。钱包码、收款码进通讯录必须拒绝。
- OnChina 登录第 1 步改收钱包码，`validate_login_identity` 改 `account_id → CidByAccountId → CidRegistry Active → AdminRecord Active` 单向反查，三级全 fail-closed。
- 不修改 `citizenchain/runtime/`、`citizenchain/pallets/`、chainspec。
- 无兼容、无过渡、无残桩：clean cutover。

## 预计修改目录

- `memory/01-architecture/qr/qr-protocol-spec.md`：§3 表加 `k=5`、§8 改写为码值回收记录、新增钱包码规格节、`k=4` 标预留。
- `memory/01-architecture/qr/qr-protocol-fixtures/`：新增 `wallet_code.json` golden 样本。
- `citizenapp/lib/qr/`：`QrKind` 加 `walletCode(5)`、新增 `bodies/wallet_code_body.dart`、envelope 分派、router 分类。
- `citizenapp/lib/8964/profile/`：用户主页出用户码；`user_qr_page.dart` 删身份分流。
- `citizenapp/lib/wallet/`：账户详情与身份卡入口出钱包码。
- `citizenapp/lib/chat/`：收付款入口暂出钱包码。
- `citizenwallet/lib/qr/`：`QrKind` 同步、新增钱包码 body、删收款码解析。
- `citizenwallet/lib/ui/account_detail_page.dart`：删 `buildOfflineReceiveQr` 与「5 分钟内有效」文案，改出钱包码。
- `citizenchain/onchina/src/core/qr/`：Rust `QrKind` 加 `WalletCode = 5`、新增钱包码解析、改 `UserTransfer` 过时注释。
- `citizenchain/onchina/src/auth/login/`：登录第 1 步改收钱包码、判据改单向反查。
- `citizenchain/onchina/frontend/core/`：`citizenQr.ts` 加 `k=5` 与 `isFixedKind`、`ScanAccountModal` 收敛为只认钱包码、`LoginView` 改判据。
- `citizenchain/node/frontend/shared/qr/`：`citizenQr.ts` 加 `k=5`、`parseAddressQr` 收敛为只认钱包码。
- `memory/07-ai/unified-naming.md`、`unified-protocols.md`、`memory/05-modules/`：`k=5` 拉黑表述改写、三码分类回写。

## 主要风险

- `k` 枚举有**五处**定义点（citizenapp / citizenwallet / onchina Rust / onchina 前端 TS / node 前端 TS），漏一处即跨端解析裂开。onchina 前端还有 `isFixedKind()` 单点必须同改。靠 golden fixture 五端同读兜底。
- OnChina 登录判据改动是鉴权入口。新单向反查三级（`CidByAccountId` 有记录 / `CidRegistry` Active / `AdminRecord` Active）任一不成立必须拒绝，不允许「读不到就放行」。
- `k=4` 保留解析但本轮无过期校验：外部构造的收款码仍能扫进转账。开发期零用户下接受，收款码落地时必补。
- `k=5` 旧语义污染：野外零存量（开发期零用户）+ body 字段集精确匹配双保险，无需专门拒绝逻辑。
- `k=4` 零生成方不是残桩而是预留，必须在 spec 显式标注 + 本卡挂住，避免后续清扫审计误删。

## 完成标准

- 三个入口各出各的码，`user_qr_page.dart` 运行时分流整块删除，「CID 绑到 `//n` 时二维码静默降级」缺陷消失。
- 五处 `k` 枚举 + `isFixedKind()` 全部含 `k=5`，golden fixture 五端同读通过。
- CitizenWallet 账户详情出固定钱包码，不再出现任何时效字段与「5 分钟内有效」文案。
- OnChina 管理员登录第 1 步可由 CitizenWallet 自己出示钱包码完成，不再需要 CitizenApp 参与。
- 通讯录扫钱包码被拒绝。
- `flutter analyze` 冷热两端 0 问题、两端测试全绿、`cargo check` onchina 通过、两个前端 `tsc` 通过。
- 文档六处 `k=5` 拉黑表述改写完毕，`user-qr-single-source` 记忆扩充为三码分类。

## 执行步骤

1. 协议真源：spec 改写 + `wallet_code.json` fixture。
2. 五处 `k` 枚举 + 钱包码 body（四端 + Rust）。
3. CitizenApp 三入口分类 + 删分流。
4. CitizenWallet 账户详情改钱包码 + 删 `buildOfflineReceiveQr` + 删收款码解析。
5. OnChina 后端登录判据改钱包码 + 单向反查。
6. OnChina / node 前端扫码收敛为只认钱包码。
7. 测试补齐 + 文档与记忆回写。
