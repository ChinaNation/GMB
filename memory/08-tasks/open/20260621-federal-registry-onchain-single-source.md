# 联邦注册局统一为链上单一真源（admins-change）

## 任务需求

- 机构管理员唯一真源统一为链上 `admins-change` pallet 的 `AdminAccounts`，任何系统不得再造第二套管理员真源。
- 联邦注册局只有一个：`总统府联邦注册局`（china_zf `ZS001-GZF0P-249474503-2026`），链上一个机构、一份 admins，链不区分省份；省份划分纯属 SFID 业务层给联邦注册局管理员分配的权限范围（联邦注册局管理员负责本省市注册局）。
- 联邦注册局管理员补齐为 43 省 × 5 = 215 个：每省 = 现有 SFID 该省注册局管理员 1 个（保留在最前）+ 弃用池补 4 个。
- SFID 后端不再内置注册局管理员、不再用 postgres `admins` 表当真源，改为只读链 + 保留省份标签映射。
- 创世把所有内置管理员（cb/ch 已写；补 zf/sf/jc/jy/lf 及总统府联邦注册局）写入 admins-change。

## 现状核实结论（只读已查实）

- `admins-change` 创世 `build()` 当前只遍历 `CHINA_CB`(国储会/省储会)、`CHINA_CH`(省储行)，**zf/sf/jc/jy/lf 常量库虽有内置 admins，但创世未写入** → 这是要补的核心缺口。
- china_zf 每机构内置 admins 历史 9 个，c13e0f82「重构身份ID协议规则」起砍到 5 个（保留前 5、删后 4）。84080b6a 版 54 机构 × 9，可从 git 完整找回被删 4 个。
- 弃用池（84080b6a 各机构第 6~9 个，去重）= 216；剔除已被现机构复用的 25 个 → 真正可用 191；需要 43×4=172，够（余 19）。
- SFID `federal_registry_admins.rs` = 43 省 × 1 管理员；助记词用户已备份，复用其公钥安全。
- 215 构造已校验：43 省 main + 172 弃用补位，全唯一，main 与弃用池零交集。

## 整组方案（本卡范围）

### A. runtime 常量：china_zf 总统府联邦注册局 admins 5 → 215
- 文件 `citizenchain/runtime/primitives/china/china_zf.rs`，仅改 `ZS001-GZF0P-249474503-2026` 一个机构的 `admins`。
- 顺序 = SFID `FEDERAL_REGISTRY_MAINS` 的 43 省序；每省 5 个 = [该省现有 SFID 管理员] + [弃用池顺序取 4]。
- **整体替换**总统府联邦注册局原 5 个 admins（用户已确认丢弃原 5）。
- 弃用池取法：84080b6a 版 china_zf 各机构 admins[5:]，去重、剔除当前在用，按出现序取前 172。

### B. runtime 创世：admins-change build() 扩展写入
- 文件 `citizenchain/runtime/governance/admins-change/src/lib.rs`，build() 增加遍历 `CHINA_ZF/CHINA_SF/CHINA_JC/CHINA_JY/CHINA_LF`，按各自 org 标签 `build_builtin_institution` 写入 admins-change（与 cb/ch 同路径），总统府联邦注册局随 zf 一起写入。
- **需单独 runtime 二次确认后才动。**

### C. SFID 后端：删内置 + 改读链
- `sfid/backend/admins/federal_registry_admins.rs`：删 `FEDERAL_REGISTRY_MAINS` 内置常量（管理员真源迁链）；保留 `admin→省` 只读 scope 映射（非第二真源，仅权限范围属性）。
- postgres `admins` 表退出真源角色（CHECK 仅 FEDERAL/CITY、无机构号列，承载不了统一模型）。
- 新增按机构 sfid_number 读链 admins 的能力（联邦注册局读总统府联邦注册局 admins）。

### D. 重新创世
- 往创世写 zf/sf/jc/jy/lf + 215 联邦注册局管理员属于改创世状态，必须新 WASM 重生 chainspec（开发链重新创世，非 setCode）。
- 重新创世后须重跑 SFID 机构注册表 / CitizenApp 公权机构数据包生成器。

## 预计修改目录

- `citizenchain/runtime/primitives/china/china_zf.rs`（代码：总统府联邦注册局 admins 5→215；**runtime 二次确认已获**，本卡 A 步执行）
- `citizenchain/runtime/governance/admins-change/`（代码：创世 build() 扩展写 zf/sf/jc/jy/lf；**待单独 runtime 二次确认**）
- `sfid/backend/admins/`（代码+残留清理：删内置注册局管理员常量、改读链、保留 scope 映射）
- `memory/`（文档：本任务卡 + 后续 ADR）

## 本卡执行步骤与状态

