# SFID 改造 Step 1:省管理员 3-tier + 删除 KEY_ADMIN

- 状态:in_progress(Phase 1 完成,Phase 2-7 待续)
- 创建日期:2026-05-01

## 进度记录

### 2026-05-01 Phase 1 完成

- 重命名 `sfid/backend/sheng-admins/` → `sheng_admins/`
- 重命名 `sfid/backend/shi-admins/` → `shi_admins/`
- `main.rs` 删除两个 `#[path]` shim,改用标准 `mod sheng_admins;` `mod shi_admins;`
- 同步重命名文档目录 `memory/05-modules/sfid/backend/sheng-admins/` → `sheng_admins/`(及 `shi-admins`)
- 更新 `models/mod.rs` / `sfid/generator.rs` 注释里的旧路径指引
- `cargo check` 通过,3 条 dead_code warning 与本次改造无关(VillageCode/TownCode 字段)

**Phase 1 暂未实施**(降级到 Phase 2/3,避免编译断点):
- `models/mod.rs` 1021 行拆分 → 留到 Phase 2(KEY_ADMIN 删除时模型类型必然重写)
- 新建空骨架(citizens/、institutions/policies/、chain/sheng_signer/、chain/sheng_admin/ 新文件)→ 留到 Phase 3/4(content-driven 创建)

### 残留(Phase 2-7 必须扫除)

- `db/migrations/013_rename_roles_sheng_shi.sql` 注释含旧路径(历史 migration,保留)
- `main.rs:871,875` 路由 path 字符串 `/api/v1/admin/sheng-admins`(API 路径,Phase 5 路由收敛时统一)
- `key-admins/sheng_signer_cache.rs` 自身待删(Phase 2)
- `memory/05-modules/sfid/` 下文档内容里的旧路径表述(Phase 7 全量重写)

### 2026-05-01 拆分 4 张执行子卡

- `memory/08-tasks/open/20260501-sfid-step1-phase23-delete-key-admin-and-sheng-3tier.md` —— Phase 2+3 原子(删 KEY_ADMIN + 省管理员 3-tier)
- `memory/08-tasks/open/20260501-sfid-step1-phase45-chain-push-and-routes.md` —— Phase 4+5(chain push + 路由收敛,推链先 mock)
- `memory/08-tasks/open/20260501-sfid-step1-phase6-frontend-3tier-views.md` —— Phase 6(前端)
- `memory/08-tasks/open/20260501-sfid-step1-phase7-acceptance-and-cleanup.md` —— Phase 7(验收 + 文档 + 联调)

### 推荐执行顺序(每张卡新开聊天线程,引用任务卡 + ADR-008)

phase23 已细分为 5 张连环子卡(每张 build 绿),按字母顺序连跑:

1. **phase23a** — `models/mod.rs` 1021 行 split → 6 文件 facade
2. **phase23b** — `key_admins::rsa_blind` → `institutions/anon_cert/`
3. **phase23c** — `business/*` 内容并入 `scope/`,删 `business/`
4. **phase23d** — `operate/*` 迁入 `citizens/`,删 `operate/`
5. **phase23e** — 拆 `chain_keyring`,最终删 `key-admins/` + `chain/{key_admins,balance}/` + `models::AdminRole::KeyAdmin` + AppState + `main_tests.rs` 重写
6. **phase45** — 后端 chain push + 路由收敛
7. **phase6** — 前端
8. **Step 2 区块链 runtime 改造**(独立任务卡,需新拆)→ 链上 4 个 Pays::No extrinsic 上线
9. **phase7** — 联调 mock → real + 文档 + 残留 → Step 1 收口


- 模块:`sfid/backend` + `sfid/frontend`
- 关联 ADR:ADR-008(待创建,本卡同步起草)
- 关联跨步:Step 2(`citizenchain/runtime` 改造)+ Step 3(其他系统适配)将由独立任务卡跟进

## 任务需求

把 SFID 系统从"KEY_ADMIN 主导 + 单省一管理员"模型,彻底改造为"无 KEY_ADMIN + 每省 3 admin 自治(main + backup_1 + backup_2)"模型。链上 0 prior knowledge of SFID,所有 SFID 相关数据内置在 SFID 系统;chain → SFID 单向 pull,SFID 推链仅限 sheng_admin 名册操作与 sheng_signer 激活/rotate(全部 Pays::No)。

