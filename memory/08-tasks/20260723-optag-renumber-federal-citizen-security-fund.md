# 任务卡：op_tag 重编号 + 联邦公民安全基金 + 联邦安全局局长岗位

状态：★ Step 1 + Step 2 全部完成并验证全绿（2026-07-23）★
- `cargo test --workspace` = **81 套件 ok，零失败/零 error/零 panic**；`cargo test -p primitives`（派生金标）74+1+1 绿；`cargo build -p citizenchain --release`（no_std WASM）通过。
- 金标已用 `scripts/sync-derive-vectors.sh --write` 重生（canonical + CitizenApp Dart 副本）；check 模式相对**已提交版**报 diff 属预期（本次是有意的地址变更，随创世一并提交后即消失）。
- Step 2 决策（用户确认）：创世期岗位**不设任期**（`term_required = false`），任期规则由运行期业务模块逐个规范。
- Step 2 实现：`public-manage/institution/role.rs` 抽出通用 `store_vacant_genesis_role`，LR 与新增 `store_genesis_director_role` 共用（避免重复代码）；`seeder.rs::insert_public_institution` 内按 `institution_code == FSC` 追加局长岗位。FSC 的联邦公民安全基金账户由约束表**自动派生播种**，未改 seeder 账户逻辑。

## 五端验证进度（2026-07-23）★ 全部完成，五端全绿 ★

| 端 | 结果 |
|---|---|
| citizenchain | `cargo test --workspace` **81 套件 / 1222 用例通过 / 0 失败 / 0 panic**（cargo 自身退出码 0）；`cargo test -p primitives` 派生金标 + 签名金标绿；`cargo build -p citizenchain --release`（WASM）通过 |
| CitizenApp | `flutter test --concurrency=1` **793 过 / 5 skip / 0 失败**；`flutter analyze lib` 零问题；`dart format` 0 changed |
| CitizenWallet | `flutter test` **190/190**；`flutter analyze lib` 零问题；`dart format` 0 changed |

> 验证方法提醒：`cargo test --workspace \| tail -40` 拿到的退出码是 `tail` 的、日志也只剩尾部，**不能作为链端全绿证据**（本轮踩过一次）。必须 `> log 2>&1` 后直接取 cargo 退出码。

`sync-derive-vectors.sh` check 模式相对**已提交版**仍报 diff，属预期（本次是有意的地址变更 + ADR-040 键名改造，随创世一并提交后消失）。

**已定位并修复的六个真实缺口（全部是「改了模型没改生成器」，被本次重派生暴露）：**
1. `citizenapp/lib/citizen/shared/account_derivation.dart` 的 Dart op_tag 镜像仍是旧编号 → 已同步新编号 + 补 `kOpFcsf`。修复后**派生金标 11/11 全过**。
2. `citizenchain/runtime/primitives/tests/account_derive_golden.rs` 仍写 `address_hex`/`creator_hex`，Dart 侧已按 ADR-040 用 `account_id`/`creator_account_id` → 已改生成器 + canonical fixture 键名并重生。
3. `scripts/generate_citizenapp_governance_registry.mjs` 仍发 `mainAccount` 等旧字段名，模型已改 `mainAccountId` 等 → 已改生成器并重生（CitizenApp + CitizenWallet 各 89 机构）。
4. `citizenapp/test/governance/shared/account_derivation_test.dart` 硬编码 `0x07` → 已改 `0x00`。
5. **同一生成器发 `OrgType.account`，模型 `64a95ec7` 已把该常量改名 `OrgType.institution`** → 生成物编译失败，**18 个测试文件全部 load 失败**（约 130 个用例根本没跑起来，故基线看起来只有 663 过）。18 项失败**同一根因、同一行**。已改生成器 line 149 并重生。
6. **同一生成器把 AccountId 发成裸 hex（无 `0x`）**，ADR-040 规范形式是 `^0x[0-9a-f]{64}$`（单源 `isAccountIdText()`）→ 已加 `dartAccountId()` 统一补 `0x` 并做格式断言，防止再漂。

**顺带补的两处根因防护：**
- 生成器末尾接 `dart format`，生成物直接落在格式稳定态（此前提交版是手工 format 过的，重生就会格式打架，掩盖真实 diff）。
- `institution_info.dart` 里 `mainAccountId` 的文档注释仍写「不含 0x」，与 ADR-040 相反 → 已改正（错误注释正是缺口 6 的助推器）。

