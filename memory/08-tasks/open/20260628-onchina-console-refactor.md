# 任务卡：onchina 多机构统一控制台重构（主卡 / 17 分步）

## 任务需求

把 registry 后端从"联邦/市注册局双角色"重构为全机构统一控制台（onchina）：节点身份机构码化、三档鉴权（Session/Passkey/PasskeyColdSign）+ 默认拒绝、能力模型（CID 码主 + CID 号辅 + 实例覆盖）、web 端复杂提案、统一入口 `onchina.local:8964`，最后目录/crate 改名。

## 所属模块

citizenchain/registry（→ onchina），自动分工：CID Agent（后端身份/鉴权/机构/能力 + 前端）、Blockchain Agent（链上集合读取/提案 extrinsic 构造，零 runtime 改动）。

## 输入文档

- memory/04-decisions/ADR-030-onchina-multi-institution-console.md
- memory/04-decisions/ADR-029-registry-into-citizenchain.md
- memory/04-decisions/ADR-027-legislation-yuan.md

## 边界铁律（必须遵守）

- **卡 01–16 全部代码改动只在 `citizenchain/registry/` 内**（含 src 与 frontend）。
- **唯一外部步骤 = 卡 17**（目录/crate 改名 + env 改名 + node/scripts/tauri 同步），动工前单独沟通确认。
- 功能改造期严禁触碰 crate 名 / node / scripts / tauri.conf.json / runtime / 移动端。
- 不碰 `QR_V1` / `GMB` / CID 码表 / `china.sqlite` / 链上 pallet/index / `CID_*` 身份 env。
- 链开发期：彻底改 + 不兼容 + 零残留；二值→多值身份直接切换不留旧分支。
- 注释描述当前实现，禁止"从 X 改 Y / 原来 / 之前 / 现已改"历史措辞。

## 链端事实（已核实，决定 R3 可纯在 registry 内交付）

- NRC/PRC/PRB（固定治理档）→ `GenesisAdmins`（idx12，创世内置）。
- 私权公司 → `PrivateAdmins`（idx30，已真实接线：AdminAccounts + `propose_admin_set_change` 内部投票 + organization-manage `AdminAccountLifecycle`）。
- PMUL（个人多签）→ `PersonalAdmins`（idx7），**控制台不收**，CitizenApp 客户端功能。
- 四 pallet 经 `AdminAccountQuery` 统一查询，registry 零链端改动即可服务全机构。

## 待后续单独 ADR（不阻塞本轮）

- R4 实例覆盖位的链上配置真源（宪法/治理派生），本轮仅签名占位。

## 分步卡

### Phase 0 · 平台地基（全局阻塞前置）
- **01** 节点身份多值化：`chain_runtime::node_institution_identity` 从 `is_federal:bool` 扩展为机构码 + admin_level + 省码 + 市码 + scope_level；`AdminPallet` 补 `PrivateAdmins`；机构码→3 pallet 路由（FRG/固定治理→Genesis、公权→Public、私权→Private）。文件：`src/core/chain_runtime.rs`。依赖：无。风险：高。
- **02** `AdminAuthContext` 扩展多值字段 + 审计改写全部 `registry_org_code` 访问点（约 123 处 / 18 文件）+ DTO 同步。文件：`src/admins/login/{model,onchain_gate}.rs`、`src/admins/model.rs`。依赖：01。风险：高。

### Phase 1 · auth 主线
- **03** `admins/` → `auth/` 纯位移（login/actions/签名/grant），更新全库 `use` 路径 + main.rs mod。依赖：02。风险：中。
- **04** 统一入口：绑 `0.0.0.0:8964` + 新建 `platform/mdns.rs`（mdns-sd 广告 onchina.local）+ CORS 加 onchina.local。**纯内部**（env 改名挪到卡 17）。依赖：03。风险：高。
- **05** 鉴权枚举三档化 `Session|Passkey|PasskeyColdSign` + 穷尽 match 默认拒绝 + 编译期守卫单测。文件：`auth/{operation_auth,actions,security_model}.rs`。依赖：03。风险：高。
- **06** passkey 模块：WebAuthn 注册/断言/吊销 + 双因子提交绑定（webauthn-rs，独立 `PasskeySignedPayload` 隔离 QR_V1，失败不降档）。依赖：05。风险：高。

