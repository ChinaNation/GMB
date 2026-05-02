# SFID 后端 `chain/` 技术说明

- 最后更新:2026-05-02
- 任务卡:
  - `memory/08-tasks/done/20260501-sfid-chain-folder-restructure.md`
  - `memory/08-tasks/done/20260502-sfid-duoqian-info-layout.md`
  - `memory/08-tasks/open/20260502-114447-按业务边界重新设计并落地-sfid-省管理员相关前后端与-runtime-目录结构.md`

## 0. 架构铁律

> **常规链 → SFID 单向 HTTP pull**:链需要 SFID 数据时主动调本目录端点;
> **SFID 自身独立维护数据**,不读链不依赖链做业务决策。
> **第 1 步机构备案上链是明确例外**:SFID 市管理员使用省管理员签名密钥主动推送最小备案 payload,只写链上备案记录,不完成链上正式多签机构注册。

历史上 chain ↔ SFID 双向交互、轮询、attestor HMAC 鉴权等多种模式并存,
本次重构(2026-05-01)统一收敛为单向 pull,并把所有"和链交互"的代码集中
到 `sfid/backend/src/chain/` 目录。2026-05-02 起,机构信息交互目录统一命名为
`duoqian_info/`,不再使用 `institution_info/`。省管理员相关链交互统一命名为
`sheng_admins/`,与前端 `sfid/frontend/chain/sheng_admins/`、runtime
`sfid-system/src/sheng_admins/` 对齐。

### 0.1 机构备案推链例外(2026-05-02)

第 1 步"机构备案上链"只解决 SFID 机构信息留痕,不创建链上多签机构。市管理员备案时推送到区块链的信息只包含:

```text
sfid_id
institution_name
account_name
```

签名与边界:

- 市管理员是 SFID 系统操作人。
- 省管理员签名密钥是链上业务授权签名。
- 照片、章程、许可证、股东会决议、法人授权书等材料不上链,只留在 SFID 系统内部。
- 备案记录不得写入 `DuoqianManage::Institutions`,不得激活机构账户。
- 第 2 步链上正式机构注册仍由 wuminapp 或节点软件发起多签注册流程承接。

## 1. 目录结构

```
sfid/backend/src/chain/
├── mod.rs                  # 子模块声明
├── url.rs                  # SFID_CHAIN_WS_URL 入口(给过渡态推链 helper 用)
├── runtime_align.rs        # SCALE 编码 + 域常量 + genesis_hash 缓存 + 凭证签发
│
├── citizen_binding/        # 公民身份绑定(过渡:含 admin push extrinsic)
├── citizen_vote/           # 公民投票凭证签发
├── duoqian_info/           # SFID 与 DUOQIAN 链之间的机构信息交互(含清算行)
├── joint_vote/             # 联合投票:获取公民人数快照凭证
└── sheng_admins/           # 省管理员三槽名册、签名公钥、待签缓存等链交互
```

## 2. 当前在用端点(链端真实消费)

| 端点 | 模块 | 调用方 | 用途 |
|---|---|---|---|
| `GET /api/v1/app/voters/count` | `joint_vote/` | citizenchain/node | 创建联合投票时拉人口快照 |
| `POST /api/v1/app/vote/credential` | `citizen_vote/` | wuminapp | 公民投票时拉凭证再上链 |
| `GET /api/v1/app/institutions/search` | `duoqian_info/` | wuminapp / 节点桌面 | 通用机构搜索 |
| `GET /api/v1/app/institutions/:sfid_id` ★ | `duoqian_info/` | 同上 + 节点桌面"创建多签机构" | 单机构详情 + chain pull 凭证(register_nonce + signature)|
| `GET /api/v1/app/institutions/:sfid_id/accounts` | `duoqian_info/` | 同上 | 机构账户列表(脱敏) |
| `GET /api/v1/app/clearing-banks/search` | `duoqian_info/` | wuminapp | 已激活清算行(分页) |
| `GET /api/v1/app/clearing-banks/eligible-search` | `duoqian_info/` | 节点桌面 | 候选清算行(可未激活) |
| `GET /api/v1/admin/sheng-admin/roster` | `sheng_admins/` | SFID 省管理员后台 | 拉本省三槽名册 |
| `POST /api/v1/admin/sheng-admin/roster/add-backup` | `sheng_admins/` | 同上 | 推链添加 backup 槽 |
| `POST /api/v1/admin/sheng-admin/roster/remove-backup` | `sheng_admins/` | 同上 | 推链移除 backup 槽 |
| `POST /api/v1/admin/sheng-signer/activate` | `sheng_admins/` | 同上 | 激活当前槽签名公钥 |
| `POST /api/v1/admin/sheng-signer/rotate` | `sheng_admins/` | 同上 | 轮换当前槽签名公钥 |
| `GET /api/v1/chain/sheng-admin/list` | `sheng_admins/` | 链/节点反向调用 | 公开拉取省管理员三槽名册 |