**结构性原因（值得单独记一笔）：** `/scripts/` 整目录在 `.gitignore` 里，生成器**不受版本管理**。所以每次模型改名，diff/review 都看不见生成器，漂移只能等下一次重生时以编译失败的形式爆出来。这就是同一个生成器一次性攒出 3 处（缺口 3/5/6）陈旧引用的原因。

## 遗留（非本卡缺口）
- 提交时需一并提交刷新后的两份金标 fixture + 两份机构注册表生成物。

## 生成器纳入版本管理（2026-07-23，用户确认后执行）

根因治理：`/scripts/` 原为「本机私密脚本区」整目录忽略，生成器不受版本管理 → 模型改名时 review 看不见它。用户确认**只纳入 A 类 4 个生成器**，B 类 9 个 AI 工作流脚本保持本机私密。

`.gitignore` 改法（**关键陷阱**：父目录被排除时 `!` 反选无效，必须排除目录内容再反选）：
```
/scripts/*
!/scripts/generate_citizenapp_governance_registry.mjs
!/scripts/rederive_accounts.py
!/scripts/rebake_china_codes.py
!/scripts/sync-derive-vectors.sh
```
纳入前已做安全扫描：4 个脚本无密钥/token/本机绝对路径/个人信息。已核验 B 类 9 个 + `__pycache__` 仍全部被忽略。

## 落地进度
**Step 1 全部完成，验证全绿：**
- `account_derive.rs`：op_tag 重编号（OP_NAME=0x00 永久冻结 … OP_CLEARING=0x07）+ `OP_FCSF=0x08` + 保留名/枚举/全部 match·payload·名称路由分支 + 字节空间分区注释（0x00-0x0F 派生 / 0x10-0x1D 签名 / 0x1E-0xFF 保留，块 A 用尽从 0x20 开块 B，新增 tag 不改既有地址）。
- `cid/code.rs`：+`pub const FSC`。
- `institution_constraints.rs`：+`FSC_PROTOCOL_ACCOUNT_KINDS`、import、match 分支（接在另一线程「清算行资格=SFGF+父级SFGF的UNIN」最终签名上）；+`ROLE_CODE_DIRECTOR=b"DIRECTOR"`、`ROLE_NAME_DIRECTOR="局长"`。
- node/onchina 三处协议账户→短名匹配补 `federal_citizen_security_fund` 分支；onchina 过期注释 `OP_NAME=0x07`→`0x00`。
- `configs.rs`：+`is_federal_citizen_security_fund_account`（public_manage 反查索引）、`can_spend` 该基金 **fail-closed 全拒**、`is_reserved` 防占号。
- `scripts/rederive_accounts.py`：**补覆盖 `citizenchain.rs`**（原七文件清单遗漏，导致金标先红）+ 更新过期 docstring。
- 重派生已写回：main 297 / fee 297 / stake 43 / safety_fund / 639 保留地址；金标经 `scripts/sync-derive-vectors.sh --write` 重生（canonical + CitizenApp Dart 副本）。
- 验证：`cargo test -p primitives` 74+1+1 绿；**`cargo test --workspace` 19 套件全绿、零失败**；`cargo check --workspace` 绿。

**Step 2 剩余（唯一）：局长岗位创世播种**
- 基金账户**无需改 seeder**：`insert_public_institution` 已按 `required_protocol_account_kinds` 动态派生并播种，FSC 的 FCSF 账户自动生成。
- 待做：给 FSC 在创世写入 `DIRECTOR`（局长）岗位。参照 `public-manage/src/institution/role.rs:202 store_default_legal_representative_role`（含 `store_role_permissions_from_fixed_directory` 权限目录接线），在 `insert_public_institution` 内按 `institution_code == FSC` 追加。
- **待用户拍板**：局长岗位 `term_required` 取 true 还是 false？（LR 为 false=无任期；局长若属任期制应为 true。）

## 任务需求
1. **op_tag 重编号**（一次性、永久受益）：把自定义命名账户永久钉在 `0x00`，协议账户从 `0x01` 起依次排列，今后新增协议账户只取"当前最大+1"，**永不再重编号**。
2. 新增协议账户 **联邦公民安全基金**（`OP_FCSF=0x08`），专属联邦安全局（FSC）。
3. 给已是创世机构的 **FSC** 增加该基金账户 + 创世岗位 **局长（`DIRECTOR`）**。

