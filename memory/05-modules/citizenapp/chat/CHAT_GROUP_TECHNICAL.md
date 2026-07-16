# CitizenApp 私密小群 E2E 技术架构

> 本文档是私密小群(MLS 端到端群聊,成员 ≤1989,Cloudflare 零存储)的单一技术真源,与 1:1 的 [CHAT_TECHNICAL.md](CHAT_TECHNICAL.md) 并列。任务卡:`memory/08-tasks/open/20260715-citizenapp-chat-group-private-e2e.md`。与卡3(大频道,非 E2E)双轨互不干扰。

## 0. 边界与不变量

- 单群成员 ≤ **1989**(发送端 + 建群/加人 Dart 拦 + Rust `group_add_members` 硬拦,双守)。
- 文本 / 贴纸 / emoji / 缩略图 **始终 E2E 零存储**;媒体按会员分级(见 §11)。
- 加密 = OpenMLS 群(**单次加密**),投递 = **发送端扇出 N 信封**(同一密文,N 个 `recipient_account`),服务端只瞬时中转、零存储不变。
- 成员变更走 MLS **Commit/Welcome**(前向/后向保密由 MLS 保证)。
- 名册以 **MLS 群成员为唯一真源**,Isar 为镜像,每次 Commit 后对账。
- 门禁沿用会话签发时的**链上活账户校验**,不放宽;不动 citizenchain;开发期零用户 → Isar 群 schema 直接重建,无迁移/兼容。

## 1. 总体架构

```text
发送端 A                              Cloudflare(零存储)          每个成员设备
sendGroupText
 └ group_create_message(1 次加密) ── N×ChatEnvelope ──▶ relay/wake ──▶ group_process
      同一份密文,N 个 recipient_account                              └ 解密 application

建群/加人
 └ group_add_members([kp…])
      ├ Commit  ── 扇给现有成员 ──▶ group_process → merge_staged_commit(进 epoch e+1)
      └ Welcome ── 扇给新成员   ──▶ group_process → StagedWelcome.into_group(入群 @ e+1)

删人/退群
 └ group_remove_members([idx…])
      └ Commit  ── 扇给剩余成员 + 被删者 ──▶ 剩余:重钥;被删者:自身 leaf 移除→群失活
```

投递复用既有事实:`chat_cloud_transport.dart` 从 `envelope.recipientAccount` 取收件人 → `cloudflare/src/chat/service.ts:submitChatEnvelope` 按此 `relayChatPayload`+`sendChatWake`,不解析密文。群消息即"同一 `mls_wire_message` 封 N 个不同 `recipient_account` 的信封",逐个走**现有** `sendEncryptedEnvelope`,**阶段 1-2 服务端零改**。

## 2. 协议与路由(无 proto 改动)

- **不新增 wire kind**:复用 `MlsWireMessageKind` 的 `WELCOME`/`APPLICATION`。Commit 也是 MLS protocol message,发送时按 `application` 标签封装;收端由 Rust `group_process` 依 `MlsMessageBodyIn` 重新判别 welcome/commit/application,envelope 标签仅供 Dart 参考,不参与正确性判定。
- **群 ID**:`group_id = "grp:<creatorAccount>:<nonce16>"`,即 `conversation_id`,全成员一致(经 Welcome 里的 `GroupId` 分发)。
- **群 vs 私聊路由**:`conversation_id` 前缀 `grp:` → 群路径(`ChatGroupFlow`);`dm:` → 现有 1:1 路径。同时 `ChatConversationEntity.conversationKind` 持久化便于列表渲染。

## 3. Rust FFI(`rust/src/chat_mls.rs` 扩展)

沿用现有 `gmb_chat_mls_*` 的 `#[no_mangle] extern "C"` + JSON-in/JSON-out + `error_out` + `crate::{string_into_raw,set_error}` + `smoldot_free_string` 模式,复用 `load_provider`/`save_provider`/`ensure_device_signer`/`mls_group_config`/`group_id_from_conversation`。

对任务卡函数清单两处微调:①把 `process_commit`+`group_process` 合并为单 `group_process`(底层 `MlsGroup::process_message` 本就统一入口);②另加只读 `group_state`(名册对账 + Rust 侧 1989 硬拦)。

