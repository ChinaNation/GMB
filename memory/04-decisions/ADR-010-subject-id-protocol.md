# ADR-010 治理主体 ID(SubjectId)的 SubjectKind 协议规范

- 状态:Accepted
- 决议日期:2026-05-06
- 关联任务卡:`memory/08-tasks/done/20260506-unified-subject-id-protocol.md`
- 关联前置:ADR-008(SFID step2b 双层验签)、ADR-009(personal-manage 拆分)、ADR-015(账户级内部投票管理员模型)
- 关联后续:`memory/08-tasks/done/20260506-rename-institution-id-to-subject-id.md`

## 背景

链上治理三类主体(内置主体 / SFID 注册机构 / 个人多签)共用同一个 48 字节 `SubjectId`(C 阶段命名修正后,A/B 阶段曾叫 `InstitutionPalletId`)作为 `admins-change::Subjects` 与 `votingengine` 反向索引的 storage key。

A 阶段(2026-05-04 引入 SubjectKind)**之前**的派生协议是"裸右填零":三类主体派生函数各写一份,字节布局都是 `payload + zeros`,**没有主体类型字节区分**。结果:

1. 内置主体(`sfid_number` ASCII)与 SFID 机构(`sfid_number` ASCII)字节空间高度重叠,理论上存在撞 key 风险(运营约定靠 GFR-/SFR-/SCR- 前缀人为隔离,无协议级保护)
2. 个人多签(32B blake2 hash)与 ASCII 主体的撞 key 概率 ≈ 2^-32(哈希熵巧合,工程上可忽略但不是结构性互斥)

## 决议

引入 `primitives::derive::SubjectKind` enum 作为永久 ABI 协议字节,把 `SubjectId`(原 `InstitutionPalletId`)重新规范为 **kind tag(1B) + payload(47B)** 结构化布局。

### 协议规范(永久不可变更)

```
SubjectId = [u8; 48]
布局:
  byte[0]:    kind tag(SubjectKind 字节值)
    0x00       留洞(防与零填充冲突)
    0x01       Builtin            (内置主体:NRC/PRC/PRB)
    0x02       SfidInstitution    (SFID 注册机构)
    0x03       PersonalDuoqian    (个人多签)
    0x04       OnchainAsset       (链上发行代币 — ADR-011 / 2026-05-07 新增)
    0x05       InstitutionAccount (机构账户级内部投票主体 — ADR-015 / 2026-05-08 新增)
    0x06..0xFE 保留(未来主体类型扩展)
    0xFF       Reserved 哨兵      (协议升级时启用)
  byte[1..48]: payload (47 字节,kind 决定语义)
    Builtin:           sfid_number ASCII 字节(≤47B)右填零
    SfidInstitution:   sfid_number 字节(≤47B)右填零,仅表示同一机构归属/检索
    PersonalDuoqian:   32B AccountId + 15B 零填充
    OnchainAsset:      4B asset_id(u32 LE) + 43B 零填充
    InstitutionAccount:32B AccountId + 15B 零填充
```

### 增量条款(0x04 OnchainAsset / 2026-05-07,ADR-011 v2 落地)

- 用于代表"链上发行代币"治理主体,**仅作 storage key 派生**(不是发行人主体身份)
- `asset_id` 为 `pallet_assets::AssetId`(u32 LE 编码),链端通过该字节段反查内核资产
- payload 仅 4B,因 NextAssetId 自增已结构性互斥,SubjectId 不会撞 key
- 详细业务语义、监管六条铁律见 ADR-011

> v2 修订记录(2026-05-07):**v1 曾设计 8B issuer_subject_short(blake2_128 摘要) + 4B asset_id**,review 时识别为冗余:
> 1. 8B 摘要不可逆,无法反查发行人完整身份,反查走 `OnchainIssuance::Assets[SubjectId].issuer_subject_id`(48B 完整);
> 2. asset_id 自增已全局唯一;
> 3. 引入哈希依赖增加 primitives 复杂度。
> 协议位 0x04 还未上线,简化零迁移成本。

### 增量条款(0x05 InstitutionAccount / 2026-05-08,ADR-015 第1步落地)

- 用于代表 SFID 注册机构下面的某个具体可操作账户。
- payload 为账户 `AccountId` SCALE 编码后的前 32 字节，后续 15 字节零填充。
- `0x02 SfidInstitution` 继续保留，用于按 SFID 机构归属检索同一机构下的多个账户，不再作为新增账户级内部投票管理员主体的推荐 ID。
- 个人账户仍使用 `0x03 PersonalDuoqian`。
- 治理机构账户仍映射到 `0x01 Builtin`，同一治理机构共享固定管理员集合。

### 全局唯一性保证

各类主体的 SubjectId 第 1 字节互不相同,即使后续字节内容偶然相同,kind tag 也强制隔离。撞 key 概率从 2^-32(哈希熵巧合)降到 0(结构性互斥)。

