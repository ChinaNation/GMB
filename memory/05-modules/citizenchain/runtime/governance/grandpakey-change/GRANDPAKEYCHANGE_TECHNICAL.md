# GRANDPA 验证密钥更换模块技术文档

## 1. 模块职责

`grandpakey-change` 只负责国家储委会（NRC）和 43 个省储委会（PRC）各自节点的
GRANDPA authority 更换。省储行（PRB）没有 GRANDPA authority，不能使用本模块。

本模块提供两条互斥路径：

| 路径 | 适用情形 | 授权与证明 | 投票 |
| --- | --- | --- | --- |
| 正常更换 | 旧私钥仍可签名 | 目标机构 `CID + 委员岗位码 + 委员 account_id` 授权；旧、新 GRANDPA 私钥共同签署同一证明 | 不投票 |
| 紧急恢复 | 旧私钥丢失或无法签名 | 目标机构委员岗位发起；新 GRANDPA 私钥签署持钥证明 | 仅目标机构自己的委员内部投票 |

紧急恢复不是联合投票。NRC 只由 NRC 的 19 个委员岗位投票，机构阈值为 13；每个
PRC 只由本 PRC 的 9 个委员岗位投票，机构阈值为 6。投票资格、快照、阈值、计票和
终态全部由现有投票引擎提供，本业务模块不实现投票流程。

代码目录：

- `citizenchain/runtime/governance/grandpakey-change/src/`

## 2. Runtime 接线

配置位置：

- `citizenchain/runtime/src/configs.rs`

关键配置：

- `InternalVoteEngine = VotingEngine`：仅紧急恢复创建内部投票。
- `InstitutionRoleAuthorization`：统一读取机构岗位任职和岗位业务权限真源。
- `GrandpaChangeDelay`：两条路径都调用 `pallet-grandpa::schedule_change` 延迟生效。
- `WeightInfo`：两项外部调用使用各自 benchmark 权重。

当前没有创世数据迁移，`STORAGE_VERSION = 0`，不得增加开发期 migration。

## 3. 外部调用

### 3.1 `call_index(0) propose_emergency_grandpa_key_recovery`

字段顺序：

1. `actor_cid_number`
2. `actor_role_code`
3. `new_public_key`
4. `proof_nonce`
5. `proof_expires_at`
6. `new_public_key_signature`

约束：

- 调用者必须是目标 NRC/PRC 委员岗位的有效任职人，并同时满足
  `CID + 岗位码 + account_id`。
- 岗位必须拥有紧急恢复的 `Propose` 权限。
- 新私钥必须对本次完整证明载荷签名，证明节点已经持有新私钥。
- 每个机构同时最多存在一项未终结的紧急恢复。
- 创建的 `VotePlan` 只包含目标机构，不得加入其他 NRC、PRC、PRB 或公民。
- 投票通过后由投票引擎回调本模块调度 authority set；全链已有 GRANDPA pending
  change 时返回可重试结果。
- 被否决或确定不可执行时释放新公钥占用。

### 3.2 `call_index(1) schedule_grandpa_key_rotation`

字段顺序：

1. `actor_cid_number`
2. `actor_role_code`
3. `new_public_key`
4. `proof_nonce`
5. `proof_expires_at`
6. `old_public_key_signature`
7. `new_public_key_signature`

约束：

- 调用者必须是目标 NRC/PRC 委员岗位的有效任职人，并同时满足
  `CID + 岗位码 + account_id`。
- 岗位必须拥有正常更换的 `Propose` 权限。
- 旧、新 GRANDPA 私钥必须分别签署同一份完整证明载荷。
- 不创建提案、不进入投票引擎，校验通过后直接调度延迟生效。
- 目标机构存在未终结紧急恢复时，不允许正常更换越过紧急恢复。

两个 call index 连续使用 `0`、`1`，不保留空洞或旧 wrapper。

## 4. 持钥证明

`GrandpaKeyProofPayload` 固定绑定：

- 当前链 `genesis_hash`
- `actor_cid_number`
- `actor_role_code`
- 当前发起 `account_id`
- 当前旧 GRANDPA 公钥
- 新 GRANDPA 公钥
- 当前 GRANDPA `set_id`
- 机构级递增 `proof_nonce`
- `proof_expires_at`
- `change_kind`（正常更换或紧急恢复）

签名消息唯一使用：

`primitives::sign::signing_message(OP_SIGN_GRANDPA_KEY_CHANGE, SCALE(payload))`

因此，签名不能跨链、跨机构、跨岗位、跨账户、跨密钥、跨 set、跨路径或重复使用。
`proof_nonce` 只在完整调用成功后递增。

## 5. 公钥校验

两条路径共同执行以下校验：