| FFI `gmb_chat_mls_*` | Request | Response |
|---|---|---|
| `group_create` | `{state_store_dir, owner_account, device_id, group_id}` | `{group_id, epoch}` |
| `group_add_members` | `{…, group_id, key_packages_hex:[…]}` | `{group_id, epoch, commit_wire_hex, welcome_wire_hex, ratchet_tree_hex}` |
| `group_remove_members` | `{…, group_id, member_identities:["acct:dev",…]}` | `{group_id, epoch, commit_wire_hex, removed_identities:[…]}` |
| `group_create_message` | `{…, group_id, plaintext_hex}` | `{group_id, epoch, application_wire_hex}` |
| `group_process` | `{…, group_id, wire_message_hex, ratchet_tree_hex?}` | `{group_id, message_kind, status, message_epoch, group_epoch, plaintext_hex?, added_members:[…], removed_members:[…], self_removed}` |
| `group_state` | `{…, group_id}` | `{group_id, epoch, member_count, member_identities:[…]}` |

OpenMLS 对应:`new_with_group_id` / `add_members`+`merge_pending_commit` / `members()`匹配 identity→`LeafNodeIndex`+`remove_members`+merge / `create_message` / `process_message`→`StagedCommitMessage`→`merge_staged_commit` / `load`+`epoch()`+`members()`。

**`group_process` 分派与 epoch 判定**(收端唯一入口):

```text
peek message_epoch = protocol_message.epoch()      // 处理前先读,不改状态
current = group.epoch()
Welcome     → StagedWelcome::into_group          ; status=applied, kind=welcome
Commit      → message_epoch > current : status=out_of_order(不处理)
              message_epoch < current : status=stale(丢弃)
              == : process_message → merge_staged_commit
                   added = staged.add_proposals(), removed = staged.remove_proposals()
                   self_removed = removed 含本机 leaf ; status=applied, kind=commit
Application → message_epoch != current : status=out_of_order/stale
              == : decrypt → plaintext_hex ; status=applied, kind=application
```

**1989 硬拦**:`group_add_members` 先 `member_count + N ≤ 1989` 否则 Err。Rust 只做密码学,不判应用层权限(权限在 Dart)。

## 4. Dart crypto 层

```dart
// crypto/mls_group_boundary.dart —— 可注入接口,单测用 fake
abstract class MlsGroupCrypto {
  Future<GroupCreated> createGroup(String groupId);
  Future<GroupCommitBundle> addMembers(String groupId, List<MlsKeyPackage> kps);
  Future<GroupCommitBundle> removeMembers(String groupId, List<String> identities);
  Future<MlsWireMessage> groupCreateMessage(String groupId, List<int> plaintext);
  Future<GroupInbound> groupProcess(MlsWireMessage wire);
  Future<GroupState> groupState(String groupId);
}
// GroupCommitBundle{commit:MlsWireMessage, welcome:MlsWireMessage?, epoch}
// GroupInbound{kind, status(applied|outOfOrder|stale), messageEpoch, groupEpoch,
//              plaintext?, addedMembers[], removedMembers[], selfRemoved}
// GroupState{epoch, memberAccounts[]}
```

`NativeMlsCrypto` 实现之(`MlsNativeBindings` 加 6 个 `lookupFunction`)。

## 5. `lib/chat/group/`

| 文件 | 职责 | 可测性 |
|---|---|---|
| `group_model.dart` | `ChatGroup{groupId,name,creator,epoch,adminSet,roster}` | 纯 |
| `group_fanout.dart` | `(wire, recipients[], sender/convId/deviceId) → List<ChatEnvelope>`;同密文异 recipient | **纯,零传输依赖** |
| `group_membership.dart` | ≤1989 上限守 + 权限(仅 `adminSet` 加/删) | 纯 |
| `group_epoch.dart` | 乱序 Commit 缓冲/回放(注入 process+buffer seam) | **纯** |
| `group_flow.dart` | create/add/remove/leave/sendText/processIncoming 编排;复用 `deliverer`+`ChatStore` | 核心可测(注入 crypto/store) |

**扇出对象**:add 的 Commit→现有成员(减自己)、Welcome→新成员;remove 的 Commit→剩余成员 **+ 被删者**;text 的 application→全体名册(减自己)。

## 6. epoch 有序性(最大正确性风险)

