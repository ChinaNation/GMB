# CPMS 站点与机构关联的元组匹配铁律

## 背景

sfid 后端存在两个看起来都叫 `sfid_id` 的东西,**但它们不是同一个值**:

1. **`multisig_institutions.sfid_id`** — 任务卡 2 两层机构模型的主键,由
   `generate_sfid_code` 按 `(a3, p1, province, city, institution, account_pubkey)`
   派生,account_pubkey 来自创建机构时指定的多签地址种子。

2. **`cpms_site_keys.site_sfid`** — 生成 QR1 CPMS 安装二维码时,由
   `generate_cpms_institution_sfid_qr` 使用 `Uuid::new_v4()` 作为
   account_pubkey 派生。每次生成都是新值,与机构主键无关。

## 关联方式

**只能通过 `(admin_province, city_name, institution_code)` 三元组匹配**。

公安局 category (`ZF + institution_name == "公民安全局"`) 下每个市唯一一个机构,
所以三元组足以保证一一对应。其他 category 理论上可能冲突,但目前 CPMS 功能
只服务公安局,无歧义风险。

## 涉及接口

- `GET /api/v1/admin/cpms-keys/by-institution/:sfid_id`
  传入 `multisig_institutions.sfid_id`,后端读机构拿三元组,扫描
  `cpms_site_keys.values()` 返回首条匹配记录。

- `cleanup_orphan_cpms_sites` 启动钩子
  构建 `multisig_institutions` 全部三元组集合,硬删
  `cpms_site_keys` 中三元组不在集合的孤儿站点。

## 铁律

- **永远不要把 `multisig_institutions.sfid_id` 当作 `site_sfid` 去查
  `cpms_site_keys`**,会查不到。
- **永远不要反向**:把 `site_sfid` 当作机构主键去查 `multisig_institutions`。
- 前端 `CpmsSitePanel` Props 接收机构 sfid_id 命名为 `institutionSfidId`,
  与 `site_sfid` 区分明确,避免混淆。
- 如果未来引入"CPMS 站点挂到非公安局机构"的需求,必须给 `cpms_site_keys`
  加一个显式的 `institution_sfid_id: Option<String>` 字段作为外键,
  不再依赖元组匹配。

## 参考

- 任务卡 `20260408-sfid-public-security-cpms-embed`
- `sfid/backend/src/sheng-admins/institutions.rs::get_cpms_site_by_institution`
- `sfid/backend/src/app_core/runtime_ops.rs::cleanup_orphan_cpms_sites`
