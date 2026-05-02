# SFID Step 1 / Phase 7:Step 1 验收 + 文档全量更新 + 残留清理

- 状态:open
- 创建日期:2026-05-01
- 模块:`sfid/backend` + `sfid/frontend` + `memory/05-modules/sfid/`
- 上游:`memory/08-tasks/open/20260501-sfid-step1-sheng-admin-3tier-and-key-admin-removal.md`(主卡)
- 关联 ADR:`memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md`
- 前置依赖:Phase 2+3 卡 + Phase 4+5 卡 + Phase 6 卡 全部完成
- 联调依赖:Step 2(citizenchain runtime)— 链上 extrinsic 上线后,本卡完成 mock → 真实推链联调

## 任务需求

Step 1 SFID 改造收尾:全量文档更新、注释完善、残留清理、端到端验收(等 Step 2 上线后链上联调替换 mock)。

## 建议模块

- `memory/05-modules/sfid/`:全量文档重写
- `sfid/backend/src/` + `sfid/frontend/src/`:注释 + 残留扫除
- `sfid/backend/src/chain/sheng_admin|sheng_signer/`:mock → 真实联调

## 影响范围

### 文档全量更新

| 路径 | 动作 |
|---|---|
| `memory/05-modules/sfid/backend/key-admins/KEY_ADMINS_TECHNICAL.md` | DELETE 整文件(整目录可删) |
| `memory/05-modules/sfid/backend/sheng_admins/SHENG_ADMINS_TECHNICAL.md` | 全量重写:3-tier 模型 + activation + rotate + roster |
| `memory/05-modules/sfid/backend/shi_admins/SHI_ADMINS_TECHNICAL.md` | 重写:签名密钥来自登录省管理员 cache |
| `memory/05-modules/sfid/backend/login/LOGIN_TECHNICAL.md` | 删 KEY_ADMIN 角色提及 |
| `memory/05-modules/sfid/backend/sfid/SFID_TECHNICAL.md` | 更新模块图,删 sheng-admins/operate/business 旧路径,加 ProvinceAdmins 结构说明 |
| `memory/05-modules/sfid/backend/models/MODELS_TECHNICAL.md` | 更新拆分后的 6 文件结构 |
| `memory/05-modules/sfid/backend/chain/` | 加 sheng_admin / sheng_signer 模块文档 |
| `memory/05-modules/sfid/SFID-CPMS-QR-v1-impl-plan.md` | 更新 sheng-admins → sheng_admins 路径 |
| `memory/05-modules/sfid/frontend/` | 加 RosterPage / ActivationPage / RotatePage 说明 + 删 keyring 文档 |
| `memory/05-modules/sfid/README.md` | 顶层概览同步 |

### 注释完善

- 每个 Step 1 新增模块顶部加 1-3 行中文用途说明
- `chain/sheng_admin/` `chain/sheng_signer/` 推链流程详注(Pays::No 原因 + 1010 错误如何避免)
- `sheng_admins/bootstrap.rs` 注释签名 seed 加密 / wrap key HKDF 派生流程
- `models/{role,slot,session}.rs` 顶部说明各类型职责

### 残留清理(全量扫描)

```
grep -rEn "KeyAdmin|key-admin|key_admin|key-admins" sfid/ memory/05-modules/sfid/  # 必须零结果
grep -rn "#\[path"     sfid/backend/src/                                            # 必须零结果
find sfid/backend/src/ -name "*-*.rs" -o -type d -name "*-*"                        # 必须零结果
grep -rn "set_sheng_signing_pubkey" sfid/backend/src/sheng_admins/                  # 必须零结果(已搬到 chain/sheng_signer)
grep -rn "operate/\|business/\|chain/balance/\|chain/key_admins/" sfid/backend/src/ # 必须零结果
grep -rn "sheng_signer_cache" sfid/backend/src/key-admins/                          # 必须零结果(整目录已删)
grep -rEn "keyring|key_admin" sfid/frontend/src/                                    # 必须零结果
```

### 数据库验证

- `sfid/backend/db/migrations/` 新增 migration 已落地:`DROP TABLE key_admins`
- 本地 PostgreSQL 跑全部 migration → schema 与代码对齐

### Step 2 联调(等 citizenchain runtime extrinsic 上线)

- `chain/sheng_admin/{add_backup,remove_backup}.rs`:mock → 真实推链
- `chain/sheng_signer/{activation,rotation}.rs`:mock → 真实推链
- 删除 `// TODO(step2-联调)` 标记
- e2e 验证(详 主卡 Phase 7 验收清单):
  - 安徽 main 登录 → activation 推链成功 → 链上 `ShengSigningPubkey[AH][main_pubkey]` 写入 + Pays::No 不收费
  - 安徽 main → roster 加 backup_1 → 链上 `ShengAdmins[AH][Backup1]` 写入
  - 安徽 backup_1 私钥登录 → 验签匹配链上 backup → 登录通过
  - 安徽 backup_1 激活 → 与 main 签名密钥独立
  - 河北 main 登录 → 跨省查看 OK → 跨省写拒绝
  - backup_1 试图 add backup_2 → 链上拒绝
  - 安徽 main rotate → 链上 signing pubkey 替换
  - 联合投票 `GET /chain/joint-vote/snapshot?province=AH` 返回安徽人口子签

