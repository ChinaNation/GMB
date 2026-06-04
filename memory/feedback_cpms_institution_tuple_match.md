# CPMS 授权与公安局机构关联铁律

## 背景

SFID 侧 CPMS 两码方案已经改为直接使用公安局机构 `sfid_number` 作为对外机构标识。

一个市公安局机构对应一个 CPMS 安装授权:

- `multisig_institutions.sfid_number` 是公安局机构主键。
- CPMS 授权记录内部保存的机构号内容必须等于对外 `sfid_number`。
- 对外 API、协议和前端类型统一使用 `sfid_number`。

## 关联方式

`GET /api/v1/admin/cpms-keys/by-institution/:sfid_number` 传入公安局机构
`sfid_number`,后端读取该机构所在省分片,再用公安局机构的省、市和机构代码匹配
对应 CPMS 授权记录。

该匹配只用于机构详情页反查授权记录;协议载荷本身已经携带 `sfid_number`。

## 铁律

- 对外协议字段只能叫 `sfid_number`。
- INSTALL 安装码必须携带公安局机构 `sfid_number`。
- ARCHIVE 的 `geo_seal` 必须绑定同一个 `sfid_number`。
- 不得让档案号 `ano` 自身编码省、市或机构信息。

## 参考

- `memory/05-modules/sfid/SFID-CPMS-QR-v1.md`
- `sfid/backend/cpms/handler.rs::generate_cpms_install_qr`
- `sfid/backend/core/runtime_ops.rs::cleanup_orphan_cpms_sites`
