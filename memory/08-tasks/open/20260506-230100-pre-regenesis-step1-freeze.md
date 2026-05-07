# 重新创世前总审计 · 第 1 步冻结清单（2026-05-07）

- 任务关联：[20260506-230100-pre-regenesis-comprehensive-audit.md](20260506-230100-pre-regenesis-comprehensive-audit.md)
- 本文件是第 1 步交付物：只冻结审计结论与后续动作，不直接修改业务代码、不删除文件
- 审计基线：第 1 步冻结时 `git ls-files` 共 1975 个 tracked 文件；P0-1 执行后为 1953 个 tracked 文件（删除 23 个残留，新增 1 个冻结清单）
- 本轮复核：2026-05-07 已重新执行 Git 基线、残留扫描、协议扫描、文档结构扫描和启动协议验收

---

## 0. 本步结论

当前仓库**还不能直接重新创世**。必须先修完 P0 与 P1 中标为“创世前必做”的项目，否则新创世后会把旧协议、旧 storage、旧构建残留或错误文档真源一起固定下来。

第 2 步执行时按以下 3 批推进：

| 批次 | 目标 |
|---|---|
| A | 版本库洁净：移除 tracked 本地链状态、网络密钥、构建产物、缓存和备份（P0-1 已执行） |
| B | 协议真源统一：统一 runtime / wumin / wuminapp / CI / 文档的 call、storage、era、fixture |
| C | 创世冻结整理：清 runtime 旧物理残留，重排 memory/docs/tasks 结构和任务卡命名 |

---

## 1. 扫描范围

### 1.1 tracked 文件分布

| 根目录 | tracked 文件数 | 审计判断 |
|---|---:|---|
| `wuminapp/` | 582 | 热钱包与轻节点主区，存在 P0 协议漂移 |
| `memory/` | 509 | 系统记忆主目录，存在结构编号、旧文档真源、任务卡命名问题 |
| `citizenchain/` | 392 | runtime/node 主区，存在本地链状态 tracked 和部分 legacy runtime 残留 |
| `sfid/` | 220 | 在线身份系统，实际目录符合 SFID 新边界 |
| `wumin/` | 138 | 冷钱包，decoder/CI 脚本与当前协议有漂移 |
| `cpms/` | 69 | 离线实名系统，存在 SFID 路径引用漂移 |
| `website/` | 28 | 官网源码，暂无本步 P0 |
| `.github/` | 11 | CI 中存在旧 `supportedSpecVersions` 同步残留 |
| `docs/` | 8 | 根目录文档/演示资产定位未登记 |
| `tools/` | 4 | 存在 `__pycache__` tracked 与旧注释生成模板 |
| `scripts/` | 2 | 暂无本步 P0 |

### 1.2 误判过滤规则

以下命中不直接当问题：

- `memory/08-tasks/done/` 中的历史记录，除非违反当前硬规则（如任务卡文件名长度）
- 明确标 `LEGACY / 已废弃` 的历史技术文档，除非仍自称“唯一事实源”
- `wuminapp/smoldot-pow/` 上游代码里的 TODO/legacy
- `sfid/backend/sfid/city_codes/` 行政区划数据中自然出现的“旧”
- `cpms/backend/src/` 自身目录名；CPMS 后端仍是正常 Rust `src/` 布局，不属于 SFID 禁止项

---

## 2. P0：创世前必须解决

### P0-1：版本库里 tracked 了本地链状态、网络密钥和生成物（已执行）

原始冻结时 `git ls-files` 包含 23 个不应入库文件：