### Phase 2 · institution 主线
- **07** `accounts/` + `subjects/` → `institution/` 纯位移。依赖：02。风险：中。
- **08** scope 多档化：`VisibleScope` 按 `admin_level` 层级派生（删 Federal/City 硬编码），所有 list API 必过 `filter_by_scope`。依赖：07。风险：中。
- **09** admin model/catalog/city 泛化：`FederalRegistry/CityRegistry` → `Tier1/Tier2 + institution_id`；guards `require_admin_federal/city` → `require_admin_tier`。依赖：02/03/08。风险：高。
- **10** seed 泛化：联邦创世引导 → Tier1 seed。依赖：09。风险：中。

### Phase 3 · 能力模型 + governance
- **11** 能力模型 R4 草图：`platform/capability.rs`（静态模板 ⊕ 链上覆盖位占位）+ `platform/tab_registry.rs`。依赖：02。风险：低。
- **12** `governance/` web 端立法提案构造 + QR 冷签载荷（对接 legislation-yuan/vote，SCALE 逐字段对齐，零 runtime 改动）。依赖：04。风险：中。

### Phase 4 · 收尾
- **13** `domains/` 平移（gov/private/citizens/docs/education）。依赖：02/07。风险：中。
- **14** 前端身份字段二值→多值对齐 + 能力位渲染 tab + localStorage 缓存版本 bump + 形状校验自愈。文件：`frontend/{App.tsx,auth/,admins/,hooks/useScope.ts,utils/storedAuth.ts}`。依赖：02。风险：中。
- **15** 注释去历史化：全 registry 内改成描述当前实现。依赖：09/10/13。风险：低。
- **16** 残留清理 + memory/ADR 回写（验证零 `FederalRegistry/CityRegistry/is_federal/registry_org_code` 残桩）。依赖：15。风险：低。
- **17** 目录/crate 改名 registry→onchina（**唯一外部步骤，触及 9 文件，动工前沟通**）：git mv + Cargo.toml + workspace member + node/registry_proc + tauri.conf.json + scripts + env 改名。依赖：16。风险：高。

## 验收标准

- 每卡落地后 `cargo build -p registry`（卡 17 后 `-p onchina`）+ 相关单测通过。
- 三档鉴权穷尽匹配，新增 action 漏标分档则编译失败。
- 节点身份能正确路由 FRG/固定治理/公权/私权四类到对应 pallet；PMUL 拒入。
- 前后端身份字段对齐，无旧二值缓存读空。
- 零残留：无 `is_federal`/`RegistryOrgCode`/双角色死分支。

## 进度

