# Grandpa Address 模块技术文档

## 0. 功能需求

- 页面需要支持上传确定性投票节点私钥，并显示当前绑定机构。
- 模块需要校验私钥格式，并能从私钥推导出 ed25519 公钥。
- 模块需要保证上传私钥必须匹配机构清单中的 GRANDPA authority 公钥，避免错误机构或错误密钥被保存。
- 模块需要把运行所需的 `gran` 密钥写入本地节点 keystore。
- 模块需要清理旧的 GRANDPA keystore 密钥，避免节点同时加载多把历史 authority key。
- 当节点正在运行时，上传成功后需要自动重启节点，并校验节点已进入 authority/validator 角色。

## 1. 模块位置

- 路径：`node/src/settings/grandpa-address/mod.rs`
- 对外命令：
  - `get_grandpa_key`
  - `set_grandpa_key`

## 2. 模块职责

- 管理"确定性投票节点私钥"的上传、校验、存储与读取。
- 从结构化机构清单 `settings/institution-catalog.json` 读取 GRANDPA authority 公钥清单。
- 将投票私钥推导公钥与 authority 清单匹配，确认机构归属。
- 将 GRANDPA 私钥同步写入本地节点 keystore（`gran` key type）。
- 清理历史遗留的 `gran` 密钥文件，保证节点只保留当前机构对应的 GRANDPA 密钥。
- 与节点启动流程协同：存在投票私钥时以 `--validator` 模式启动并校验生效。

## 3. 存储设计

- 节点 keystore 文件：`<node-data>/chains/*/keystore/6772616e<grandpa_pubkey_hex>`
  - 文件内容：`"0x<private_hex>"`
  - 通过原子写入落盘，避免异常中断时文件损坏。
- 本地元数据文件：`<app_data_dir>/grandpa-meta.json`
  - `institution_name`
  - `pubkey_hex`

## 4. 关键流程

### 4.1 上传投票节点私钥 `set_grandpa_key`

1. 校验设备开机密码。
2. 校验 GRANDPA 私钥格式（64 位 hex）。
3. 推导 ed25519 公钥。
4. 公钥必须匹配 GRANDPA authority 清单中的机构。
5. 保存机构元数据（含 pubkey_hex）。
6. 同步写入节点 keystore 的 `gran` 密钥文件。
   - 清理旧的 `gran` 密钥文件，只保留当前公钥对应的密钥。
7. 若节点运行中，执行 `stop_node -> start_node`，并进行生效校验。
8. 若写入后重启或校验失败，回滚旧的元数据和 `gran` keystore 文件，避免留下半提交状态。

### 4.2 节点启动协同

- `home::home_node::start_node` 启动流程中调用 `prepare_grandpa_for_start`：
  - 检查 meta 中的 pubkey_hex 是否在当前 authority 清单中；
  - 确认 keystore 文件存在；
  - 返回 `enable_grandpa_validator=true`。
- `home::home_node::start_node` 在 `enable_grandpa_validator=true` 时追加 `--validator`。
- `home::home_node::start_node` 启动后调用 `verify_grandpa_after_start`：
  - 最长等待约 20 秒，校验 `system_nodeRoles` 包含 `authority` 或 `validator`；
  - 校验本地 keystore 已存在匹配的 `gran` 密钥文件。

### 4.3 失败回滚

- `set_grandpa_key` 在落盘前会先备份旧的 `grandpa-meta.json` 与现有 `gran` keystore 文件。
- 如果新配置写入后节点重启失败，模块会恢复旧持久化状态；若节点原本在运行，还会尝试按旧配置重新拉起。

## 5. 对外协作接口

- `prepare_grandpa_for_start(app, unlock_password) -> Result<bool, String>`
- `verify_grandpa_after_start(app, unlock_password) -> Result<(), String>`

## 6. 性能优化

- 机构清单使用 `OnceLock<Vec<InstitutionCatalogEntry>>` 惰性缓存，编译期内嵌 JSON 仅解析一次。
- Keystore 操作委托 `shared/keystore` 通用模块，与 bootnode 模块共享目录扫描和密钥写入逻辑。
