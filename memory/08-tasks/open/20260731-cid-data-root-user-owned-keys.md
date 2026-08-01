# CID 稳定数据根与新钱包接管

状态：in_progress（2026-07-31，可跟踪仓库实现与验收已通过；待确认补齐私有部署控制台）

## 目标

- `cid_number` 是唯一身份主键；业务数据永久归 CID，不归钱包账户。
- CID 换绑 finalized 后，新绑定 `account_id` 是唯一签名与控制凭证。
- 新账户接管同一 CID 的动态、文章、通讯录、聊天等私有数据，不重新加密全部业务密文。
- 接管不得读取、要求或联系此前账户私钥、助记词、设备或本地缓存。
- 新账户数据根包装和用途子钥验证就位后，才清理旧账户包装与旧凭证。

## 已确认缺陷

提交 `87d97365` 把数据根改成：

```text
HKDF(钱包母种子, cid_number)
```

这个实现把 CID 数据根重新绑定到某一只钱包的助记词。A 钱包丢失并由注册局把 CID
换绑到完全不同的 B 钱包后，B 的母种子只能派生另一把数据根，无法解密 A 时期密文，
违反“换绑成功即由新钱包完整接管 CID”的产品规则。

## 定稿三层模型

1. 业务层：每个 CID 只有一把首次随机生成、永久稳定的 `CidDataRoot`。
2. 当前账户层：当前绑定账户 child 只派生本机 KEK，包装同一数据根。
3. 独立恢复层：Worker 使用只存在于 Secret 的恢复密钥，按创世与 CID 派生独立 KEK，
   密封保存该 CID 数据根；D1 不保存明文数据根。

恢复层是“旧钱包完全不可用仍能恢复同一数据根”的必要信任边界。它不替代链上授权：
只有 finalized 当前绑定账户通过一次性钱包签名挑战，Worker 才能发放对应 CID 的数据根。

## 接管协议

签名载荷绑定：

- 固定动作 `activate_cid_binding`
- 创世哈希
- `cid_number`
- `binding_revision`
- 当前 `account_id`
- 本次 X25519 接收公钥
- 一次性 `challenge_id`
- `expires_at`

Worker 在发放前后两次读取 finalized 当前绑定；挑战使用 D1 条件更新原子消费。数据根
不会出现在 HTTP 明文 JSON 中：Worker 生成一次性 X25519 发送密钥，与 App 临时接收密钥
协商 AES-256-GCM，AAD 再绑定完整接管上下文和数据根摘要。

## 正确接管顺序

1. 读取 finalized `(cid_number, binding_revision, account_id)`。
2. 当前新账户签一次性恢复挑战。
3. App 解密并校验稳定数据根摘要。
4. 新账户 child 包装数据根，读回摘要验证。
5. 写 CID 数据根缓存并派生用途子钥。
6. 删除此前本地包装和旧账户级密钥名残留。
7. 当前新账户登记本设备子钥。
8. Worker 确认新设备子钥上岗后，删除旧 revision / 旧账户会话、设备子钥、Chat
   KeyPackage 与 Chat 设备，关闭旧实时连接。

任一步失败都不伪造完成；重试仍从 finalized 当前绑定收敛。全过程没有旧账户输入。

## 防重放

- 签名域固定走 `signing_message(OP_SIGN_SQUARE_ACTION)`。
- 挑战号使用安全随机数且有专用前缀，禁止与登录挑战交叉消费。
- 挑战绑定创世、CID、revision、当前账户、临时接收公钥和过期时间。
- 条件 `UPDATE ... used_at IS NULL AND expires_at > now` 原子消费。
- 发放前后双读 finalized；绑定变化立即拒绝。
- 本机禁止绑定版本回退与同 revision 的账户/摘要冲突。

## 预计修改目录

