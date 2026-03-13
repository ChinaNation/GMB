# Bootnodes Address 模块技术文档

## 0. 功能需求

- 页面需要支持上传区块链引导节点私钥，并显示当前绑定的 PeerId 与机构名称。
- 模块需要校验上传的私钥格式，并能从私钥稳定推导出 PeerId。
- 模块需要保证上传的私钥只能对应创世引导节点清单中的合法机构，避免任意节点私钥被误配置。
- 模块需要把私钥安全存储在系统安全存储中，不向前端回传明文。
- 当节点正在运行时，上传成功后需要自动重启节点并确认本机 PeerId 已切换为目标引导节点。
- 模块需要向首页/网络模块提供 PeerId 到机构名映射能力，供角色展示和网络统计复用。

## 1. 模块位置

- 路径：`nodeui/backend/src/settings/bootnodes-address/mod.rs`
- 对外命令：
  - `get_bootnode_key`
  - `set_bootnode_key`
  - `get_genesis_bootnode_options`

## 2. 模块职责

- 管理“区块链引导节点私钥”的上传、校验、存储与读取。
- 从结构化机构清单 `settings/institution-catalog.json` 读取引导节点名称与 PeerId 清单。
- 校验上传私钥是否匹配创世引导节点。
- 节点运行中上传后自动重启，并校验本机 PeerId 已切换为目标引导节点。

## 3. 存储设计

- 系统安全存储（Keychain/Keyring）键：`bootnode-node-key`
- 本地元数据文件：`<app_data_dir>/bootnode-meta.json`
  - `peer_id`
  - `institution_name`

## 4. 关键流程

### 4.1 上传引导节点私钥 `set_bootnode_key`

1. 校验设备开机密码。
2. 校验 `node-key` 格式（64 位 hex）。
3. 由私钥推导 `PeerId`。
4. 校验推导 `PeerId` 必须在创世引导节点清单内。
5. 私钥加密写入系统安全存储。
6. 保存 `bootnode-meta.json`。
7. 若节点运行中，执行 `stop_node -> start_node`。
8. 轮询 `system_localPeerId`，确认重启后 PeerId 与目标一致。

### 4.2 节点启动协同

- `home::home_node::start_node` 内部启动流程会调用 `load_bootnode_node_key` 读取已保存私钥。
- 启动参数通过 `--node-key-file` 注入，避免私钥出现在命令行。

## 5. 对外协作接口（给 home-node）

- `load_bootnode_node_key(app, unlock_password)`
- `verify_bootnode_secret_unlock(unlock_password)`
- `genesis_bootnode_options() -> Result<Vec<GenesisBootnodeOption>, String>`
- `find_genesis_bootnode_name_by_peer_id(peer_id) -> Result<Option<String>, String>`

## 6. 共享模块依赖

- 机构清单解析复用 `grandpa-address::load_institution_catalog()`（OnceLock 缓存）。
- Keystore 操作委托 `shared/keystore` 通用模块进行目录扫描和密钥写入。
