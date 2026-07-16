# 卡2 · CitizenApp 私密小群聊天（MLS 端到端，成员 ≤1989，Cloudflare 零存储）

任务需求：在现有 1:1 端到端聊天之上新增**私密小群**：多人 E2E 群聊，单群成员上限 **1989**；沿用 MLS 加密与 Cloudflare 零存储瞬时中转，不在服务端存明文/密文。与卡3（大频道，非 E2E，10万+）双轨互不干扰。
所属模块：citizenapp / chat（+ rust/chat_mls FFI + cloudflare/chat 中转）

## 定稿方向（承接 roadmap 决策，见 [[project_chat_media_group_roadmap_2026_07_15]]）

- 10万人+E2E+零存储三约束互斥、业界无成品；**私密小群走 E2E（MLS 群），封顶 ≤1989**（与 WhatsApp/Signal 同量级，MLS 容量可行）。
- 大群零存储扇出下"离线收大媒体"体验降级；**群媒体口径是本卡待决子项**（见下"关键决策"）。
- 10万+ 的公共广播归卡3（复用广场，非强制 E2E），不在本卡。

## 现状事实（已核实，非推断）

- **MLS 已是群原生**：`rust/src/chat_mls.rs` 用 OpenMLS `MlsGroup`（`new_with_group_id`/`add_members`/`create_message`/`process_message`），当前仅按会话建 2 人群（1:1）。FFI 经 `lib/chat/crypto/mls_native.dart`（`createKeyPackage`/`encrypt`/`decrypt`/`processIncoming`）。→ 群能力**主要缺 FFI 批量成员操作 + Commit 处理的暴露**，不是从零造密码学。
- **信封是单收件人**：`chat/proto/chat_envelope.proto` 的 `ChatEnvelope.recipient_account` 单值。→ 群消息=**发送端扇出**：MLS 群密文只加密一次，按成员数封 N 个信封（同 `mls_wire_message`、不同 `recipient_account`）。
- **投递=瞬时中转+唤醒**：`cloudflare/src/chat/service.ts` 用 `relayChatPayload`（WebSocket 实时中转）+ `sendChatWake`（离线推送唤醒），不存内容。→ 群扇出复用此机制，按成员逐个 relay/wake。
- **会话/密钥本地态**：Isar（`lib/isar/app_isar.dart`）+ MLS 状态库（`crypto/mls_state_store.dart`）。→ 群需扩：群会话（group_id/名称/成员名册/epoch/创建者）、成员名册。
- 门禁：广场/聊天会话签发时已校验签名钱包链上活账户（≥ED，见 [[project_square_chat_onchain_wallet_gate]]）。群沿用。

## 架构

```
发送端                                   Cloudflare(零存储)          每个成员设备
sendGroupText
  └ MLS group.create_message(1次加密)  ── N×ChatEnvelope ──▶ relay/wake ──▶ processIncoming
        (同密文, N 个 recipient)                                              └ group.process_message → 解密

建群/加人
  └ MLS group.add_members([kp…]) → Commit + N×Welcome
        Commit  ── 扇出给现有成员 ──▶ 各自 process_message(更新 ratchet/epoch)
        Welcome ── 发给新成员     ──▶ 新成员 process_welcome(入群)
```

- **单加密多扇出**：群密文一次加密（MLS 群 epoch 密钥），发送端按名册封 N 信封。服务端只中转，零存储不变。
- **成员变更走 MLS Commit/Welcome**：加人产 Commit（现有成员处理）+ Welcome（新成员入群）；删人/退群产 Commit（前向保密/后向保密由 MLS 保证）。
- **epoch 有序性**：Commit 改 epoch；乱序 Commit 需缓冲/重放（MLS 要求按 epoch 顺序处理），复用现有 pending inbound 缓冲思路。

## 目录结构（新增/改）

