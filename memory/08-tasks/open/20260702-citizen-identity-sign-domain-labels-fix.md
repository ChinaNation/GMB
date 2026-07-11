# 公民身份签名域统一残留修复 + 确认页字段标签补全

## 背景(诊断结论)

公民身份上链"录入钱包"流程(OnChina `chain_identity.rs` ↔ 钱包扫码签名)在
仓库 HEAD 上验签链逐字节一致:三端统一 `blake2_256(GMB || 0x10 || payload)`
(citizenwallet `qr_signer.dart` / onchina `chain_identity.rs` / runtime
`configs/mod.rs`),该域三处同生于提交 5c8374185(2026-06-30)。现场
"签名不正确"由旧构建钱包(5c8374185 之前的中间构建直签裸 payload)与新后端
域验签不匹配产生。诊断中确认全仓仍存四项缺陷:

1. 跨端一致性缺回归防线:citizenapp 侧 0x10 域无测试锚点;现场设备需重装新构建。
2. citizenapp `signingBytesForHex` 缺 `citizen_identity`(0x10)分支——
   "统一签名域"提交(617348d8d)只补了 0x1A;且该函数手工拼 GMB 前缀,
   违反 `signing.dart` 单源纪律。
3. citizenapp `MyIdSignPage`(电子护照→扫码签名)是残桩:载荷门卡在已废弃的
   `citizen-identity-v1` 管道文本格式(全仓唯一残留),且直签裸 payload 不走
   0x10 域;公民用 CitizenApp 持有投票钱包时该页是唯一签名入口,现流程走不通。
4. citizenwallet 离线签名确认页 `_fieldLabel` 翻译表漏登记 6 个公民身份
   reviewFields key(registrar_account / wallet_account / citizen_age_years /
   valid_range / citizen_status / residence),全部显示"未知字段"。

## 任务目标

- citizenapp `signingBytesForHex` 补 0x10 分支,并将 0x10/0x1A 两分支收敛到
  `signing.dart::signingMessage` 单源,删除本地 GMB 前缀副本。
- 重写 `MyIdSignPage`:解码链上 `VotingIdentityPayload` SCALE 载荷(两色识别,
  解不开拒签),向公民展示中文字段确认后,按 0x10 域签名;删除
  `citizen-identity-v1` 残桩。
- citizenwallet 确认页字段翻译表补 6 个公民身份条目,并抽到独立可测文件。
- 补跨端回归测试:citizenapp 0x10 域测试、载荷解码测试、citizenwallet 标签测试。
- 完成后更新 `qr-action-registry.md`、完善中文注释、清理残留、回写本卡。

## 涉及范围

- `citizenapp/lib/signer/qr_signer.dart`
- `citizenapp/lib/my/myid/myid_sign_page.dart`
- `citizenapp/lib/my/myid/voting_identity_payload.dart`(新)
- `citizenapp/test/signer/qr_signer_test.dart`
- `citizenapp/test/my/myid/voting_identity_payload_test.dart`(新)
- `citizenwallet/lib/signer/field_labels.dart`(新)
- `citizenwallet/lib/qr/offline_sign_page.dart`
- `citizenwallet/test/signer/field_labels_test.dart`(新)
- `memory/01-architecture/qr/qr-action-registry.md`

## 边界

- 链端(runtime / onchina 后端)零改动——验签链已一致,不碰。
- 不改 QR envelope 协议与动作码。
- 不执行 `git push`,不创建 PR。
- 现场设备重装新构建由用户执行(代码侧只提供回归防线)。

## 验收

- citizenapp `signingBytesForHex(action=2)` 输出与
  `signingMessage(kOpSignCitizenIdentity, payload)` 逐字节相等,测试断言。
- `MyIdSignPage` 能解码 `VotingIdentityPayload`、展示中文字段、签 0x10 域;
  `citizen-identity-v1` 全仓 0 引用。
- citizenwallet 确认页公民身份 7 个 reviewFields key 全部显示中文标签,
  测试断言无"未知字段"。
- `flutter analyze` 两端无新告警,相关测试全绿。
- `qr-action-registry.md` a=2 行登记 CitizenApp 签名方。

## 实现记录

- 2026-07-02 完成,链端零改动,全部落在两个移动端与文档:
  - citizenapp `signingBytesForHex` 补 `citizen_identity`(0x10)分支,
    0x10/0x1A 两分支统一改调 `signing.dart::signingMessage`,删除本地
    历史 GMB 前缀和聊天钱包绑定签名域副本（单源纪律归位）。
  - 新增 `citizenapp/lib/my/myid/voting_identity_payload.dart`:
    `VotingIdentityConsentPayload` 独立解码 SCALE 载荷(严格校验:年龄≥16、
    日期合法、状态 0/1、恰好消费完字节),输出中文确认条目;文件头登记
    链端/钱包端/本文件三处同步纪律。
  - 重写 `myid_sign_page.dart`:删除 `citizen-identity-v1` 管道文本残桩与
    裸 payload 直签;新流程 = 扫码 → 独立解码展示中文字段(两色识别,
    解不开拒签)→ 公民确认 → 0x10 域签名 → 展示回执二维码;补签名前
    过期复查。
  - citizenwallet 确认页字段翻译抽到 `lib/signer/field_labels.dart`
    (公开可测),补 6 个公民身份条目:注册机构账户/公民钱包账户/周岁年龄/
    护照有效期/身份状态/居住地;`offline_sign_page.dart` 改引单源。
  - 测试:citizenapp 18 绿(qr_signer 0x10 域断言与 signingMessage 逐字节
    相等 + 载荷解码 9 用例),citizenwallet 25 绿(新增 field_labels 7 用例;
    既有 0x10/0x1A 域金标用例保持)。`dart analyze` 两端改动文件无告警。
  - 残留检查:`citizen-identity-v1` 全仓 0 引用;citizenapp 手拼 GMB 前缀
    仅剩 `signing.dart` 单源一处。
  - 文档:`qr-action-registry.md` a=2 行登记 CitizenApp 签名方与解码器路径;
    新增 `memory/01-architecture/citizenchain/CITIZEN_IDENTITY_FLOW.md`
    (建档→上链→人口统计→投票引擎消费全链路 + 签名域四端同步纪律)。
  - 待用户执行:现场设备重装 ≥5c8374185 构建的 CitizenWallet/CitizenApp,
    OnChina 节点二进制同步重建部署。