- [x] 现状核实（china_zf 9→5、弃用池、215 构造校验）
- [x] A. 改 china_zf.rs：总统府联邦注册局 admins → 215（runtime 二次确认已获）
- [x] A 验收：`cargo check -p primitives` 通过；`cargo test -p primitives` 16/16（含 china_other check_arr! 派生断言）；总统府联邦注册局 admins=215，其余 58 机构仍 5，全文件 hex 零重复；`cargo fmt -p primitives --check` 退出码 0
- [x] B. 创世 build() 扩展（runtime 二次确认已获）：`admins-change/src/lib.rs` build() 增 5 个 china import + 局部宏 `insert_pup_builtin!` 把 CHINA_ZF/SF/JC/JY/LF 以 ORG_PUP 写入 admins-change（总统府联邦注册局随 zf 写入）。
- [x] B 验收：测试 mock `MaxAdminsPerInstitution` 32/64→1989（admins-change/duoqian-transfer/organization-manage/personal-manage 共 4 文件 8 处，因联邦注册局 215 超旧 mock 上限）；admins-change 边界用例 `institution_account_at_max_admins_works` 账户生成改按 i 唯一(原 `(i&0xff)` 在 >256 重复)；`cargo test --workspace --exclude citizenchain` 全绿(admins-change 41/duoqian 23/org 26/personal 23/votingengine 174…)，fmt 全过。主 crate 集成测试需 WASM_FILE，留待 D 重新创世时连带验证。
- [x] B 上限澄清：两个独立 Config 项——`MaxAdminsPerInstitution`(机构多签=1989) vs `MaxPersonalAccountAdmins`(个人多签=64)。本轮只动前者(4 mock 各有 test_genesis_config_builds 构建含 215 联邦注册局的全链创世,故都需 1989)；后者 64/16 全程未动。与真实 runtime(configs/mod.rs:1087/1089)一致。
- [ ] B-follow：`integrity_test` 的 `required` 仅算 NRC/PRC/PRB(19)，未含 ORG_PUP 内置(联邦注册局 215)；真实 runtime 1989 安全，但建议补 max(china_zf admins) 校验防误配（低优先）。
- [x] C. SFID 删内置联邦注册局清单（按用户指令：内置管理员真源已在 china_zf 链上常量，SFID 不再内置、不在 SFID 读链——读链不在本卡范围）：
  - 删除 `sfid/backend/admins/federal_registry_admins.rs`（FEDERAL_REGISTRY_MAINS 43×1 常量 + 3 个 helper）。
  - 清引用：`admins/mod.rs` 删模块声明；`main.rs` 删 `ensure_builtin_federal_registry_admins` 调用 + 无用 glob `use runtime_ops::*`；`core/runtime_ops.rs` 删该 seeding 函数 + 连带 4 个失效 import；`admins/repo.rs::find_federal_registry_scope_conn` 常量 fallback 改 `Ok(None)`（省份只认 postgres federal_registry_scope 表）；`admins/login/signature.rs::build_admin_display_name` 删常量显示名分支（参数标 `_admin_account`）；`admins/actions.rs::actor_is_initial_federal_registry` 改用 postgres `built_in` 标记判定（+ delete 守卫同步）。
  - 保留：`catalog.rs::list_federal_registry_admins` / `repo.rs::*_federal_registry_admins_by_province_conn`（查 postgres admins 表的 API，非被删文件）。
  - 验收：`cargo check` 通过，**零 warning 零 error**；全仓被删文件符号零残留。
- [ ] D. 重新创世（**用户指令：现在不做**，部署时再由用户执行：构建新 runtime WASM→重生 citizenchain.raw.json→SFID/CitizenApp 数据包重生）

## 验收

- china_zf：总统府联邦注册局 admins 恰 215，每省首位 = SFID 该省现有管理员，其余 4 来自弃用池；全文件 admins 无新增重复；`cargo check`/`cargo build` 通过。
- 创世：admins-change 内 zf/sf/jc/jy/lf + 总统府联邦注册局均为 Active 内置；国储会等不回归。
- SFID：无任何内置注册局管理员常量、无 postgres admins 真源；联邦注册局管理员页读链展示；scope 按省过滤不变。
- 全端：CitizenApp 读同一 admins-change，端到端一致。

## 后续 follow-up 任务卡（不在本卡）

- 所有机构详情页（公权/公安/私权）新增「管理员列表」tab + 注册局权威新增/删除管理员（路 B）。
- CPMS 安装码携带管理员 + 校验 ≥1 + 去掉扫公民钱包绑管理员步骤（初始化成功直跳登录）。
- CitizenApp 同步：注册局发起的机构管理员创建/变更链上交易提交与读取。
- 普通机构/市注册局 管理员"注册局权威 vs 机构自治投票"分界落地（治理机构走投票，普通机构走注册局）。
