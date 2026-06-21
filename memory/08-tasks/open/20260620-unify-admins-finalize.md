# 统一管理员体系 — 收尾修复

- 状态：代码完成，待节点桌面三路径真机冒烟
- 完成记录(2026-06-20)：
  - Phase 1 节点前端契约对齐：tsc exit=0；invoke 键 `admins`/`org` 对齐后端，详情 DTO `org`/`adminsLen`/`adminsSs58` 对齐 serde
  - Phase 2 链端改名：`AdminsOf`/`InvalidAdminsLen`/`AdminsLenMismatch`/`InternalAdminsLenProvider`/`RuntimeInternalAdminsLenProvider`，删 5 处 `EMPTY_DUOQIAN_ADMINS`+py 正则，补漏 `InvalidAdminOrg→InvalidOrg`(索引 13 不变)；`cargo check` lib 全过(organization-manage/personal-manage/admins-change/votingengine)
  - Phase 3 客户端+文档：citizenapp/citizenwallet `adminCount→adminsLen`/`adminOrg→org`/`DuoqianAdminSnapshot→AdminSnapshot`/`newAdmins→admins`，chain_rpc 错误标签同步；修一处 citizenwallet 解码器命名碰撞(`adminsVecLen`)；flutter analyze 0 + citizenapp 26 测试 + citizenwallet 117 测试全过
  - 全仓库(范围内)零残留；CID 注册局/CPMS 操作员 admin 域按约定不动
  - 注意：`cargo check --workspace --tests` 有既有失败(PQC vote-credential + CidInstitutionVerifier 测试 mock 签名过期)，与本次 admins 改名无关，属其它在途工作
- 历史状态：进行中
- 创建：2026-06-20
- 背景：提交 `25798a52`(统一管理员体系) 把节点后端 Tauri 命令参数 + serde 字段 + 链端字段/事件改名为 `admins`/`org`/`admins_len`/`adminsSs58`，但节点前端 TS 未同步、链端只改字段层未改类型/错误/trait/常量层。本卡把统一收口为「全仓库零例外」。
- 口径基线：后端新名为规范，旧的一侧一律向新名对齐，不做兼容层。
- 范围边界：统一所有机构和个人多签管理员，唯一真源为 `admins-change::AdminAccounts`；CID 注册局机构管理员和 CPMS 机构管理员都必须收口到机构 `admins`，CPMS 本地非机构人员只叫 `operators`。

## 改名口径表
- 新增或替换管理员参数统一使用 `admins`；不再保留旧管理员公钥数组字段。
- 管理员数量字段统一为 `admins_len`；多签管理员 SS58 展示字段统一为 `admins_ss58`(serde→`adminsSs58`)。
- `DuoqianAdminsOf<T>` → `AdminsOf<T>`（`DuoqianTransfer`/`DuoqianAccountOf`/`PersonalDuoqians` 属多签业务域名，保留）
- `EMPTY_DUOQIAN_ADMINS` → 删除（死常量）
- `InvalidAdminCount`/`AdminCountMismatch` → `InvalidAdminsLen`/`AdminsLenMismatch`
- `InternalAdminCountProvider`/`RuntimeInternalAdminCountProvider` → `InternalAdminsLenProvider`/`RuntimeInternalAdminsLenProvider`
- 客户端 `adminCount`/`DuoqianAdminSnapshot`/`newAdmins`(局部) → `adminsLen`/`AdminSnapshot`/`admins`

## Phase 1 — CRITICAL 节点前端契约对齐（纯 TS，先发）
- admins_change/api.ts：invoke 键 `newAdmins`→`admins`
- organization-manage/api.ts + create-multisig.tsx：invoke 键/字段 `adminOrg`→`org`、`adminPubkeys`→`admins`
- organization-manage/types.ts + institution-detail.tsx + offchain section/types/node-register/admin-unlock：`adminOrg`→`org`、`adminCount`→`adminsLen`、`duoqianAdminsSs58`→`adminsSs58`
- 验收：`npm run build`(tsc) 0 error + 真机三路径(机构创建/管理员更换/机构详情)

## Phase 2 — 链端命名彻底统一（encoding-neutral，随下次 runtime 升级带出）
- `DuoqianAdminsOf→AdminsOf`、删 `EMPTY_DUOQIAN_ADMINS`(+ scripts/fill_china_admins.py 正则)、Error 变体、Provider trait
- 无 storage 迁移、不碰 chainspec 创世值、不询问 spec_version
- 验收：`cargo build` + `cargo test` + `clippy -D warnings` + `rg` 零残留

## Phase 3 — 客户端 + 文档收尾（citizenapp/citizenwallet/节点前端内部命名 + 技术文档）
- citizenapp/lib/governance、citizenapp/lib/wallet/capabilities、citizenwallet/lib/signer：`adminCount→adminsLen`、`DuoqianAdminSnapshot→AdminSnapshot`、`newAdmins→admins`
- 文档：ADMINSCHANGE / ORGANIZATION_MANAGE / CROSS_MODULE_INTEGRATION / GOVERNANCE_TECHNICAL 同步 trait 名
- 验收：`flutter analyze` 0 + dart test + `rg` 零残留

## 分工
- Phase 1：CPMS/Mobile Agent（节点前端）
- Phase 2：Blockchain Agent（runtime）
- Phase 3：Mobile Agent（钱包）+ 文档回写