## 主要风险点

- **文档重写工作量被低估**:`memory/05-modules/sfid/` 下 ~10 份技术文档要更新,每份 100-300 行,总量超 2000 行文档改动。
- **联调期 Step 2 不就绪**:本卡部分内容(mock → real)依赖 Step 2 完成;若 Step 2 延迟,本卡只能完成文档/注释/残留三件,联调推迟。
- **数据库迁移不可逆**:`DROP TABLE key_admins` 一旦执行,生产数据丢失;部署前需 backup。
- **e2e 测试自动化缺失**:目前 SFID 没有完整 e2e 套件,Step 1 验收靠手工。建议本卡顺手补一个 minimal e2e bash 脚本(走 curl)。

## 是否需要先沟通

- 否(收尾性任务,边界已清楚)

## 建议下一步

1. 文档全量更新(`memory/05-modules/sfid/` 重写)
2. 注释完善扫描:`grep -rn 'TODO\|FIXME' sfid/backend/src/` 全部消化
3. 残留扫描脚本固化到 `memory/scripts/sfid-step1-residue-check.sh`,CI 调用
4. 等 Step 2 完成 → mock 切真,e2e 跑一遍
5. 主卡 + 4 张子卡全部 close,挪到 `memory/08-tasks/done/`,更新 index.md
6. 落地 user memory 备忘:Step 1 完成、3-tier 模型固化

## 验收清单

- 所有 grep 残留扫描通过(零结果)
- 数据库 migration 跑通 + schema 校验脚本通过
- 文档与 ADR-008 全对齐
- e2e 8 项手工(或脚本)验证全绿
- 主卡 + 4 张子卡 close + 归档

## 工作量预估

- 文档:~1d
- 注释完善:~0.5d
- 残留清理 + e2e:~0.5d(等 Step 2)
- 合计:~2d(不含 Step 2 等待)

## 提交策略

- feature branch:`sfid-step1-phase7-acceptance-and-cleanup`
- 单 PR 落地,作为 Step 1 收口 PR
- PR 描述:贴出全部 grep 残留检查输出 + e2e 结果

## Progress(2026-05-01,SFID Agent 工作线程)

### 已完成 — chain push 4 个 extrinsic mock → real

| 文件 | 改动 |
|---|---|
| `sfid/backend/src/chain/client.rs` | 整体重写:`MockTxHash` → `TxHash`(保留 `.hex` 字段,handler 调用点零变化);`submit_immortal_paysno_mock` → `submit_immortal_paysno(label, call_bytes)` 真实推链;新增 V4 BARE 包装 helper `wrap_v4_bare`(version_byte=0x04 + Compact len 前缀);新增 32 字节随机 `generate_sheng_nonce`;subxt OnlineClient 单例 `tokio::OnceCell` + 指数退避重试 3 次(200/400/800ms);`TxStatus::InBestBlock` 收口;1010 / Invalid → `ChainPushError::InvalidTx`;`ChainPushError::NotImplemented` 已删除 |
| `chain/sheng_admin/add_backup.rs` | service 改 `add_backup(province, slot, new_pubkey, signer_pair)`,内部生成 nonce + 计算 `blake2_256((ADD_BACKUP_DOMAIN, province, slot, new_pubkey, nonce).encode())` + sr25519 签名 + 裸 SCALE 编码 call_bytes(pallet_idx=10, call_idx=2)+ 调 `submit_immortal_paysno`;handler 从 `state.sheng_signer_cache.any_for_province` 取 signer pair |
| `chain/sheng_admin/remove_backup.rs` | 同上(call_idx=3,domain `b"remove_sheng_admin_backup_v1"`) |
| `chain/sheng_signer/activation.rs` | 同上(call_idx=4,domain `b"activate_sheng_signing_pubkey_v1"`),signer pair 从 `cache.get(province, &admin_pubkey)` |
| `chain/sheng_signer/rotation.rs` | 同上(call_idx=5,domain `b"rotate_sheng_signing_pubkey_v1"`) |
| `chain/sheng_admin/mod.rs` / `chain/sheng_signer/mod.rs` | 顶部 `//!` 注释更新为 phase7 真实推链 |
| `sheng_admins/roster.rs` | 历史 mock(无活跃调用方)注释 + 函数名改 `push_chain_placeholder`,标注被 `chain/sheng_admin/` 取代 |
| `chain/sheng_admin/query.rs` / `handler.rs` | warn 字符串 / 错误文案去 mock 字眼(query.rs 仍是占位实现,留独立卡;chain pull 不在本卡范围) |

