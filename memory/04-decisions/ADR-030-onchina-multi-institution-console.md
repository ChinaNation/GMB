# ADR-030 onchina 多机构统一控制台

## 状态

已接受（2026-06-28）。取代 registry 的"联邦/市注册局双角色"定位，扩展为全机构控制台。承接 [ADR-029](ADR-029-registry-into-citizenchain.md) 的去中心化进程模型。

## 背景

registry 后端最初只服务联邦注册局 + 市注册局两类管理员（前端 `registry_org_code` 二值枚举、后端曾以本机节点机构身份预判登录边界）。随着立法院、监察院、政府、储备、私权公司等机构都要在电脑端登录发起提案，需要一套统一控制台：同一套软件、同一登录入口，按链上管理员所属机构显示不同子 tab、不同权限。

## 决策

### 1. 平台定位
registry 重定位为通用 CID 机构控制台，产品名 **onchina**（链上中国，china≈chain 双关）。注册局只是其中一个机构租户。去中心化：每机构在自己办公室部署节点（内嵌 onchina 后端 + 本地 PostgreSQL）。

### 2. 统一入口
全节点统一访问字符串 `https://onchina.local:8964`：后端绑 `0.0.0.0:8964`，mDNS 广告 `onchina.local`（`_onchina._tcp.local`），TLS 自签证书目标主机为 `onchina.local`。各机构管理员输入同一字符串，连的是各自办公室本地节点。"统一"指统一字符串与体验，非指向同一台服务器。

节点程序启动后默认不启动 onchina。用户需要在节点设置页“链上中国平台”行点击“启动”并完成二次确认后，节点桌面端才拉起 onchina 子进程；退出节点程序时一并清理该子进程。设置页只启动服务，不自动打开浏览器。

### 3. 机构范围 = 控制台准入
服务 `primitives::cid::code` 中可进入网页控制台的机构管理员。平台启动时不预设机构，管理员冷钱包登录后用 `verified_pubkey` 反查链上 active admin 集合；是否能进入 OnChina 由机构码准入表决定。

| 机构类别 | 判定 | 链上真源 | OnChina 准入 |
|---|---|---|
| 联邦注册局 FRG | `== FRG` | `PublicAdmins::FederalRegistryProvinceGroups`（29） | 可登录，完整注册局能力 |
| 市注册局 CREG | `== CREG` | `PublicAdmins::AdminAccounts`（29） | 可登录，本市业务能力 + 只读本省联邦注册局 |
| 国家司法院 NJD | `== NJD` | `PublicAdmins::AdminAccounts`（29） | 可登录，本期只读本机构管理员 |
| 其它公权法人（政府/立法/监察/司法/教育/公安等） | `is_public_legal_code` | `PublicAdmins::AdminAccounts`（29） | 可登录，本期只读本机构管理员 |
| 私权法人（股权/股份/有限合伙/公益/协会/私立学校等） | `is_private_legal_code` | `PrivateAdmins::AdminAccounts`（30） | 可登录，本期只读本机构管理员 |
| 非法人组织 | `is_unincorporated_code` | `PublicAdmins::AdminAccounts` / `PrivateAdmins::AdminAccounts` 双探测 | 可登录，本期只读本机构管理员 |
| 国家储委会 / 省储委会 / 省储行 | `NRC` / `PRC` / `PRB` | `PublicAdmins::AdminAccounts`（29） | 不登录 OnChina，使用节点桌面端 |

个人多签 PMUL（personal-admins，idx7）**不登录控制台**：无 CID、不跑节点，纯 CitizenApp 客户端功能。

### 4. 登录绑定与权限模型

- 启动只做运行健康检查：`ONCHAIN_WS_URL` 可连接、本地数据库可用、HTTPS 服务可用、平台进程健康接口可达。
- 冷钱包签名验证后，后端用 `verified_pubkey` 扫描链上 active admin 集合，生成该管理员可登录机构候选。
- 非 active admin 不能登录；国家储委会 / 省储委会 / 省储行返回桌面端专用错误；个人多签返回个人多签不支持错误。
- 本节点未绑定机构时：一个候选也必须在页面显示机构信息并二次确认绑定；多个候选由管理员选择一个后确认绑定。
- 本节点已绑定机构后：后续登录只允许该绑定机构的 active admin；管理员被链上移除后由后台复查清退会话。
- 本节点解绑 / 换机构：必须由当前本机会话管理员发起 `NODE_BINDING_UNBIND` 安全动作，并由冷钱包签名确认；commit 成功后 active binding 置为 `INACTIVE` 并清退本节点管理员会话。换机构不走影子兼容流程，必须先解绑，再由新机构 active admin 重新扫码登录并确认绑定。
- 本地 `node_institution_bindings` 只保存“本节点已绑定哪个机构”的结果与缓存展示字段，不是权限真源；权限真源始终是链上 active admin 关系。
- 登录后 UI 由后端 `capabilities` 单源下发：FRG/CREG 显示注册局业务 tab；NJD、普通公权、私权和非法人组织本期只显示“本机构管理员”只读 tab，并允许管理员在自己的行设置 / 更新 passkey。

