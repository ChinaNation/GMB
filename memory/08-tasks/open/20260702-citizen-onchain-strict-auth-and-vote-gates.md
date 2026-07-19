# 公民上链最严鉴权(passkey+冷签)+ 候选人性别上链 + 消费端护照有效期门

> 2026-07-18 流程鉴权部分已由 `20260718-citizen-onchain-single-signature-flow.md` 取代：不得再恢复 prepare/complete 各消费一次 grant。当前唯一口径为一次 Passkey、一次公民钱包签名、一次管理员最终链签；本卡其余投票门禁历史记录保留。

## 任务目标

1. 注册局上链操作升最严档:OnChina 公民身份上链 prepare/complete 两接口挂
   `PasskeyColdSign`(passkey 断言 + 管理员冷钱包扫码签名 grant),新增动作
   `CITIZEN_ONCHAIN_PUSH`,前端接入统一 grant 流程。
2. 链上身份字段定稿:投票公民 = CID号/钱包账户(存储键)/护照有效期起止/
   身份状态/居住地省市镇码(现状已符);参选公民新增 `citizen_sex`(性别)——
   `CandidateIdentityPayload` + `CandidateIdentity` 存储同步加,链上 breaking。
3. 消费端全量校验:`can_vote` 在身份存在(CID↔钱包一对一绑定)、状态 NORMAL、
   作用域匹配之外,新增护照有效期窗口校验(valid_from ≤ 今日 ≤ valid_until)。
4. 护照过期不能投票:经 `TimeProvider: UnixTime`(pallet-timestamp)取链上时间,
   按 UTC+8 折算 YYYYMMDD 与护照有效期比对;`can_be_candidate` 继承同门。
5. 公民发行确认(无改动):`citizen-issuance` 已在首次注册回调中把公民币
   `deposit_creating` 直发公民钱包账户,按档位/上限/CID+账户双重防重。

## 涉及范围

- `citizenchain/runtime/otherpallet/citizen-identity/`(lib.rs + tests)
- `citizenchain/runtime/src/configs/mod.rs`(TimeProvider 接线)
- `citizenchain/runtime/issuance/citizen-issuance/tests/integration_citizen_identity.rs`
- `citizenchain/onchina/src/auth/operation_auth.rs`
- `citizenchain/onchina/src/domains/citizens/chain_identity.rs`
- `citizenchain/onchina/frontend/admins/admin_security_api.ts`
- `citizenchain/onchina/frontend/citizens/api.ts` + `CitizenDetailPage.tsx` + dist 重建
- `memory/01-architecture/citizenchain/CITIZEN_IDENTITY_FLOW.md`

## 边界

- 人口计数器(国/省/市/镇)保持按状态增量,不按护照到期自动减——链上无到期
  事件;护照过期公民在注册局更新状态前仍计入公投分母,投票被 `can_vote` 拒。
- runtime 字段/校验变更 = breaking,按链开发期规则重新创世,不写 migration。
- 候选人流程当前无客户端/OnChina 构造方,`citizen_sex` 仅链端定稿。
- 不执行 `git push`,不创建 PR。

## 验收

- prepare/complete 无 grant 或 grant 不匹配(动作/目标/载荷/所有者)一律 403。
- `CITIZEN_ONCHAIN_PUSH` 归 PasskeyColdSign,as_str/parse 往返一致,测试断言。
- 过期/未生效护照 `can_vote == false`,窗口内为 true;pallet 单测覆盖。
- `CandidateIdentity{,Payload}` 含 `citizen_sex`,升级/更新调用写入存储。
- `cargo test -p citizen-identity`、citizen-issuance 集成测试、onchina 编译
  与前端构建全绿。

## 实现记录

- 2026-07-02 完成:
  - **OnChina 最严档**:`operation_auth.rs` 新增 `CitizenOnchainPush`
    (`CITIZEN_ONCHAIN_PUSH`)归 PasskeyColdSign,不占 Tier1 治理能力边界;
    `chain_identity.rs` 的 prepare/complete 各消费一次 grant(target=cid_number,
    载荷绑定 `{cid_number, wallet_account}`);前端 `citizens/api.ts` 两接口经
    `createScanSignSecurityGrant` 取 grant 后携 `x-cid-security-grant` 调用,
    `CitizenDetailPage` 接入 `useScanSignGrant` 弹管理员冷签确认;dist 已重建。
  - **候选人性别上链**:citizen-identity 新增 `CitizenSex`(Male=0/Female=1),
    `CandidateIdentityPayload`/`CandidateIdentity` 增 `citizen_sex`,升级/更新
    调用写入存储;候选人链上公开档案 = 出生地三级码+姓名+性别。
  - **护照有效期投票门**:Config 新增 `TimeProvider: UnixTime`(runtime 接
    `pallet-timestamp`),`current_date_int()` 按 UTC+8 折算 YYYYMMDD
    (Hinnant civil-from-days,no_std,时间戳缺失 fail-closed 返 0),
    `can_vote` 增护照窗口校验(valid_from ≤ 今日 ≤ valid_until),
    `can_be_candidate` 继承;计数器口径不变(分母含过期公民,投票被拦,
    lib.rs 有注释说明)。
  - **公民发行确认**:无改动——`citizen-issuance` 首次注册回调
    `deposit_creating(公民钱包, 奖励)` 直发,档位/上限/双重防重齐全。
  - 测试:citizen-identity 14(新增过期禁投/未生效禁投/性别存储/日期折算 4 用例)、
    citizen-issuance 12+5、onchina 132(新增动作档位往返用例)、runtime 30 全绿;
    runtime 测试 `new_test_ext` 统一设链上时间戳;前端 `tsc -b && vite build` 通过。
  - 文档:`CITIZEN_IDENTITY_FLOW.md`(最严档 + 字段定稿 + 消费端全量校验 +
    分母口径)、`BACKEND_TECHNICAL.md`(CITIZEN_ONCHAIN_PUSH 登记)。
  - **部署注意**:runtime 为 breaking 改动(候选人结构 + Config),按链开发期
    规则重新创世;onchina 节点二进制与前端 dist 需重建部署。
