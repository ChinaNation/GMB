# ADR-008 SFID 省管理员 3-tier 自治 + 删除 KEY_ADMIN

- 状态:accepted
- 日期:2026-05-01
- 决策人:Architect 主入口(Claude)+ 用户
- 关联任务卡:`memory/08-tasks/open/20260501-sfid-step1-sheng-admin-3tier-and-key-admin-removal.md`

## 上下文

SFID 三角色原设计:
- KEY_ADMIN(全国):管理省管理员 + 派发省级签名密钥 + 全国业务
- SHENG_ADMIN(单省):每省 1 把硬编码 main pubkey + 派生签名密钥 + 本省业务
- SHI_ADMIN(单市):本市业务,签名走省级签名密钥

实际运行暴露 4 个问题:

1. **KEY_ADMIN 多此一举**:省管理员可自治,无需"超级管理员"代为发起省级签名密钥派发
2. **省管理员单点故障**:每省仅 1 把 admin pubkey,私钥泄露/丢失即全省瘫痪
3. **链上 trust anchor 矛盾**:用户坚持"链上 0 prior knowledge of SFID",chain runtime 不应硬编码任何 SFID admin pubkey
4. **推链 1010 错误**:SFID main 账户链上零余额,所有 push extrinsic 因 fee 检查被 TxPool reject(实际事件:`bootstrap_sheng_signer` 推 `set_sheng_signing_pubkey` 失败)

## 决策

### 1. 角色模型彻底简化

- **删除 KEY_ADMIN**:整角色废止,所有相关代码 / 路由 / UI / 数据库 schema 删除,不留兼容
- **省管理员 3-tier**:每省 main / backup_1 / backup_2 三槽
  - `main` 公钥:SFID `sfid/province.rs` const 硬编码 43 把(沿用现有,不变)
  - `backup_1` / `backup_2` 公钥:由 main 登录后通过 SFID 前端动态添加,链上 storage `ShengAdmins[Province][Slot]` 持久化
- **市管理员**:沿用,签名时使用本省**当前登录省管理员**的签名密钥(三槽各自独立)

### 2. 链与 SFID 单向交互

- **链 → SFID**:pull-only(机构信息、人口快照、公民绑定状态、公民投票状态)
- **SFID → 链**:仅限以下 push,全部 `Pays::No`(SFID main 账户零余额下也能成功)
  - `add_sheng_admin_backup(province, slot, new_pubkey, sig_by_main)`
  - `remove_sheng_admin_backup(province, slot, sig_by_main)`
  - `activate_sheng_signing_pubkey(province, admin_pubkey, signing_pubkey, sig)`
  - `rotate_sheng_signing_pubkey(province, admin_pubkey, new_signing_pubkey, sig)`

### 3. 签名密钥模型

每省 **3 把独立签名密钥**(每个 admin slot 各一把,互不共享):

- 加密 seed 落盘:`storage/sheng_signer/{province_code}_{admin_pubkey_hex}.enc`
- 加密算法:AES-256-GCM
- wrap key:`HKDF(SFID_MASTER_KEK, salt = admin_pubkey)`
- 登录态 cache:`Mutex<HashMap<(Province, AdminPubkey), Sr25519Pair>>`
- 业务凭证签发(institutions / citizens / shi_admins 的所有 credential 签发)统一从 cache 取 keypair

### 4. 链上 trust anchor 解决

- **拒绝在 runtime 硬编码 SFID admin pubkey**:链上 `ShengAdmins` storage 初始为空
- **Activation 走 first-come-first-serve**:任意 admin 公钥首次调 `activate_sheng_signing_pubkey` 时被记录到 `ShengAdmins[Province][Slot]`,后续 add/remove backup 由记录在案的 main pubkey 签名授权
- **安全 trade-off**:接受"首激活方即被信任"的风险(部署期窗口短,可控);收益是 chain runtime 与 SFID 完全解耦,SFID 内部模型可独立演进

### 5. 国家级业务签名分布化

人口快照等国家级业务的链上验签:每省独立签子集 → 业务方(节点桌面/runtime)聚合 43 份子签 → 链端逐一验签。**不**做 KEY_ADMIN 集中签名。

## 影响

### SFID 后端(详 Step 1 任务卡)

- 删除 ~3300 行(KEY_ADMIN 整模块 + 推链失败路径)
- 新增 ~2700 行(省管理员 3-tier 业务 + chain push 模块)
- 净增 ~600 行
- 数据库:DROP `key_admins` 表

### citizenchain/runtime(Step 2 任务卡跟进)

- 新增 storage:`ShengAdmins: DoubleMap<Province, Slot, Pubkey>` ; `ShengSigningPubkey: DoubleMap<Province, AdminPubkey, SigningPubkey>`
- 新增 4 个 extrinsic(全 `Pays::No`)
- 删除任何"硬编码 SFID admin pubkey"残留(若有)
- 删除任何依赖 KEY_ADMIN 的 verifier 分支
- spec_version + 1,需 `on_runtime_upgrade` migration(开发期可裸升)

### 节点桌面 / wuminapp(Step 3 任务卡跟进)

- 联合投票流程改造:并行拉 43 省人口快照子签 + 聚合
- 删除 KEY_ADMIN 视图(若有)

## 替代方案与拒绝原因

| 方案 | 拒绝原因 |
|---|---|
| 保留 KEY_ADMIN 仅作为"恢复通道" | 增加复杂度,违反"开发期一次性彻底切换"原则 |
| 链上硬编码 43 把 main pubkey | 违反"链上 0 prior knowledge of SFID"原则,SFID 模型变更需 setCode |
| 保留每省单 admin + 加冷备私钥 | 单点故障未解,管理员私钥泄露依然全省瘫痪 |
| activate 用 sudo 强制写入 | 引入 sudo 即破坏去中心化,且 sudo 退役后无法操作 |

## 验收

- ADR-008 标记 accepted 后,Step 1 / Step 2 / Step 3 任务卡按本 ADR 执行
- Step 1 完工后:Grep `KeyAdmin|key-admin|key_admin` 在整个 GMB 工作区全部零结果
- Step 2 完工后:链上 4 个 Pays::No extrinsic 可调用;SFID main 账户零余额下推链成功
- 端到端:任意省 main / backup_1 / backup_2 登录,各自独立签名密钥,本省写权限,跨省只读
