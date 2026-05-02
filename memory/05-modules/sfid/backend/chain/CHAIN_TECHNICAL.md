# SFID 后端 `chain/` 技术说明

- 最后更新:2026-05-01
- 任务卡:`memory/08-tasks/open/20260501-sfid-chain-folder-restructure.md`

## 0. 架构铁律

> **链 → SFID 单向 HTTP pull**:链需要 SFID 数据时主动调本目录端点;
> **SFID 自身独立维护数据**,不读链不推链(过渡期 3 处保留,见 §4)。

历史上 chain ↔ SFID 双向交互、轮询、attestor HMAC 鉴权等多种模式并存,
本次重构(2026-05-01)统一收敛为单向 pull,并把所有"和链交互"的代码集中
到 `sfid/backend/src/chain/` 目录,7 个二级目录按功能划分。

## 1. 目录结构

```
sfid/backend/src/chain/
├── mod.rs                  # 子模块声明
├── url.rs                  # SFID_CHAIN_WS_URL 入口(给过渡态推链 helper 用)
├── runtime_align.rs        # SCALE 编码 + 域常量 + genesis_hash 缓存 + 凭证签发
│
├── institution_info/       # 链/钱包 pull 机构信息(含清算行)
├── joint_vote/             # 联合投票:获取公民人数快照凭证
├── citizen_binding/        # 公民身份绑定(过渡:含 admin push extrinsic)
├── citizen_vote/           # 公民投票凭证签发
├── key_admins/             # key-admins 推链(过渡:rotate / sheng_signing / state 查)
├── sheng_admin/            # 省级管理员链上 signing pubkey 清理(过渡)
└── balance/                # admin 后台主账户链上余额展示
```

## 2. 当前在用端点(链端真实消费)

| 端点 | 模块 | 调用方 | 用途 |
|---|---|---|---|
| `GET /api/v1/app/voters/count` | `joint_vote/` | citizenchain/node | 创建联合投票时拉人口快照 |
| `POST /api/v1/app/vote/credential` | `citizen_vote/` | wuminapp | 公民投票时拉凭证再上链 |
| `GET /api/v1/app/institutions/search` | `institution_info/` | wuminapp / 节点桌面 | 通用机构搜索 |
| `GET /api/v1/app/institutions/:sfid_id` ★ | `institution_info/` | 同上 + 节点桌面"创建多签机构" | 单机构详情 + chain pull 凭证(register_nonce + signature)|
| `GET /api/v1/app/institutions/:sfid_id/accounts` | `institution_info/` | 同上 | 机构账户列表(脱敏) |
| `GET /api/v1/app/clearing-banks/search` | `institution_info/` | wuminapp | 已激活清算行(分页) |
| `GET /api/v1/app/clearing-banks/eligible-search` | `institution_info/` | 节点桌面 | 候选清算行(可未激活) |
| `GET /api/v1/admin/chain/balance` | `balance/` | SFID admin keyring 视图 | 主账户链上余额展示 |

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

实现位置:[chain/institution_info/handler.rs:`app_get_institution`](sfid/backend/src/chain/institution_info/handler.rs)
+ [chain/runtime_align.rs:`build_institution_credential_with_province`](sfid/backend/src/chain/runtime_align.rs)。

## 3. 凭证签发签名口径(冻结)

`runtime_align.rs` 维护 4 套凭证签发函数,全部用 SFID main 私钥签:

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

## 4. 过渡态:SFID 推 extrinsic / 主动读链

下面 3 处仍含"SFID 主动连链"代码,与铁律不符,但本轮先不动业务流程:

| 模块 | 触发场景 | 链交互动作 |
|---|---|---|
| `chain/citizen_binding/push.rs` | admin 在 SFID 后台点"推链绑定" | 提交 `bind_sfid` / `unbind_sfid` extrinsic |
| `chain/key_admins/rotate.rs` | KEY_ADMIN 走主备账户轮换 commit | 提交 `rotate_sfid_keys` extrinsic |
| `chain/key_admins/sheng_signing.rs` | 省登录管理员首次登录 / KEY_ADMIN 替换省管理员 | 提交 `set_sheng_signing_pubkey` extrinsic |
| `chain/sheng_admin/clear_sheng_signing.rs` | 同上(替换时清旧 pubkey) | 同上 |
| `chain/key_admins/chain_keyring_query.rs` | 启动期同步 SFID 主备账户 | 读链 `state_getStorage` |
| `chain/key_admins/state_query.rs` | balance / 其他模块共用 RPC helper | 读链 |
| `chain/balance/handler.rs` | admin keyring 视图 | 读链 `System::Account` |

后续配套链端"chain pull 凭证→外部代提"流程就绪后,本节内容应整体下架。

## 5. 删除清单(2026-05-01 一次性下架)

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

总计:删除约 1300 行 + 移动整合约 1500 行;迁移结束后 `cargo test --bin sfid-backend` 77/77 通过。

## 6. 节点客户端反序列化契约(P0 修复点)

故障:节点桌面"添加清算行"输入完整 sfid_id 后报"SFID 响应解析失败:error decoding response body"。

根因:节点端 `EligibleSearchEnvelope { data: { items } }` 套了一层 `items` 信封,
而 SFID 端 `ApiResponse.data` 直接是 Vec;字段还混了 camelCase / snake_case;
`institution_name` 在两步式未命名时缺字段;`main_chain_status` 是 SCREAMING_SNAKE_CASE。

修复:`citizenchain/node/src/offchain/sfid.rs` 拆双 DTO:
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

## 7. 验收

- `cargo check -p sfid-backend` 零 error,3 个无关 warning(province.rs 字段读未读)
- `cargo test --bin sfid-backend` 77/77 通过
- `cargo check -p node --tests`(citizenchain)零 error
- 节点桌面输入 `FFR-AH001-ZG1C-887947508-20260430` 能搜到候选(P0 解决)
- 全仓 grep 残留扫描:`subxt|chain_http_url|chain_ws_url` 仅在 `chain/` 子目录出现
- `verify_vote_eligibility / chain_voters_count / chain_binding_validate / chain_reward / attestor_public_key / sync_institution_chain_state / clearing_bank_watcher` 全 SFID 后端零残留(只剩 ↑ 历史注释)
