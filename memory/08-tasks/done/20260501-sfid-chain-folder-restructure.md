# SFID 后端 chain/ 目录重构与 dead route 清理

- 日期: 2026-05-01
- 状态: open / in-progress
- 归属: SFID Agent
- 触发原因: 节点桌面"添加清算行"输入 `FFR-AH001-ZG1C-887947508-20260430` 报 "SFID 响应解析失败:error decoding response body"(P0 故障);追溯发现 `chain/` 目录承载语义混乱、含 6 条 dead route、与按业务域整理 chain ↔ SFID 交互能力的预期相悖。

## 架构铁律(本任务确立)

> 链 → SFID 是唯一交互方式(链需要时主动 HTTP pull),SFID 自身不依赖链做主动业务决策。但当前 key-admins / operate-binding push-chain 等历史能力**先不动**,只做"位置归整"——把所有 chain ↔ SFID 交互代码统一收敛到 `sfid/backend/src/chain/` 下。

## 范围

### chain/ 二级目录目标形态(7 业务模块 + 3 共享文件)

```
sfid/backend/src/chain/
├── mod.rs                       # INSTITUTION_DOMAIN 等共享域常量
├── url.rs                       # SFID_CHAIN_WS_URL 入口
├── runtime_align.rs             # SCALE 编码 + genesis_hash 缓存
│
├── institution_info/            # 链/钱包 pull 机构信息(含清算行)
├── joint_vote/                  # 联合投票:人口快照 pull
├── citizen_binding/             # 公民身份绑定(含 admin push-chain 暂留)
├── citizen_vote/                # 公民投票凭证 pull
│
├── key_admins/                  # key-admins 推链(rotate / register / sheng_signing)
├── sheng_admin/                 # 省级管理员链上状态查询(给前端 UI)
└── balance/                     # 链余额查询(给前端 UI)
```

### 删除清单(0 caller dead routes)

1. `POST /api/v1/vote/verify` + `chain/vote.rs` 整文件
2. `GET /api/v1/chain/voters/count` + `chain/voters.rs` 整文件
3. `POST /api/v1/chain/binding/validate` / `chain/reward/ack` / `chain/reward/state` + `chain/binding.rs` 整文件
4. `GET /api/v1/attestor/public-key` + `app_core/http_security.rs::attestor_public_key`
5. `POST /api/v1/app/institutions/:sfid_id/chain-sync` + `institutions/handler.rs::sync_institution_chain_state`
6. `chain/clearing_bank_watcher.rs` 整文件 + `start_watcher` 调用 + AppState 缓存字段(下游 `app_search_clearing_banks` 改为不再过滤"已加入清算网络",由 wuminapp 自己读链 — 第二步任务承接)

### 顺手修复 P0

7. `citizenchain/node/src/offchain/sfid.rs` + `types.rs` 修 SCALE/JSON 契约 mismatch:
   - SFID 返回 `data: Vec<Row>`(非 `data.items`)
   - SFID 返回 snake_case 字段(非 camelCase)
   - `institution_name` 是 Option(skip_serializing_if=is_none)
   - `main_chain_status` 用 SCREAMING_SNAKE_CASE 枚举(NOT_ON_CHAIN/...)
   - 拆 deserialize/serialize 双 DTO,前者 snake_case,后者 camelCase + 友好 enum

## 不做

- 不动 key-admins / operate / sheng-admins 主体业务代码(只搬出 chain RPC 段)
- 不删 subxt 依赖
- 不删 chain/url.rs / chain/runtime_align.rs 读链分支
- 不动 SFID 前端 view(后端能力保留,UI 数据源不变)
- 不动 wuminapp / wumin / citizenchain runtime

## 子任务

- T1. 创建 7 个二级目录骨架
- T2. 搬 institution_info 5 endpoint
- T3. 搬 app_voters_count → joint_vote/
- T4. 搬 app_vote_credential → citizen_vote/
- T5. 搬 operate/binding 推链 + credential → citizen_binding/
- T6. 搬 key-admins 推链段 → key_admins/
- T7. 搬 sheng-admins/catalog 推链段 → sheng_admin/
- T8. 搬 balance.rs → balance/
- T9. 删 6 项 dead routes
- T10. 修 P0(node/src/offchain/sfid.rs+types.rs)
- T11. main.rs 路由表整理
- T12. 重写 CHAIN_TECHNICAL.md / SFID_TECHNICAL.md
- T13. cargo check + 测试 + 全仓 grep 残留扫描

## 验收

- `cargo check -p sfid-backend` 零 error
- `cargo test --workspace` 既有测试不破坏
- 节点桌面添加清算行能搜到 `FFR-AH001-...` 候选(P0 解决)
- `grep "subxt|chain_http_url|chain_ws_url"` 仅出现在 `chain/` 子目录内
- `grep "verify_vote_eligibility|chain_voters_count|chain_binding_validate|chain_reward|attestor_public_key|sync_institution_chain_state|clearing_bank_watcher"` 全 SFID 后端零残留
- `CHAIN_TECHNICAL.md` 按新 7 模块结构重写
- 前端 sheng-admins 视图链上状态列 + keyring 链余额行行为不变