- `citizenapp/lib/security/`：删除母种子派生真源，只保留 CID 数据根、用途派生和当前账户包装代码。
- `citizenapp/lib/wallet/`：恢复新账户安装接口，删除助记词补录分支，按新账户所属钱包生成设备子钥。
- `citizenapp/lib/my/myid/`：按 finalized 当前账户编排恢复、安装、设备登记和失败重试。
- `citizenapp/lib/8964/services/`：增加 X25519 加密数据根恢复客户端和严格响应校验。
- `citizenapp/cloudflare/src/`：恢复并加固 CID 数据根恢复路由、密封存储、防重放和凭证清理。
- `citizenapp/cloudflare/schema/`：恢复 CID 数据根恢复封装表并升级开发期唯一 schema 基线。
- `citizenapp/test/`、`citizenapp/cloudflare/test/`：覆盖不同助记词、旧设备全失、同一密文解密、
  重放、跨链、过期、绑定变化、失败回滚和正确清理顺序。
- `memory/01-architecture/`、`memory/03-security/`、`memory/05-modules/`、`memory/07-ai/`：
  更新信任边界、协议、钱包说明、注释和残留。

## 验收门槛

- A、B 使用完全不同助记词；删除 A 的全部私钥、助记词、设备和缓存后，注册局换绑到 B，
  B 取得的数据根摘要与 A 时期相同并能解密 A 时期密文。
- 自主换绑仍由链上当前旧账户签授权、新账户提交；数据恢复阶段只接受 finalized 新账户签名。
- 注册局换绑的数据恢复阶段不要求旧账户签名。
- 重放、跨创世、过期、旧 revision、临时接收公钥替换和 finalized 变化全部失败关闭。
- Worker 测试、类型检查、Flutter 测试、Dart 分析和真实本地 Worker/D1 HTTP 验收通过。
- 文档、中文注释和全仓残留搜索与目标状态一致。

## 不在本步范围

- 不修改 `citizenchain/runtime/`。
- 不执行 GitHub 推送、远端 CI、部署或创世。

## 2026-07-31 实施与验收记录

- Worker 已恢复每 CID 随机稳定数据根、Secret 密封恢复层、finalized 双读、专用一次性
  挑战和 X25519 加密发放；登录挑战与接管挑战前缀不能交叉消费。
- CitizenApp 已删除母种子派生、助记词补录和旧账户解包分支；finalized 当前新账户独立
  恢复、安装并读回验证新包装，再安装用途子钥和当前新钱包设备子钥。
- Worker 只有确认新设备子钥成功落库后，才清旧挑战、Session、设备子钥、Chat 设备、
  KeyPackage 与旧实时连接；CID 业务数据与稳定数据根不删除、不迁移、不重加密。
- Worker `npm run typecheck` 通过；Vitest 32 文件、222 项全绿，其中包含完整 Worker
  路由 + 真实 Miniflare D1/KV binding 加密接管，以及换绑后真实 AES-GCM 通讯录解密。
- CitizenApp `dart analyze lib test` 零问题；Flutter 1056 项通过、5 项按既有原生条件
  跳过、0 项失败。A/B 使用完全不同助记词，A 全部秘密删除后 B 仍解开 A 时期密文。
- Wrangler 4.114.0 已在独立 `/tmp` 状态执行唯一 schema：62 条语句成功，
  `cid_data_roots` 表与 9 个字段可读；本地 Worker `GET /health` 返回 200，临时状态已删除。
- 可跟踪仓库残留扫描与 `git diff --check` 通过；未修改 runtime，未推送、未触发 CI、
  未部署、未创世。

## 待补充确认的私有部署控制台闭环

`citizenconsole/` 被 Git 整目录忽略，且不在本步已确认的预计修改目录中。只读复核确认它
仍明确断言 `cid_data_roots` 不应存在，且部署 Secret 白名单不会注入
`CID_DATA_ROOT_RECOVERY_KEY`；如果不修，未来部署会缺恢复密钥并失败关闭。需要用户确认
追加该私有目录后，才能修改其现有 server、Cloudflare 动作、原生 Keychain 白名单、
网页生成入口和安全测试；不新增文件，不纳入 Git。
