# 投票引擎补全四种投票 + 选举投票命名收口 + 白皮书/官网同步

任务需求：
- 投票引擎补全为四种投票，并按用途重述边界：
  1. 内部投票 internal-vote：各机构 + 个人/机构多签的内部事项（转账等），管理员快照+阈值
  2. 联合投票 joint-vote：仅治理机构（国家储委会/省储委会/省储行）跨机构共同治理（含联合公投阶段）
  3. 立法投票 legislation-vote：仅立法机构修法表决（市自治会/市教委会/市立法会/省联邦立法院参议会和众议会/国家立法院参议会和众议会/国家教委会）
  4. 选举投票 election-vote：选举公职人员=普选（公民选）+互选（机构成员互选）
- 独立选举 pallet 统一为 `election-vote`（crate/目录/标识符/construct_runtime `ElectionVote` idx22；历史空骨架已接入选举框架）
- 白皮书 5.2 + 官网 citizenweb（Governance.tsx/Technology.tsx）同步四种投票，CitizenApp 只描述为投票交互入口
- 修残留：旧 HTML 解析脚本 OUT 路径 governance/legislation-yuan → public/legislation-yuan
- 2026-06-28 追加：`election-vote` 已从空骨架接入普选/互选框架，新增候选人/选民快照、投票、计票、超时结算、终态回调和清理状态机。

宪法依据（2026-06-28 已按用户确认更新）：公职人员章——普选产生总统、护宪大法官、市立法会委员、市教委会委员、省参议员、省众议员、市/镇自治会员，以及国家立法院参议员/众议员；其中国家立法院参议员由各省行政区公民在本省联邦立法院参议会现任参议员中普选产生，国家立法院众议员由各省行政区公民在本省联邦立法院众议会现任众议员中普选产生。互选仍用于机构现任成员内部选举院长、主席、参议长、众议长等职位。选举相关统一归入"选举投票"。

边界铁律：投票引擎只保留内部投票、联合投票、立法投票、选举投票四类。联合投票的第二阶段统一叫联合公投；立法特别案/核心修宪的公民参与阶段统一叫立法公投；不得再把公民参与阶段描述成独立第五类投票。

所属模块：Blockchain Agent（citizenchain runtime）+ 文档 + 官网

验收标准：
- runtime cargo check 绿 + 相关测试过
- 全仓旧独立投票类型命名改为 election-vote；联合公投内部账本统一使用 Referendum / JointReferendum 语义
- 白皮书 5.2 四种投票 + 官网同步
- 旧 HTML 解析脚本 残留修复
- 文档更新、残留清理

已完成补充：
- `election-vote/src/popular.rs`：普选创建/投票入口。
- `election-vote/src/mutual.rs`：互选创建/投票入口。
- `election-vote/src/types.rs`：本地运行态快照类型；职位/任期由业务模块传入，不进 `runtime/primitives` 常量库。
- `election-vote/src/tally.rs`：多候选、多席位得票多数计票；同票暂拒绝，待选举法规则接入。
- `election-vote/src/snapshot.rs`：候选人/选民快照校验与写入。
- `election-vote/src/cleanup.rs`：选举账本分块清理。
- 2026-07-04：按用户确认追加收口 runtime 核心旧描述；旧清理阶段名改为 `JointReferendumVotes`，internal-vote 测试辅助和测试名改为 joint_referendum 语义。
- 2026-07-04：同步 OnChina、CitizenApp、citizenweb、memory 当前文档，把旧独立投票类型描述改为四类投票、联合公投或立法公投。