| 类型 | 路径 |
|---|---|
| 本地链 RocksDB | `citizenchain/.local-node/chains/citizenchain/db/full/000038.sst` 到 `000049.sst` 等 |
| 本地链 RocksDB 元数据 | `CURRENT`、`IDENTITY`、`LOCK`、`LOG`、`MANIFEST-000057`、`OPTIONS-*`、`db_version` |
| 本地网络密钥 | `citizenchain/.local-node/chains/citizenchain/network/secret_ed25519` |
| 前端构建产物 | `citizenchain/node/frontend/dist/index.html` 与 `dist/assets/*` |
| Python 缓存 | `tools/__pycache__/fill_china_admins.cpython-314.pyc` |
| 旧 light sync 备份 | `wuminapp/assets/light_sync_state.json.spec9-backup` |

执行结果（2026-05-07）：

- 已从 Git 索引和工作区移除上述 23 个文件
- 已补 `.gitignore`：`citizenchain/.local-node/`、`citizenchain/node/frontend/dist/`、`**/__pycache__/`、`*.pyc`、`*.spec*-backup`
- `secret_ed25519` 视为已泄露；重新创世后重新生成，禁止复用
- 验收命令 `git ls-files | rg '(^citizenchain/\.local-node/|^citizenchain/node/frontend/dist/|^tools/__pycache__/|\.pyc$|\.spec.*-backup$|secret_ed25519)'` 已无输出
- `git check-ignore` 已确认四类路径会被忽略

### P0-2：机构多签创建协议 runtime / wumin / wuminapp 三端不一致

执行状态：2026-05-07 已完成首轮统一，执行任务卡为
[`20260507-p0-2-propose-create-institution.md`](20260507-p0-2-propose-create-institution.md)。

已落地结果：

- `wuminapp` 机构创建 caller 已从旧 `17.0` 单账户入口改为 `OrganizationManage(17).propose_create_institution(5)`
- `wuminapp` 提交前读取 SFID `/registration-info`，使用 `account_names/register_nonce/signature/province/signer_admin_pubkey`
- `wuminapp` 创建页面改为按账户列表填写每个账户初始资金，并按 `registration-info.account_names` 顺序编码
- `wumin` 冷钱包 decoder 已删除 `a3 / sub_type / parent_sfid_number` 旧尾字段解析，解码后禁止剩余字节
- `citizenchain node` 冷钱包 display 已补齐 `province / signer_admin_pubkey`
- 已恢复 `wuminapp/test/duoqian/duoqian_manage_service_test.dart` 字节级回归

验收记录：

- `flutter test test/duoqian/duoqian_manage_service_test.dart`：通过
- `flutter test test/signer/payload_decoder_test.dart`：通过
- `flutter analyze` 目标文件：通过
- `rustfmt --check citizenchain/node/src/offchain/organization_manage/signing.rs`：通过
- `cargo check -p node`：已执行，被 `WASM_FILE` 仓库硬规则阻断

以下为执行前审计证据，保留用于追溯：

当前三端状态互相打架：

| 位置 | 当前事实 |
|---|---|
| [organization-manage/src/lib.rs:530](../../../citizenchain/runtime/governance/organization-manage/src/lib.rs:530) | runtime 当前入口是 `OrganizationManage::propose_create_institution`，`call_index=5` |
| [institution/create.rs:39](../../../citizenchain/runtime/governance/organization-manage/src/institution/create.rs:39) | runtime 当前参数是 `sfid_number / institution_name / accounts / admin_count / duoqian_admins / threshold / register_nonce / signature / province / signer_admin_pubkey` |
| [duoqian_manage_service.dart:40](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:40) | wuminapp 仍保留 `_proposeCreateCallIndex = 0` |
| [duoqian_manage_service.dart:66](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:66) | wuminapp 注释和编码仍是旧 `propose_create` 参数 |
| [institution_duoqian_create_page.dart:351](../../../wuminapp/lib/duoqian/institution/institution_duoqian_create_page.dart:351) | QR display 仍写“链端复用 propose_create Registry” |
| [payload_decoder.dart:625](../../../wumin/lib/signer/payload_decoder.dart:625) | wumin decoder 识别 `17.5`，但注释/解析还期待 `a3 / sub_type / parent_sfid_number` |
| [payload_decoder.dart:734](../../../wumin/lib/signer/payload_decoder.dart:734) | decoder 在 `signer_admin_pubkey` 后继续读 `a3`，与当前 runtime 参数不一致 |
| [duoqian_manage_service_test.dart:1](../../../wuminapp/test/duoqian/duoqian_manage_service_test.dart:1) | wuminapp 该协议测试已清空，只剩空 `main()` |
| [20260502-wuminapp-propose-create-institution-caller-fix.md](20260502-wuminapp-propose-create-institution-caller-fix.md) | 任务卡 Progress 写“已完成”，但当前代码不符合该 Progress |