```text
收到 Commit 信封:
  r = groupProcess(wire)
  applied    : epoch→r.groupEpoch; group_state 对账名册; drainBuffer(groupId, r.groupEpoch)
  outOfOrder : bufferPut(groupId, r.messageEpoch, envelope)   // ChatGroupPendingCommitEntity
  stale      : 丢弃
drainBuffer(groupId, e):
  while row = bufferTake(groupId, messageEpoch == e):
     groupProcess(row); 若 applied 使 epoch→e+1: e=e+1 继续
入群前(未处理 Welcome)到达的 Commit/Application:
  复用现有 pending-inbound(键 conversationId=groupId);Welcome 处理成功后 drain
```

application 若因群已推进而 stale(密钥已 ratchet 掉)→ 标"需重发",不静默丢。

## 7. 成员生命周期

| 操作 | 流程 |
|---|---|
| 建群 | `group_create`(创建者=唯一成员)→ 落 `ChatGroupEntity`+名册(自己=admin) |
| 加人 | Dart 上限/权限拦 → 逐个领 KeyPackage(复用 1:1 fetch/consume)→ `group_add_members`(批量 N,1 Commit+1 Welcome)→ Commit 扇现有、Welcome 扇新人 → 更新 epoch/名册 |
| 删人 | admin 权限拦 → `group_remove_members` → Commit 扇剩余+被删者 → 更新 |
| 退群 | 本机即刻标 `leftLocally` 并停发 + 发 `leave_request` application 给群;admin 在线时自动 `group_remove_members([leaver])` 重钥。不新增 FFI |
| 被删感知 | `group_process(commit).selfRemoved==true` → 本机标群已移除、停处理 |

> 退群取舍:标准 MLS 成员不能自我 commit 移除,v1 选"admin 代提交"(落在 6 FFI 内)。admin 无关的即时密码学退群需另加 self-remove proposal FFI,默认不加。

## 8. Isar 实体(开发期直接重建)

```dart
ChatGroupEntity           { groupId(唯一), groupName, creatorAccount, ownerAccount(索引),
                            epoch, memberCount, leftLocally, createdAtMillis, updatedAtMillis }
ChatGroupMemberEntity     { groupId(索引), memberAccount, role(admin|member), joinedAtMillis }
ChatGroupPendingCommitEntity { groupId(索引), messageEpoch(索引), envelopeBytesHex, createdAtMillis }
ChatConversationEntity 加 : conversationKind = "dm" | "group"
```

消息流复用 `ChatMessageEntity`(`conversation_id=group_id`);会话列表读 `ChatGroupEntity` 取群名/人数。

## 9. 上限 / 权限 / 门禁

- 上限单源常量 `chat_group_limits.dart: kMaxGroupMembers = 1989`;Dart(建群邀请+创建者、加人 `当前数+N`)+ Rust(MLS 实际成员数+N)双拦。
- 权限:`adminSet`(默认=创建者)可加/删;退群任何人可。
- 门禁:群会话签发沿用链上活账户校验,阶段 1-2 零改。

## 10. 扇出与投递

单密文 → `group_fanout` 生成 N 个 `ChatEnvelope`(逐个换 `recipient_account`)→ `ChatFlow.deliverWithTransport` 逐个投递;transport 按 envelope 内 `recipient_account` 路由;离线成员由 `sendChatWake` 唤醒 + 发送端队列重试(复用 1:1 队列)。阶段1 串行发,背压/批量降级留阶段2。

## 11. 群媒体(阶段3,口径见 [[project_chat_media_tiered_relay_2026_07_15]])

- 文件上限按会员档(单源来自会员卡 `MembershipPlan.chat_file_max_bytes`):无订阅/自由 ≤10MB、民主 ≤100MB、星火 ≤5GB;`chat_media_limits.dart` 由固定值改按档读。
- **≤100MB**:WebRTC P2P 端到端,群内=发送端扇出(在线直传 + 缩略图/blurhash 入群占位 + 离线降级)。
- **>100MB**(仅星火):客户端群 epoch 密钥流式分块 AEAD 加密 → Cloudflare 瞬时中转密文,全员拉取解密(E2E 不破),拉完/短 TTL 后删。**需引入 R2+过期,反转"Chat 禁 R2",阶段3 前显式认**。会员窗口 ADR-036 已确认该 transport 归本卡阶段3。

## 12. 分阶段