```
rust/src/chat_mls.rs                 # FFI 扩:create_group / add_members(N) / remove_members / process_commit / group_create_message / group_process
lib/chat/crypto/mls_native.dart      # 上述 FFI 的 Dart 绑定
lib/chat/crypto/mls_boundary.dart    # 群操作的边界类型(GroupCommit/Welcome/Roster)
lib/chat/group/
  group_model.dart                   # ChatGroup(group_id/name/roster/epoch/creator/adminSet)
  group_flow.dart                    # 建群/加人/删人/退群/发群消息(扇出) 编排,可测核心
  group_membership.dart              # 名册增删 + ≤1989 上限守卫 + 权限(仅 admin 加/删)
  group_fanout.dart                  # 单密文 → N 信封扇出(可测,与传输解耦)
lib/isar/app_isar.dart               # ChatGroupEntity(group_id 唯一/name/epoch/creator) + ChatGroupMemberEntity(group_id 索引/account/role)
lib/chat/chat_runtime.dart           # sendGroupText/createGroup/addMembers/removeMembers/leaveGroup 接线 + 群扇出投递 + Commit/Welcome 收发
lib/chat/storage/chat_store.dart     # 群会话/名册读写 + 群消息落库(复用消息表, conversation_id=group_id)
lib/chat/group/ui/
  group_create_page.dart             # 建群(选联系人, 拉其 key package)
  group_manage_page.dart             # 成员管理(加/删/退, 仅 admin)
  (群聊详情复用 chat_page, 传 group adapter)
cloudflare/src/chat/service.ts       # 若需:批量扇出端点(可选;也可发送端逐个调现有单发端点)
```

## 分阶段实现（每阶段先出细化方案确认后执行）

- **阶段1 · MLS 群原语 + 文本扇出（地基）**：Rust FFI 暴露批量 add_members/remove_members/create_group/process_commit/group message；`mls_native` 绑定；`group_flow` 建群+发文本；`group_fanout` 单密文扇 N 信封；成员 Commit/Welcome 收发与 epoch 有序处理；Isar 群/名册实体；≤1989 上限守卫。**只做文本群，验证 3+ 成员端到端往返 + 加人/退群**。
- **阶段2 · 成员管理 UI + 权限**：建群页（选联系人拉 key package）、成员管理页（加/删/退，仅 admin）；群聊详情复用 chat_page；名册变更实时反映。门禁沿用链上活账户校验。
- **阶段3 · 群媒体（见关键决策定口径后做）**：按下方决策实现群内图片/视频；沿用 2a 流式 + 四门门控 + 大小上限（图100MB/视频文件5GB）。

## 关键决策（群媒体，本卡必须先拍板一处）

1:1 媒体走 WebRTC P2P 直传；**1989 人 P2P 网格不可行**。三个候选：
- **A（推荐 v1）· 仅在线直传 + 缩略图入群**：大媒体只在发送时对**当前在线成员**逐个 WebRTC 直传（复用 2d 传输）；缩略图/blurhash 走 E2E 信封让所有成员可见占位；离线成员上线后"内容已过期，可请求重传"（发送方在则补，不在则不可得）。**维持零存储、无新服务端存储**，代价=离线成员大媒体体验降级（roadmap 已认此取舍）。
- **B · 短 TTL 加密中转**：媒体密文（群 epoch 密钥加密）经服务端**短 TTL**（如 24h）中转桶暂存，所有成员可拉。破"零存储"红线（虽密文+短 TTL），需单独批准。
- **C · 群内禁大媒体**：群仅文本+贴纸+emoji+缩略图占位，大媒体给"转 1:1 发送"引导。最省、最稳。

→ 默认按 **A**；若要 B（破零存储）或 C（禁大媒体）请在起卡时定。

## 必须遵守

- **Cloudflare 零存储不变**（除非选决策 B 并显式批准）：群密文只瞬时中转，不新增群媒体存储路由。
- MLS 群 epoch 处理必须**有序**（乱序 Commit 缓冲重放），否则群解密链断裂——这是最大正确性风险。
- 成员上限 **1989 硬守卫**（发送端 + 建群/加人处双拦）。
- 权限：仅创建者/admin 可加删成员;退群任何人可。
- 不动 citizenchain;门禁沿用链上活账户校验,不放宽。
- 开发期零用户：Isar 群 schema 直接重建，不做迁移/兼容。

## 验收标准

- 3+ 成员建群、发文本，全员端到端收发并正确显示；加人（新成员收 Welcome 入群、看后续消息）、删人/退群（被删者收不到后续、MLS 后向保密）验证通过。
- 成员数达 1989 上限时加人被拒并提示。
- Cloudflare 抓路由确认无群内容落库（零存储）。
- 群媒体按选定决策验收（A：在线成员可见、离线降级提示）。
- `flutter analyze lib/chat` 0 问题；群 flow/fanout/membership/epoch 有序处理有单测；对抗式审查每阶段跑。