- [x] 需求分析 + 方案设计 + ADR-030
- [x] 链端 4 pallet 接线核实
- [x] 主任务卡创建
- [x] 01 节点身份多值化（chain_runtime.rs：NodeInstitutionIdentity={机构码,候选pallet,主账户}；准入路由：固定治理档NRC/PRC/PRB拒入、FRG→Genesis、公权→Public、私权→Private、非法人→[Public,Private]探测、个人/PMUL拒入；fetch 候选探测）
- [x] 02 身份二值→多值（registry_org_code→institution_code+admin_level）：6 DTO + AdminUser + repo 56处 + db schema(列改名迁移+去CHECK+索引) + onchain_gate + 12 consumer 文件共 160 处；cargo check+test 绿(53 passed)；零残留
- [x] 03 auth 位移（用户改主意执行）：`git mv src/admins → src/auth`;全库 `crate::admins::`→`crate::auth::`(词边界避开 city_registry_admins,~51 处)+ main.rs `mod admins`→`mod auth` 与 bare `admins::`→`auth::`;58 测试绿
- [x] 04 统一入口 + mDNS（main.rs 绑定默认 127.0.0.1:8899→0.0.0.0:8964；新 platform/mdns.rs 用 mdns-sd 广告 _onchina._tcp.local 主机名 onchina.local:8964[best-effort,CID_MDNS_NAME 可覆盖]；serve_registry bind 后起广告；CORS 默认 origins 加 onchina.local + http://localhost:8964,allow_headers 补 x-passkey-assertion + x-cid-security-grant；cargo test 57 过；部署 env：LAN 用 onchina.local 须 CID_ENABLE_TLS=on + CID_PASSKEY_RP_ID=onchina.local + CID_PASSKEY_ORIGIN=https://onchina.local:8964[WebAuthn secure context],本机 http://localhost:8964 直接可用）
- [x] 05 三档鉴权 + 默认拒绝（AdminOperationAuth{Session,Passkey,PasskeyColdSign}；auth_type 穷尽 match；is_session；新增三档守卫单测；Session/PasskeyColdSign=原 LoginState/ScanSign 行为保留，Passkey 档保留待 06 接通，无安全空窗；cargo test 54 passed；档名注释全现在时化）
- [x] 06 passkey 模块（webauthn-rs v0.5.5；3 表[credentials/ceremonies/assertions]+admins/passkey 模块[register/assert begin·finish 4 端点]+require_passkey_assertion 一次性令牌；三档强制：prepare 仅 PasskeyColdSign、commit 流+UpdateX handler 消费断言；UpdateX 提升 Passkey；FRG 门禁解耦为 requires_federal_admin 保行为；fail-closed 不降档；前端 passkeyClient.ts[原生 navigator.credentials+base64url]；cargo test 57 passed 含 SoftPasskey 全流程往返；frontend tsc 绿）
- [x] 07 institution 位移（用户改主意执行）：`accounts/`+`subjects/` git mv 嵌入新 `institution/`(新 institution/mod.rs 声明两 pub(crate) 子模块);`crate::accounts::`→`crate::institution::accounts::`、`crate::subjects::`→`crate::institution::subjects::`(~87 处)+ main.rs 合并 mod;58 测试绿
- [x] 08 scope 多档化（用户执行 08→09→10→12 路径）：`VisibleScope` 重写为五档(全国/省/市/镇/私权自机构) + 镇维度 + `nationwide` 标志;`get_visible_scope` 按 `admin_level` 派生,**FRG 先于 admin_level 特判为省级**(FRG 码属 NATIONAL 但管理员按省分区);新 `chain_runtime::node_scope_{province,city,town}` 共享 env 来源;**新 `repo::derive_admin_scope_conn` 作 onchain_gate 签发与 guards 重建唯一来源**(口径一致 + 按层级只填该档维度);onchain_gate 镇校验(`CID_RUNTIME_SCOPE_TOWN_NAME`∈china.sqlite);ctx+3 DTO 加 `scope_town_name`;6 处机构 scope 检查加 `includes_town`;8 个 get_visible_scope 单测。**对抗式验证 sound(high)×4**。**镇档语义**:记录无镇维度(town 空,手动创建机构 town_code 恒空)= 不限镇对镇级可见,只排除明确属其他镇的对账机构(includes_town + B SQL 一致 lenient)。**公民不按镇**(A 撤销:公民省/市级精度,镇非其 scope 轴)。
- [⏸] 09 admin 泛化（**延后**:Tier1/Tier2 泛化待具体机构功能落地)
- [⏸] 10 seed 泛化（**延后**:随 09)
- [x] 08-补 B/C/D（card 08 收尾,对抗式验证 sound×4）：**B** gov 公权机构列表按镇过滤(`list_official_institutions_scope` 加 town_code 入参 + SQL `$7` lenient;handler 串 locked_town/town_code;query DTO 加 town_name);**C** 前端 useScope 泛化为 admin_level 五档(删 FRG/CREG 硬编码,镜像后端)+ `scope_town_name`(types/api/3 构造点)+ storedAuth v4→v5;**D** `search_parent_institutions` 补 scope 管辖校验(原丢弃 ctx,任一管理员可跨省/市搜父机构=预存越权洞)。68 测试 + node + tsc 绿
- [x] 17-补 启动横幅修(card 17 漏扫 dev 脚本)：run.sh/clean-run.sh 产品名 `注册局 Web`→`链上中国平台`、URL `127.0.0.1:8964`→统一入口 `onchina.local:8964`(+dev/passkey 直连 127.0.0.1 因安全上下文);框架 `注册局`→`机构`,保留 FRG dev 身份/护照域引用
- [x] 11 能力模型（后端单源 + 会话下发 + 前端镜像）：新 platform/capability.rs（CapabilitySet serde camelCase 对齐前端 RoleCapabilities，capabilities_for 内置 FRG/CREG、其它空能力占位）；AdminAuthOutput/AdminIdentifyOutput 加 capabilities，4 构造点(handler×2/onchain_gate/qr_login)派生；前端 capabilityMap 删硬编码 FRG/CREG 表只留类型+EMPTY 兜底，AuthContext 改读 auth.capabilities，types/api/LoginView×2/App 带 capabilities；cargo test 57 过 + tsc 绿；其它机构功能后续实现时在 capability.rs 逐个补能力位
- [⏸] 12 governance web 提案（**延后**:立法院 web 提案构造,待结构地基稳定后单独推进)
- [x] 13 domains 平移（用户改主意执行）：`gov/`+`private/`+`citizens/`+`docs/` git mv 嵌入新 `domains/`(citizenapp 独立 BFF 不动;新 domains/mod.rs);`crate::{gov,private,citizens,docs}::`→`crate::domains::*`(citizens 词边界不误伤 citizenapp)+ main.rs 合并 mod;**踩坑**:gov/service.rs 7 处 `#[path="../../../runtime/.../china_*.rs"]` 因下沉一层补成 `../../../../`;`cargo fmt` 规范 perl 编辑格式;58 测试绿
- [x] 14 前端身份对齐 + 能力 tab + passkey UX（types/api/AuthContext/LoginView/App 切 institution_code+admin_level；新 platform/capabilityMap.ts 镜像后端权限[FRG/CREG 内置,其它占位]；useScope 按机构码；storedAuth v4 bump+形状校验自愈；后端新增 GET passkey/status；passkey 操作列按钮 repurpose 为 self-only[删 codex 的 'key' 换账户错误分支],红点驱动真实状态,未注册登录默认跳管理员列表；passkeyClient.getPasskeyStatus + usePasskey hook；tsc+cargo 绿；残留仅 admin_security_api 错误码契约,保留正确）
- [x] 15 注释去历史化（全 onchina/src + frontend 扫描零"从X改Y/原来/之前/现已"历史化措辞;改名期注释全程现在时书写,无需返工)
- [x] 16 残留清理 + 回写（零 `registry_org_code`/`RegistryOrgCode`/`is_federal`/双角色残桩;db.rs 死迁移片段[registry_org_code→institution_code rename DO-block + DROP CONSTRAINT/INDEX]清除,基表已直定义 institution_code;ADR-030 + 本卡回写)
- [x] 17 目录/crate 改名 registry→onchina（用户批准提前做）：git mv registry→onchina；onchina/Cargo.toml(name/bin/description) + citizenchain/Cargo.toml workspace member；main.rs env REGISTRY_FRONTEND_DIST→ONCHINA_FRONTEND_DIST + serve_registry→serve_console + 日志/CLI/服务标识/内嵌PG库名(registry→onchina)/TLS证书名(onchina-cert/key.pem)/兜底路径(/opt/onchina) 全产品名残留清零；node：git mv registry_proc→onchina_proc 全重写(ONCHINA_CHILD/onchina 二进制·资源 onchina-bin·env ONCHINA_FRONTEND_DIST·日志[onchina]) + main.rs/desktop 调用方；tauri.conf.json resources(onchina-bin/onchina-frontend) + node/resources git mv + .gitignore；scripts(prepack.sh/.ps1/run.sh/clean-run.sh 全改 + 端口 8899→8964 + git mv registry-{backup,restore}.sh·postgresql.conf.sample→onchina-*)；保留=注册局领域名(federal_registry/city_registry/federal_registry_scope)与 node 内 governance::registry/prometheus_registry(非本次产品名;注:registry_org_code 旧列名已在卡16连同 db.rs 死迁移清除)；cargo check onchina+node 绿 + onchina 57 测试过 + frontend tsc 0 err；零产品名残留

