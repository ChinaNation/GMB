# CitizenApp Cloudflare D1 schema 收敛为唯一基线

状态：open（2026-07-31，代码改动完成，待用户执行清空重建）

## 缺陷

CitizenApp 设备绑定失败的最终根因：D1 缺 `cid_data_roots` 表。

`POST /v1/square/identity/takeover/challenge` 返回 **500 `internal_error`**。
返回 500 而非 401 证明 `requireCurrentFinalizedBinding` 已通过——Worker 读链、
CID 绑定校验、Tunnel、`CHAIN_URL` 全部正常（`GET /v1/constitution` 亦返回 200 佐证）。
故障收敛在 D1 写入。

实测线上表清单：`square_login_challenges` 存在，**`cid_data_roots` 不存在**——
`0003_cid_data_root_takeover.sql` 从未被执行。

## 根因：迁移链与重建流程互相矛盾

- `wrangler deploy` **不会**自动应用 D1 migrations。
- 部署脚本步骤 5 只做 `SELECT 1 AS ready` 连通性检查，不建表、不校验完整性。
- 唯一建表入口 `reset_d1()` **只执行 `0001`**，`0002` / `0003` 永远不会被应用。
- 而 `0001` 文件头原本就写明「唯一目标基线，不保留历史迁移链」——后加的
  `0002`、`0003` 破坏了该约定。

结果：代码升级到需要新表，线上库停留在旧 schema，且部署全程无告警。
**每次改 schema 都会重演。**

## 改动

1. `migrations/0001_square_core.sql` → `schema/citizenapp.sql`，并入 `0003` 的
   `cid_data_roots` 建表与索引。
2. 删除 `0002`（仅一句 `DELETE FROM square_contacts;`，对新建空库无意义）与 `0003`。
3. 删除空的 `migrations/` 目录——该词暗示迁移链，与单一基线约定冲突。
4. `wrangler.toml` 删 `migrations_dir`（D1 用）。**`[[migrations]]` 段保留**：
   那是 Durable Objects 的 `new_sqlite_classes`，与 D1 无关。
5. 同步引用点：`cloudflare.sh` 的 `reset_d1()`、`package.json` 的 `db:local` / `db:production`。
6. 护栏：部署时校验关键表完整性（缺表即失败）；测试断言 `schema/` 下有且仅有
   `citizenapp.sql` 且含 `SCHEMA VERSION`。

## 版本约定

文件名固定 `citizenapp.sql`，版本号写在文件内容（`SCHEMA VERSION: v1.0.0`）+ 变更日志。
文件名不带版本，避免每次升级都要同步改脚本、测试与文档路径。

- **开发期（零用户）**：改本文件 + 升版本号 + 追加日志 → 清空重建。不写迁移、不做兼容。
- **上线冻结后**：本文件转只读，变更走 `0002_` 起的增量迁移，并由 `schema_baseline`
  标记自动禁用清空重建。该阶段实现待接近上线时再落地，本卡只写死约定。

## 待用户执行

控制台 →「清空并重建全部数据」（需 Touch ID）。

**该动作 DROP D1 全部表**。开发期零用户，清掉的是空表与过期挑战记录。
媒体在 R2 / Images / Stream、密钥在本机钥匙串，均不在 D1，不受影响。

完成后验证 `cid_data_roots` 存在，再在 App 点「重试」完成设备绑定。
