# 任务卡：CitizenWallet 命名精简统一 + 残留清理

## 任务需求

执行 2026-07-11 命名审计「公民钱包 CitizenWallet」一类 21 条方案([[project_naming_audit_2026_07_11]]),对 `citizenwallet/` 精简+统一+清除;完成后更新文档、完善注释、彻底删除旧代码/注释/文档残留。不管迁移兼容。

**数量口径校正(用户指令):行政区/机构码数量以 `citizenchain/runtime/primitives/cid/code.rs` 为准。已核实 `INSTITUTION_CODE_INFOS = 104`、`PROVINCE_CODE_INFOS = 43`。CitizenApp 上一轮误改的 92 已回退为 104;CitizenWallet `institution_code.dart` 已是 104,无需改。**

## 执行

- 清洁改名:sigAlg→alg、_ss58Format→_ss58Prefix、k 前缀去除(kNationalCouncils→nationalCouncils 等 5 个)、submitOffchainBatchV2Call→submitOffchainBatchCall、registerCitizenIdentity→registerVotingIdentity、InstitutionType→OrgType、'公民冷钱包'→'公民钱包'。
- 文件/去重:qr/offline_sign_page.dart→ui/;_ScanOverlayPainter/_ScanCornerPainter(与 scan_page 逐字节重复)抽 ui/scan_overlay.dart 单源;scanBoxSize/scanBoxOffsetY 单源;'全部'×17→allGroup 常量。
- Isar 实体(build_runner 重生 wallet_isar.g.dart):删死字段 balance/walletIcon(含 wallet_manager.dart WalletProfile 镜像与管道)、ss58→ss58Prefix。
- 注释:signMode「只有热钱包」自相矛盾注释修正。

## 未做(读源码后判断,同 CitizenApp 口径)

challengeRaw/initialCode/raw→raw、_seedHex/私钥→seed 若为跨文件/UX 术语纠缠则按需处置;发现审计错误建议即跳过并报告。

## 验收

`flutter analyze` 零新增(基线 No issues)、build_runner 重生通过、`flutter test` 无因改名新增失败;残留旧名零引用;code.rs 数量口径一致(104/43)。

## 验收结果（2026-07-11 完成）

`flutter analyze` = **No issues found**(基线亦 clean,零新增)。`flutter test` = **144/144 全通过**(零失败零编译错误)。`build_runner build` 成功重生 wallet_isar.g.dart(删 balance/walletIcon、ss58→ss58Prefix)。残留旧名零引用。

**已执行:** 清洁改名(sigAlg→alg、_ss58Format→_ss58Prefix、k 前缀去除×5、submitOffchainBatchV2Call→submitOffchainBatchCall、registerCitizenIdentity→registerVotingIdentity、InstitutionType→OrgType、公民冷钱包→公民钱包、challengeRaw/initialCode→raw)+ 去重(_ScanOverlayPainter/_ScanCornerPainter 抽 ui/scan_overlay.dart 单源、scanBoxSize/scanBoxOffsetY 单源、'全部'×15→allGroup 单源)+ 文件移动(qr/offline_sign_page→ui/)+ Isar 实体(删死字段 balance/walletIcon、ss58→ss58Prefix,含 wallet_manager WalletProfile 与全部测试,build_runner 重生)+ 注释(signMode「只有热钱包」矛盾注释改「冷签设备恒本机签名」)。

**未执行:** `_seedHex/getSeedHex ↔ UI '私钥' → seed`——这是「代码标识符 vs 用户可见中文安全标签」的跨层术语,不是干净改名;把面向用户的「私钥」改成 seed 属产品/UX 决策且有安全语义风险,已跳过并报告。

**数量口径校正:** 已核实 code.rs = 104 机构码 / 43 省;CitizenApp 上轮误改的 92 已回退 104;CitizenWallet institution_code.dart 本就 104;stale memory [[feedback_cid_code_table_level_final]] 已更正为「以 code.rs 为准 = 104」。
