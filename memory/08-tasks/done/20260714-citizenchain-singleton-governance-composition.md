# CitizenChain 单例治理机构与成员组成永久约束

## 状态

- 已完成（2026-07-14）。
- 第 1/5 至第 5/5 步均已完成。

## 任务目标

- 固定 NRC、NJD、FRG 国家级单例和 43 省 PRC、PRB 省级单例。
- 固定上述 89 个创世治理机构的管理员人数与内部投票阈值。
- 固定 NLG、NSN、NRP、NED、NSP、PRS 六个创世身份的全链单例，不把其完整组织结构纳入 89 个精确治理骨架。
- NSN 的 `SENATOR` 岗位及 admins 保持 105–155 人；NRP 的 `REPRESENTATIVE` 岗位及 admins 保持 305–355 人；NED 的 `COMMITTEE_MEMBER` 岗位及 admins 保持 105–155 人。
- NLG 由 NSN 与 NRP 组成；NLG、NSP、PRS 的岗位和管理员人数由运行期治理增删改。六个国家单例均不建立账户级动态阈值。

## 设计边界

- 六个国家机构保留现有 block#0 机构身份、CID、主账户和费用账户。
- 六个国家单例允许从创世“尚未组成”状态原子写入岗位、任职和 admins；NSN、NRP、NED 组成后不得删除指定成员岗位或退回人数下限以下。
- 三个成员机构的 admins 必须等于指定成员岗位有效任职钱包去重集合，避免非成员获得代表机构投票资格。
- 其他辅助岗位允许运行期增删改，但不得引入指定成员岗位之外的新 admin 钱包。
- 不修改 ConstitutionGuard，不放宽 NodeGuard 的发行、PoW、CID、GenesisPallet 等既有策略。
- 不保留旧流程、双轨兼容或影子真源。

## 修改范围

- `citizenchain/runtime/primitives/`：单例身份、立法院组成关系、岗位代码和人数区间真源。
- `citizenchain/runtime/entity/public-manage/`：运行期组成结果与 admins 闭环校验。
- `citizenchain/runtime/votingengine/internal-vote/`：固定治理机构阈值快照约束。
- `citizenchain/runtime/genesis/`：保留六个机构并补齐未组成状态断言。
- `citizenchain/node/src/core/node_guard/`：单例身份、成员组成和固定阈值永久守卫。
- CitizenChain runtime/node 技术文档：更新制度边界、验收结果和残留口径。

## 验收要求

- block#0 精确存在约定单例身份，新增同码不同 CID、关闭或替换单例均被 runtime 与 NodeGuard 拒绝。
- NSN/NRP/NED 创世未组成状态通过；首次组成必须原子满足人数区间与 admins 集合闭环。
- 已组成后的合法换届通过，低于下限、高于上限、删除指定岗位、引入非成员 admin 均拒绝。
- NLG/NSP/PRS 的岗位和管理员人数合法变化不被永久组成策略拒绝；六个国家单例不建立永久机构级动态阈值。
- NRC/PRC/PRB/NJD/FRG 的内部投票阈值快照不能被 runtime 升级改变。
- 完成专项测试、整组测试、fresh block#0 与当前本地链真实 RPC 验收。
- 更新文档、完善中文注释并清理旧实现与旧口径残留。

## 执行结果

### 纠正执行记录