## 待统一修复（对抗式审查发现）——✅ 已全部修复并对抗式验证（2026-06-28）

card 06/14 审查发现 8(App.tsx 默认跳转覆盖手动切 tab→hasInitializedView 守卫)早修。发现 5/1/3/6 在结构重构完成后一次性修,并经 5-agent 对抗式验证(每发现一怀疑者+完备性评审,全读真实代码证伪):

- **[HIGH] 发现 5 城市管理员被锁死不能更新机构/上传文档**:✅ 从 `operation_auth.rs::requires_federal_admin` 删 `InstitutionUpdate | InstitutionUploadDocument` + 边界守卫单测。验证 **sound(high)**:两 handler(subjects/admin.rs:224 update_institution、docs/handler.rs:209 upload_document 经 ensure_institution_visible_to_admin)各自从会话 scope(非请求体)做省/市校验,删联邦门禁不开跨省洞;两动作 Session 档,通用 prepare/commit 拒 Session 档无旁路。
- **[HIGH] 发现 1 challenge/grant 查询缺 admin 先验隔离**:✅ `get_action_challenge_conn`/`get_security_grant_conn` 加 `actor_account` 入参 + `AND lower(actor_account)=lower($2)`;3 callsite(actions.rs commit×2 + require_admin_security_grant)全传 ctx.admin_account。验证 **sound(high)**:DB 层 + app 层(same_admin_account)双隔离,纯防御加固不破合法流。
- **[MEDIUM] 发现 3 prepare 缺目标机构管辖预检**:✅ preview_action_conn 加两预检——`precheck_institution_target_scope_conn`(CreateAccount/DeleteAccount/DeleteDocument:按 target 取号→get_institution_with_accounts_conn→includes_province/city)+ `precheck_institution_create_scope`(InstitutionCreate:逐字段镜像 create_institution_inner 的 locked_province/city,仅拒非空且不等于锁定值,留空交 handler 回填,绝不更严)。完备性评审独立确认与真 handler 等价、无越权洞。**更正**:先前误判"docs handler 无 scope",实则 docs 经 `ensure_institution_visible_to_admin`(subjects/http.rs)校验,发现 3 是纵深防御非堵活洞。
- **[HIGH→改正] 发现 6 CREG city 属省校验**:初版在 `province_scope_for_registry_org_conn` 读路径硬 fail-closed→**对抗式验证判 FLAWED**:`issue_session_after_onchain_gate`(onchain_gate.rs:121)每次登录用裸 `CID_RUNTIME_SCOPE_CITY_NAME` env 覆写 city_name 且从不校验,节点 env 城市名与 china.sqlite 不逐字一致(如"三山"vs"三山市")会每请求 403 锁死合法管理员。✅ **改正**:撤销读时校验,移到 onchain_gate 的 env→会话写入边界——非 FRG 登录时校验 node_province/node_city ∈ china.sqlite(province_code_by_name + city_code_by_name),不一致 `GateError::Config` 拒登录(明确报错,非每请求锁死)。