### 2.1 `app_get_institution` 凭证响应增强(2026-05-01)

`GET /api/v1/app/institutions/:sfid_id` 响应在既有 `MultisigInstitution` 字段
之外**追加** 2 个签名字段:

| 字段 | 用途 |
|---|---|
| `register_nonce` | 防重放 nonce(本次响应生成的 32 字节随机 hex)。链端 `UsedRegisterNonce[hash(nonce)]` 标记已用,同凭证不可重放 |
| `signature` | **省级签名密钥**对凭证 payload 的 sr25519 签名(64 字节 hex)。链端 `propose_create_institution` 用 `signing_province` 反查 `ShengSigningPubkey[province]` 验签 |

`signing_province` 字段**不下发**——节点桌面发起 extrinsic 时直接用响应里的
`province` 字段塞 `signing_province` 入参(永远等同),避免冗余。

旧调用方(钱包等仅展示场景)收到多 2 个字段忽略即可,展示路径无变化。

实现位置:`sfid/backend/src/chain/duoqian_info/handler.rs::app_get_institution`
+ `sfid/backend/src/chain/runtime_align.rs::build_institution_credential_with_province`。

## 3. 凭证签发签名口径(冻结)

`runtime_align.rs` 维护 4 套凭证签发函数,全部用 SFID main 私钥签。这里的机构凭证服务于第 2 步链上正式机构注册,不是第 1 步机构备案 payload:

```rust
build_bind_credential(state, account_pubkey, binding_seed, bind_nonce)
build_vote_credential(state, account_pubkey, binding_seed, proposal_id, vote_nonce)
build_population_snapshot_credential(state, account_pubkey, eligible_total, snapshot_nonce)
build_institution_credential(state, sfid_id, name, register_nonce)
```

签名 payload:`blake2_256(scale_encode(DUOQIAN_DOMAIN ++ OP_SIGN_<TAG> ++ payload))`
- `DUOQIAN_DOMAIN`:`b"DUOQIAN_V1"`(10 字节)
- `OP_SIGN_BIND` = 0x10 / `OP_SIGN_VOTE` = 0x11 / `OP_SIGN_POP` = 0x12 / `OP_SIGN_INST` = 0x13

字节布局必须与 `citizenchain/runtime/src/configs/mod.rs` 的链端 verifier 严格对齐;
任一字段顺序变更都需要双端同步,否则 `sr25519_verify` 必败。

链上 verifier 锚点:
- 绑定 payload:`citizenchain/runtime/src/configs/mod.rs::RuntimeSfidVerifier::verify`
- 投票 payload:`citizenchain/otherpallet/sfid-system/src/lib.rs::verify_and_consume_vote_credential`
- 人口快照 payload:`citizenchain/runtime/src/configs/mod.rs::verify_population_snapshot`
- 机构注册 payload:`citizenchain/runtime/src/configs/mod.rs::verify_institution_registration`

## 4. 省管理员链交互目录(2026-05-02)

`chain/sheng_admins/` 是省管理员功能与区块链交互的唯一后端目录,不再拆成
`sheng_admin/` 与 `sheng_signer/` 两套入口。

```text
sheng_admins/
├── mod.rs              # 模块边界说明与子模块声明
├── query.rs            # ShengAdmins[Province][Slot] 三槽名册 pull
├── handler.rs          # roster HTTP handler 与公开 pull endpoint
├── add_backup.rs       # add_sheng_admin_backup extrinsic
├── remove_backup.rs    # remove_sheng_admin_backup extrinsic
├── activate_signer.rs  # activate_sheng_signing_pubkey extrinsic
├── rotate_signer.rs    # rotate_sheng_signing_pubkey extrinsic
└── pending_signs.rs    # 冷钱包 prepare/submit-sig nonce 暂存
```

非链上的省管理员本地业务仍放在 `sfid/backend/src/sheng_admins/`。其中
`province_admins.rs` 承载 43 省 main 公钥和 Slot/ProvinceAdmins 模型,
`sfid/backend/src/sfid/province.rs` 只保留 SFID 号码生成需要的省市代码。

## 5. 过渡态:SFID 推 extrinsic / 主动读链

下面 3 处仍含"SFID 主动连链"代码,与铁律不符,但本轮先不动业务流程:

| 模块 | 触发场景 | 链交互动作 |
|---|---|---|
| `chain/citizen_binding/push.rs` | admin 在 SFID 后台点"推链绑定" | 提交 `bind_sfid` / `unbind_sfid` extrinsic |
| `chain/sheng_admins/add_backup.rs` | 省管理员 main 添加 backup | 提交 `add_sheng_admin_backup` extrinsic |
| `chain/sheng_admins/remove_backup.rs` | 省管理员 main 移除 backup | 提交 `remove_sheng_admin_backup` extrinsic |
| `chain/sheng_admins/activate_signer.rs` | 当前槽激活签名公钥 | 提交 `activate_sheng_signing_pubkey` extrinsic |
| `chain/sheng_admins/rotate_signer.rs` | 当前槽轮换签名公钥 | 提交 `rotate_sheng_signing_pubkey` extrinsic |

