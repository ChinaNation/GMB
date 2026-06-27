# 投票引擎补全四种投票 + citizen-vote 改名 election-vote + 白皮书/官网同步

任务需求：
- 投票引擎补全为四种投票，并按用途重述边界：
  1. 内部投票 internal-vote：各机构 + 个人/机构多签的内部事项（转账等），管理员快照+阈值
  2. 联合投票 joint-vote：仅治理机构（国储会/省储会/省储行）跨机构共同治理（含联合公投阶段）
  3. 立法投票 legislation-vote：仅立法机构修法表决（市自治会/市教委会/市立法会/省参议会众议会/国家参议会众议会/国家教委会）
  4. 选举投票 election-vote（原 citizen-vote/公民投票）：选举公职人员=普选（公民选）+互选（机构成员互选）
- 改名 `citizen-vote → election-vote`（crate/目录/标识符/construct_runtime `CitizenVote→ElectionVote` idx24；空骨架无生产存储，链上名一并改，不波及 Dart 端）
- 白皮书 5.2 + 官网 website（Governance.tsx/Technology.tsx）同步四种投票 + 公民投票→选举投票
- 修残留：parse_constitution.py OUT 路径 governance/legislation-yuan → public/legislation-yuan

宪法依据（已核）：公职人员章——普选产生市立法会委员/市教委会委员/省参议员/省众议员/护宪大法官/总统+市镇自治委员；互选产生储委会主席副主席/国家参议员众议员/各级教委会/储行董事长/立法院院长等。故"公民投票"名不副实，改"选举投票"。

边界铁律：核心 votingengine 的 `CitizenVotes` storage / `PendingCleanupStage::CitizenVotes` / CITIZEN kind / CidEligibility / internal-vote 的 citizen_vote_* 测试 = 联合投票的联合公投阶段（公民凭证模式），**不改**；只改独立的 idx24 citizen-vote pallet。

所属模块：Blockchain Agent（citizenchain runtime）+ 文档 + 官网

验收标准：
- runtime cargo check 绿 + 相关测试过
- 全仓 citizen-vote/citizen_vote/CitizenVote 改名（联合公投 CitizenVotes 除外）
- 白皮书 5.2 四种投票 + 官网同步
- parse_constitution.py 残留修复
- 文档更新、残留清理
