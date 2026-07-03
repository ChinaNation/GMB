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