具体要做：

- 已建立统一协议入口：`memory/07-ai/unified-protocols.md`，本条以后按 `P-TX-001：OrganizationManage.propose_create_institution` 执行
- 口径修正：扫码协议仍只有 `WUMIN_QR_V1`，本条统一的是 `sign_request.payload_hex` 内层交易载荷格式，不是新增扫码协议
- 以当前 runtime 为真源，冻结 `17.5` 的 SCALE 参数顺序
- wuminapp：把机构创建 caller 从 `17.0` 改为 `17.5`，字段改为当前 runtime 10 参数
- wumin：删除 `a3 / sub_type / parent_sfid_number` 解析或同步 runtime 是否重新加入这些字段，二者必须二选一
- wuminapp：恢复字节级测试，测试头两字节必须为 `[0x11, 0x05]`
- 更新任务卡 Progress，撤销“已完成”误导

### P0-3：`wuminapp` 仍读已删除的 `OrganizationManage::DuoqianAccounts`（已执行）

执行状态：2026-05-07 已完成，执行任务卡为
[`20260507-p0-3-duoqian-storage-truth.md`](20260507-p0-3-duoqian-storage-truth.md)。

已落地结果：

- 新增 `DuoqianStorageCodec` 统一维护多钱相关 storage key 与 SCALE decoder。
- `fetchPersonalMeta` 已改读 `PersonalManage::PersonalDuoqianInfo`。
- `fetchDuoqianAccount` 已改为注册机构路径 `AddressRegisteredSfid -> Institutions + InstitutionAccounts`，并回退到个人多签 `PersonalManage::PersonalDuoqians`。
- `InstitutionAdminService` 已改为统一读 `AdminsChange::Subjects`，覆盖 `duoqian:`、`personal:`、内置机构三类 subject。
- `wuminapp/lib` 与 `wuminapp/test` 已无旧 `OrganizationManage::DuoqianAccounts` 活跃读取。

验收记录：

- `flutter test test/duoqian/duoqian_storage_codec_test.dart test/duoqian/duoqian_manage_storage_test.dart test/institution/institution_admin_service_test.dart test/duoqian/duoqian_manage_service_test.dart`：通过
- `flutter test test/duoqian test/institution`：通过
- `flutter analyze lib/duoqian/shared lib/institution test/duoqian test/institution`：通过
- `rg -n "DuoqianAccounts|OrganizationManage.*PersonalDuoqianInfo|AdminsChange\\.Institutions|admins-change Institutions" wuminapp/lib wuminapp/test`：无输出

以下为执行前审计证据，保留用于追溯：

runtime 当前以 `InstitutionAccounts` / `PersonalDuoqians` 为真源；执行前 wuminapp 曾构造旧 storage：

- [duoqian_manage_service.dart:348](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:348)：注释曾称从 `DuoqianAccounts` 解码
- [duoqian_manage_service.dart:357](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:357)：构造 `OrganizationManage::DuoqianAccounts`
- [institution_admin_service.dart:40](../../../wuminapp/lib/institution/institution_admin_service.dart:40)：阈值来源曾写 `DuoqianAccounts.threshold`
- [institution_admin_service.dart:76](../../../wuminapp/lib/institution/institution_admin_service.dart:76)：注册多签/个人多签都走 `DuoqianAccounts`
- [institution_admin_service.dart:143](../../../wuminapp/lib/institution/institution_admin_service.dart:143)：构造旧 storage key

