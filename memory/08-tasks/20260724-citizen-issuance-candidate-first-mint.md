# 公民发行:竞选身份首次上链补发公民币(方案 A)

任务需求：
公民首次上链即发放一次性公民币,无论首次上链是投票身份还是竞选身份都应发币。
当前发币回调只挂在 `register_voting_identity`(call 0),`upgrade_to_candidate_identity`(call 1)
可作为首次上链却不触发回调,导致"一开始就竞选、直接以参选身份首上链"的公民永久漏发。
按 5 条产品口径定稿:
1. 首次上链即发币,投票/竞选都发;
2. 按公民 CID + 钱包账户双重去重,不可重复领;
3. 竞选身份不需先有投票身份,不得强制先投票;
4. 发币前必须字段完整,不完整不允许上链且不发币;
5. 硬顶+档位先到先得,达顶永久停发。

所属模块：citizenchain/runtime/misc/citizen-identity(触发端)+ citizenchain/runtime/issuance/citizen-issuance(发行端)

输入文档：
- memory/05-modules/citizenchain/runtime/issuance/citizen-issuance/CITIZENISS_TECHNICAL.md
- memory/07-ai/definition-of-done.md
- 链端节点守卫 citizenchain/node/src/core/node_guard/citizen_issuance.rs(状态复核基线)

必须遵守：
- 不可突破模块边界:发行逻辑不进 citizen-identity,只补回调触发点
- 不得强制"先投票后竞选"(第 3 条红线)
- 双重去重(CID+账户)与硬顶/档位先到先得规则不得改动
- 链开发期:重新创世即可,无需 migration;不改存储布局、不改 call 索引/签名
- 字段完整性校验在写身份+触发回调之前,保持"不完整不上链不发币"结构性保证

范围(方案 A,不扩围)：
- 仅在 `upgrade_to_candidate_identity` 中,当 `old.is_none()`(该 CID 首个投票身份、即竞选作为首次上链)
  时,于 `bind_account_id` 之后补触发 `OnVotingIdentityRegistered::on_voting_identity_registered`;
  `old.is_some()`(先投票后升竞选)不触发,发行侧 CID/账户永久去重亦会拦截。
- call 1 的 `#[pallet::weight]` 追加 `on_voting_identity_registered_weight()`(worst-case 上界)。
- 节点守卫无需改:守卫按"本块新建投票身份+双向绑定+去重"状态复核,竞选首上链与投票首上链状态同构。
- onchina / 前端无需改:已允许直接选参选身份(正是第 3 条要的)。

输出物：
- 代码:citizen-identity/src/lib.rs 两处(回调触发 + weight)
- 中文注释
- 测试:citizen-issuance 集成测试补
  - 竞选身份首次上链触发一次性发币
  - 先投票拿币后升竞选不二次发币
  - (可选)竞选首上链后重复登记被去重拦截
- 文档更新:CITIZENISS_TECHNICAL.md 定位口径改为"投票或竞选身份首次上链"
- 残留清理:无兼容分支

验收标准：
- 竞选身份首次上链后同块 on_finalize 到账,金额=当前档位,写入永久去重标记
- 先投票已领币的 CID 升竞选不再发币
- cargo test -p citizen-issuance / -p citizen-identity 通过
- cargo check -p citizenchain-runtime 通过(weight 注解合法)
- CITIZENISS_TECHNICAL.md 已同步
- Review 问题已处理

状态：已完成(2026-07-24)

落地记录：
- citizen-identity/src/lib.rs:call 1 `upgrade_to_candidate_identity` 在 `old.is_none()` 时,
  于 `bind_account_id` 后补触发 `on_voting_identity_registered`;`#[pallet::weight]` 追加
  `on_voting_identity_registered_weight()`(worst-case,基准场景 old.is_some() 不重复计权)。
- citizen-issuance 集成测试新增:竞选首次上链发币 + 先投票后升竞选不二次发币。
- 文档:citizen-issuance/src/lib.rs 头注释、CITIZENISS_TECHNICAL.md 定位口径已同步。
- 验证:cargo test -p citizen-issuance(21)、-p citizen-identity(36)全绿;cargo check -p citizenchain 通过。
- 节点守卫未改:按状态复核,竞选首上链与投票首上链状态同构,天然兼容。
- onchina/前端未改:已允许直接选参选身份(第 3 条不强制)。