## 建议模块

- `sfid/backend/`(目录重组 + 删 KEY_ADMIN + 新增 sheng_signer 业务 + chain 推链能力)
- `sfid/frontend/src/views/`(删 keyring 视图 + 加 sheng_admin 名册页 / 激活页 / rotate 页)

## 影响范围

### 后端文件级影响

- 重命名:`sheng-admins/` → `sheng_admins/` ; `shi-admins/` → `shi_admins/`
- 删除整目录:`key-admins/` ; `chain/key_admins/` ; `chain/balance/` ; `chain/sheng_admin/`(旧文件) ; `business/` ; `operate/`(内容迁 `citizens/`)
- 删除模块/字段:`models::AdminRole::KeyAdmin` 枚举值 + 所有 match 分支 + `scope::filter_by_scope` KEY 分支 + `main.rs` 中 `/key-admin/*` 路由 + AppState 中 KEY 相关字段
- 拆分:`models/mod.rs`(1021 行)→ `role.rs` / `slot.rs` / `session.rs` / `permission.rs` / `error.rs`
- 新建子目录骨架:
  - `citizens/{mod.rs,handler.rs,binding.rs,vote.rs}`(原 operate/ 合并)
  - `institutions/policies/{mod.rs,private.rs,gov.rs,public_security.rs}`
  - `chain/sheng_admin/{mod.rs,handler.rs,query.rs,add_backup.rs,remove_backup.rs}`
  - `chain/sheng_signer/{mod.rs,handler.rs,activation.rs,rotation.rs}`
- 新增静态数据:`sfid/province.rs` 加 `ProvinceAdmins { main, backup_1, backup_2 }`(只 main 是 const,backup 链上来源)
- 新增业务模块:`sheng_admins/{login,bootstrap,signing_cache,roster,catalog}.rs`
- 新增加密 seed 持久化:`store_shards/sheng_signer.rs`(或新建 `store_shards/` 内子模块)
- 路由收敛:`main.rs` 路由表精简到方案中的"路由表全景"

### 前端文件级影响

- 删除整目录:`sfid/frontend/src/views/keyring/`
- 重命名:`views/sheng-admins/` → `views/sheng_admin/`(保持与后端一致)
- 新建页面:`views/sheng_admin/{Roster,Activation,Rotate}Page.tsx`
- 新建表单组件:`views/institutions/policies/{Private,Gov,PublicSecurity}Form.tsx`
- API client:`api/sheng_admin.ts`、`api/sheng_signer.ts` 新增
- Header 删 KEY 角色切换

### 数据库

- DROP TABLE `key_admins`(及关联外键)
- 新建 schema:省管理员加密签名 seed 持久化(按需要)
- 三表存量数据保留:`multisig_institutions` / `accounts` / `cpms_site_keys`(已分片)

### 跨模块依赖(阻塞)

- **Phase 4 联调阻塞 Step 2**:`add_sheng_admin_backup` / `activate_sheng_signing_pubkey` / `rotate_sheng_signing_pubkey` 三个 extrinsic + `ShengAdmins` storage 由 Step 2 提供;Step 1 实现先 mock 推链返回值,等 Step 2 落地后联调

## 主要风险点

- **推链 Pays::No 错误风险**:现有 `key-admins::bootstrap_sheng_signer` 推链失败(1010 错误)的根因是 SFID main 账户零余额。新设计 activate/rotate 全部 Pays::No,绕过 fee 检查;Step 2 必须严格按 Pays::No 实现,否则 Step 1 推链段一样炸
- **KEY_ADMIN 删除连锁影响**:KEY_ADMIN 引用横跨 `business/scope.rs` / `key-admins/` / `chain/key_admins/` / `models/mod.rs` / `main.rs` / `scope/rules.rs` / `operate/binding.rs` / `login/mod.rs` / `institutions/handler.rs` / `sheng-admins/{catalog,institutions,operators}.rs` / `store_shards/{migration,shard_types}.rs` 等 26+ 文件,删除工作量大且容易破坏编译
- **rename 破坏性**:`sheng-admins/` 横线改下划线虽是 Rust 标识符要求,但当前用 `#[path]` shim 绕开,删除 shim 后所有 `mod sheng_admins` import 路径改写
- **operate/ 合并入 citizens/ 边界模糊**:`operate/binding.rs` `operate/cpms_qr.rs` `operate/status.rs` 三文件不全是公民身份业务,合并需细分
- **数据库 DROP TABLE 不可逆**:开发期可接受,生产上线前必须 backup
- **冷钱包 / 节点桌面 RPC 契约**:节点桌面 `chain/sheng-admin/list` endpoint 输入输出契约必须与 Step 2 链上 storage 一致