### 关键约束已对齐

- pallet_index = 10(SfidSystem,与 `runtime/src/lib.rs` `#[runtime::pallet_index(10)]` 对齐)
- 4 个 call_index = 2/3/4/5
- Slot SCALE 编码:Main=0 / Backup1=1 / Backup2=2(单字节,与链端 `pub enum Slot` 对齐)
- ProvinceBound encoded as `Vec<u8>`(`Compact(len) ++ bytes`)
- ShengNonce = `[u8; 32]`(无长度前缀)
- domain 常量 `[u8; N]` 数组(死规则 `feedback_scale_domain_must_be_array.md`)
- payload 顺序与链端 `Pallet::*_payload` helper 严格一致
- V4 BARE wire 格式(`Compact(len) ++ 0x04 ++ pallet_idx ++ call_idx ++ args`)与
  substrate `Preamble::Bare(LEGACY_EXTRINSIC_FORMAT_VERSION)` 编码等价

### 验证结果

```
cargo check -p sfid-backend  # 全绿(仅 3 条 baseline dead_code warning 在 sfid/province.rs)
cargo test   -p sfid-backend  # 79 passed; 0 failed(baseline 66 → +13 新增)
cargo clippy -p sfid-backend --all-targets  # chain/ 子目录零新增 error/warning(baseline 51 warnings 在 store_shards/main.rs 不变)

grep -rn "submit_immortal_paysno_mock\|chain push mocked\|chain pull mocked\|MockTxHash" sfid/backend/src/  # 0
grep -rn "ChainPushError::NotImplemented" sfid/backend/src/                                                   # 0
```

### 新增测试覆盖

- `chain::client::tests`:`wrap_v4_bare_prepends_compact_len_and_version_byte` / `wrap_v4_bare_handles_longer_payloads`(2 字节 Compact 模式)/ `generate_sheng_nonce_returns_random_32_bytes` / `tx_hash_from_h256_uses_lowercase_0x_hex` / `chain_push_error_display_carries_payload`
- `chain::sheng_admin::add_backup::tests`:payload 签名 + sr25519 自验证 / `Slot::Main` 拒绝路径 / call_bytes 头字节断言 / **完整 V4 BARE wire 字节构造端到端断言**
- `chain::sheng_admin::remove_backup::tests`:payload 签名 + 拒绝 Main 槽
- `chain::sheng_signer::{activation,rotation}::tests`:payload 签名自验证

### 已识别架构卡点(留 Step 2 联调时统一解决)

- **链端 4 个 unsigned extrinsic 验签语义与 SFID 后端可签密钥不匹配**:
  - `add_sheng_admin_backup` / `remove_sheng_admin_backup` 链端要求由本省 `ShengAdmins[Province][Main]` 公钥签发;SFID 后端不持有 admin slot 私钥(admin 私钥仅在冷钱包内)。
  - `activate_sheng_signing_pubkey` / `rotate_sheng_signing_pubkey` 链端要求由 `admin_pubkey` 签发;SFID 后端 `sheng_signer_cache` 持有的是 **signing pair**(bootstrap 派生),公钥即 `signing_pubkey`,与 `admin_pubkey` 是不同密钥。
  - **当前实现**:4 个 helper 用 `sheng_signer_cache` 的 signing pair 作为 stand-in 签名,链端 `Sheng3TierSignatureInvalid` 验签会失败,直至引入冷钱包签名通路或链端 verifier 调整。
- **e2e 测试自动化降级**:phase7 任务卡指引"如启动 docker 链端复杂可降级为 mock 单元测试,真实 e2e 留人工"。本期补了完整 wire 字节构造断言(`add_backup_constructs_correct_unsigned_extrinsic_wire`)+ payload 签名自验证,真实链联调留 Step 2 上线后人工验收。
- **chain pull(`query.rs::fetch_roster`)未切真**:仍走常量 + None 占位。本卡严格只覆盖 4 个 push extrinsic,chain pull 切真留独立任务卡。

### 后续建议任务卡

1. **冷钱包签名通路**(优先级 P0):wumin/wuminapp 增加 4 个 unsigned extrinsic 的 QR 扫码签名两色识别,SFID 后端改为接收前端推链请求体里的 sig 字段(而非自己签)。
2. **chain pull 全量切真**:`chain/sheng_admin/query.rs` + `chain/citizen_binding` 等 4 个 chain pull 业务模块走 subxt `storage().fetch()`。
3. **`sheng_admins/roster.rs` 删除**:无活跃调用方,Phase 7 收尾时一并删除。
