# 任务卡：注册局两个缺陷修复（公民列表回填 + CPMS 安装码键对齐）

状态：代码完成（待用户端到端 QA）
创建：2026-06-09
完成：2026-06-09
模块：SFID（frontend + backend）
来源：用户报告两个现象并确认修复方向。

## 背景 / 根因（已代码级确认）

### 缺陷 1：新增身份ID绑定成功但公民列表查不到
公民列表是「精确检索」视图，空关键字直接返回空（前端 `CitizensView.tsx` refreshList:37-42 + 后端 `main.rs` list_citizens_exact:642-651）。绑定成功回调 `onBound` 用的是当前空 `searchKeyword`（CitizensView.tsx:281），刷新后仍空。数据其实已入库（`citizen_bind` 在 `upsert_citizen_row` 成功后才返回 ok，binding.rs:357）。

### 缺陷 2：CPMS 安装码二维码再次进入就消失、要重新生成
写入键 ≠ 读取键：生成接口用 `(province,city,institution_code)` 三元组经 `find_cpms_target_institution`（`ORDER BY created_at DESC LIMIT 1`，cpms/handler.rs:142-167）解析出 sfid 当 `cpms_sites` 主键写入；再次进入用机构 `inst.sfid_number` 读（GovDetailPage.tsx:240 → get_cpms_site:58-72）。`subjects` 对该三元组无唯一约束（db.rs:327-359），解析到的「最新」一条可能≠正在看的那条 → 读 `None` → 200+`data:null`（handler.rs:748）→ `cpmsSite=null` → 面板隐藏、生成按钮复现（GovDetailPage.tsx:332,383）。

## 目标
1. 绑定成功后自动用返回的新身份ID回填搜索并查询，使新公民立即出现在列表、搜索框同步回填。
2. CPMS 生成接口以机构自身 `sfid_number` 作为 `cpms_sites` 写入键（= 详情页读取键），根治再次进入二维码消失。

## 实施清单

### 缺陷 1（前端 2 文件）
- [x] `sfid/frontend/citizens/BindModal.tsx`：`onBound` 改 `(boundSfidNumber?: string) => Promise<void> | void`；:157 改 `await onBound(result.sfid_number)`。
- [x] `sfid/frontend/citizens/CitizensView.tsx`：`Form.useForm()` 绑定搜索表单；新增 `handleBound`：拿到 boundSfid 时 `searchForm.setFieldsValue({keyword})` + `setSearchKeyword` + `setCursorStack([])` + `refreshList(boundSfid,null,true)`，无值回退原逻辑；`<BindModal onBound={handleBound}>`。

### 缺陷 2（前端 3 文件 + 后端 2 文件）
- [x] `sfid/frontend/gov/GovDetailPage.tsx`：onGenerateCpms 保持 grant payload `{province,city,institution}` 不变（passkey 公民钱包签名内容不动），generate 请求体增 `sfid_number: inst.sfid_number`；loadCpms 真错误改 `notice.error` 提示、不再静默置 null（返回 null=未生成仍正常置空）。
- [x] `sfid/frontend/cpms/api.ts`：`generateCpmsInstallQr` 入参增 `sfid_number: string`；顺手修正 getCpmsSiteByInstitution 过时注释。
- [x] `sfid/backend/cpms/model.rs`：`GenerateCpmsInstallInput` 增 `sfid_number: String`（保留 province/city/institution 供 grant 绑定）。
- [x] `sfid/backend/cpms/handler.rs`：新增 `find_cpms_target_institution_by_sfid(sfid_number)`（按 subjects 主键 sfid 查 PUBLIC/ACTIVE，返回省/市/编码/名称）；`generate_cpms_install_qr` 改用 `input.sfid_number` 查机构+校验 scope（sheng admin: institution.province==ctx.admin_province）+ 以该 sfid 写 `CpmsSiteKeys.sfid_number`；删除三元组解析写入键的旧 `find_cpms_target_institution`（无其它调用者）+ 一并删除仅此处用的 `MAX_PROVINCE/CITY/INSTITUTION_CHARS` 常量。grant_payload 仍取 `{province,city,institution}` 不变。

