# 任务卡：注册局机构详情页搬入「注册局」Tab + 管理员列表内嵌

## 目标

把「注册局机构」（org_code = `CITY_REGISTRY` 市注册局 / `FEDERAL_REGISTRY` 联邦注册局）的机构详情页搬进「注册局」tab，并把对应管理员列表内嵌进机构详情页。功能逻辑不变，只是把管理员列表挪进机构详情页。

- 「公权机构」列表（GOV_INSTITUTION）不再显示注册局机构（CITY_REGISTRY / FEDERAL_REGISTRY）。
- 「注册局」tab 两个子 tab 改名：`市注册局机构管理员列表`→`市注册局`，`联邦注册局机构管理员列表`→`联邦注册局`。
- **市注册局**：联邦注册局机构管理员 城市网格→点市→该市注册局机构详情页（内嵌本市市注册局机构管理员列表，可增删改）；市注册局机构管理员 直接进本市注册局机构详情页（管理员列表只读）。
- **联邦注册局**：两角色都进全国唯一联邦注册局机构详情页；内嵌联邦注册局机构管理员列表只显示登录管理员本省（逻辑不变）；联邦注册局机构管理员可增删改、市注册局机构管理员只读。
- 注册局机构详情页 = 完整机构详情（机构信息+账户列表+资料库+操作记录），管理员列表插在**机构信息卡与账户列表卡之间**。其它普通机构详情页零行为变化。
- 联邦注册局位于中枢省，现有 scope 校验会 403 拦截外联邦注册局机构管理员 → 后端新增只读接口，仅返回联邦注册局这一个机构、绕过 scope。

## 预计修改目录

- `sfid/backend/main.rs`：DB 层新增 `get_federal_registry_with_accounts`（按 `org_code='FEDERAL_REGISTRY'` 查）；路由注册 `GET /api/v1/institution/registry/federal`。
- `sfid/backend/subjects/admin.rs`：新增 handler `get_federal_registry`（require_admin_any，不做 scope，返回 InstitutionDetailOutput）。
- `sfid/frontend/gov/api.ts`：新增 `getFederalRegistry`。
- `sfid/frontend/gov/GovDetailPage.tsx`：新增可选 props `adminListSection` / `loadDetail` / `backLabel`，onBack 改可选；普通机构零行为变化。
- `sfid/frontend/gov/GovListTable.tsx`：GOV_INSTITUTION 列表过滤掉注册局 org_code。
- `sfid/frontend/subjects/labels.ts`：导出 `REGISTRY_ORG_CODES` 单一源。
- `sfid/frontend/admins/RegistryAdminsView.tsx`：解析市注册局 sfid、加载联邦注册局 detail。
- `sfid/frontend/admins/adminUtils.ts`：RegistryAdminsSharedState 增字段。
- `sfid/frontend/admins/ProvinceDetailView.tsx`：子 tab 改名 + leaf 换成机构详情页（避免 Card 套 Card）。
- `memory/08-tasks/`、`memory/`：任务流转与回写。

## 验收

- 公权机构列表（任一城市 / 中枢省）不再出现注册局机构。
- 注册局 tab 两子 tab 改名为 市注册局 / 联邦注册局。
- 三角色进入路径符合目标表（联邦注册局机构管理员可增删改、市注册局机构管理员只读）。
- 注册局机构详情页：管理员列表位于机构信息卡与账户列表卡之间。
- 联邦注册局：外联邦注册局机构管理员可正常读取机构详情（200），管理员列表仅本省。
- 普通机构详情页零行为变化（无管理员列表）。
- 后端 `cargo check` + `cargo test` 通过；前端 `npm run build`（含 tsc）0 error。

## 完成记录

- 后端 `main.rs` DB 层新增 `get_federal_registry_with_accounts()`：按 `org_code='FEDERAL_REGISTRY'` 定位唯一机构 sfid 后复用 `get_institution_with_accounts`。
- 后端 `subjects/admin.rs` 新增只读 handler `get_federal_registry`：require_admin_any，**不做 scope 校验**，返回与 `get_institution` 一致的 `InstitutionDetailOutput`。
- 路由注册 `GET /api/v1/institutions/federal-registry`（放在 `/institutions/` 段，结构上避开 `:sfid_number` 动态段冲突）。
- 前端 `subjects/labels.ts` 导出 `REGISTRY_ORG_CODES`（单一源）。
- 前端 `gov/api.ts` 新增 `getFederalRegistry`。
- 前端 `gov/GovListTable.tsx`：GOV_INSTITUTION 列表用 `visibleRows` 过滤掉注册局 org_code（公安局列表不受影响）。
- 前端 `gov/GovDetailPage.tsx`：新增可选 props `adminListSection`/`loadDetail`/`backLabel`，`onBack` 改可选；管理员列表渲染在机构信息卡与账户列表卡之间；普通机构不传这些 props → 零行为变化。
- 前端 `admins/adminUtils.ts`：`RegistryAdminsSharedState` 增 4 字段（federalRegistryDetail/Loading、cityRegistrySfid/Loading）。
- 前端 `admins/RegistryAdminsView.tsx`：新增两个 effect——挂载时加载联邦注册局 detail（scope-bypass）；按活动省市解析市注册局 sfid（从 official 目录筛 CITY_REGISTRY）。
- 前端 `admins/ProvinceDetailView.tsx`：子 tab 改名为「市注册局 / 联邦注册局」；leaf 改为 GovDetailPage + 内嵌管理员列表（CityRegistryAdminsView / FederalRegistryAdminSubTab，均包 inner Card）；用 `useCallback` 稳定 `loadFederalRegistry` 复用预加载 detail，避免重复请求；联邦注册局机构管理员城市网格→点市→详情、市注册局机构管理员直接进本市详情、返回按钮按角色隐藏。