## 已确认决策（用户 2026-07-23 逐条确认）
- `OP_FCSF=0x08`，保留名「联邦公民安全基金」。
- 该基金由联邦安全局**经投票引擎**支出，属业务模块内容 → 本卡 `can_spend` 对该基金 **fail-closed 全拒**，支出路径留待业务模块设计时再放行。
- 局长岗位名「局长」，岗位码 **`DIRECTOR`**（对齐 SENATOR/REPRESENTATIVE/COMMITTEE_MEMBER/LR 风格）。
- 走**重新编号**方案（非"冻结 OP_NAME"），用户明确要求一次性解决。

## op_tag 终态 + 字节空间分区（写入注释固化）
```
0x00  OP_NAME      自定义命名账户  ← 永久冻结
0x01  OP_MAIN      0x02 OP_FEE     0x03 OP_STAKE
0x04  OP_SAFETY    0x05 OP_HE      0x06 OP_PERSONAL
0x07  OP_CLEARING  0x08 OP_FCSF    联邦公民安全基金(新增,专属 FSC)
0x09–0x0F  派生块 A 余量
0x10–0x1D  签名域(sign.rs,禁用)
0x1E–0xFF  未分配保留;块 A 用尽从 0x20 开块 B —— 新增 tag 不影响任何既有地址
```
硬规则：派生 tag 绝不得与签名 tag（0x10–0x1D）撞值，两者共用 `GMB‖op_tag‖…` 域分离。

## 关键事实（排查所得）
- op_tag **进哈希**（`blake2_256(GMB‖op_tag‖ss58_le‖payload)`）→ 重编号会**重新派生全机构所有账户地址**。
- **FSC 已在常量库且已创世播种**：机构码 `FSC`、CID `ZS001-FSC0W-434172688-2026`、在 `CHINA_ZF` 挂总统府名下，`seeder.rs` 已按 `CHINA_ZF` 播种 → Step 2 只需加基金账户 + 局长岗位，不是新建机构。
- onchina `institution/accounts/derive.rs` 有硬断言 `OP_NAME=0x07`，须同步改 `0x00`。
- 重生工具链已存在：`scripts/rederive_accounts.py`（**从 account_derive.rs 读 op_tag，单源自动跟随**）、`scripts/sync_account_derive_vectors.sh --write`、机构注册表生成器。

## 改动清单
**Step 1（重编号 + 基金账户类型）**
- `primitives/src/account_derive.rs`：8 tag 重编号 + `OP_FCSF`；+RESERVED_NAME、AccountKind、InstitutionProtocolAccountKind 及各分支；固化分区注释。
- `primitives/src/institution_constraints.rs`：`FSC_PROTOCOL_ACCOUNT_KINDS=[Main,Fee,FederalCitizenSecurityFund]` + required 分支。
- `runtime/src/configs.rs`：+`is_federal_citizen_security_fund_account`（单一派生常量比对，仿 safety fund）；`is_reserved` 防占号；`can_spend` 该基金 **恒 false**。
- 五端：onchina derive.rs（含 0x07→0x00 断言）、node、CitizenWallet、CitizenApp 派生镜像。
- `scripts/rederive_accounts.py`：覆盖 OP_CLEARING + OP_FCSF；**更新过期 docstring**（现仍写 0x06=OP_INSTITUTION）。

**Step 2（FSC 基金账户 + 局长岗位）**
- china 常量：FSC 的基金地址（重派生脚本产出）。
- `genesis/src/institution/seeder.rs`：FSC 播种第三协议账户 + `DIRECTOR` 岗位（LR 自带）。

## 重生顺序
扩展 rederive 脚本 → 跑重派生（china 全部常量）→ `sync_account_derive_vectors.sh --write` → 重跑机构注册表生成器。

## 验收标准
- `cargo test -p primitives`（派生金标）→ `cargo test --workspace` → `cargo build -p citizenchain --release`（WASM）→ 五端 analyze/test → `sync_account_derive_vectors.sh` check 模式无 diff。
- 全仓无旧 op_tag 残留；onchina 断言已改。

## 排期约束
全机构地址会变 → **必须赶在 account_id 统一任务第 10 步「总验收 + 重新创世」之前完成**。
