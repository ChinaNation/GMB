# 任务卡:joint_vote 冷钱包展示 institution_id

- 时间:2026-04-22(跟进项)
- 状态:open
- 归属:Mobile Agent(wumin)
- 承接:`20260422-cold-wallet-two-color-recognition.md`(主任务卡 PR-C)

## 背景

`VotingEngineSystem::joint_vote(proposal_id, institution_id_48, approve)`
的 call payload 包含 48 字节 `institution_id`(shenfen_id),决定"以哪个
机构身份投票"。

当前 wumin `payload_decoder.dart:303-320` 的 `_decodeJointVote` 在解析
时跳过该字段,`decoded.fields` 只回填 `proposal_id` / `approve`。
用户在冷钱包上无法看到"投哪个机构身份",存在一票投错机构的 UX 风险。

## 现状

- Registry `qr-action-registry.md` joint_vote fields = `proposal_id`, `approve`
  (已对齐 decoder 现状 2026-04-22,PR-C 落地)
- decoder `_decodeJointVote` 跳过 institution bytes
- 节点 Tauri UI `signing.rs:270` 也不发 institution_id 到 display.fields
- wuminapp `runtime_upgrade_detail_page.dart:332-343` 同不发

## 目标

冷钱包扫 `joint_vote` QR 时展示 `institution_id → 机构中文名`,
用户可独立确认"投哪个机构"。

## 落地步骤

1. decoder `_decodeJointVote` 解出 48B institution bytes,trim 尾零后
   按 `institutions.dart` 的 `institutionName()` 查表转机构中文名
   (找不到回退原 shenfen_id 字符串)
2. 返回 `decoded.fields` 增加 `institution_id` 项
3. Registry `qr-action-registry.md` joint_vote fields 改回
   `(proposal_id, institution_id, approve)`,注释修改
4. 节点 Tauri UI `signing.rs:270` `build_joint_vote_sign_request` 的
   `display.fields` 增加 `institution_id` 条目(value 查节点端
   institution 查表,与 decoder 输出字面一致)
5. wuminapp `runtime_upgrade_detail_page.dart:332` 的 SignDisplayField
   补 `key: 'institution_id'` 条目(value 取 `widget.institution.name`
   或从 institution.shenfenId 查表)
6. wumin `payload_decoder_test.dart` joint_vote 测试增补 institution_id
   字段断言
7. Grep 无 `institution_id` 残留不一致

## 验收

- wumin 扫 joint_vote QR → 🟢 绿色,fields 包含机构中文名
- fields 三端字面对齐
- 手工构造 institution_id 48B 非法内容 → decoder 回退原 bytes 展示,
  不影响识别判定

## 关联

- 主任务卡:`20260422-cold-wallet-two-color-recognition.md`
- Registry:`memory/05-architecture/qr-action-registry.md`
- decoder:`wumin/lib/signer/payload_decoder.dart` `_decodeJointVote`
- UI 入口:`wuminapp/lib/governance/runtime_upgrade_detail_page.dart` +
  `citizenchain/node/src/ui/governance/signing.rs`
