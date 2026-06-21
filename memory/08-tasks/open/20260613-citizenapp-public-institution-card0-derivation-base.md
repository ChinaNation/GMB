# 任务卡:公权机构卡0 — account_derivation 归位 + 补 op_tag(共享底座)

属 ADR-018 §九。公权机构详情页本地派生主/费/自定义账户地址的**前置底座**;也兑现 §九"derive_duoqian_account 唯一 Dart 实现归位 shared"。

状态:**代码完工(2026-06-13)**。account_derivation 归位 shared + 补 op_tag(0x00/0x01/0x06 + 路由 0x02/0x03/0x04 + 个人 0x05);golden 向量 11/11 与链上注册表 hex 逐一吻合(国储会/中枢省 main/fee + 安全基金 OP_AN + 两和基金 OP_HE);旧 personal_duoqian_derive.dart 已删、调用方迁移、零残留;analyze 0。

## 背景
- 目前派生只有个人多签:`governance/personal-manage/personal_duoqian_derive.dart`(仅 OP_PERSONAL=0x05)。
- 公权机构账户地址需本地派生:主 0x00、费 0x01、自定义 0x06。派生错=地址错=余额全错,**op_tag 必须与链端单一源 `citizenchain/runtime/primitives/src/core_const.rs` 逐一核对**。

## 完工清单
- [ ] 新建 `governance/shared/account_derivation.dart` 作为全 app 唯一派生入口。
- [ ] 补齐 op_tag 常量(对齐 core_const.rs):`OP_MAIN=0x00`、`OP_FEE=0x01`、`OP_CUSTOM=0x06`、保留 `OP_PERSONAL=0x05`。
- [ ] 两种 preimage 形态(与 CID `accounts/derive.rs` / 链端一致):
  - 机构账户:`DUOQIAN || op_tag || ss58_le(2027) || cid_number || (account_name 仅 0x06)`
  - 个人多签:`DUOQIAN || 0x05 || ss58_le || creatorPubkey(32B) || account_name`
  - 末端统一 `blake2b256` → 32B,prefix=2027 SS58。
- [ ] API:`deriveInstitutionAccount({cidNumber, opTag, accountName?})` + 保留 `derivePersonalAccount({creatorPubkey, accountName})`。
- [ ] 迁移现有个人多签调用方 import 到 shared,删旧 `personal_duoqian_derive.dart`,零行为变化。
- [ ] 核对 organization-manage 是否有派生点(现状用 registry hex 不派生,确认即可,不改口径)。

## 单测
- [ ] golden 向量:用 governance registry 里已知机构的 cid_number→mainAccount/feeAccount(若同派生)做断言;否则用链上已激活公权机构地址做向量。
- [ ] 自定义账户 0x06 名字进 preimage、主/费 0x00/0x01 名字不进,各一组向量。
- [ ] 个人多签迁移后输出与旧实现逐字节一致。

## 验收
- [ ] flutter analyze 0 + flutter test 全过(含派生向量)。
- [ ] 旧 personal_duoqian_derive.dart 零引用、已删。

## 不做(边界)
- 不改链端、不改 CID;只动 citizenapp 派生底座。
- 不引入公权机构 UI(卡 B/C)。

## 改动目录(中文注释)
- 新增 `citizenapp/lib/governance/shared/account_derivation.dart`:统一派生原语,代码。
- 删 `citizenapp/lib/governance/personal-manage/personal_duoqian_derive.dart`:归位后清残留。
- 改个人多签派生调用方:import 切到 shared,代码。
