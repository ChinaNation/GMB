# CitizenApp CidRecord 空居住地码解析修复

状态：done（2026-07-31 建卡并完成，用户已确认方案与可选项）

## 缺陷

CitizenApp 判「是否已注册 CID」时解析链上 `CidRegistry` 记录，
`cidRecordIsActive` 读 `residence_province_code` / `residence_city_code`
未允许空 `BoundedVec`，抛出的 `FormatException` 被本函数 `catch (_)` 吞成 `false`。

而链端这两个字段**恒为空**：

- 创世 `initial_cid_bindings` 写 `AreaCodeBound::default()`（`citizen-identity/src/lib.rs:884`）
- `self_occupy_cid` 与注册局占号两条路径都传 `empty_area`（同文件 1312 / 1532）
- 换绑只 clone 旧值（同文件 1269）

结果：链上任何 CID 记录都被判非 Active → 绑定闭环失败 → `readByAccountId` 返回
`null` → `IdentityRegistrationGate` 落 `unregistered`。

## 影响

- 等级 CRITICAL，影响全体用户（开发期零用户，无迁移问题）。
- 阻断全部需 CID 的功能：聊天、广场、通讯录、会员、创作者。
- `readBindingsByCidNumbers` 走同一函数，通讯录按 CID 反查绑定账户一并失效。

## 现场证据（2026-07-31 本地节点 127.0.0.1:9944 finalized 实测）

程伟创世身份三张 storage 正反闭环全部成立：

```
CidByAccountId[0x0cb1d05c…dca4b06b] = "CN220-CTZN2-198805200-2026"
AccountIdByCid[CN220-…]             = 0x0cb1d05c…dca4b06b
BindingRevisionByCid[CN220-…]       = 1
CidRegistry[CN220-…] 67 字节:
  68 5a533030312d46524730372d3234393437343530332d32303236   registrar "ZS001-FRG07-249474503-2026"
  93c7cc569ee4c38bead016a83ffd5e76d1c6a628555b55805cf18d3f06779a66  commitment
  00 00                                                      省码空 / 市码空  ← 原解析在此抛
  00 00000000 00                                             Active / registered_at=0 / 未吊销
VotingIdentityByCid = 空 → 匿名已注册
```

## 任务需求

1. `cidRecordIsActive` 两处 `_readBoundedBytes(..., 16)` 补 `allowEmpty: true`。
2. （用户确认一起做）把「解析失败」与「确实非 Active」拆开：布局非法上抛
   `FormatException` 由调用方按链读失败处理，不再压成 `false` 冒充未注册。
   `revoked_at` 按 `Option` 双分支正确解析（`None`=1 字节 / `Some`=5 字节）。
3. 测试夹具从 `_bounded('GD')` / `_bounded('0755')` 改为空居住地码——链上从不
   出现非空形态，旧夹具正是 bug 躲过测试的原因。
4. 用上面链上实测 67 字节做金标向量钉死，防回归。

## 已确认边界

- **不修改 `citizenchain/`**：链端写空居住地码是设计如此（匿名 CID 无省市归属），
  遵守 `no-chain-changes`。
- 全 App 只有 `citizen_identity_chain_reader.dart` 一处解析 `CidRecord`，
  citizenwallet 无同类逻辑，无需跨端同步。
- `votingIdentityLayoutIsValid` 的「布局损坏降级为匿名」是有意的安全降级，本卡不动。
- 程伟钱包账户 = `0x0cb1d05c…dca4b06b` 已由用户确认与链上绑定一致，非本卡范围。

## 预计修改目录

- `citizenapp/lib/my/myid/citizen_identity_chain_reader.dart`：解析修复 + 三态语义。
- `citizenapp/test/my/myid/identity_account_resolver_test.dart`：夹具订正 + 金标向量。

## 验收

- `flutter test test/my/myid/ --concurrency=1` 全绿。
- `dart analyze lib/my/myid` 零问题。
- 真机：导入钱包后进「聊天」不再显示「需要注册身份」。

## 执行结果（2026-07-31）

`citizen_identity_chain_reader.dart`：

- `cidRecordIsActive` 两处居住地码解析补 `allowEmpty: true`。
- 删掉整个 `try/catch (_) => false`，改三态：不存在 / `Revoked` / Active 带撤销块号
  → `false`；布局越界、尾部截断、尾随字节、`revoked_at` 标记非法 → 抛 `FormatException`。
- `revoked_at` 按 `Option` 双分支解析（`None` 1 字节 / `Some` 5 字节），原实现把
  `Some` 一律当成长度不符而返回 false，现在按自相矛盾显式判非 Active。
- `readBindingsByCidNumbers` 文档补「任一条布局无法解析则整批上抛」。

`identity_account_resolver_test.dart`：

- 两处 `CidRegistry` 夹具的 `_bounded('GD')` / `_bounded('0755')` 收敛到
  `_activeCidRecord()`，居住地码改空。
- 新增 group「CidRecord 解析」7 条：创世金标 67 字节判 Active、非空省市码不回归、
  记录不存在、`Revoked`、Active 带撤销块号、截断上抛、尾随字节上抛。
- 新增 group「创世匿名身份端到端」2 条：`readByAccountId` 命中匿名快照、
  `IdentityAccountResolver.resolve().isRegistered == true`。

验收：`flutter test test/my/myid/ --concurrency=1` 71 passed（原 62 + 新增 9）；
`flutter test --concurrency=1` 全量 1051 passed / 5 skipped（既有 skip）；
`dart analyze` 全仓 No issues found。真机验证待用户执行。

## 主要风险

- `readBindingsByCidNumbers` 批量刷新时单条布局损坏会整批上抛。这是刻意的
  fail-closed：解析器与链端结构不同步时，整批失败并提示重试，远好于静默把
  全部联系人判成「无有效绑定」。两个调用点 `refreshContactBindings` /
  `resolveCurrentContact` 的既有契约本就是「链读失败直接失败，禁止回退旧地址」。