## 风险

- **epoch 有序性 / 乱序 Commit**：MLS 群最易碎处,乱序或丢 Commit 致该成员解密链断。缓冲+重放+缺失检测必须做实,重点测。
- **扇出放大**：1989 成员每条消息 1989 信封,发送端网络/电量成本;需批量+背压,弱网降级。
- **Welcome/KeyPackage 供给**：加人需被加者有可用 key package;耗尽处理（复用 1:1 的 fetch/consume）。
- **成员一致性**：名册与 MLS 群成员必须一致,否则漏发/多发;以 MLS 群成员为准,名册为镜像。

## 定案更新(2026-07-16)

- **完整技术方案已落文档**:`memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md`(单一技术真源,与 1:1 CHAT_TECHNICAL.md 并列)。以该文档为准。
- **无 proto 改动**(工具链复核后优化):`protoc-gen-dart` 缺失,且收端本靠 Rust `group_process` 依 `MlsMessageBodyIn` 判别 welcome/commit/application,envelope `mls_message_kind` 仅 Dart 参考标签、不参与正确性。故复用现有 `WELCOME`/`APPLICATION` 两 wire kind;群 vs 私聊按 `conversation_id` 前缀 `grp:` 路由 + `ChatConversationEntity.conversationKind`。
- **FFI 微调**:`process_commit`+`group_process` 合并为单 `group_process`(底层 `process_message` 统一入口);另加只读 `group_state`(名册对账 + Rust 1989 硬拦)。共 6 个:create/add_members/remove_members/create_message/process/state。
- **群媒体口径定案 = A(分级) + >100MB 星火 Cloudflare 瞬时密文中转**([[project_chat_media_tiered_relay_2026_07_15]]):≤100MB P2P;>100MB(仅星火)客户端群密钥流式加密→中转→拉完/短 TTL 删(需 R2+过期,反转 Chat 禁 R2,阶段3 前显式认)。会员窗口 ADR-036 已确认该 transport 归本卡阶段3、本卡只用其档位/限额值。文件上限单源来自会员 `MembershipPlan.chat_file_max_bytes`(freedom 10MB/democracy 100MB/spark 5GB)。
- **执行位置**:主检出 `/Users/rhett/GMB`(遵死规则),不碰 worktree。

## 阶段1 落地(2026-07-16 完成,测试通过)

- **Rust**:6 群 FFI + 1989 硬拦;`cargo test chat_mls::` 3 绿(含群多方 round-trip 后向保密)。
- **Dart**:`crypto/mls_group_boundary`+`mls_native`(6 绑定)、`group/{model,limits,fanout,membership,epoch,flow}`、3 Isar 实体 +`conversationKind`、`chat_store` 群方法、`chat_runtime` 接线(建群/加删/退群/群发 + `grp:` 入站路由)。
- **测试**:`test/chat/group/` 8 绿(纯模块 fanout/membership/epoch + flow 全链路 + 非 admin 拒);`flutter analyze lib/chat` 0。
- **微调**:无 proto 改动(grp: 前缀路由)、删人账户级、名册全量覆盖对账、process_commit 并入 group_process、加 group_state。
- **未做转阶段2**:退群暂为本机 leftLocally(admin removeMembers 已保证密码学后向保密);建群/成员管理/群聊 UI;群媒体转阶段3。
- as-built 详见 `memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md §15`。

## 关联

[[project_chat_media_group_roadmap_2026_07_15]] 双轨决策 · [[project_chat_media_tiered_relay_2026_07_15]] 媒体分级中转 · [[project_membership_identity_decoupling_2026_07_15]] 会员解耦(另窗 ADR-036) · [[project_square_chat_onchain_wallet_gate]] 门禁 · 卡3=大频道 `20260715-citizenapp-channel-broadcast-square.md` · 复用媒体传输见 `CHAT_TECHNICAL.md`(2a-2d) · 完整方案 `memory/05-modules/citizenapp/chat/CHAT_GROUP_TECHNICAL.md`。
