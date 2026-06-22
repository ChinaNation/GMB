# 任务卡：机构名称单一真源 = cid_full_name / cid_short_name + 两 UI 显示简称 + 清理第二源

- 任务编号：20260621-institution-name-single-source-cid-short-name
- 状态：done（2026-06-21 全落地验证；仅 2 项 follow-up）
- 所属模块：citizencode/backend（gov 名称生成 + auth 投影）+ citizencode/frontend（两 UI + 清理第二源）
- 当前负责人：CID Agent
- 创建时间：2026-06-21

---

## 背景（用户拍板）

机构只有两个名字字段:全称 `cid_full_name`、简称 `cid_short_name`。系统里却"到处再造机构名称真源",
导致同一个"联邦注册局"被多处硬编码/派生制造。要求:① 单一真源就是这两个字段;② 管理员列表页头部 +
右上角徽标都显示**简称**;③ 清理所有再造机构名称的第二源头。联邦注册局:全称=`总统府联邦注册局`,简称=`联邦注册局`。

## 根因（已核实）

`gov/service.rs::official_name_pair()` 没有 `总统府联邦注册局` 的分支,落到默认 `_ => (name, name)`,
导致 `cid_short_name == cid_full_name == 总统府联邦注册局`——库里压根没有"联邦注册局"这个简称。
于是各处只能另造"联邦注册局"字符串(org_code 派生标签 / registry_org_code 角色硬编码 / 死常量)。

## 改动清单

### A. 根修(后端 DB 真源)
1. `gov/service.rs official_name_pair`：加分支 `"总统府联邦注册局" => ("总统府联邦注册局","联邦注册局")`(默认臂之前)。
2. 改完跑 `reconcile-gov --changed-only` 把 `cid_short_name=联邦注册局` 推进开发库。
- 注:4 个兄弟联邦局(安全/情报/特勤/人事局)同样 简称=全称,但它们无 CID 登录管理员、不在本次两 UI,简称值未经用户确认,**留作 follow-up**,不擅自造名。

### B. 简称投影到 auth(后端,供右上角徽标)
3. `admins/login/model.rs`：`AdminAuthContext`/`AdminAuthOutput`/`AdminIdentifyOutput` 各加 `institution_short_name: Option<String>`。
4. `admins/repo.rs`：新增 `resolve_home_institution_short_name_conn`(+ `&Db` 非 conn 包装),按 registry_org_code 查 subjects.cid_short_name(联邦=org_code='FEDERAL_REGISTRY' 单行;市=org_code='CITY_REGISTRY' AND province_name AND city_name)。
5. `guards.rs admin_auth`（check 路径,conn）/`handler.rs admin_auth_identify`(非 conn)/`admin_auth_verify`(conn)/`qr_login.rs`(非 conn) 四处填充。

### C. 两 UI 改读简称(前端)
6. 头部(place 1)`FederalRegistryAdminSubTab.tsx`：新增 prop `federalRegistryShortName`,Card title 由硬编码"联邦注册局管理员列表"改 `{简称} · {省}`（"联邦注册局 · 中枢省"）。`ProvinceDetailView.tsx FederalRegistryView` 传 `federalRegistryDetail.institution.cid_short_name`。
7. 徽标(place 2)`App.tsx resolveHeaderAdminIdentity`：删 registry_org_code 三元硬编码,改读 `auth.institution_short_name`(右段仍 admin_display_name)。`auth/api.ts`(AdminAuthCheck/AdminIdentifyResult)+`auth/types.ts`(TokenAdminAuth)加字段;`App.tsx` bootstrap 合并 + `LoginView.tsx` 两处 nextAuth 透传。

### D. 清理第二源(前端)
8. 删死常量 `auth/types.ts RegistryOrgCodeLabel`(零引用)。
9. `ProvinceDetailView.tsx nameText` 兜底 `${city_name}注册局`(伪造机构名)→ `'-'`。
- 保留(经判定属"机构类型/角色"标签,非机构名第二源,且与名字是不同概念)：`subjects/labels.ts ORG_CODE_LABEL`(机构类型列)、`PrivateDetailLayout CREATED_BY_ROLE_LABEL`(创建者角色)、`build_admin_display_name`(管理员个人名兜底)。已在回报中向用户说明,待异议再动。

## 验证（已完成 2026-06-21）
- 后端：`cargo fmt` 干净 + `cargo check` 通过 + `cargo test` **64+5 全过**。
- reconcile：`reconcile-gov --changed-only` 实跑开发库 `updated=3896 removed=0`；实测 federal `short=总统府联邦注册局→联邦注册局`(name 同步)；city 早已正确(如 固市身份注册局/固市注册局)。
- 前端：`npx tsc --noEmit` **0 error**；残留扫描——`registryOrgCodeLabel`/`RegistryOrgCodeLabel`/`${city}注册局` 兜底 **零代码残留**(仅注释提及 tab 名)。
- 数据流终态：place 1 头部读 `federalRegistryDetail.institution.cid_short_name`(联邦注册局 · 中枢省)；place 2 徽标读 `auth.institution_short_name`(联邦=联邦注册局 / 市={市}注册局,右段仍 admin_display_name,简称空时整段隐藏不显伪造名)。

## Follow-up
- 4 兄弟联邦局简称(需用户确认 安全/情报/特勤/人事局 的简称值)。
- 市注册局子 tab 头部("市注册局管理员"硬编码)同模式可改读简称(本次未在用户点名两 UI 内)。
