# CitizenApp 电子护照重构:纯默认用户 + 三档身份状态 + 切换即跟随

> 页面单卡呈现已由 `20260715-citizenapp-electronic-passport-three-cards` 彻底替代；本卡只保留默认热钱包、链上身份解码和状态模型的历史实施记录。

任务需求:
把「我的-电子护照」从"扫全钱包认身份"改为**只认默认用户**(`WalletManager.getDefaultWallet()`
= 最靠前热钱包),实时读该账户链上身份,按三档渲染卡片,默认用户切换即跟随;卡片宽度适应
屏幕、高度适应内容。

所属模块:citizenapp / lib/my/myid(电子护照)

## 决策锁定(多轮定案)
- 身份钥匙 = 默认用户唯一;放弃扫全钱包/冲突判定(链上一人一 CID 一账户一身份,`AccountByCid`
  一对一 + `CidAlreadyRegisteredToAnotherAccount`/`CidAlreadyOccupied` 保证)。
- 默认用户切换 → 护照自动跟随(监听 `WalletManager.walletsRevision`,与 Chat/广场同机制)。
- 三档状态:匿名访客 / 投票公民 / 竞选公民。
- 冷钱包身份不认(默认用户只可能是热钱包)——纯默认用户版固有取舍。
- 匿名访客不显钱包地址，投票账户只在当前公民身份卡出现。

## 三档链上公开字段(citizen-identity pallet)
- 投票公民 `VotingIdentityByAccount[account]`:cid_number / residence_省市镇_code / valid_from~until / status。
- 竞选公民 再有 `CandidateIdentityByAccount[account]`(**增量存储**,不含 voting):
  birth_省市镇_code / citizen_full_name / citizen_sex(0男1女)。

## 改动清单
- `lib/my/myid/myid_service.dart`:重写 `getState()` 纯默认用户;新状态模型 `MyIdTier`/`MyIdStatus`;
  扩 `_decodeVotingIdentity`(保留居住码)+ 新增 `_decodeCandidateIdentity`;新增允许空 vec 读取器
  (避免空区划码把公民误判访客);居住/出生选区经 `formatAreaPath`+`provinceDisplayNameByCode`
  在 service 层异步预 join;状态今天日期按 UTC+8(与链 `can_vote` 窗口一致);徽章快照只写默认用户。
- `lib/my/myid/myid_page.dart`:三档卡片渲染;`walletsRevision` 监听自动重读;宽度 `ConstrainedBox`
  (maxWidth 560,窄屏填满=适应屏幕)+ 高度随内容(Column mainAxisSize.min);扇贝徽章 `CitizenBadge` 按档分色。
- 测试:`test/myid_service_test.dart`(逻辑+解码 golden vector,mock WalletManager/ChainRpc/Store)、
  重写 `test/myid_page_test.dart`(三档 widget)。

必须遵守:
- 不改链端;不改徽章快照 store 契约(仍 visitor/voting/candidate);默认用户判定与 `getDefaultWallet` 一致。
- MyIdState/MyIdService 无外部消费方(仅本页),可自由重构。

输出物:代码 + 中文注释 + 测试 + 文档更新 + 残留清理。

验收标准:
- `flutter test` 相关用例通过;`dart analyze` 无新增告警。
- 三档状态正确渲染;切默认用户自动跟随;匿名访客不显示公民身份信息;卡片宽高自适应。
- 无旧"扫全钱包/conflict/notOnchain"残留;文档与注释更新。

关联:`project_citizen_identity_strict_auth_vote_gates`(链上身份字段真源)、
`project_seed_biometric_binding_design`(默认用户=统一身份来源)。

## 完成情况(2026-07-10,已验收)

- `lib/my/myid/myid_service.dart`:重写为纯默认用户;`MyIdTier{visitor,voting,candidate}` +
  `MyIdStatus{normal,notYetValid,expired,revoked,queryFailed}`;`_decodeVotingIdentity` 保留居住码、
  新增 `_decodeCandidateIdentity`、新增 `_readUtf8VecAllowEmpty`(空区划码不误判访客);居住/出生选区
  经 `formatAreaPath`+`provinceDisplayNameByCode` 在 service 层预 join;状态按 UTC+8;徽章快照只写默认用户。
- `lib/my/myid/myid_page.dart`:三档卡片;`walletsRevision` 监听切换即跟随;`ConstrainedBox(maxWidth 560)`
  宽度适应屏幕、`Column mainAxisSize.min` 高度随内容;`CitizenBadge` 按档分色;queryFailed 显读取失败+重试。
- 消费方迁移(旧 MyIdState API 已删,零残留):`lib/my/user/user.dart`(个人页徽章:isCertified→isCitizen、
  identityWalletAccount→votingAccount、删 conflict 分支)、`lib/wallet/pages/wallet_page.dart`(身份钱包标记同迁)、
  `test/my/profile_page_lazy_chain_test.dart`。
- 测试:新增 `test/my/myid/myid_service_test.dart`(7 例:三档 + 未生效/过期/吊销 + 链读失败 + 空镇码)、
  重写 `test/myid_page_test.dart`(5 例三档 widget)。
- 验证:`dart analyze lib test` = 0 error;相关 `flutter test` 全绿(19 例)。
- 附带行为对齐:wallet_page 身份钱包标记也收敛到默认用户(与护照同一身份口径),非默认钱包持有的
  身份不再标记——与"只用默认用户"锁定一致。
- 访客状态不显钱包地址；页面呈现以 2026-07-15 三身份卡任务的最终方案为准。
