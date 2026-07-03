# 任务卡:公权机构卡C — 详情页 + 更多账户 + 动态(余额/提案/管理员)+ 订阅

属 ADR-018 §九。公权机构详情页与动态展示。依赖卡0(派生)、卡A(数据)、卡B(导航);余额复用卡⑤ ChainReadCache(已完工)。

状态:**代码完工(2026-06-13,v1)**。详情页 `public_institution_detail_page.dart` 扩充:info 卡 + 账户卡(主/费本地派生 + 余额)+ 更多账户页 `public_institution_accounts_page.dart`(主+费+自定义全派生 + 批量余额)+ 提案卡 + 管理员卡 + 右上角订阅按钮(写卡A store)。账户派生 `data/public_institution_accounts.dart`(deriveAccountRows,golden 与链上 hex 吻合)。链读隔离在 `data/public_institution_chain_data.dart`(balances/admins/proposals 接口 + LivePublicInstitutionChainData 复用 ChainRpc/InstitutionAdminService/DuoqianTransferProposalFeed;提案按 `subject_cid_numbers` 包含机构 CID 过滤年缓存;管理员走 institutionAccount org=4)。chainData + walletPubkeyProvider 注入可测。测试 3/3(派生行/详情展示/订阅切换)+ analyze 0 + dart format + citizen/public 子树 17/17 + **全量 232/232 无回归**。

**v1 范围/真机延后**:发起提案/换管理员本期不做(detail 不放入口);真实余额/管理员/提案需联网(LivePublicInstitutionChainData),走既有基础设施,**真机验证待跑**(注入 fake 已覆盖 UI 逻辑)。finalized 订阅刷新未接(v1 进页加载一次;订阅集动态刷新留 follow-up)。

## 需求(用户口径)
详情页展示:名称、身份ID(cid_number)、主账户+余额、费用账户+余额、更多账户(卡片→该机构所有账户页)、提案卡片(→ 发起提案;v1 隐藏非管理员)、管理员列表(→ 管理员列表页)、提案列表。右上角**订阅按钮** → 订阅进"关注"。

## 完工清单
- [ ] 新建 `citizen/public/public_institution_detail_page.dart`,复用 organization-manage/institution_detail_page 布局(~80%)。
- [ ] **抽取共享详情 widget 到 `governance/shared/`**(账户卡 / 余额行 / 管理员列表 / 提案列表),org-manage 与 citizen/public 都 compose(避免跨功能 relative import)。
- [ ] 账户地址全部**本地派生**(卡0):主 0x00 / 费 0x01 / 自定义 0x06(名字来自卡A custom_account_names,空则无)。
- [ ] 「更多账户」卡片 → 全部账户页:列 主+费+全部自定义;地址全本地派生;一次 `fetchFinalizedBalances([全部地址])` 走 ChainReadCache(卡⑤)。
- [ ] 主/费账户余额:精确整键批量 + 块内缓存(只在打开详情时读)。
- [ ] 提案列表:复用 `DuoqianTransferProposalFeed.currentYearProposals`(卡①),按 `subject_cid_numbers` 包含机构 CID 过滤。
- [ ] 管理员列表:`AdminsChange::AdminAccounts` 精确读(复用 InstitutionAdminService)→ 管理员列表页。
- [ ] 右上角订阅按钮:toggle 写卡A 订阅 store;订阅后纳入 finalized 订阅动态刷新集。
- [ ] 动态刷新:`ChainEventSubscription` finalized 头驱动余额/提案刷新,**仅打开/已订阅机构**(有界);**禁轮询**(R2)。
- [ ] 发起提案/换管理员:按 `ProposalContextResolver` 判定"我是否本机构管理员",非管理员**隐藏**;v1 不接入发起流程(复用页面留下一期)。

## 单测/widget 测
- [ ] 派生地址→余额批量(fake ChainRpc)装配正确;无自定义时更多账户只 2 个。
- [ ] 提案过滤 `subject_cid_numbers` 命中本机构 CID;管理员列表渲染。
- [ ] 订阅 toggle 写入/移除;非管理员隐藏发起入口。

## 验收
- [ ] flutter analyze 0 + flutter test 全过。
- [ ] 真机:国家储委会外某市公权机构详情显示名称/ID/主+费余额/更多账户/提案/管理员;订阅后出现在关注;余额走 finalized,无轮询。
- [ ] R1 自检:无长前缀 keysPaged;账户发现 100% 本地派生。

## 不做(边界)
- v1 不做发起提案/换管理员全流程(下一期,复用 duoqian-transfer + admins-change)。
- 不动链端、不动 CID 写入。

## 改动目录(中文注释)
- 新增 `citizenapp/lib/citizen/public/public_institution_detail_page.dart` + 更多账户/管理员列表子页,代码。
- 改 `citizenapp/lib/governance/shared/`:抽取共享详情 widget(账户卡/余额行/管理员/提案列表),代码。
- 复用(不改口径)`rpc/`(fetchFinalizedBalances/ChainReadCache/ChainEventSubscription)、`transaction/duoqian-transfer`(ProposalFeed)。
