# 公民身份上链删除自报年龄,年龄单源 birth_date

任务需求：
删除公民身份上链载荷中的自报字段 `citizen_age_years`。年龄语义单源化:
- 投票身份不带年龄(只需 status=正常 + 护照有效期窗口 + 注册局 voting_eligible 判定能否投票);
- 竞选身份年龄一律由 `birth_date` 链上实时计算(age_from_birth_date ≥ 16 门保留)。
- onchina BFF 侧对投票/竞选保留 birth_date ≥ 16 防误推门(不入链,仅拦注册局误操作)。

背景:上一轮字段完整性核对发现 `citizen_age_years` 冗余(竞选已有 birth_date)且投票无需链上算龄。
用户裁决:第 4 条"只用 birth_date";第 1 条投票只需能否投票+状态正常。

所属模块：citizenchain/citizen-identity(冷签载荷真源)+ onchina + citizenwallet + citizenapp + qr-protocol registry

必须遵守：
- 冷签 SCALE 载荷字节布局跨四层锁步:链结构 ↔ onchina 编码器 ↔ citizenwallet 解码器 ↔ citizenapp 解码器,逐字节一致,否则 decodeFailed/盲签
- qr-protocol registry(fields.yaml + actions.yaml)与解码器 reviewFields 必须一致(registry_consistency 守卫)
- 生成物 qr_action_registry.g.dart(citizenapp + citizenwallet 两份字节相同)必须重生
- 链开发期:重新创世即可,无 migration;MIN_ONCHAIN_CITIZEN_AGE_YEARS / UnderVotingAge 保留(竞选仍用)
- 无残桩:删干净所有 citizen_age_years / ageYears 触点

输出物：
- 链:VotingIdentityPayload 删字段、ensure_valid_voting_payload 删≥16校验、revoke_identity 构造删字段
- onchina:chain_identity.rs 编码器删 age 字节 + 输出结构 + CitizenIdentityPayloadBytes;api.ts 删类型字段;保留 BFF ≥16 门
- citizenwallet:payload_decoder 删 age 读取/展示 + 长度校验 -1
- citizenapp:voting_identity_payload.dart 删 ageYears + 用法
- registry:fields.yaml 删定义 + actions.yaml 5 处 required_fields 删引用 → 重生两份 .g.dart
- 测试:链 4 处载荷 helper 删字段;under_sixteen 投票用例改为竞选 birth_date<16;钱包 2 测试;app 测试
- 文档 + 注释更新 + 残留清理

验收标准：
- cargo test -p citizen-identity / -p citizen-issuance / -p qr-protocol(registry_consistency + repo_guard)通过
- cargo check -p citizenchain / onchina 通过
- flutter analyze/test(citizenwallet 签名解码 + citizenapp myid)通过
- 全仓 grep citizen_age_years / citizenAgeYears / ageYears 零残留(除非有意保留说明)
- 两份 qr_action_registry.g.dart 已重生且一致

状态：已完成(2026-07-24)

落地记录：
- 链 citizen-identity:VotingIdentityPayload 删 citizen_age_years;ensure_valid_voting_payload 删≥16校验;
  revoke_identity 构造删字段。MIN_ONCHAIN_CITIZEN_AGE_YEARS / UnderVotingAge 保留(竞选按 birth_date 用)。
- onchina:chain_identity.rs 编码器删 age 字节 + 输出结构/PayloadBytes 删字段;api.ts 删类型字段;
  保留 birth_date≥16 BFF 防误推门(citizen_age_years() helper + CitizenDetailPage calculateAgeYears 都保留)。
- 冷钱包 payload_decoder + 公民 app voting_identity_payload:删 age 读取/展示,字节长度校验 -1。
- registry:fields.yaml 删定义 + actions.yaml 5 处;重生两份 qr_action_registry.g.dart(字节一致、零残留)。
- 测试:链 4 处 helper 删字段;under_sixteen 用例改为竞选 birth_date<16(UnderVotingAge);钱包 2 测试 + app 测试字节构造删 age。
- 文档:BACKEND_TECHNICAL / CITIZEN_IDENTITY_FLOW / qr-action-registry / CITIZENCHAIN_TECHNICAL 四处同步。
- 验证全绿:qr-protocol registry_consistency(7)、citizen-identity(36)、citizen-issuance(7)、
  citizenchain+onchina 编译、citizenwallet 签名(115)、citizenapp 载荷(9)。
- 遗留(非本次):qr-protocol repo_guard 因 citizenwallet/lib/signer/pallet_registry.dart:161 一条含
  "decodeFailed" 字样的既有未提交注释而红;该文件本次未改,committed 版无此串,属既有问题待另处理。