具体要做：

- 注册机构：改读 `OrganizationManage::InstitutionAccounts` 和 `OrganizationManage::Institutions`
- 个人多签：改读 `PersonalManage::PersonalDuoqians`
- 管理员与阈值：统一读 `AdminsChange::Subjects`
- 删除旧 `_buildDuoqianAccountsKey` 或改名为 legacy 测试 helper

### P0-4：热钱包签名 era 策略未统一（已执行）

执行状态：2026-05-07 已完成，执行任务卡为
[`20260507-p0-4-immortal-era.md`](20260507-p0-4-immortal-era.md)。

已落地结果：

- 已在统一协议文件登记 `P-SIGN-001：Citizenchain signed extrinsic era`。
- `wuminapp` 新增统一 `SignedExtrinsicBuilder`，所有在线签名 extrinsic 固定 `eraPeriod = 0 / era = 0x00 / blockNumber = 0 / blockHash = genesisHash`。
- 已替换 `OnchainRpc`、`InternalVoteService`、`RuntimeUpgradeService`、`TransferProposalService`、`DuoqianManageService`、`OnchainClearingBankRpc` 六条在线签名路径。
- `ChainRpc.fetchLatestBlock()` 仅保留给 UI 展示、事件查询和诊断，不再参与 signed extrinsic 构造。

验收记录：

- `flutter test test/rpc/signed_extrinsic_builder_test.dart`：通过
- `flutter test test/duoqian test/proposal test/trade`：通过
- `flutter analyze lib/rpc lib/proposal lib/duoqian/shared lib/offchain/rpc test/rpc test/duoqian test/proposal test/trade`：通过
- `rg -n "_eraPeriod\\s*=\\s*64|mortal era=64|Mortal era" wuminapp/lib wuminapp/test`：无输出
- `rg -n "SigningPayload\\(|ExtrinsicPayload\\(" wuminapp/lib`：只命中 `wuminapp/lib/rpc/signed_extrinsic_builder.dart`

以下为执行前审计证据，保留用于追溯：

链端 node 已使用 immortal era，但 `wuminapp` 多条在线签名路径仍使用 `_eraPeriod = 64`：

- [onchain.dart:27](../../../wuminapp/lib/rpc/onchain.dart:27)
- [internal_vote_service.dart:36](../../../wuminapp/lib/proposal/shared/internal_vote_service.dart:36)
- [runtime_upgrade_service.dart:35](../../../wuminapp/lib/proposal/runtime_upgrade/runtime_upgrade_service.dart:35)
- [transfer_proposal_service.dart:45](../../../wuminapp/lib/proposal/transfer/transfer_proposal_service.dart:45)
- [duoqian_manage_service.dart:49](../../../wuminapp/lib/duoqian/shared/duoqian_manage_service.dart:49)
- [onchain_clearing_bank_rpc.dart:25](../../../wuminapp/lib/offchain/rpc/onchain_clearing_bank_rpc.dart:25)

具体要做：

- 如果 PoW 链规则是“所有推链交易 immortal”，则以上全部改为 immortal，并移除 latest block hash/number 依赖
- 如果热钱包允许 mortal，则必须新增权威协议文档，明确“node/冷钱包/离线签名 immortal，热钱包在线签名 mortal”的边界

### P0-5：Step2D fixture 漂移，冷钱包与热钱包同名用例不一致（已执行）

执行状态：2026-05-07 已完成，执行任务卡为
[`20260507-p0-5-step2d-fixture.md`](20260507-p0-5-step2d-fixture.md)。

已落地结果：