### Rust API

```rust
// primitives/src/derive.rs
pub enum SubjectKind {
    Builtin = 0x01,
    SfidInstitution = 0x02,
    PersonalDuoqian = 0x03,
    OnchainAsset = 0x04,
    InstitutionAccount = 0x05,
}

pub fn build_subject_id(kind: SubjectKind, payload: &[u8]) -> Option<[u8; 48]>;
pub fn parse_subject_id(id: &[u8; 48]) -> Option<(SubjectKind, &[u8])>;

// 语义 helper(全工程统一入口)
pub fn subject_id_from_account<A: Encode>(account: &A) -> SubjectId;  // PersonalDuoqian
pub fn subject_id_from_registered_sfid_number(sfid_number: &[u8]) -> Option<SubjectId>; // SfidInstitution
pub fn subject_id_from_sfid_number(sfid_number: &str) -> Option<SubjectId>;             // Builtin
pub fn subject_id_from_onchain_asset(asset_id: u32) -> SubjectId;                       // OnchainAsset
pub fn subject_id_from_institution_account<A: Encode>(account: &A) -> SubjectId;         // InstitutionAccount
```

### 长度约束

- `MaxSfidNumberLength` 从 `ConstU32<96>` 收紧到 `ConstU32<47>`(BoundedVec 入链强制守门)
- `MaxAccountNameLength` 不变(账户名不入 institution_id 协议)
- 内置主体 sfid_number 实测 33B,远小于 47B 上限,兼容

## 不变量

1. **永久 ABI**:`SubjectKind::Builtin=0x01 / SfidInstitution=0x02 / PersonalDuoqian=0x03 / OnchainAsset=0x04 / InstitutionAccount=0x05 / Reserved=0xFF` 一旦上线不可改
2. **payload 上限 47B**:任何超过 47B 的 sfid_number 注册请求在 BoundedVec 阶段拒绝
3. **kind tag 不在已启用集合中的 SubjectId**:`parse_subject_id` 返回 None,`admins-change` 等下游应视为非法
4. **0x00 留洞**:防与零填充冲突,任何全零的 48B `[u8; 48]` 都不是合法 institution_id

## 客户端契约

- **wuminapp `admin_institution_codec.dart`**:PersonalDuoqian 检查 byte[0]==0x03 + byte[33..48] 全零；InstitutionAccount 检查 byte[0]==0x05 + byte[33..48] 全零；SfidInstitution 检查 byte[0]==0x02 + 提取 byte[1..] 去尾零
- **wuminapp `institution_admin_service.dart::_sfidNumberToFixed48`**:`out[0] = 0x01` + `out.setAll(1, raw)`
- **wumin `payload_decoder.dart`**:institution_id 字段透传,不解码内部字节(无影响)
- **node `storage_keys.rs::subject_id_from_sfid_number`** + `admin_subjects_key`:offline storage key 计算同步加 0x01 kind tag
- **sfid backend**:不解析 institution_id 字节(无影响)

## 协议升级路径

未来若需要引入新主体类型(例如"立法机构 Legislative"):

1. 在 `SubjectKind` enum 加新 variant + 分配 `0x06..0xFE` 中的字节值
2. `parse_institution_id` 接受新 variant 解码
3. 业务字段 `org` 视需要分配新值(NRC/PRC/PRB/REN/PUP/OTH 之外)

若需要 payload > 47B 的扩展(例如未来 sfid_number 增长到 64B):

1. 启用 `0xFF Reserved` 哨兵作为新协议版本标记(`SubjectKind::ExtendedV2`)
2. 在新 storage_version 的 storage migration 中迁移旧数据
3. 客户端按新 kind tag 0xFF 路由到新解码器

## 决议执行结果

- 链端 cargo test 全过(primitives 19 / citizenchain --lib 37 / duoqian-transfer 20 / admins-change 31)
- wumin flutter test 105/105 passed
- wuminapp duoqian flutter test 30/30 passed
- 残留扫描 3 项全零(旧函数名 / MaxSfidNumberLength=96 / 别名)
- 链未上线,fresh genesis 即生效;无 storage migration

2026-05-08 追加执行结果:

- `0x05 InstitutionAccount` 已在 `primitives::derive::SubjectKind`、`parse_subject_id`、`subject_id_from_institution_account` 中落地。
- `cargo test --manifest-path citizenchain/Cargo.toml -p primitives --lib`：24 passed。
- `cargo test --manifest-path citizenchain/Cargo.toml -p admins-change --lib`：34 passed。

## 与 ADR-008 / ADR-009 的边界

- ADR-008 step2b/step2d:SFID 机构注册凭证 (province, signer_admin_pubkey) 双层验签 — D 阶段不影响
- ADR-009:personal-manage 拆分 — 已落地,本协议进一步统一三类主体派生
