# OnChina 地址库技术文档

## 1. 功能定位

OnChina 地址库模块负责读取本地 `china.sqlite.addresses`，并构造 `AddressRegistry` 链上地址变更 call data。

模块路径：

```text
citizenchain/onchina/src/domains/address/
├── mod.rs              # 地址域聚合入口
├── model.rs            # API DTO
├── repo.rs             # china.sqlite 只读查询
├── handler.rs          # HTTP handler
├── chain_call.rs       # AddressRegistry SCALE call data 编码器
└── version.rs          # 地址库版本常量

citizenchain/onchina/frontend/address/
├── api.ts              # 前端地址 API
└── AddressManageView.tsx # 地址管理页面
```

## 2. 数据边界

- 地址主数据仍在 `citizenchain/onchina/src/cid/china/china.sqlite`。
- 后端只读打开 SQLite，不在运行态复制或改写地址主数据。
- 链上 call data 只用于地址变更冷签，不在 OnChina 后端直接提交 extrinsic。
- 前端只展示查询结果和生成的 call data，不绕过 QR_V1/冷签流程。

## 3. API

| 方法 | 路径 | 用途 |
|---|---|---|
| `GET` | `/api/v1/admin/address/names` | 查询某省市镇下的地址名称列表 |
| `GET` | `/api/v1/admin/address/items` | 查询某地址名称编号下的完整地址列表 |
| `POST` | `/api/v1/admin/address/chain-call` | 构造 AddressRegistry 裸 SCALE call data |

## 4. 权限

- 后端按登录态 `scope_province_name / scope_city_name` 转为省市码过滤。
- FRG 只能访问本省地址。
- CREG 只能访问本市地址。
- runtime 会再次校验签名管理员与 `registrar_account` 是否具备本省/本市地址更新权。

## 5. 链上调用

OnChina 使用 `address/chain_call.rs` 构造以下 call data：

```text
AddressRegistry(35).set_catalog_version(0)
AddressRegistry(35).set_address_name(1)
AddressRegistry(35).remove_address_name(2)
AddressRegistry(35).set_address(3)
AddressRegistry(35).remove_address(4)
```

链交易动作码统一为：

```text
action = (35 << 8) | call_index
```

## 6. 验收

```text
cargo check --manifest-path citizenchain/Cargo.toml -p onchina
npm --prefix citizenchain/onchina/frontend run build
python3 citizenchain/onchina/src/cid/china/check_code_immutable.py
sqlite3 citizenchain/onchina/src/cid/china/china.sqlite "PRAGMA integrity_check"
```