- 已在统一协议文件登记 `P-TX-002：JointVote.cast_referendum`。
- 已新增统一 fixture：`memory/06-quality/fixtures/step2d_credential_payload.json`。
- `cast_referendum` fixture 已统一到 `JointVote(23).cast_referendum(1)`，前缀固定 `0x1701`。
- 已删除 `wumin/test/fixtures/step2d_credential_payload.json` 与 `wuminapp/test/fixtures/step2d_credential_payload.json` 两份重复副本。
- `wumin` 与 `wuminapp` 测试都改读统一 fixture，并补 `23.1 / 0x1701` 断言。

验收记录：

- `flutter test test/signer/payload_decoder_test.dart test/signer/pallet_registry_test.dart`：通过
- `flutter test test/proposal/runtime_upgrade/runtime_upgrade_service_test.dart`：通过
- `flutter analyze test/signer`：通过
- `flutter analyze test/proposal`：通过
- `rg -n '"pallet_index": 9|"call_index": 2|0x0902|test/fixtures/step2d_credential_payload\\.json' wumin/test wuminapp/test memory/06-quality`：无输出

以下为执行前审计证据，保留用于追溯：

- [wumin fixture:20](../../../wumin/test/fixtures/step2d_credential_payload.json:20)：`cast_referendum` expected 是 `0x1701...`
- [wuminapp fixture:20](../../../wuminapp/test/fixtures/step2d_credential_payload.json:20)：同名 expected 是 `0x0902...`
- 两边 [metadata:9](../../../wumin/test/fixtures/step2d_credential_payload.json:9) 都仍写 `pallet_index=9 / call_index=2`

具体要做：

- 以当前 runtime `JointVote(23).cast_referendum(1)` 为真源
- 同步 `expected_call_data_hex` 与 metadata
- 两端测试统一从同一份生成源或同一份 fixture 派生

### P0-6：CPMS 编译脚本旧 SFID 路径（已执行）

- [cpms/backend/build.rs](../../../cpms/backend/build.rs)：默认路径已改为 `../../sfid/backend/sfid`
- [cpms/backend/build.rs](../../../cpms/backend/build.rs)：错误提示已改为 `/path/to/sfid/backend/sfid`
- [province_codes.rs](../../../cpms/backend/src/dangan/province_codes.rs)：注释已同步为新路径
- 当前实际目录固定为 `sfid/backend/sfid/`，不得恢复 `sfid/backend/src/`

验收：

- `cargo check --manifest-path cpms/backend/Cargo.toml` 已通过，证明 build.rs 能找到 `province.rs` 和 `city_codes/`；CPMS 仍有 19 个既有 warning，后续单列清理

---

## 3. P1：创世前建议同批处理

### P1-1：`wumin` CI 与本地脚本 spec 残留（已执行）

- [.github/workflows/wumin-ci.yml:40](../../../.github/workflows/wumin-ci.yml:40)
- [wumin-run.sh:23](../../../wumin/scripts/wumin-run.sh:23)
- [pallet_registry.dart:7](../../../wumin/lib/signer/pallet_registry.dart:7) 已明确 `supportedSpecVersions / isSupported` 物理移除

执行结果：

- 已删除 CI 和本地脚本中的 `supportedSpecVersions` 写源码 sed
- 当前脚本仅同步仍存在的 pallet/call index 常量，不再读取链上 spec_version 后写冷钱包源码

### P1-2：`organization-manage` 机构创建代投物理残留（已执行）

- 已删除 `MaxAdminSignatureLength`、`AdminSignatureOf<T>`、`AdminSignaturesOf<T>`。
- 已删除无 emit 路径的 `CreateFinalized` 事件。
- 已删除无使用路径的 `UnauthorizedSignature`、`DuplicateSignature`、`InvalidSignature`、`InsufficientSignatures`。
- [lib.rs](../../../citizenchain/runtime/governance/organization-manage/src/lib.rs)：`MalformedSignature` 仍活跃并已保留。

验收：

- `cargo check -p organization-manage` 通过。
- `cargo test -p organization-manage --lib` 通过，24 个测试全绿。
- `cargo test -p duoqian-transfer --lib` 通过，20 个测试全绿。
- `cargo check -p citizenchain --lib` 在设置 `WASM_FILE=target/wasm/citizenchain.compact.compressed.wasm` 后通过。

