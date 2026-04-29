# SFID 清算行资格白名单（Step 1）

> 配套：[ADR-007 清算行三阶段拆分](../../04-decisions/ADR-007-clearing-bank-three-phase.md)

## 资格定义

机构 `inst` 是否有"清算行候选"资格，由以下规则判定：

```
is_clearing_bank_eligible(inst, parent_lookup) =
  match inst.a3:
    "SFR" → inst.sub_type == "JOINT_STOCK"
    "FFR" → 存在 parent = parent_lookup(inst.parent_sfid_id)
            ∧ parent.a3 == "SFR"
            ∧ parent.sub_type == "JOINT_STOCK"
    其他 → false
```

**口径**：
- "私法人股份公司" = `a3=SFR ∧ sub_type=JOINT_STOCK`
- "私法人股份公司所属的非法人" = `a3=FFR ∧ parent_sfid_id 指向 a3=SFR 且 sub_type=JOINT_STOCK 的机构`
- 其他类型一律不允许：GFR 公权机构、SF 公安局、SFR-LIMITED_LIABILITY/PARTNERSHIP/SOLE_PROPRIETORSHIP/NON_PROFIT、FFR 但 parent 不合规、FFR 无 parent_sfid_id 等

## 6 种 case 自检表

| # | a3 | sub_type | parent | 结果 |
|---|---|---|---|---|
| 1 | SFR | JOINT_STOCK | — | ✅ |
| 2 | SFR | LIMITED_LIABILITY | — | ❌ |
| 3 | SFR | NON_PROFIT | — | ❌ |
| 4 | FFR | (任意) | parent.SFR + parent.JOINT_STOCK | ✅ |
| 5 | FFR | (任意) | parent.SFR + parent.LIMITED_LIABILITY | ❌ |
| 6 | FFR | (任意) | parent_sfid_id 缺失 / parent 不存在 | ❌ |

GFR / SF 等其他 a3 一律 ❌。

## 后端实现位置

| 资产 | 文件 |
|---|---|
| 资格判定函数 | [sfid/backend/src/institutions/service.rs](../../../sfid/backend/src/institutions/service.rs) `is_clearing_bank_eligible` |
| 已激活清算行搜索 | [sfid/backend/src/institutions/handler.rs](../../../sfid/backend/src/institutions/handler.rs) `app_search_clearing_banks`（收紧版） |
| 候选清算行搜索（含未激活） | [sfid/backend/src/institutions/handler.rs](../../../sfid/backend/src/institutions/handler.rs) `app_search_eligible_clearing_banks` |
| 路由 | [sfid/backend/src/main.rs](../../../sfid/backend/src/main.rs) `/api/v1/app/clearing-banks/search` 与 `/api/v1/app/clearing-banks/eligible-search` |

## 跨省 parent 查询机制

由于 SFID 数据按省分片（`sharded_store.read_province(prov, |shard| ...)`)，FFR 候选的 parent 可能在另一省 shard。实现采用 **2 轮跨省读取**：

1. 第 1 轮（跨 43 省）：收集所有 `SFR ∧ JOINT_STOCK` 的 sfid_id（写入 `HashSet<String>` 全国快查表）
2. 第 2 轮（跨 43 省）：收集本省候选机构（SFR-JOINT_STOCK 直接通过 / FFR 查 parent_sfid_id 是否在第 1 轮的 HashSet 里）

跨省读取通过 sharded_store 已有的 `read_province` 并发能力，单次 search 总耗时 ≈ 2 × 43 × P95(单省读)。

## 前端实现位置

| 资产 | 文件 |
|---|---|
| 前端版资格判定 | [sfid/frontend/src/utils/clearingBankEligible.ts](../../../sfid/frontend/src/utils/clearingBankEligible.ts) |
| 列表 badge | [sfid/frontend/src/views/institutions/InstitutionListTable.tsx](../../../sfid/frontend/src/views/institutions/InstitutionListTable.tsx) |
| 详情 badge | [sfid/frontend/src/views/institutions/InstitutionDetailPage.tsx](../../../sfid/frontend/src/views/institutions/InstitutionDetailPage.tsx) |
| 第二步选 sub_type 时提示 | [sfid/frontend/src/views/institutions/PrivateInstitutionLayout.tsx](../../../sfid/frontend/src/views/institutions/PrivateInstitutionLayout.tsx) |

**前端 badge 显示规则**：
- SFR + JOINT_STOCK → 蓝色 badge `可作为清算行`（直接判定，单条机构信息足够）
- FFR + parent_sfid_id 已设置 → 详情页显示"待校验所属法人股份公司"提示（详情页可查 parent 后判定）；列表页**不显示** badge（避免误判和额外查询）
- 其他 → 不显示

## API 契约

### `GET /api/v1/app/clearing-banks/search`（已有，本次收紧）

返回**已加入清算网络候选**：资格白名单 ∩ 主账户 `ACTIVE_ON_CHAIN`。

响应字段（在原 7 字段基础上扩展）：
```json
{
  "sfid_id": "...",
  "institution_name": "...",
  "a3": "SFR" | "FFR",
  "sub_type": "JOINT_STOCK" | null,
  "parent_sfid_id": "..." | null,
  "parent_institution_name": "..." | null,
  "parent_a3": "SFR" | null,
  "province": "...",
  "city": "...",
  "main_account": "...",
  "fee_account": "..."
}
```

### `GET /api/v1/app/clearing-banks/eligible-search`（本次新增）

返回**资格候选**：仅用资格白名单过滤，不要求主账户已经 `ACTIVE_ON_CHAIN`（NodeUI"添加清算行"用，因为可能正在创建中）。

响应在 `app_search_clearing_banks` 基础上增加：
```json
{
  "main_chain_status": "NOT_ON_CHAIN" | "PENDING_ON_CHAIN" | "ACTIVE_ON_CHAIN" | "REVOKED_ON_CHAIN",
  "main_account": "..." | null,
  "fee_account": "..." | null
}
```

参数：仅 `q`（关键字模糊匹配 sfid_id 或机构名）+ `limit`（最大 50，默认 20）。**无 province/city 过滤**（sfid_id 全局唯一，精确定位）。

## NodeUI 调用地址规则

NodeUI 的"添加清算行"页通过 `citizenchain/node/src/offchain/sfid.rs`
转发调用 `/api/v1/app/clearing-banks/eligible-search`。

SFID 基地址由 `citizenchain/node/src/sfid_config.rs` 统一决定：

- `SFID_BASE_URL` 环境变量优先
- 本地 debug 构建默认访问 `http://127.0.0.1:8899`
- 正式 release 构建默认访问 `http://147.224.14.117:8899`

本地局域网 IP 只用于手机或其他设备联调，NodeUI 本机开发不依赖局域网 IP，
避免 Wi-Fi 地址变化导致清算行搜索请求失败。

## 不在 Step 1 范围

- 链上 ClearingBankNodes storage / register_clearing_bank extrinsic → Step 2
- bank_check::ensure_can_be_bound 收紧 → Step 2
- NodeUI 清算行 tab → Step 2
- wumin decoder / wuminapp UI → Step 3
