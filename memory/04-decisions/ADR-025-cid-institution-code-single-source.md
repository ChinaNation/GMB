# ADR-025 机构分类统一为 CID 号机构码唯一真源（删 org_code + 链 ORG_xx + subject_property）

- 状态：**全栈代码完成并验证（2026-06-22），2026-06-25 收敛到 runtime primitives `code.rs` 唯一常量真源**，未提交未推送，重新创世 gated 待用户部署。
  - 链端 `cargo test --workspace` ~695 测试 0 失败；node 174；冷钱包 citizenwallet 81/81；热钱包 citizenapp 88 + `flutter analyze` 0；citizencode 后端 76、前端 `tsc` 0。
  - 全产品旧分类残留 grep = 0（链端旧 u8 分类、后端旧分类列、前端旧短码、Dart 旧分类字段）。
- 关联：[[ADR-021]] 行政区单源 / [[ADR-024]] 账户派生单源（同"单一真源"思想，本 ADR 末尾创世与 ADR-024 Tier 3 合并）；任务卡 `20260622-cid-classification-unify-t3t4.md`
- 取代：[[ADR-010]] subject-id-protocol（SubjectKind/K1 主体属性段已废）

## 背景（问题）

同一件事——**机构是什么类型**——在仓库里被编码了**三套互相平行的分类标准**，且各自漂移：

| 平行分类 | 位置 | 取值 |
|---|---|---|
| `subject_property`（K1） | CID 号段 + 后端 `SubjectProperty` 枚举 + DB 列 | G/F/S/M（公/非法人/私/公民） |
| `org_code` | 后端 `subjects.org_code`/`gov.org_code` 列 + 50+ SQL/DTO/前端 label | `NATIONAL_PRESIDENT_OFFICE`/`CITY_POLICE`/… 字符串 |
| `ORG_xx` | 链端 `votingengine::types` 常量 `org: u8` | 0..=5（NRC/PRC/PRB/REN/PUP/OTH） |

三套都在重复表达"这个主体是政府/储委会/公司/个人多签…"。用户铁律：**不要再搞出任何两套的分类标准**。CID 号里本就嵌了**机构码**（`R5-seg2-N9-D4` 的 seg2），它天然唯一标识机构类型——应作为唯一真源，其余全部删除或从它派生。

## 决策

**机构分类唯一真源 = CID 号机构码（institution_code，92 码）。** 所有"是不是公权/私权/个人/某档治理机构"一律从机构码派生，绝不另立第二套。

### 92 码体系（双布局，总长 26 不变）
- 常量唯一真源 `citizenchain/runtime/primitives/src/code.rs`:
  `CountryCode`、`ProvinceCode`、`InstitutionCode`、`INSTITUTION_CODE_INFOS`、
  `cid_short_name`、盈利策略、行政层级和治理谓词全部在此维护。
- CID 后端 `citizencode/backend/number/code.rs` 只 re-export / wrap `primitives::code`,继续服务
  CID 号生成、解析和校验;不得恢复第二份机构码枚举、第二份 `ALL` 码表或第二份中文标签表。
- 码段二布局：3 字符码（国家/省部）= `码(3)+盈利位(1,公权恒0)+校验(1,mod-36)`；4 字符码（市/镇/私权/个人）= `码(4)+M1(1)`。靠 seg2 index3 数字/字母分流。K1 主体属性段删除。
- 国家/省级代码也纳入 `code.rs`:国家为 `CN`,省级行政区为 43 个两位大写 `ProvinceCode`。市镇代码仍由 CID `china.sqlite` 管理。

### 三阶段消除
- **Phase 1（后端 number/）**：删 `subject_property`/`SubjectProperty` 枚举 + DB 列；机构类别一律从机构码派生。92 码双布局落地。
- **Phase 2（后端 org_code）**：删 `subjects.org_code`/`gov.org_code` 列 + 50+ 消费方；改为 primitives 谓词 `admin_level()`、`is_city_police_code()` 和纯码匹配派生。`registry_org_code`（管理员授权范围 FEDERAL/CITY_REGISTRY，与机构分类无关）**保留**。citizencode 前端删除旧机构标签 DTO；citizenapp 删除旧机构分类字段并重生 Isar。
- **Phase 3（链端 ORG_xx，重新创世）**：旧 `org: u8` 全替换为 `institution_code: [u8;4]`（~499 引用/48 文件）；阈值 `DoubleMap` **保留结构**只改腿类型（用户"阈值存储键保持"）；固定治理档 NRC/PRC/PRB=13/6/6 不动。china_*.rs 282 内建 cid_number 重烤为专属码（脚本 `scripts/rebake_china_codes.py`，base36 校验位同后端）+ `scripts/gmb.py --apply` 重派生账户。客户端线格式 `org` 1 字节 → 机构码 4 字节（冷/热钱包解码器 offset、node TS invoke 参数）。

### 治理档派生（取代 ORG_xx 语义）
| 旧 ORG | 新派生 |
|---|---|
| ORG_NRC/PRC/PRB（固定阈值） | `is_fixed_governance_code` + `fixed_governance_pass_threshold(&code)`（china 内建表校验） |
| ORG_REN（个人多签） | `is_personal_code`（PMUL，管理员来自 personal-manage） |
| ORG_PUP/OTH（机构账户） | `is_institution_code`（公权/私权法人，管理员来自 organization-manage） |

## 影响
- **重新创世**：改 CID 号 = 改 china_*.rs 常量 = 派生账户变 = 必须重新创世（用户同意，pre-genesis）。china/账户/代码已就绪，重生 `citizenchain.raw.json`+出 deb+重启节点为用户部署步骤。
- 链端阈值/票数/人数（13/6/6、19/1/1、19/9）统一派生 `primitives::count_const`，桌面端不再硬编码。
- error/storage 命名去 org：`ProposalsByOrg`→`ProposalsByCode`、`*OrgMismatch`/`InvalidOrg`/`InvalidInternalOrg`→`*Code*`（原地改保留 SCALE index）。
- 展示侧机构码中文名只允许来自 `cid_short_name`;不得恢复旧标签字段或任意第二份标签表。
- `OrgType`（node 展示枚举 NRC/PRC/PRB）保留为薄标签，值全部 primitives 单源，非独立分类。

## 备选方案
- **链端用全量变体 enum 镜像后端**：否决——等于把码表复制一份到链上再维护。选 `[u8;4]` 原始码字节 + 少量谓词，最贴合"直接用机构码"。
- **ORG_xx 改名 GOVTIER_xx 保留 u8 标签**：用户否决——仍是第二套分类，要求直接用机构码。
- **阈值 DoubleMap 删 org 腿（AccountId 已全局唯一）**：用户选"阈值存储键保持"，故只改腿类型 u8→[u8;4]，降低风险。

## 后续动作
- [ ] 用户部署：重生 `citizenchain.raw.json` + 出 deb + 重启 6 节点；重跑公权机构数据包 + citizenapp 机构注册表生成器（china 重派生后必须）。
- [ ] 提交（用户授权后）。