## 验收
- 缺陷 1：注册局点「新增身份ID绑定」走完流程 → 列表自动出现新公民、搜索框回填新身份ID。
- 缺陷 2：某市公安局生成安装码 → 离开 → 再次进入 → 二维码仍在（待激活、可下载），无「生成」按钮。
- `cargo check`（sfid backend）+ `cargo test`（cpms/citizens）+ 前端 `tsc`/`build` 0 error。

## 完成记录

- 缺陷 1 根治：绑定成功后 `BindModal` 把 `result.sfid_number` 回传 `onBound`；`CitizensView.handleBound` 用它回填搜索框 + 触发查询 → 新公民立即按身份ID命中显示（精确检索语义不变，数据本就已入库）。
- 缺陷 2 根治：写/读键统一为机构自身 `sfid_number`。生成接口改由前端传 `sfid_number`、后端按 sfid 反查机构并以该 sfid 落键；详情页再次进入按同一 sfid 读 → 命中已存站点、二维码持续显示。passkey 公民钱包签名内容（grant payload）保持 `{province,city,institution}` 不变，未触达 citizenwallet 解码器。
- 附带：`loadCpms` 真错误改提示而非静默吞 null；删除无引用的三元组解析函数与 3 个仅此处用的常量。
- 验证：`cargo check` 通过；`cargo test` 52/52 全过；前端 `tsc -b` 0 error；`vite build` 成功。
- 待用户端到端 QA：①注册局新增身份ID绑定 → 列表自动出现；②市公安局生成安装码 → 离开再进入 → 二维码仍在。
- 注意：开发态 postgres 中旧错位 `cpms_sites` 行（键为旧三元组解析值）不会被复用，修复后对该机构重新生成一次即按 `inst.sfid_number` 正确落键。

## 追加（2026-06-10）：事故定性 + 第二道防线

实测事故（锦程市公安局 ZS001-GZF0F-149201859-2026）查清：
- 用户在公安局页面生成的安装码，实际落到了**锦程市农业局 ZS001-GZF0R-748040000-2026** 名下，且该安装码已被一台 CPMS 初始化消费（ACTIVE/USED、公钥已绑定）。
- 根因比「同名最新」更糟：前端发的 `institution=inst.institution_code='ZF'` 是**类别码**（全锦程市 63 个政府机构共用、全国 29 万 GOV_INSTITUTION），旧三元组查询 `institution_code='ZF' OR name='ZF'` 命中 63 行；批量建库 created_at 全部相同 → `ORDER BY created_at DESC LIMIT 1` 取任意行 → 农业局。
- 第二漏洞：「只有公安局才有安装码」此前**仅靠前端按钮可见性**约束，后端对 category 零校验，直调 API 可给任何 PUBLIC 机构发码。

加固（已落地）：
- [x] `generate_cpms_install_qr` 增加服务端铁律：`subjects.category != 'PUBLIC_SECURITY'` 一律 403（`find_cpms_target_institution_by_sfid` 返回 category 七元组）。
- [x] `cargo build`（重建 target/debug 主二进制——教训：cargo check/test 不产出主二进制）+ `cargo test` 52/52。
- [x] 2026-06-10 18:04 用户重启 sfid-run.sh，新进程确认运行 11:17 新二进制（含 category 防线）。
- [x] 用户已在公安局页面重新生成安装码：新行 `ZS001-GZF0F-149201859-2026`（PENDING）落键正确，QR payload 内嵌 sfid 与机构一致（库内三方核对通过）——修复链路实证生效。
- [x] 删除错位脏行 `cpms_sites ZS001-GZF0R-748040000-2026`（农业局，ACTIVE/USED），现库内仅剩公安局一行。
- [ ] 待用户：用新安装码重新初始化那台 CPMS（旧装机持有农业局站点身份，其已签发档案码内嵌错号；脏行已删，旧档案码验真将正确失败）。
- [x] 2026-06-10 按用户指令删除经旧档案码绑定的公民 `ZS000-MZG12-124983574-2026`（citizens/subjects/ids 三表同事务删除，删后三表 CITIZEN 计数均为 0，号已全局释放）。
