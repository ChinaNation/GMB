# AddressRegistry 技术文档

## 1. 功能定位

`address-registry` 是 CitizenChain runtime 的地址变更上链模块，路径为 `citizenchain/runtime/otherpallet/address-registry/`。

本模块只保存地址库版本、单条地址当前哈希和链上事件，不保存完整地址库，不保存旧地址历史，不保存墓碑。

## 2. 链上模块

```text
citizenchain/runtime/otherpallet/address-registry/
├── Cargo.toml              # pallet 依赖与 feature
└── src/lib.rs              # storage、事件、权限抽象和 extrinsic
```

## 3. Runtime 挂载

- pallet index：`33`
- runtime 类型名：`AddressRegistry`
- 费用模型：`VoteFlat`
- 权限配置：`RuntimeAddressAuthority`

## 4. Extrinsic

| call index | 调用 | 用途 |
|---:|---|---|
| `0` | `set_catalog_version` | 设置当前地址库版本号和整体哈希 |
| `1` | `set_address_name` | 新增或修改镇下地址名称 |
| `2` | `remove_address_name` | 删除镇下地址名称 |
| `3` | `set_address` | 新增或修改完整地址 |
| `4` | `remove_address` | 删除完整地址 |

## 5. 权限规则

- `set_catalog_version` 只允许 FRG 省级组管理员通过该省级组主账户发起。
- FRG 省级组管理员可以更新本省任意地址。
- CREG 管理员只能更新本市地址。
- 地址模块不直接读取管理员 storage；它通过 `AddressUpdateAuthority` 抽象把权限判断交给 runtime 配置层。

## 6. 版本与存储

- `CatalogVersion` 保存行政区地址库版本字符串，例如 `v1.0.0`。
- `CatalogHash` 保存当前本地 `china.sqlite` 的 32 字节哈希。
- `AddressNameVersions` / `AddressVersions` 保存单条地址当前版本号。
- `AddressNameHashes` / `AddressHashes` 保存单条地址当前内容哈希。
- 删除地址时只移除当前哈希并发事件，版本号递增，不保存旧数据和墓碑。

## 7. 地址键

地址名称键：

```text
province_code + city_code + town_code + address_name_code
```

完整地址键：

```text
province_code + city_code + town_code + address_name_code + address_local_no + address_detail
```

字段约束：

- `address_name_code` 固定 3 位数字，禁止 `000`。
- `address_local_no` 为空或固定 4 位数字，禁止 `0000`。
- `address_detail` 可为空。

## 8. 验收

```text
cargo check --manifest-path citizenchain/Cargo.toml -p address-registry
cargo check --manifest-path citizenchain/Cargo.toml -p citizenchain
```