后续配套链端"chain pull 凭证→外部代提"流程就绪后,本节内容应整体下架。

## 6. 删除清单(2026-05-01 一次性下架)

| 项 | 原因 |
|---|---|
| `chain/vote.rs::verify_vote_eligibility` | 0 caller dead route |
| `chain/voters.rs::chain_voters_count` | 与 `app/voters/count` 重复 |
| `chain/binding.rs::chain_binding_validate / chain_reward_*` | 0 caller dead routes |
| `chain/clearing_bank_watcher.rs` | SFID 不再读链,改由 wuminapp 自己读 ClearingBankNodes 过滤 |
| `chain/app_api.rs` | 拆到 `joint_vote/` + `citizen_vote/` |
| `app_core/http_security.rs::attestor_public_key` | 0 caller |
| `institutions/handler.rs::sync_institution_chain_state` | 0 caller |
| `app_core/http_security.rs` 中 chain HMAC 鉴权全套 | 与 dead routes 配套 |
| `main.rs::prepare_chain_request` / `ensure_chain_request_db` | 同上 |
| `models/mod.rs` 中 `ChainRequestAuth / VoteVerifyInput / ChainVotersCount* / ChainBindingValidate* / RewardAck* / RewardState*` | 全部 dead 数据结构 |
| `key-admins/chain_proof.rs` | 仅服务于已下架的 `attestor/public-key` |
| `chain/sheng_admin/` / `chain/sheng_signer/` / `chain/sheng_pending/` | 2026-05-02 合并为 `chain/sheng_admins/` |

总计:删除约 1300 行 + 移动整合约 1500 行;迁移结束后 `cargo test --bin sfid-backend` 77/77 通过。

## 7. 节点客户端反序列化契约(P0 修复点)

故障:节点桌面"添加清算行"输入完整 sfid_id 后报"SFID 响应解析失败:error decoding response body"。

根因:节点端 `EligibleSearchEnvelope { data: { items } }` 套了一层 `items` 信封,
而 SFID 端 `ApiResponse.data` 直接是 Vec;字段还混了 camelCase / snake_case;
`institution_name` 在两步式未命名时缺字段;`main_chain_status` 是 SCREAMING_SNAKE_CASE。

修复:`citizenchain/node/src/offchain/duoqian_manage/sfid.rs` 拆双 DTO:
- `SfidEligibleRow`(snake_case + Option + SfidMultisigChainStatus 枚举)→ deserialize SFID 响应
- `EligibleClearingBankCandidate`(camelCase + 友好状态字符串)→ serialize 给 Tauri 前端

字段映射表(冻结):

| SFID 端字段(snake_case) | 节点 → TS 字段(camelCase) | 备注 |
|---|---|---|
| `sfid_id` | `sfidId` | |
| `institution_name`(可缺失) | `institutionName`(缺失填空串) | 两步式未命名 |
| `a3` | `a3` | |
| `sub_type`(可缺失) | `subType` | |
| `parent_sfid_id`(可缺失) | `parentSfidId` | FFR 候选才有 |
| `parent_institution_name`(可缺失) | `parentInstitutionName` | |
| `parent_a3`(可缺失) | `parentA3` | |
| `province` / `city` | 同名 | |
| `main_account`(可缺失) | `mainAccount` | hex |
| `fee_account`(可缺失) | `feeAccount` | hex |
| `main_chain_status` enum | `mainChainStatus` 字符串 | NOT_ON_CHAIN→Inactive / PENDING→Pending / ACTIVE→Registered / REVOKED→Failed |

任一字段或映射改动必须同时更新两端 + 本表。

## 8. 验收

- `cargo check -p sfid-backend` 零 error,3 个无关 warning(province.rs 字段读未读)
- `cargo test --bin sfid-backend` 77/77 通过
- `cargo check -p node --tests`(citizenchain)零 error
- 节点桌面输入 `FFR-AH001-ZG1C-887947508-20260430` 能搜到候选(P0 解决)
- 全仓 grep 残留扫描:`subxt|chain_http_url|chain_ws_url` 仅在 `chain/` 子目录出现
- `verify_vote_eligibility / chain_voters_count / chain_binding_validate / chain_reward / attestor_public_key / sync_institution_chain_state / clearing_bank_watcher` 全 SFID 后端零残留(只剩 ↑ 历史注释)


## ADR-008 Phase 23e 更新（2026-05-01）

全局密钥管理员角色已废止；省管理员 3-tier 自治（main / backup_1 / backup_2）。
旧全局密钥环、轮换挑战、旧权限 helper 和旧目录名仅作为历史迁移背景保留；
实际行为以 `memory/04-decisions/ADR-008-sheng-admin-3tier-and-key-admin-removal.md` 与代码为准。