### P1-3：`organization-manage/src/institution/close.rs` 错误边界占位（已执行）

- `citizenchain/runtime/governance/organization-manage/src/institution/close.rs` 已删除。
- [institution/mod.rs](../../../citizenchain/runtime/governance/organization-manage/src/institution/mod.rs)：已移除 `pub mod close;`。
- 当前事实：机构关闭在 `organization-manage`，个人关闭在 `personal-manage`

### P1-4：活跃代码/文档旧 `duoqian-manage` 模块名（已执行）

执行结果：

- 活跃 runtime 注释已统一写 `organization-manage` / `personal-manage`。
- `tools/duoqian.py` 生成的制度保留地址注释已同步为 `organization-manage`。
- `wuminapp/lib/institution/institution_data.dart` 中注册多签来源已改为 `organization-manage` 的机构主账户。

保留：

- 历史任务卡中的旧名记录留到 PR-D 归档/冻结阶段处理。
- `DuoqianManageService` / `DuoqianManageDetailPage` 属于 wuminapp “多钱管理”业务命名，本轮不重命名。

### P1-5：活跃文档旧 storage 真源（已执行）

执行结果：

- `ORGANIZATION_MANAGE_TECHNICAL.md` 已删除把旧 `DuoqianAccounts` 当当前 storage 的叙述。
- `DUOQIAN_TRANSFER_TECHNICAL.md` 已改为 `OrganizationManage::InstitutionAccounts`。
- `GOVERNANCE_TECHNICAL.md` 已改为 `AdminsChange.Subjects`、`OrganizationManage::InstitutionAccounts`、`PersonalManage::PersonalDuoqians`。
- `CHAIN_TECHNICAL.md` 已改为按 `organization-manage` / `personal-manage` / `admins-change::Subjects` 边界校验。

保留：

- 明确写“已删除 / 替代旧 storage / migration 清理旧 storage”的 legacy/history 说明。

### P1-6：QR 协议文档自称唯一事实源，但内容已过期且目录编号不合规

- 状态：已执行（2026-05-07）
- 新真源目录：[memory/01-architecture/qr/](../../01-architecture/qr/)
- 任务卡：[20260507-p1-6-qr-protocol-source.md](20260507-p1-6-qr-protocol-source.md)

已完成：

- QR spec、签名识别方案、action registry、fixture 已从旧非标准 QR 架构目录迁到 `memory/01-architecture/qr/`。
- `memory/07-ai/unified-protocols.md` 与 `memory/07-ai/unified-naming.md` 已登记新路径。
- action registry 已更新为当前 `InternalVote(22)` / `JointVote(23)` / `OrganizationManage(17)` / `PersonalManage(7)` / `VotingEngine(9)` 生命周期入口。
- 已删除当前识别规则中的 `supportedSpecVersions` 要求。
- 已删除 wumin / node / sfid / cpms 侧已下线 `user_duoqian` kind 残留。
- 已对齐 `propose_close_institution` / `propose_close_personal` 与 wumin decoder 输出。

---

## 4. P2：结构、命名和任务卡冻结

2026-05-07 已新增统一命名入口：`memory/07-ai/unified-naming.md`。后续所有新建或重命名目录、文件、字段、变量、类、模块、API 字段、storage 字段、QR display 字段、任务卡文件名、文档文件名，都必须先按该文件执行；命名不确定时必须先报告确认。

2026-05-07 已新增统一必读入口：`memory/07-ai/unified-required-reading.md`。后续每次设计、编程、改协议、改命名、改文档、改流程前，都必须先按该文件确认必读清单。

### P2-1：repo-map 根目录与实际 tracked 根目录不一致

[repo-map.md:7](../../01-architecture/repo-map.md:7) 固定根目录只列：

