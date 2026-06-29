# ADR-030 onchina 多机构统一控制台

## 状态

已接受（2026-06-28）。取代 registry 的"联邦/市注册局双角色"定位，扩展为全机构控制台。承接 [ADR-029](ADR-029-registry-into-citizenchain.md) 的去中心化进程模型。

## 背景

registry 后端最初只服务联邦注册局 + 市注册局两类管理员（前端 `registry_org_code` 二值枚举、后端 `node_institution_identity` 仅放行 FRG/公权两类）。随着立法院、监察院、政府、储备、私权公司等机构都要在电脑端登录发起提案，需要一套统一控制台：同一套软件、同一登录入口，按机构身份显示不同子 tab、不同权限。

## 决策

### 1. 平台定位
registry 重定位为通用 CID 机构控制台，产品名 **onchina**（链上中国，china≈chain 双关）。注册局只是其中一个机构租户。去中心化：每机构在自己办公室部署节点（内嵌 onchina 后端 + 本地 PostgreSQL）。

### 2. 统一入口
全节点统一访问字符串 `http://onchina.local:8964`：后端绑 `0.0.0.0:8964`，mDNS 广告 `onchina.local`（`_onchina._tcp.local`）。各机构管理员输入同一字符串，连的是各自办公室本地节点。"统一"指统一字符串与体验，非指向同一台服务器。

### 3. 机构范围 = 全部
服务 `primitives::cid::code` 全部能发号机构。节点身份按机构码路由到 3 个链上管理员集合容器：

| 机构类别 | 判定 | 链上 pallet（index） |
|---|---|---|
| 联邦注册局 FRG / 固定治理档 NRC/PRC/PRB | `== FRG` 或 `is_fixed_governance_code` | `GenesisAdmins`（12） |
| 其它公权法人（政府/立法/监察/司法/教育/储委以外公权/注册局/公安等） | `is_public_legal_code` | `PublicAdmins`（29） |
| 私权法人（股权/股份/有限合伙/公益/协会/私立学校等 SF*） | `is_private_legal_code` | `PrivateAdmins`（30） |

个人多签 PMUL（personal-admins，idx7）**不登录控制台**：无 CID、不跑节点，纯 CitizenApp 客户端功能。

### 4. 权限模型 = CID 码（主）+ CID 号（辅）+ 实例覆盖
- **CID 码**（主键）：决定可见 tab / 能力基线、`admin_level`（国/省/市/镇）、所属 admin pallet。
- **CID 号 R5**（辅键，省码+市码）：决定数据 scope 与跨地区写边界。节点启动即带自身 `CID_RUNTIME_ISSUER_CID_NUMBER`，码与地区当场解析。
- **实例覆盖位**（R4 第三层）：同机构码、同层级、跨地区能力可不同。采用「机构码静态模板 ⊕ 链上按 cid 号能力覆盖位」，缺省纯模板；覆盖位本期仅签名占位，配置真源（宪法/治理派生）留后续 ADR。
- 登录即隔离：节点身份 = 权限边界，复用 `onchain_gate`「signer ∈ 本机构链上 Active 集合」机制。
- **scope 单一来源 + 边界校验**：非联邦注册局机构的省/市 scope 来自节点 `CID_RUNTIME_SCOPE_*` env，唯一在 `onchain_gate` 的 env→会话签发边界校验其落在 `china.sqlite` 真源内（省存在、市属省），不一致即拒登录（`GateError::Config`，明确报错）。绝不在读路径对该 env 写入的 city_name 反复 fail-closed（会因 env 与 china.sqlite 不逐字一致而每请求锁死合法管理员）。联邦注册局省走 `federal_registry_scope`（china_zf 创世映射）、无市维度。所有机构特殊操作经 `get_visible_scope`/`includes_province`/`includes_city` 统一 fail-closed 闸（含 docs/账户/机构创建，prepare 预检 ⊕ 业务 handler 双层）。

### 5. 三档鉴权 + 默认拒绝
固定三档：`Session`（一般操作）/ `Passkey`（重要操作，WebAuthn）/ `PasskeyColdSign`（特殊操作/链上提案，叠冷签）。每个 action 必穷尽 `match` 标注其一，漏标编译失败；三档之外一律拒绝，无第四档、无 `_ =>` 兜底。

### 6. Web 端复杂提案
onchina web 端承接复杂提案（立法投票等），走 `PasskeyColdSign`：web 构造 extrinsic → 冷钱包扫码签 → 提交链，复用 legislation-yuan（idx27）/legislation-vote（idx28），链端零改动。移动端本期不动。

### 7. 改名边界
目录/crate `registry → onchina` 触及 registry 以外 9 个文件（workspace Cargo.toml、node/registry_proc、tauri.conf.json、scripts/{prepack,run,clean-run}.sh + env 改名）。功能改造（卡 01–16）全部锁在 `citizenchain/registry/` 内；改名作为独立最后一步（卡 17），外部文件一次性平移。

## 不碰清单
签名协议 `QR_V1`、签名域 `GMB`、`primitives/cid/code.rs` 机构码表、`china.sqlite`、链上 pallet/事件名/index、`CID_*` 身份 env、移动端 CitizenApp/CitizenWallet。

## 影响
- 落地任务卡：[20260628-onchina-console-refactor](../08-tasks/open/20260628-onchina-console-refactor.md)（17 张分步卡）。
- **落地状态（2026-06-28）**：结构重构 + 纯位移 + scope 多档(card 08)完工——卡 01–08/11/13–17 + 审查发现（5/1/3/6）+ B/C/D 收尾 + 启动横幅修,全部经多轮 5/4-agent 对抗式验证(sound)。scope 已五档(国/省/市/镇 + 私权本市);公民按省/市级(不挂镇);目录终态:`auth/`(原 admins)、`institution/{accounts,subjects}`、`domains/{gov,private,citizens,docs}`、citizenapp 独立。仅 admin 泛化 09 / seed 10 / governance web 12 延后到各机构登录功能实现期(依赖各机构 capabilities/界面)。`cargo test -p onchina` 68 passed + `cargo check -p node` 绿 + frontend tsc 0 err。
- 关联记忆：[[project_admin_single_source_admins_change_2026_06_21]]、[[project_cid_classification_unify_t3t4_2026_06_22]]、[[feedback_signing_layer_selection_rule]]、[[project_legislation_yuan_adr027]]。
