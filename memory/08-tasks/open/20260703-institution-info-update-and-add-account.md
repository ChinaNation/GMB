# 机构信息可维护:改名 + 新增账户(entity 公权/私权)

> 背景:ADR-031 链是机构信息唯一真源(公权/私权统一,注册局本地库为同步副本)。创世只铸定初始集,今后新增省/市/镇/机构、改名、加账户全走交易上链。当前 entity 只有 create/close,缺"改信息"和"给已存在机构加账户"两个入口。

## 字段可变性分析(2026-07-03,已逐字段核实)

**绝对不能改**(改了就塌):
- CID 号(身份锚点,账户派生/管理员建键/档案引用全靠它)
- 机构码、省市码(**物理编码在 CID 里**,改它=改 CID;还参与校验和/路由/费率)
- 账户地址(CID+账户名 确定性派生,有余额)
- 协议账户名及 `created_at`；协议账户是否可关闭由 `InstitutionProtocolAccountKind` 唯一判定，不保存第二份布尔标记

**支持修改**(本卡新增交易):
- cid_full_name / cid_short_name(公权+私权统一上链;私权原存空是旧框架,本卡改为存名)

**既有机制,不归本卡**:
- 管理员(admins-change)、余额(转账)、status(生命周期 close)

## 本卡实现(链端为主)

### A. 私权名上链(修正旧框架)
- `private-manage/create.rs`:`stored_full/short_name` 从 `default()` 改为用参数(公权已如此);私权机构名称随创建上链,链为唯一真源。genesis 无私权,不受影响。

### B. `update_institution_info` call(public + private)
- 参数:cid_number + 新 cid_full_name/cid_short_name + 注册局凭证(issuer/nonce/signature/scope,复用 register 授权套件);
- 校验:机构存在、非 Closed、新名非空、注册局授权(RegistryAuthority + CidInstitutionVerifier);
- 只 mutate `Institutions[cid]` 的两个名字字段,机构码/CID/created_at 不给参数、不动;
- 发 `InstitutionInfoUpdated` 事件。

### C. `add_institution_account` call(public + private)
- 参数:cid_number + 新账户名(单个或列表)+ 授权凭证;
- 校验:机构存在且 Active、账户名非空/非保留冲突/未重复、派生地址未被占用/非保留/非保护(复用 register 校验链);
- 派生地址 → 写 `InstitutionAccounts[(cid_number, account_name)]`（`initial_balance=0`）与 `AccountRegisteredCid`；不保存状态、默认标志或重复正向表；
- 发 `InstitutionAccountAdded` 事件。
- **授权双路径**:①注册局授权(同 create,直接生效);②机构自己管理员经 internal_vote 提案通过(治理自治)。本卡先做注册局路径,internal-vote 路径列 follow-up。

### D. 事件/错误/权重/测试
- 新增 Event `InstitutionInfoUpdated`/`InstitutionAccountAdded`,Error `InstitutionNotFound`/`InstitutionClosedCannotUpdate`;
- 两 pallet 对称实现(结构近同,close.rs 曾被证逐字节一致);
- 测试:改名成功+拒空名+拒 Closed+非授权拒;加账户成功+拒重复/保留名+拒非授权+派生地址正确。

## Follow-up(后续卡)
- internal-vote 机构自治路径(管理员发起改名/加账户提案);
- onchina 两阶段冷签流程(复用卡2 chain_submit):更新机构信息 / 新增账户;
- App reconcile 增量源接链(改名/加账户后 App 跟上);
- 私权名上链后,onchina 私权注册流程同步把名字带进链上交易。

## 验收
- `cargo test -p public-manage -p private-manage` 全绿(含新用例);
- `cargo test -p citizenchain --lib` 通过;
- 改名只动名字、机构码/CID 不可改(参数层就没有);加账户三索引一致、地址确定性派生。

## 进展

- 2026-07-03:**链端(A/B/C/D)完成**,public 38 + private 37 测试全绿、全 runtime(含 benchmarks)+ onchina 编译过:
  - **A 私权名上链**:`private-manage/create.rs` 从存空改为用参数存名(`_cid_short_name`→`cid_short_name`,拒空全称/简称);翻转 3 个旧行为测试(私权名现上链、空简称现拒绝)。
  - **B/C 两个 call**：`update_institution_info`（call 6）只更新名称；`add_institution_account`（call 7）派生地址并写 `InstitutionAccounts` 与 `AccountRegisteredCid`。外层授权统一为 `actor_cid_number + origin`，注册局凭证仍走唯一消息构造函数。
  - **D 事件/错误/权重/测试**:新增 Event `InstitutionInfoUpdated`/`InstitutionAccountAdded`、Error `InstitutionNotFound`;weights 复用 register 上界;费类走 `RuntimeCall::PublicManage(_)`/`PrivateManage(_)` 已有 VoteFlat 兜底(1 元/次),无需改分类器。
  - 踩坑:测试 mock verifier 原要求 account_names 非空(改名传空会挂),放宽为可空(登记入口自身已在 verifier 前拒空账户名);Rust 字节串字面量不能含非 ASCII,中文账户名用 `"…".as_bytes()`。

## 剩余 follow-up(不阻塞链端)

- internal-vote 机构自治路径(管理员发起改名/加账户提案,非注册局路径);
- onchina 两阶段冷签流程(复用卡2 chain_submit):更新机构信息 / 新增账户 call_data 编码 + 两阶段 handler;私权注册流程把名字带进链上交易;
- App reconcile 增量源接链(改名/加账户后 App 跟上)。

## 状态
- 2026-07-03:建卡;链端(改名+新增账户+私权名上链)完成并全绿;onchina/App/internal-vote 路径列 follow-up。
