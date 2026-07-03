# CID 占号先行与吊销墓碑(占号体系 卡2)

> 设计真源:`memory/04-decisions/ADR-031-cid-occupy-registry.md`(D2/D3/D6/D7/D8)。2026-07-02 代码核查补充的硬约束:
> - onchina 现无任何自动提交交易通路(全部=裸 call_data→冷签→钱包提交),须按 D7 补「组装+dry-run+author_submitExtrinsic」后端提交骨架(参照 node/src/governance/signing.rs:612-683),QR 仍只签不提交;
> - 机构关闭现状=账户级物理删除、`Institutions` 永不删但状态从未置 Closed;须按 D3 落地 Closed 墓碑语义 + 堵 register(call 2)不查本 pallet Institutions 的重注册缺口;
> - citizens.onchain_tx_hash 等列现无写入者,须按 D8 经 indexer 事件回写闭环;
> - 新增 `occupy_cids_batch`(≤10,000 项/笔)供公民批量建档;占号/吊销费类 **Free(2026-07-02 已决,ADR-031 Q4)**,滥用由链上注册局授权门槛拦截。
> - 2026-07-03 修订:机构存量改全量创世直铸(卡3 D5),本卡占号先行流程只服务**运行期新增**(公民建档、未来新设机构);机构批量注册 extrinsic 本轮不实现。

## 设计定稿(用户已确认)

- 唯一性仲裁 = 链上写入时原子「验格式 + 查重 + 登记」;链下 RPC 预查仅作快速失败优化,不承担唯一性保证(查询防不住并发,写入才是仲裁)。
- 建档流程 = 本地生成号 → 提交占号交易(携档案承诺哈希)→ InBestBlock 成功 → 才写本地机构/公民档案。
- 占号即终身绑定:号码从此不可能发给任何其他主体。
- 落库失败恢复 = 幂等续用:占号记录含登记机构 + 档案承诺哈希,重试建档时发现「本注册局为同一档案承诺」已占 → 直接落库,不二次占号,孤号不产生。
- 清档 = 发吊销交易(链上状态 Active → Revoked 墓碑)+ 清本地档案;链上记录永不删除、号码永不复用(对齐 revoke 保留映射、行政区码墓碑 ADR-021)。
- 隐私边界(已认可):全部公民建档即占号,链上只有 cid_number + 承诺哈希 + 登记机构 + 块高,无姓名生日;链上可枚举每省建档总量。
- 建档依赖链活性,链不可用即建档失败(fail-closed)。
- runtime breaking → 重新创世,零兼容零残留。

## 目标

- `citizen-identity`(或独立归口)新增:
  - `CidRegistry` 存储:cid_number → { 登记机构, 承诺哈希, 状态 Active/Revoked, 块高 }。
  - 占号 extrinsic(注册局标准 extrinsic 签名,复用 `CitizenIdentityAuthority` 省市 scope 授权;遵守签名分层铁律,不引入 op_tag)。
  - 吊销 extrinsic(墓碑,不删除存储项)。
- `register_voting_identity` 前置要求:CID 已占号、状态 Active、归属一致。
- 机构侧:`public/private-manage` 注册即占号(现有 `Institutions` + sibling `cid_exists` 写入时查重即真源,不重复建表);核对 onchina 机构两步流严格「链上成功才转正」。
- onchina 建档流程改造:公民/机构统一占号先行;碰撞报错走 nonce 后缀重试(上限对齐 SFID n9 桶 1000 次重试规则,同时治愈确定性种子同名同生日碰撞即 409 无恢复的问题);清档接吊销交易;`onchain_tx_hash` 等字段回写。
- 占号/吊销费用归入 `primitives::fee_policy` 明确分支。

## 修改范围

- `citizenchain/runtime/otherpallet/citizen-identity/`
- `citizenchain/runtime/entity/public-manage/`、`private-manage/`(核对为主)
- `citizenchain/runtime/src/`(注册、费用分支、benchmark)
- `citizenchain/runtime/primitives/`(承诺哈希定义、fee_policy)
- `citizenchain/onchina/src/domains/citizens/`、`institution/`、`cid/`
- `citizenchain/onchina/frontend/`(建档/清档流程状态提示)

## 验收

- 并发占同号:一成一败(`CidAlreadyRegistered` 类错误)。
- 占号成功 + 本地落库失败 → 重试直接续用,链上不出现二次占号。
- 清档 → 链上 Revoked 墓碑,该号任何主体不可再占。
- 同名同生日同镇两位公民:第二位经 nonce 重试获得新号,建档成功。
- `cargo test` 相关 pallet 全绿;`cargo check -p onchina` 通过;`npm --prefix citizenchain/onchina/frontend run build` 通过。

## 状态

- 2026-07-02:建卡。依赖卡1(链端校验先行)。
