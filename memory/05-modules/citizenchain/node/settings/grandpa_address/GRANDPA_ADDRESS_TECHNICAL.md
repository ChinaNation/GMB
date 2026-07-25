# Grandpa Address 模块技术文档

## 0. 功能需求

- 页面需要支持上传确定性投票节点私钥，并显示当前绑定机构。
- 模块需要校验私钥格式，并能从私钥推导出 ed25519 公钥。
- 初始上传私钥必须匹配机构清单中的创世 GRANDPA authority 公钥。
- 模块需要把运行所需的 `gran` 密钥写入本地节点 keystore。
- 正常更换与紧急恢复期间必须同时保留旧、新两把私钥，不能提前清理旧私钥。
- 只有 finalized 状态确认新 authority 生效且旧 authority 移除后，才能自动删除旧私钥。
- 当节点正在运行时，上传成功后需要自动重启节点，并校验节点已进入 authority/validator 角色。

## 1. 模块位置

- 路径：`node/src/settings/grandpa_address.rs`
- 对外命令：
  - `get_grandpa_key`
  - `set_grandpa_key`
- 更换编排：`node/src/core/grandpa_rotation.rs`
- 对外命令：
  - `build_grandpa_key_change_request`
  - `submit_grandpa_key_change`
  - `get_grandpa_key_change_status`

## 2. 模块职责

- 管理"确定性投票节点私钥"的上传、校验、存储与读取。
- 初始导入从结构化机构清单读取创世 GRANDPA authority 公钥，确认机构归属。
- 运行期更换根据链上目标 CID 的当前公钥和 finalized authority set 确认归属，
  不要求新公钥存在于不可变创世清单。
- 将 GRANDPA 私钥同步写入本地节点 keystore（`gran` key type）。
- 初始导入只保存当前密钥；更换期保留旧、新密钥；finalized 后精确删除旧密钥。
- 与节点启动流程协同：存在投票私钥时以 `--validator` 模式启动并校验生效。

## 3. 存储设计

- 节点 keystore 文件：`<app_data>/chains/*/keystore/6772616e<grandpa_public_key>`
  - 文件内容：`"0x<private_hex>"`
  - 通过原子写入落盘，避免异常中断时文件损坏。
- 本地元数据文件：`<app_data_dir>/grandpa-meta.json`
  - `cid_full_name`
  - `grandpa_public_key`
- 待更换状态：`<app_data_dir>/grandpa-key-change.json`
  - 只保存目标 CID、旧新公钥、路径、nonce、有效期、交易哈希和状态。
  - 不保存旧、新 GRANDPA 私钥或管理员签名私钥。

## 4. 关键流程

### 4.1 上传投票节点私钥 `set_grandpa_key`

1. 校验设备开机密码。
2. 校验 GRANDPA 私钥格式（64 位 hex）。
3. 推导 ed25519 公钥。
4. 公钥必须匹配 GRANDPA authority 清单中的机构。
5. 保存机构元数据（含 `grandpa_public_key`）。
6. 同步写入节点 keystore 的 `gran` 密钥文件。
   - 初始导入清理非当前公钥文件；运行期更换不调用该单密钥初始化入口。
7. 若节点运行中，执行 `stop_node -> start_node`，并进行生效校验。
8. 若写入后重启或校验失败，回滚旧的元数据和 `gran` keystore 文件，避免留下半提交状态。

### 4.2 节点启动协同

- `home::home_node::start_node` 启动流程中调用 `prepare_grandpa_for_start`：
  - 检查 meta 中的 `grandpa_public_key` 与本地 keystore 是否一致；
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
- `import_rotation_candidate(...)`：新增候选新私钥但保留旧私钥。
- `sign_rotation_proof(...)`：由本机旧私钥签署正常更换证明。
- `finalize_rotation_key(...)`：finalized 确认后精确删除旧私钥并更新元数据。
- `discard_rotation_candidate(...)`：仅清理未提交且已过期的候选新私钥。

## 6. 更换生命周期

1. 目标机构委员选择正常更换或紧急恢复。
2. 节点随机生成新 ed25519 私钥，写入所有链 keystore，并保留旧私钥。
3. 正常更换由旧、新私钥双签；紧急恢复只由新私钥证明持有，再进入目标机构内部投票。
4. 交易提交后，后台监视 finalized 区块下的 `CurrentGrandpaKeys` 和
   `Grandpa::Authorities`。
5. 只有“当前 CID 公钥等于新公钥、新公钥已在 authority set、旧公钥已不在
   authority set”同时成立时，才删除旧私钥、更新元数据、清理待处理状态并重启节点。
6. 仅生成但从未提交的请求在证明过期后可删除候选新私钥；已经提交的紧急恢复不能
   因证明过期被清理，因为内部投票可能仍在进行。

## 7. 性能优化

- 机构清单使用 `OnceLock<Vec<InstitutionCatalogEntry>>` 惰性缓存，编译期内嵌 JSON 仅解析一次。
- Keystore 操作委托 `shared/keystore` 通用模块，与 bootnode 模块共享目录扫描和密钥写入逻辑。