- 新公钥必须是 32 字节 ed25519 公钥，不能为全零。
- 拒绝无效曲线点和 small-order 弱公钥。
- 新公钥不能等于目标机构当前公钥。
- 新公钥不能被其他机构当前使用，也不能被另一项待处理流程占用。
- 目标机构当前旧公钥必须仍在实际 GRANDPA authority set 中。
- 替换后的 authority set 不得出现重复公钥。
- `pallet-grandpa` 或本模块已有全链 pending change 时不能再调度。

## 6. 存储

| 存储 | 用途 |
| --- | --- |
| `CurrentGrandpaKeys` | 机构最后一次已经实际生效的 GRANDPA 公钥 |
| `GrandpaKeyOwnerByKey` | 已生效公钥到机构 CID 的唯一反向索引 |
| `ReservedGrandpaKeys` | 投票中或等待生效的新公钥占用 |
| `NextGrandpaKeyProofNonce` | 每个机构下一份证明必须使用的 nonce |
| `ActiveEmergencyRecoveryByInstitution` | 每个机构唯一的未终结紧急恢复提案 |
| `PendingGrandpaKeyChange` | 全链唯一、已调度并等待实际生效的变更 |

创世时从 `CHINA_CB` 初始化 44 个 NRC/PRC 当前公钥及反向索引，并拒绝重复初始公钥。

## 7. 生效与节点私钥生命周期

1. 节点生成新 ed25519 私钥，并在提交前把新私钥加入 `gran` keystore，旧私钥继续保留。
2. 正常更换完成旧、新私钥双签名；紧急恢复只完成新私钥证明并等待本机构内部投票。
3. Runtime 调用 `pallet-grandpa::schedule_change`，旧、新私钥在延迟期内同时保存在节点。
4. `on_initialize` 读取实际 GRANDPA authorities；只有新公钥已出现且旧公钥已消失，
   才更新正反向索引、清理 pending，并发出 `GrandpaKeyActivated`。
5. 节点后台只读取 finalized 状态。确认新公钥已是该 CID 当前公钥、位于 authority
   set 且旧公钥已移除后，才删除旧私钥、更新本地元数据并重启节点。

链上事件尚未 finalized、仅达到预计区块、仅看到交易成功或仅看到
`GrandpaKeyActivated` 的非 finalized 分叉，都不能触发旧私钥删除。

## 8. 紧急恢复回调边界

紧急恢复业务动作与投票提案原子绑定。回调执行前必须重新核对：

- callback owner 和 scope；
- 内部投票 kind、stage、status；
- 目标机构代码、CID 和 action；
- 当前机构岗位授权；
- 提案是否仍是该机构登记的唯一活动紧急恢复；
- 新公钥占用是否仍属于该机构。

暂时性的 `GrandpaChangePending` 返回 `RetryableFailed`，由投票引擎现有重试机制处理。
确定不可执行或被否决时关闭本业务状态并释放公钥占用。投票引擎代码不因本模块改动。

## 9. 事件

- `EmergencyRecoveryProposed`
- `RoutineRotationScheduled`
- `EmergencyRecoveryScheduled`
- `GrandpaKeyActivated`
- `EmergencyRecoveryExecutionFailed`
- `EmergencyRecoveryClosed`

事件分别表达提案创建、调度、实际生效和紧急恢复关闭，不能把“已调度”解释为“已生效”。

## 10. 节点接口与界面

节点后端 `node/src/core/grandpa_rotation.rs` 提供：

- `build_grandpa_key_change_request`
- `submit_grandpa_key_change`
- `get_grandpa_key_change_status`

节点页面位于 `node/frontend/governance/grandpa-key/`。管理员选择目标机构委员任职，
输入本机解锁密码，完成管理员交易签名后提交。页面明确区分：

- 正常更换：无投票、旧新私钥双签、延迟生效；
- 紧急恢复：旧私钥不可用、目标机构委员内部投票。

非秘密的待处理状态保存为 `<app_data>/grandpa-key-change.json`；GRANDPA 私钥只保存
在节点 `gran` keystore，不写入该状态文件、日志或前端。

## 11. 测试与验收

Runtime 测试至少覆盖：

- call index `0/1` 连续且 SCALE 布局固定；
- 正常更换岗位授权、双签、nonce、过期和延迟生效；
- 紧急恢复只生成目标机构内部投票；
- NRC `13/19`、PRC `6/9` 的机构阈值来自现有机构配置；
- PRB、跨机构委员、无任职管理员和无权限岗位被拒绝；
- 无效、弱、相同、已使用和已预留公钥被拒绝；
- pending 冲突、回调重试、否决和确定失败清理；
- 只有实际 authority set 已切换后才更新当前公钥索引。

节点验收至少覆盖：

- 新私钥写入时保留旧私钥；
- finalized 前不删除旧私钥；
- finalized 状态确认新 authority 生效后删除旧私钥；
- 未提交且证明过期的新候选私钥可安全清理；
- 两条 call 均能被节点和离线钱包按同一二维码注册表解析。
