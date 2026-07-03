# CID 链上统一校验(占号体系 卡1)

> 设计真源:`memory/04-decisions/ADR-031-cid-occupy-registry.md`(D1)。家族断言复用现有谓词:`is_person_code`(CTZN)/`is_public_legal_code`/`is_private_legal_code`/`is_unincorporated_code`(code.rs:874-917)。

## 背景

- 链端对 CID 号没有真校验:`citizen-identity` 只查 `starts_with(b"CTZN")`,而真实公民号为 `GD000-CTZN1-…`(以省码开头,CTZN 在第二段),所有真实号必被 `InvalidCitizenCode` 拒绝——CRITICAL,公民身份上链功能整体不可用;`public-manage` / `private-manage` 注册入口只查非空。
- 该断裂被测试掩盖:pallet 单测与 citizen-issuance 集成测试全部使用 `b"CTZN-0001"` 手造假夹具,从未使用真实生成器产物。
- 校验规则单源已在 runtime 常量库 `primitives::cid::number::parse_cid_number_parts`(no_std,段结构+机构码+盈利位+校验和),pallet 可直接调用,链上链下逐字节一致。

## 目标

- ~~D0 机构码四级补齐~~ **已定稿不补码(2026-07-03 用户拍板)**:92 码表即四级完整——镇级不设立法/教委、省级不设省教委/省公安厅是制度设计;名称统一与生成条件核验已于 2026-07-03 通过(cid 测试 28 项全绿、87 储备机构零不一致、onchina 编译过)。本卡只做链端校验接入。
- 三个链上写入口统一调用 `parse_cid_number_parts` 做全量格式校验。
- 各入口断言机构码家族:`citizen-identity` = CTZN;`public-manage` = 公权类码;`private-manage` = 私权类码(家族判断谓词收敛 `primitives::cid::code` 单源)。
- 删除 `starts_with(b"CTZN")` 残桩,错误语义保留 `InvalidCitizenCode`(或按家族细分)。
- pallet 单测与集成测试夹具全部替换为 `generate_cid_number` 真实产物,禁止手造假号,防止再漂移。

## 修改范围

- `citizenchain/runtime/otherpallet/citizen-identity/`
- `citizenchain/runtime/entity/public-manage/`、`citizenchain/runtime/entity/private-manage/`
- `citizenchain/runtime/primitives/cid/`(如需补家族判断辅助函数)
- 相关 `src/tests/`、`citizenchain/runtime/issuance/citizen-issuance/tests/`

## 验收

- 真实生成号(公民/公权/私权)全部通过链端校验;篡改校验位、错家族码、旧版格式被拒。
- `cargo test -p citizen-identity -p public-manage -p private-manage -p citizen-issuance -p primitives` 通过。
- `cargo fmt --all` 通过。
- 全仓无 `b"CTZN-0001"` 类手造假夹具残留。

## 状态

- 2026-07-02:建卡。
- 2026-07-03:**完成**。落地内容:
  - `primitives::cid::number` 新增 `parse_cid_number_parts_bytes` 字节入口(单源复用)。
  - `citizen-identity::ensure_valid_voting_payload` 删 `starts_with(b"CTZN")` 残桩,改全量解析 + 机构码必须 CTZN(修真实号全拒的 CRITICAL)。
  - `public-manage` register/create、`private-manage` register/create 接入全量解析 + 家族断言(公权 `is_public_legal_code`;私权 `is_private_legal_code`||`is_unincorporated_code`),create 另断言号内机构码与 `institution_code` 参数一致;两 pallet 新增 `Error::InvalidCidNumber`。
  - 测试夹具全仓换真号:citizen-identity(15)、citizen-issuance 单测/集成/benchmark(12+5)、public/private-manage 测试与 benchmark(34+34,含 close 路径 helper 改 tag 签名)、runtime 主 crate(30,含 GCB 旧码假号)、public/private-admins 管理员档案字段字面量;新增 3 个家族拒绝用例(公民入口拒 CGOV、公权入口拒 SFLP、私权入口拒 CGOV);公权 UNIN 用例期望错误改 `InvalidCidNumber`(家族断言先于 lifecycle 检查)。
  - 顺手修工作区既有断链:entity 两 pallet 测试 mock 补 `type InstitutionQuery = ();`。
  - 验收:`cargo test -p primitives -p citizen-identity -p citizen-issuance -p public-manage -p private-manage` 与 `cargo test -p citizenchain --lib` 全绿;`cargo check --features runtime-benchmarks` 过;全仓 `b"CID-*"`/`b"CTZN-*"` 手造假号零残留;`cargo fmt --all` 过。