- **阶段1 地基**:6 Rust FFI + `MlsGroupCrypto` + `group/` + epoch 有序 + 3 Isar 实体 + 1989 双拦。只文本,验 3+ 成员端到端往返 + 加人/退群。
- **阶段2 UI+权限**:建群页、成员管理页、群聊详情复用 `chat_page`。
- **阶段3 群媒体**:按 §11。

## 13. 测试与验收

- 单测:`group_fanout`、`group_membership`(上限/权限)、`group_epoch`(乱序缓冲+回放+stale)、`group_flow`(fake crypto)。
- Rust:`cargo test` 群多方 round-trip(建群→加 2→发消息→删 1)。
- 集成:3+ 成员建群发文本全员正确显示;加/删/退(后向保密);1989 加人被拒。
- `flutter analyze lib/chat` 0;每阶段对抗式审查。

## 14. 风险

| 风险 | 对策 |
|---|---|
| epoch 乱序/丢 Commit(最碎) | 缓冲+回放+缺失检测,纯可测逻辑,重点测 |
| 名册不一致→漏发/多发 | MLS `group_state` 为真源,每 Commit 后对账覆盖镜像 |
| 扇出放大 1989× | 阶段1 串行复用队列/重试;背压/批量降级留阶段2 |
| KeyPackage 耗尽 | 复用 1:1 fetch/consume;缺则跳过+提示 |
| 退群依赖 admin 在线 | 本机即刻停发;admin 上线补重钥;可选 self-remove FFI |

## 15. 当前状态(as-built)

**阶段1 已完成并测试通过(2026-07-16):**

- **Rust**(`rust/src/chat_mls.rs`):6 个群 FFI `gmb_chat_mls_group_{create,add_members,remove_members,create_message,process,state}_json` + 名册辅助;`MAX_GROUP_MEMBERS=1989` 硬拦。`cargo test chat_mls::` 3 绿,含群多方 round-trip(建群→加 2→发文本双端解密→删 1:被删者 `self_removed=true`、剩余名册对齐,后向保密)。
- **Dart**:`crypto/mls_group_boundary.dart`(接口+边界类型)、`crypto/mls_native.dart`(6 绑定,`NativeMlsCrypto implements MlsGroupCrypto`)、`group/{group_model,chat_group_limits,group_fanout,group_membership,group_epoch,group_flow}`、3 个 Isar 实体(`ChatGroupEntity/ChatGroupMemberEntity/ChatGroupPendingCommitEntity`)+ `ChatConversationEntity.conversationKind`、`chat_store` 群方法、`chat_runtime` 接线(`createGroup/addGroupMembers/removeGroupMembers/leaveGroup/sendGroupText` + 入站按 `grp:` 前缀路由到群 flow)。
- **测试**:`test/chat/group/` 共 8 绿——纯模块(fanout 单密文扇 N、membership 1989/权限、epoch 乱序缓冲+回放)+ flow 全链路(建群→发文本→收文本→删人,fake 密码学 + 真 Isar)+ 非 admin 加人被拒。`flutter analyze lib/chat lib/isar/app_isar.dart` 0 问题。

**落地期确认的方案微调(相对上文设计):**

- **无 proto 改动**:复用 `welcome`/`application` 两 wire kind;群 vs 私聊按 `conversation_id` 前缀 `grp:` 路由。
- **删人=账户级**:`group_remove_members` 收 `member_accounts`,移除该账户在群内**全部设备叶子**;返回 `removed_accounts`。
- **名册对账**:Rust `group_process`(commit applied)与 `group_state` 都回吐**全量 `member_identities`**,Dart 据此**整体覆盖**镜像;"谁加入/谁退出"的系统消息由 Dart 差分旧镜像得出(不在 Rust introspect proposal,更鲁棒)。
- **`process_commit`+`group_process` 合并**为单 `group_process`;另加只读 `group_state`。

**阶段1 未做(转阶段2):**

- **退群**当前 = 本机 `leftLocally`(停止参与)。"leave_request → admin 自动 `removeMembers` 重钥"需一个群控制消息载荷,归阶段2;**后向保密的密码学移除已由 admin `removeMembers` 保证并测**。
- **UI**(建群页/成员管理页/群聊详情复用 `chat_page`)= 阶段2。
- 群媒体 = 阶段3(§11)。