- `.github/`
- `memory/`
- `citizenchain/`
- `sfid/`
- `cpms/`
- `wuminapp/`
- `website/`

但当前 tracked 还包含：

- `wumin/`：冷钱包正式项目，未写入根目录固定表
- `docs/`：根目录静态文档/图片/PPT，未登记定位
- `tools/`、`scripts/`：工具脚本，未登记边界
- `README.md`、`Dockerfile`、根 `Cargo.toml` 等根配置，未写清边界

具体要做：

- repo-map 增补 `wumin/`、`tools/`、`scripts/`、`docs/` 的正式定位
- 若 `docs/` 只是旧发布产物，迁入 `website/` 或 `memory/` 后删除根目录入口

### P2-2：`memory/tasks/` 旧任务目录仍 tracked

当前旧目录：

- `memory/tasks/smoldot-checkpoint-plan.md`
- `memory/tasks/smoldot-kbuckets-dht.md`
- `memory/tasks/smoldot-stability-plan.md`

具体要做：

- 未完成任务迁入 `memory/08-tasks/open/`
- 已完成/历史资料迁入 `memory/08-tasks/done/` 或模块文档
- 删除 `memory/tasks/` 入口

### P2-3：任务卡文件名当前未超过 160 UTF-8 字节，但已有贴线风险

按 AGENTS 硬规则“任务卡文件名（含 `.md` 扩展名）不得超过 160 个 UTF-8 字节”重新复核，当前 `memory/08-tasks/` 下超限数量为 0。此前“49 个超限”是把完整路径长度误计入文件名长度，不作为当前违规项。

但已有 3 个 done 任务卡文件名正好 160 字节，open 目录最长文件名为 159 字节，继续使用长中文标题会很容易撞线。

具体要做：

- 新建任务卡统一使用短文件名：`日期-短英文/拼音主题.md`
- 文件内保留完整中文标题
- 对 159/160 字节贴线文件，若后续被重新引用或迁移，优先改为短文件名并更新链接

### P2-4：`memory/08-tasks/open/` 长期 open 任务过多

当前统计：

- `open/`：90 个 `.md`
- `done/`：212 个 `.md`
- `templates/`：10 个 `.md`

具体要做：

- 每个 open 卡标为三类之一：继续执行 / 已被取代 / 已完成待归档
- 被当前重新创世总审计覆盖的旧任务，统一加 `superseded by 20260506-230100` 后归档

### P2-5：wuminapp 目录命名有大小写不统一

当前 `wuminapp/lib/` 下有 `Isar/`，而 `wumin/lib/isar/` 是小写；Dart/Flutter 目录通常应小写。

具体要做：

- 评估 `wuminapp/lib/Isar/` 是否改为 `wuminapp/lib/isar/`
- 若改名，必须同步 imports、生成文件和测试

---

## 5. 本步确认通过的事项

- 根 `AGENTS.md` 与 `memory/AGENTS.md` 一致
- 根 `CODEX.md` 与 `memory/CODEX.md` 一致
- 根 `CLAUDE.md` 与 `memory/CLAUDE.md` 一致
- `bash memory/scripts/check-startup-acceptance.sh` 通过
- `sfid/backend/src/`、`sfid/frontend/src/`、`sfid/backend/chain/`、`sfid/frontend/chain/`、`sfid/frontend/api/` 当前没有 tracked 源码目录
- `sfid/frontend/dist/`、`website/dist/` 存在于本地工作区但未 tracked，已被对应 `.gitignore` 忽略

---

## 6. 第 2 步入口顺序

第 2 步不要先做大迁移，按这个顺序执行：

1. **版本库残留已处理**：P0-1
2. **协议真源已统一**：P0-2 / P0-3 / P0-5 / P1-1
3. **era 策略已处理**：P0-4
4. **CPMS/SFID 路径已处理**：P0-6
5. **最后做文档与结构冻结**：P1-2 到 P2-5
