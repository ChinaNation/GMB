# 任务卡：链上中国地址库精简为 3+4 完整地址模型

状态：已完成

## 目标

将 `china.sqlite` 镇以下旧 `address_units` 数据精简为单表 `addresses`：

- `address_name_code`：同一镇下地址名称编号，三位数字 `001-999`
- `address_name`：现有村、路、社区、小区等镇下名称
- `address_local_no`：地址名称下精确地址号，四位数字，可为空
- `address_detail`：完整地址细段，可为空

完整地址展示为：

```text
省 + 市 + 镇 + address_name + address_local_no + address_detail
```

## 范围

- 保留省、市、镇表
- 迁移旧 `address_units.name` 到 `addresses.address_name`
- 清除旧字段 `address_unit_id`、`raw_name`、`source_code`
- 清除旧墓碑、变更日志、版本表
- 更新只读模型注释和校验脚本
- 更新 OnChina 数据安全文档

## 不做

- 不修改 runtime
- 不新增链上模块
- 不做链上同步
- 不保留旧地址历史或墓碑

## 验收

- `china.sqlite` 不再包含旧地址表和旧字段
- `addresses` 行数等于迁移前镇下地址名称行数
- 同镇下 `address_name_code` 唯一
- 同镇下 `address_name` 唯一
- `address_local_no` 与 `address_detail` 允许为空
- `check_code_immutable.py` 通过

## 执行记录

- 已将旧镇下地址名称迁入 `addresses`
- 已删除旧地址字段、墓碑表、变更日志表和旧镇名清理脚本
- 已更新校验脚本,禁止旧地址结构残留

## 验收记录

- `python3 citizenchain/onchina/src/cid/china/check_code_immutable.py` 通过
- `sqlite3 citizenchain/onchina/src/cid/china/china.sqlite "PRAGMA integrity_check"` 返回 `ok`
- `cargo check -p onchina` 通过
- `addresses` 行数：598654
- 单镇最大 `address_name_code` 数：170