## 是否需要先沟通

- **否**(已多轮分析 + 拍板,目录结构已确认)

## 建议下一步

按 Phase 1 → 7 顺序执行:

### Phase 1:目录骨架重组(纯结构,业务零变动)

1. 重命名 `sheng-admins/` → `sheng_admins/` ; `shi-admins/` → `shi_admins/`(实质用 `git mv`)
2. 删除 `main.rs` 所有 `#[path = "..."]` shim,改 `mod sheng_admins;` `mod shi_admins;`
3. 拆分 `models/mod.rs`(1021 行)→ `models/{mod,role,slot,session,permission,error}.rs`
4. 新建空骨架:`citizens/`、`institutions/policies/`、`chain/sheng_admin/`、`chain/sheng_signer/`
5. **不动**:`app_core/` / `store_shards/` / `indexer/` / `qr/` / `scope/` / `login/` / `sfid/`(已对齐设计)
6. 验收:`cargo check` 全绿;`grep -rn '#\[path' src/` 零结果;`find src/ -name '*-*.rs' -o -type d -name '*-*'` 零结果

### Phase 2:KEY_ADMIN 全删

详见方案"Phase 2:KEY_ADMIN 全删"清单。

### Phase 3:省管理员 3-tier 模型

详见方案"Phase 3:省管理员 3-tier 模型重写"。

### Phase 4:`chain/sheng_admin/` + `chain/sheng_signer/` 实现(推链先 mock,等 Step 2)

### Phase 5:路由 + AppState 收敛

### Phase 6:前端落地

### Phase 7:验收 + 更新文档 + 完善注释 + 清理残留

## 验收清单(整 Step 完工)

- `cargo check` + `cargo clippy -- -D warnings` + `cargo test` 全绿
- 残留 grep 全部零结果:`KeyAdmin|key-admin|key_admin|key-admins`、`#\[path`、`*-*.rs/*-*目录`、`set_sheng_signing_pubkey` 在 `sheng_admins/` 内、`operate/|business/|chain/balance/|chain/key_admins/`
- 浏览器 e2e:三 slot admin 登录 + activation + roster add backup + rotate + 跨省读/写权限
- 文档同步:`memory/05-modules/sfid/` 下相关文档更新;ADR-008 落地
- 注释:每个新模块顶部 1-3 行中文用途说明
- 残留清理:`grep -rn TODO|FIXME|XXX` 不增加未跟踪项

## 阶段性提交策略(开发期一次性彻底切换,但分 PR 落地便于 review)

- PR-A:Phase 1(纯结构 + 编译通过,无业务变化)
- PR-B:Phase 2(删 KEY_ADMIN + 编译通过)
- PR-C:Phase 3(省管理员 3-tier 业务)
- PR-D:Phase 4(chain push 模块)
- PR-E:Phase 5(路由收敛)
- PR-F:Phase 6(前端)
- PR-G:Phase 7(收尾 + 文档)

## 工作量预估

| Phase | 后端净改动 | 前端净改动 | 工时 |
|---|---|---|---|
| 1 目录骨架 | ~200 行(纯重命名/拆分) | 0 | 0.5d |
| 2 删 KEY_ADMIN | -2500 行 | -800 行 | 1d |
| 3 省管理员 3-tier | +1200 行 | 0 | 2d |
| 4 chain push 模块 | +800 行 | 0 | 1.5d |
| 5 路由收敛 | +100 行 | 0 | 0.5d |
| 6 前端 | 0 | +900 行 | 2d |
| 7 验收 + 收尾 | +500 行 | +200 行 | 1d |
| **合计** | **+300 行净增** | **+300 行净增** | **8.5d** |

(KEY_ADMIN 删减 ~3300 行,新功能补回 ~2700 行,净增 ~600 行)