## 验证

- `cd /Users/rhett/GMB/sfid/backend && cargo check` 通过（exit 0）。
- `cd /Users/rhett/GMB/sfid/backend && cargo test` 通过（52 passed / 0 failed）。
- `cd /Users/rhett/GMB/sfid/frontend && npm run build`（tsc -b + vite build）通过，0 type error。
- 待 user 真机手测三角色路径（见任务卡验收表）+ 确认外联邦注册局机构管理员可读联邦注册局（200）。

## 迭代二（2026-06-08，user 反馈后）

### 诊断（联邦注册局空数据 / "数据不存在" toast 根因）
联邦注册局走创世常量路径 `push_constant_target`，org_code 由 `org_code_for_constant_name` 推导，而该函数无「总统府联邦注册局」分支 → 落 `_ => "PUBLIC_ORG"`。所以 subjects 表里联邦注册局 org_code 是 `PUBLIC_ORG` 而非 `FEDERAL_REGISTRY`，旧的 `WHERE org_code='FEDERAL_REGISTRY'` 查不到 → 404 → 两个报错（toast"数据不存在"=错误码 1004 前端文案；详情页"暂无联邦注册局数据"=前端兜底）。

### 修复 + 改动
1. **后端联邦注册局按 sfid_number 定位**（robust，不依赖 org_code / 不需重新对账）：
   - `gov/service.rs` 新增 `federal_registry_sfid_number()`：从 china_zf 常量找「总统府联邦注册局」取 sfid_number。
   - `subjects/admin.rs` 的 `get_federal_registry` 改用它 + 复用 `get_institution_with_accounts`（仍不做 scope）。
   - `main.rs` 删除上一版多余的 `get_federal_registry_with_accounts` DB 方法。
2. **注册局子 tab 提升为顶级 tab**（删 `注册局` tab + 两个子 tab）：
   - `auth/AuthContext.tsx`：`canViewSystemSettings` → `canViewCityRegistry` + `canViewFederalRegistry`。
   - `App.tsx`：ActiveView 改 `city-registry`/`federal-registry`；tab 顺序 …公安局 → **市注册局 → 联邦注册局**；passkey 未绑定按角色强制进对应注册局 tab（联邦→federal-registry，市级→city-registry）；两个 RegistryAdminsView 实例按 viewResetToken 加 key。
   - `admins/RegistryAdminsView.tsx`：mode 改 `list|city-registry|federal-registry`，effect 按新 mode 分流，dispatch 到 CityRegistryView/FederalRegistryView。
   - `admins/ProvinceDetailView.tsx`：拆成 `CityRegistryView` + `FederalRegistryView`（删 SubTabBar/子 tab）。
   - `admins/RegistryAdminsView.tsx`：旧 `mode="system-settings"` → `mode="city-registry"`。
3. **管理员列表表头布局**：CityRegistryAdminsView / FederalRegistryAdminSubTab 改为 `Card`，title=「市注册局机构管理员列表 / 联邦注册局机构管理员列表」+ 计数（n/上限）紧随其右，`extra`=新增按钮置于统一行最右；删除原内部表头行 + 去掉标题里的计数括号。

### 迭代二验证
- 后端 `cargo check` + `cargo test`（52 passed）通过。
- 前端 `npm run build`（tsc + vite）通过，0 type error。
- 注意:联邦注册局机构在 subjects 表 org_code 仍是 `PUBLIC_ORG`(本次按 sfid_number 定位绕过),故 GovListTable 的 FEDERAL_REGISTRY 过滤对它不生效——中枢联邦注册局机构管理员的「公权机构」列表里它仍会以"公权机构"出现(仅影响中枢省;5 个总统府联邦局 org_code 缺映射是既有问题,未扩边)。如需一并清掉,后续单列任务补 `org_code_for_constant_name` 的 5 个联邦局分支 + reconcile 重跑。
- user 确认（迭代二收尾）：只改注册局即可；另外 4 个总统府联邦局保持现状不动；联邦注册局留在中枢省/锦程市公权列表是可接受的（"正好"）。核实 china.sqlite：中枢省(ZS) 001 市=锦程市，联邦注册局正落于此，故锦程市市注册局机构管理员与中枢省联邦注册局机构管理员都能在公权列表看到它。

