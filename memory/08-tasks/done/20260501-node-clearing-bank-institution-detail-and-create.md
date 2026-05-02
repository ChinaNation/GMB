# 节点桌面清算行机构详情与创建多签流程

- 日期: 2026-05-01
- 状态: open / in-progress
- 归属: SFID Agent + Blockchain Agent
- 承接: `20260501-sfid-chain-folder-restructure.md`(已完成)

## 触发原因

节点桌面"添加清算行"页 UX 问题:
- "查询"按钮多余(input debounce 已自动搜)
- propose-create / register-sfid 两个 info 终态长说明无法操作
- 当前 detail 页只显示节点声明信息,不显示机构本身的元数据
- 创建机构多签流程缺失(目前只能点跳"去 SFID 后台"无操作能力)

## 设计铁律

1. 链 → SFID 单向 HTTP pull
2. 机构信息只有一种,不区分私权/公权/公安局,统一走 chain/institution_info/
3. 签名钱包必须冷钱包(wumin QR 两段握手)
4. 机构整体创建,不按账户拆(用 propose_create_institution call_index 5,不是 propose_create call_index 0)
5. 不新建目录、不新建 endpoint,只在已有模块内增量

## 范围

### SFID 后端
- `chain/institution_info/handler.rs::app_get_institution` 响应追加 2 字段:`register_nonce` + `signature`
- `chain/runtime_align.rs` 新增 `build_institution_credential_with_province`(用省级签名密钥签)

### 节点 Rust
- `offchain/chain.rs` 加 `fetch_institution_detail` + `fetch_institution_proposals`
- `offchain/sfid.rs` 加 `fetch_institution_credential`
- `offchain/signing.rs` 加 `build_propose_create_institution_*`
- `offchain/commands.rs` 加 4 个 #[tauri::command]
- `offchain/types.rs` 加 InstitutionDetail / AccountWithBalance / InstitutionCredentialResp

### 节点前端
- 新建 institution_detail.tsx / create_multisig.tsx / other_accounts.tsx / admin_list.tsx
- section.tsx 状态机重构(删 register-sfid / propose-create / 老 detail 视图)
- sfid.tsx 删"查询"按钮

## 不做

- wuminapp / wumin 端任何改动(本轮)
- citizenchain runtime 任何改动
- chain/ 下新建目录 / 新建 endpoint
- 现有 SFID 4 个查询 endpoint(institution search / detail / accounts / clearing-banks search)结构改动(只在 detail 响应里追加 2 字段,旧字段全留)

## 验收

- cargo test -p sfid-backend 既有 77/77 + 新增至少 5 通过
- cargo check -p node --tests 0 error
- tsc --noEmit(node frontend) exit 0
- 节点输入 `FFR-AH001-ZG1C-887947508-20260430` 选中候选:
  - 链上不存在 → 进 create-multisig-institution 页,机构名/账户列表正确
  - 全表单填好 + 冷钱包签 → Institutions[sfid_id]=Pending
  - 投票达阈值 → status=Active → 自动跳 declare-node
  - 已存在(Active) → 进 institution-detail,展示主账户/费用账户/其他账户/管理员/提案列表
- 全仓 grep `propose-create.*info|register-sfid.*info|chain/institution_register` 零残留
- SFID 老 4 个查询 endpoint 调用方收到多 2 字段不影响

## 工作量

总计 ~3000 行新增 / ~100 行删除,跨 SFID 后端 + node Rust + node frontend 三处。