**完备性评审附带 LOW(已修)**:`ensure_institution_visible_to_admin`(subjects/http.rs)原 fail-open(scope 字段 None 时放行,今不可达但与全仓 fail-closed 不一致)→改用 `get_visible_scope`/`includes_*` 统一 fail-closed。

已排除(误报/非问题):发现 2(DELETE RETURNING 行锁原子)、发现 4(counter 回退 webauthn-rs 已处理)、发现 7(VisibleScope 泛化属 card 08/09);owner 隔离类 get_login_sign_request/get_qr_login_result 按随机 challenge_id 走预认证(无身份,正确不加 actor 过滤);auth_type 分档无错配(状态变更/上链全 PasskeyColdSign)。

验收:`cargo test -p onchina` 58 passed(含新增守卫单测)· `cargo check -p node` 绿 · 零 `registry_org_code`/`RegistryOrgCode`/`is_federal`/历史化注释残留(db.rs 死迁移片段同步清除)。

---

## 本次重构收口（2026-06-28）

**结构性重构 + scope 多档全部完成**:卡 01–08/11/13–17 + 审查发现 5/1/3/6 + B/C/D 收尾 + 启动横幅修,全部多轮对抗式验证(sound)。`cargo test -p onchina` 68 passed · `cargo check -p node` 绿 · 前端 tsc 0。下列为**后续待办,本次重构后再做**(用户 2026-06-28 决定)。

**后续待办已统一迁至新卡** → [20260628-onchina-onchain-write-and-followups](20260628-onchina-onchain-write-and-followups.md):链写凭证基座 + 机构/管理员上链录入(架构缺口:onchina 对链只读、console 创建管理员只写本地不上链→登不进)+ 09 admin 泛化 + 10 seed 泛化 + 12 governance web + R4 覆盖位真源。**本重构卡到此收口,不再追加。**
