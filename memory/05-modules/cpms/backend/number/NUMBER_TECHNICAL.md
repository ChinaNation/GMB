# CPMS Number 模块技术文档

## 1. 模块定位

`cpms/backend/src/number/` 是 CPMS 编号工具模块，负责在创建公民档案时同步生成一对编号：

- `archive_no`：档案号，供 SFID 绑定和 ARCHIVE 载荷使用。
- `passport_no`：护照号，印刷在公民护照上。

档案号和护照号创建时一对一绑定；档案软删除时两者跟随同一档案进入删除状态。
从软删除时间起满 100 年后，CPMS 才允许硬删除实名档案资料，并将这一对档案号与护照号放入回收池。

## 2. 档案号规则

- 格式：`<26位Base32>-<2位Base32校验>`。
- 明文不包含省、市、CPMS 机构号、日期。
- 生成输入包含 `install_secret`、本机序列、安全随机数、终端 ID、管理员公钥。
- 本机 `archives.archive_no` 唯一约束拒绝重复；SFID 绑定时继续以 `archive_no` 做全局唯一验收。

## 3. 护照号规则

- 格式：`<2位省代码><8位Crockford Base32主体><1位Crockford Base32校验>`。
- 总长度：11 位。
- 示例：`GD7K3M9X2QH`。
- 省代码来自 CPMS 安装码内 `sfid_number` 的 R5 段前两位，必须是 SFID 省级代码表中的两位省代码。
- 8 位主体容量为 `32^8 = 1,099,511,627,776`。
- 主体按 512 个城市隔离空间分割，每市容量为 `2,147,483,648`。
- 原始三位市代码不直接出现在护照号中；`number` 模块将市代码映射为 `0..511` 的城市隔离编号，再与本市护照序列组合。

生成公式：

```text
city_namespace = permutation_512(province_code, city_code)
scrambled_seq = permutation_2^31(province_code, city_code, local_passport_seq)
body_number = scrambled_seq * 512 + city_namespace
passport_no = province_code + crockford_base32_8(body_number) + check
```

## 4. 数据边界

- 前端不得提交护照号。
- 操作管理员不得手工指定护照号。
- `archives.passport_no` 必须非空唯一。
- `sequence_counters` 保存本机档案号序列和本市护照号序列。
- `archive_number_recycle_pool` 保存满 100 年硬删除后释放的一对档案号和护照号；唯一约束只作用于未使用池项，避免同一号码多轮复用后再次硬删除被历史池项挡住。
- CPMS 永不联网，省内不同市的护照号不靠联网查重，而靠城市隔离编号保证生成空间不重叠。

## 5. 回收规则

- 软删除就是注销，但软删除未满 100 年时，`archives` 行仍保留，档案号和护照号继续被唯一约束占用。
- 满 100 年硬删除后，`dangan/lifecycle.rs` 将档案号和护照号成对写入 `archive_number_recycle_pool`。
- 新建档案时，`number` 模块优先在同一事务内领取一对未使用的回收号码；没有可用回收号码时再生成新号码。
- 回收池领取会写入 `used_at / used_by_archive_id`；事务回滚时领取状态同步回滚，号码仍保持可用。
- 档案号和护照号必须成对领取，不允许只回收或只复用其中一个。
- `archive_id` 是内部随机 ID，不进入回收池，不复用。

## 6. 模块边界

- `number` 只负责编号生成和校验字符计算。
- `dangan` 负责 ARCHIVE 载荷、签名、`geo_seal`、有效期规则，以及档案生命周期硬删除。
- `operator_admin` 负责创建档案时调用 `number` 并写入数据库。
- `initialize` 负责提供安装码解析得到的省市代码和 `install_secret`。

## 7. 测试覆盖

- 普通 `cargo test` 覆盖护照号格式、省市隔离编号唯一性和本市最大序列边界。
- 设置 `CPMS_TEST_DATABASE_URL` 后，数据库集成测试会套用当前 `db/schema.sql`，覆盖回收池号码成对领取、领取后写入 `used_by_archive_id`、事务回滚后号码仍可用，以及空回收池时生成新号码。
- `cargo clippy --all-targets -- -D warnings` 必须通过，避免测试代码引入警告。