- [x] 第 1 步：确认 `internal-vote` 是所有机构共用的投票程序；有效准入同时要求投票引擎模式准入和业务 pallet 具体权限，不建立机构与全部业务互斥绑定的全局能力表。
- [x] 多签转账删除只识别 NRC/PRC/PRB 的内置机构解析，所有 active 机构统一从 entity 生命周期真源解析；新增 FRG、NJD 创建转账内部提案回归。
- [x] 删除 `internal-vote` 中 FRG 管理员必须恰好 5 人的错误账户校验；FRG 保持一个机构、一个主账户和 215 名管理员，省域岗位组权限不混入通用投票引擎。
- [x] `resolution-destroy` 继续在业务模块固定 NRC/PRC/PRB，新增 NJD 拒绝回归；`grandpakey-change` 现有 NRC/PRC 限权复核正确。
- [x] 第 1 步验证：`internal-vote` 88、`multisig` 24、`resolution-destroy` 15、`grandpakey-change` 17 项测试通过，runtime 整体 `cargo check` 通过。
- [x] 第 2 步删除首次组成自动写入动态阈值的错误实现；六个国家单例拒绝所有账户级动态阈值写入，一般内部事项在提案创建时按 admins 快照生成 `floor(N/2)+1` 阈值快照。
- [x] 六个国家单例首次有效治理结果只原子写岗位、任职和 admins；NSN/NRP/NED 继续强制法定岗位、人数区间和 admins 闭环，NLG/NSP/PRS 不增加岗位或人数限制。
- [x] 第 2 步验证：primitives 66、`internal-vote` 89、`public-admins` 8、`public-manage` 42、runtime 集成 40 项测试通过；runtime 默认特性与 `no_std` 编译通过。
- [x] 第 3 步：业务执行端不再只凭“提案已通过”放行。多签转账、销毁、GRANDPA 密钥变更、机构/个人生命周期统一绑定 callback scope、`ProposalOwner`、proposal kind/stage、机构码、账户、CID 和业务 action，并在执行前复核当前业务权限。
- [x] 联合业务同时接受 `STAGE_JOINT` 与 `STAGE_REFERENDUM` 的合法通过终态；决议发行和 runtime 升级新增联合公投通过回归，runtime 升级额外复算 wasm 对象哈希。
- [x] 立法交易字段和顺序保持不变；链端按层级与表决类型固定 houses/proposer/executive/legislature 法定路由，复核 active entity、机构码、账户、CID 和省市 R5，并把完整路由写入摘要供投票通过写入前二次复校验。
- [x] `election-vote` 删除两个外部创建 extrinsic 和直写 entity 路径；只保留投票、计票和结果快照。`election-campaign` 未实现真实业务前，创建与任职写入继续 fail-closed。
- [x] 第 3 步验证：`resolution-issuance` 17、`runtime-upgrade` 19、`resolution-destroy` 15、`grandpakey-change` 17、`multisig` 24、`public-manage` 42、`private-manage` 40、`personal-manage` 23、`election-vote` 2、`legislation-yuan` 32、runtime 集成 40、OnChina 法律专项 27 项测试通过；runtime `no_std` 通过。
- [x] 当前源码重新构建 WASM 后，`citizenchain-fresh --tmp` 真实启动通过：block#0 `0xb9688b1d8904a75319c6e715544c00678e4a94df7306cfec071a07a93dd03025`，`isSyncing=false`，隔离节点已正常退出。
- [x] 第 4 步：NodeGuard 治理骨架只按 89 个创世完整身份触发；普通机构的管理员、岗位、任职和组织结构不进入原生分区或扫描，任职来源、引用和任期继续归 runtime 业务合法性。
- [x] 六个国家级单例只由 CID 生命周期策略冻结机构码、CID、主账户和 Active 身份；NSN/NRP/NED 额外执行“全空未组成→原子已组成”的单向状态及法定岗位人数/admins 闭环，NLG/NSP/PRS 不进入岗位或人数守卫。
- [x] 五类固定治理码只冻结每笔内部提案的固定阈值快照；固定治理提案与规范快照双向绑定，六个国家级单例明确不进入固定阈值策略。`:code` runtime 升级会枚举并复核全部存量提案与阈值快照，不能依靠“不改旧 key”或“移走旧快照表”绕过永久阈值。
- [x] 第 4 步验证：primitives 66/66、国家机构组成专项 8/8、NodeGuard 整组 88/88 通过；当前源码重新构建后 fresh block#0 仍为 `0xb9688b1d8904a75319c6e715544c00678e4a94df7306cfec071a07a93dd03025`，`isSyncing=false`，隔离节点已正常退出。
- [x] 最新冻结已使用 Git commit `7abac7982a5c5ee25580583d456523ce2132743e` 对应的成功 CI WASM 重生唯一 chainspec、轻客户端 checkpoint、创世状态包和 49,593 条公权机构快照；未保留旧 storage 布局或双创世兼容。

- 新增 `primitives::institution_constraints` 单一真源，六个国家级单例身份、两院组成关系、三个成员岗位及人数区间不再散落手写。
- runtime 注册局拒绝新建六类单例码，public-manage 注销入口拒绝关闭；NodeGuard 同时拒绝固定治理码/六类单例码占用非规范 CID。
- 六个国家单例创世保持岗位、任职、admins 和动态阈值全空；首次治理结果原子写岗位、任职和 admins。旧的账户级最小严格过半动态阈值实现已删除。
- 新增 `national_body_composition` 原生策略；五类固定治理码的 `InternalThresholdSnapshot` 必须等于编译期阈值，FRG 省岗位组上下文同样覆盖。
- 当前累计回归：primitives 66/66、public-admins 8/8、public-manage 42/42、runtime 集成 40/40、NodeGuard 整组 88/88；其中 NodeGuard 国家组成专项 8/8、CID 生命周期专项 15/15。
- 最新正式冻结：GitHub `CitizenChain WASM` run `29530114067` artifact 校验通过；`runtime_wasm_hash=be4585ce369e658e6799be667ed5be692fc050f9c6196ab14c53f7dfa5dc6e70`，物化耗时 51 秒。
- 唯一发布锚点：`genesis_hash=0x840d5b12c541a010783e54069c9168a13d102ba63cd8f3a00263440c1803aad9`、`state_root=0x99b4cb3031baa5e87536a22190dc81bf6bf49d3678c0abae86a312268506fe09`、`chainspec_hash=5e609d166e8517d20ec0cd2095b88825146e34e64b3ebaba54152c7bde9d1f60`、`light_sync_state_hash=4b05735ed59a8ef3756bf6445f1e4fa744730d2161ad14a62be1e16856bbfb9a`、`public_institution_root=ecff487ce7d2bac6cb89d064a456187b453acd27f4bee2b140f474a48d072682`。
- 冻结一致性脚本、43 省分片及 49,593 总数校验通过；默认内嵌链规范使用正式创世状态包真实启动，RPC 返回同一 block#0/state root、`isSyncing=false`，节点正常退出。
- CitizenApp 本地 checkpoint、缓存测试夹具、Cloudflare bootstrap 默认值及 development/staging/production 三套环境链身份已同步；Flutter 相关 31/31、smoldot chain-spec 4/4、Cloudflare typecheck、Worker 167/167 测试通过，未执行远端部署。
- 烘焙脚本 RPC 轮询的内嵌 Python f-string 转义错误已修复；该错误此前会吞掉语法异常并把已经出块的节点误报为 10 分钟超时。
