# CitizenApp 聊天/发帖去授权（默认热钱包静默签名）

## 任务需求

- 聊天、发帖不再弹身份验证；用默认热钱包**静默签名**（`signWithWalletNoAuth`，已存在，不走生物识别）。
- 发帖是**自动扣款**：校验余额够 ED + 最低链上费用 0.1 元后静默签名 `扣费入块`，不弹。
- 授权只保留在"动钱/换身份"：
  - **转账/充值/提现/清算行/多签/个人账户**：保留 `authenticateForSigning()`（资金安全）。
  - **投票**：保留授权（用户明确要求）。
  - **切换默认钱包 = 切换用户身份**：新增一次授权（用户明确要求）。
- 竞选帖字段（竞选目标机构 CID + 岗位）先做**预留 + 注释**，待公民身份上链后落地。

## 建议模块

- 聊天签名：`citizenapp/lib/chat/chat_runtime.dart`
- 发帖签名：`citizenapp/lib/8964/pages/square_compose_page.dart`
- 切换默认钱包授权：`citizenapp/lib/wallet/pages/wallet_page.dart`
- 竞选字段预留：`citizenapp/lib/8964/models/square_models.dart` + Worker `citizenapp/cloudflare/src/types.ts`（注释）

## 影响范围

- 删 `chat_runtime._signWalletPayload` 的 `authenticateForSigning()` + `_authenticatedWalletIndexes` 门控；错误文案清残留（"聊天账户"→"默认用户钱包"）。
- 删 `square_compose_page._submit` 的 `authenticateForSigning()`；保留余额校验与静默签名。
- `wallet_page._onReorder`：拖拽后默认用户钱包发生变化时，先 `authenticateForSigning()` 通过才落盘，失败则回滚 UI 不切换。
- 竞选字段：`SquarePost` 加可空 `campaignInstitutionCid`/`campaignPosition` + 注释；Worker 注释预留列。
- 链端 0 改。

## 主要风险点

- 默认热钱包静默签名 = 手机在解锁态被拿走可冒充聊天/发帖（发帖仍扣本人 1 元），但转账/投票/换身份仍要验证；应用锁 PIN 为第一道闸。用户已明确接受该取舍。
- 切换默认授权失败必须回滚拖拽 UI，避免"看起来切了其实没切"。
- 清残留：`_authenticatedWalletIndexes`、"聊天账户"旧文案一并清掉。

## 是否需要先沟通

- 否。授权边界（投票留、换默认留、聊天/发帖去）用户已逐条确认。

## 预计修改目录

- `citizenapp/lib/chat/`、`citizenapp/lib/8964/`、`citizenapp/lib/wallet/`：代码。
- `citizenapp/cloudflare/src/`：竞选字段注释预留。
- `memory/05-modules/citizenapp/`、`memory/01-architecture/citizenapp/`：授权策略文档。

## 分步骤技术方案

### 步骤 1：聊天去授权
- `chat_runtime.dart`：删 `_authenticatedWalletIndexes` 字段与 `_signWalletPayload` 内 `authenticateForSigning()` 块；文案改"默认用户钱包"；补注释"聊天登录/设备绑定静默签名，不涉及转账不验证"。

### 步骤 2：发帖去授权（自动扣款）
- `square_compose_page.dart`：删 `_submit` 的 `authenticateForSigning()`；保留 `hotWalletManager = WalletManager()` 供 `signWithWalletNoAuth`；补注释"发帖自动扣款，静默签名"。

### 步骤 3：切换默认钱包授权
- `wallet_page._onReorder`：算拖前/拖后默认（`defaultUserWalletIndex`），变化则 `authenticateForSigning()`；失败回滚 `_wallets` 并提示，不落盘。

### 步骤 4：竞选字段预留
- `SquarePost` 加可空 `campaignInstitutionCid`/`campaignPosition`（解析响应存在则填，否则 null）+ 统一注释；Worker post 类型加注释预留列。

### 步骤 5：验收 + 文档 + 残留
- `dart analyze` + `flutter test`（im/8964/wallet）；Worker `typecheck`。
- 更新 Chat/WALLET/CITIZENAPP 技术文档授权策略；回写卡；`git diff --check`。

## 当前执行状态

- [x] 步骤 1：`chat_runtime.dart` 删 `_authenticatedWalletIndexes` + `_signWalletPayload` 内 `authenticateForSigning()`；文案改"默认用户钱包"；补注释。
- [x] 步骤 2：`square_compose_page._submit` 删 `authenticateForSigning()`（保留余额校验与 `signWithWalletNoAuth` 扣费入块）；补注释。
- [x] 步骤 3：`wallet_page._onReorder` 加默认用户变化检测 → `authenticateForSigning()`，失败提示且不落盘、不切换。
- [x] 步骤 4：`SquarePost` 加可空 `campaignInstitutionCid`/`campaignPosition` + 注释；`_parsePost` 解析预留；Worker `SquarePostRow` 注释预留列。
- [x] 步骤 5：清残留——卡 C 遗漏的"聊天账户"旧文案（chat_tab ×2、chat_runtime 注释、对应测试断言）改为"默认用户钱包/创建热钱包"。
- [x] 验收：`dart analyze lib test` 干净（唯一 info 为未触及文件既有 lint）；Worker `typecheck` 通过；`flutter test test/chat test/8964 test/wallet` 103 过 4 skip。
- [x] 文档：更新 WALLET_TECHNICAL 授权分层、CHAT_TECHNICAL 聊天静默签名。
- [ ] 待用户真机验收：进聊天/发帖不弹；转账/投票/切换默认钱包弹验证。