## 迭代三（2026-06-08，管理员列表表头微调）

1. **FederalRegistryAdminSubTab（联邦注册局）**：title 改「联邦注册局机构管理员列表 · {province}」（中点 `·` 用 `Space align="center"` 垂直居中）；计数「联邦注册局机构管理员：n/5」→「用户数：n/5」移到 `extra` 内、置于「新增联邦注册局机构管理员」按钮左侧。
2. **CityRegistryAdminsView（市注册局）**：title 改回纯「市注册局机构管理员列表」；计数「本市市注册局机构管理员：n/30」→「用户数：n/30」移到 `extra` 内、置于「新增市注册局机构管理员」按钮左侧。
3. **机构类型显示逻辑问题（item 3，仅答疑未改码）**：「机构类型」= `INSTITUTION_CODE_LABEL[institution_code]` + ` / ` + `ORG_CODE_LABEL[org_code]`，后半段恒为 org_code 细类标签；`公权机构` 是 `PUBLIC_ORG` 通用兜底。联邦注册局显示「政府/公权机构」即因 org_code=PUBLIC_ORG。是否改为「政府/联邦注册局」留待 user 定（方案 B 需补 org_code 映射 + 升 GOV_TEMPLATE_VERSION，副作用会从公权列表隐藏它，除非把 GovListTable 过滤改为只过滤 CITY_REGISTRY）。

### 迭代三验证
- 前端 `tsc -b` 通过（exit 0）。

## 迭代四（2026-06-08，方案 A：补全国家级/省级公权机构 org_code）

### 背景
user 发现不止注册局，连「中华民族联邦共和国住房与城镇建设部」等**所有国家级机构**都显示「政府/公权机构」。根因升级：`org_code_for_constant_name` 靠机构名字猜 org_code，但常量存的是**全名**（且经历改名 省政府→省联邦政府、省储备委员会→省公民储备委员会），简称/旧后缀全对不上 → 国家级 + 部分省级全落 `PUBLIC_ORG`。能正常显示细类的（公民安全局、省立法院…）都是区划模板写死 org_code 或后缀恰好对上的。私权机构不受影响（不用 org_code，走 机构代码+企业类型）。user 选**方案 A**（只动 SFID，不碰链）。

### 改动
1. **`gov/service.rs` 重写 `org_code_for_constant_name`**：改为按实际全名精确匹配（总统府+10部委+5联邦局+两院+3联邦署+国家储备/教育，共 ~24 条）+ 纠正 6 条省级后缀（`省联邦政府`/`省立法院`/`省司法院`/`省监察院`/`省公民储备委员会`/`省公民储备银行`）。前端 ORG_CODE_LABEL 已含全部标签，零改动。
2. **`gov/service.rs` 升 `GOV_TEMPLATE_VERSION` v3→v4**：使 `ensure-gov` 不再 skip（manifest_ready 看 template_version）→ 跑 `check_gov_catalog_db`（[:1065](sfid/backend/gov/service.rs) `org_code != target.org_code` 会判定 mismatch）→ ok=false → 走 `reconcile_official_institutions_explicit(All, force_row_sync=true)` 全量重写 org_code。catalog hash 也含 org_code（[:828](sfid/backend/gov/service.rs)）。
3. **`gov/GovListTable.tsx` 过滤改为只过滤 `CITY_REGISTRY`**：联邦注册局 org_code 修正为 FEDERAL_REGISTRY 后仍按 user 意愿保留在公权列表（"正好"）。顺手删 `subjects/labels.ts` 不再用的 `REGISTRY_ORG_CODES` 导出。

### 迭代四验证
- 后端 `cargo check` 通过（exit 0）。
- 前端 `tsc -b` 通过（exit 0）。
- **部署 gotcha（必须执行才生效）**：org_code 重写靠 reconcile，**重建后端后须跑一次 gov 目录对账**——`ensure-gov`（v4 已使其不 skip，会自动检出 org_code mismatch 并 force 重写）或 `init-gov`（无条件 force_row_sync）。`serve` 启动本身不触发对账。reconcile 后 manifest_version 变化会顺带让前端公权列表缓存失效、重新拉到正确细类。
- 待 user 真机验证：跑 ensure-gov 后，住房与城镇建设部/交通运输部/联邦注册局/各省联邦政府 的「机构类型」显示真实细类（不再"公权机构"）。