### 5. 权限模型 = CID 码（主）+ CID 号（辅）+ 实例覆盖
- **CID 码**（主键）：决定可见 tab / 能力基线、`admin_level`（国/省/市/镇）、所属 admin pallet。
- **CID 号 R5**（辅键，省码+市码）：决定数据 scope 与跨地区写边界。CID 号与作用域来自登录绑定机构的链上/本地投影候选，不再由节点启动前预填。
- **实例覆盖位**（R4 第三层）：同机构码、同层级、跨地区能力可不同。采用「机构码静态模板 ⊕ 链上按 cid 号能力覆盖位」，缺省纯模板；覆盖位本期仅签名占位，配置真源（宪法/治理派生）留后续 ADR。
- 登录即隔离：本节点 active binding = 会话机构边界；每次登录和冷签 step-up 都复查 signer 是否仍属于该机构链上 Active 集合。
- **scope 单一来源 + 边界校验**：省/市/镇 scope 来自本节点 active binding 的机构候选；候选优先由链上管理员集合命中，再用本地 `subjects/accounts` 投影补齐 CID、全称、简称和行政区。所有机构特殊操作经 `get_visible_scope`/`includes_province`/`includes_city` 统一 fail-closed 闸（含 docs/账户/机构创建，prepare 预检 ⊕ 业务 handler 双层）。

### 6. 三档鉴权 + 默认拒绝
固定三档：`Session`（一般操作）/ `Passkey`（重要操作，WebAuthn）/ `PasskeyColdSign`（特殊操作/链上提案，叠冷签）。每个 action 必穷尽 `match` 标注其一，漏标编译失败；三档之外一律拒绝，无第四档、无 `_ =>` 兜底。

### 7. Web 端复杂提案
onchina web 端承接复杂提案（立法投票等），走 `PasskeyColdSign`：web 构造 extrinsic → 冷钱包扫码签 → 提交链，复用 legislation-yuan（idx27）/legislation-vote（idx28），链端零改动。移动端本期不动。

### 8. 改名边界
目录/crate `registry → onchina` 触及 registry 以外 9 个文件（workspace Cargo.toml、node/registry_proc、tauri.conf.json、scripts/{prepack,run,clean-run}.sh + env 改名）。功能改造（卡 01–16）全部锁在 `citizenchain/registry/` 内；改名作为独立最后一步（卡 17），外部文件一次性平移。

## 不碰清单
签名协议 `QR_V1`、签名域 `GMB`、`primitives/cid/code.rs` 机构码表、`china.sqlite`、链上 pallet/事件名/index、链写凭证签名 env、移动端 CitizenApp/CitizenWallet。

## 影响
- 落地任务卡：[20260628-onchina-console-refactor](../08-tasks/open/20260628-onchina-console-refactor.md)（17 张分步卡）。
- **落地状态（2026-06-28）**：结构重构 + 纯位移 + scope 多档(card 08)完工——卡 01–08/11/13–17 + 审查发现（5/1/3/6）+ B/C/D 收尾 + 启动横幅修,全部经多轮 5/4-agent 对抗式验证(sound)。scope 已五档(国/省/市/镇 + 私权本市);公民按省/市级(不挂镇);目录终态:`auth/`(原 admins)、`institution/{accounts,subjects}`、`domains/{gov,private,citizens,docs}`、citizenapp 独立。仅 admin 泛化 09 / seed 10 / governance web 12 延后到各机构登录功能实现期(依赖各机构 capabilities/界面)。`cargo test -p onchina` 68 passed + `cargo check -p node` 绿 + frontend tsc 0 err。
- 关联记忆：[[project_admin_single_source_admins_change_2026_06_21]]、[[project_cid_classification_unify_t3t4_2026_06_22]]、[[feedback_signing_layer_selection_rule]]、[[project_legislation_yuan_adr027]]。
