# 任务卡：重构治理管理员激活机制——去掉钱包管理，改为机构页内激活

任务需求：删除独立的冷钱包管理页面（Tab），改为在机构详情页内直接激活管理员身份。所有用户看到统一页面（提案按钮灰色不可操作），管理员通过冷钱包扫码签名激活后解锁操作权限。省储行管理员激活后额外提供"设为验证者"功能。

所属模块：citizenchain/node（主）、wuminapp（联动）、wumin（联动）

## 输入文档

- memory/00-vision/project-goal.md
- memory/01-architecture/repo-map.md
- memory/03-security/security-rules.md
- memory/07-ai/module-definition-of-done/citizenchain.md
- memory/07-ai/module-checklists/citizenchain.md
- memory/07-ai/module-checklists/wuminapp.md

## 必须遵守

- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 涉及签名协议变更，必须同步更新 wumin 冷钱包端
- 不清楚逻辑时先沟通

---

## 完整技术方案

### 一、项目实际结构

```
GMB/
  citizenchain/
    node/                          ← Tauri 桌面应用 = 节点 + UI
      src/                         ← Rust 后端
        ui/governance/             ← 治理后端逻辑
          mod.rs                   ← 治理入口命令（check_admin_wallets 等）
          signing.rs               ← QR 签名协议（WUMIN_SIGN_V1.0.0）
          institution.rs           ← 链上机构 RPC 查询
          types.rs                 ← 后端类型定义
          proposal.rs              ← 提案查询
          storage_keys.rs          ← 链上存储 key 构造
          sfid_api.rs              ← SFID 接口
        ui/settings/cold-wallets/
          mod.rs                   ← 冷钱包管理（要删除的核心）
        offchain_keystore.rs       ← 加密本地存储（签名管理员用）
      frontend/                    ← React + TypeScript 前端
        governance/
          GovernanceSection.tsx     ← 治理主页（含"钱包管理" Tab）
          InstitutionDetailPage.tsx ← 机构详情页（管理员匹配+提案按钮）
          ColdWalletManager.tsx     ← 钱包管理页面（要删除）
          CreateProposalPage.tsx    ← 创建提案
          ProposalDetailPage.tsx    ← 提案详情
          VoteSigningFlow.tsx       ← 投票签名流程
          QrScanner.tsx             ← 二维码扫描
          governance-types.ts       ← 前端类型
          ...
        api.ts                     ← Tauri invoke 封装
    runtime/                       ← 链上 pallet（本任务不改）
  wuminapp/                        ← 手机在线端 (Flutter)
    lib/governance/
      proposal_context.dart        ← 提案上下文解析（冷钱包匹配管理员）
      institution_detail_page.dart ← 机构详情页
      admin_list_page.dart         ← 管理员列表
      ...
  wumin/                           ← 冷钱包离线端 (Flutter)
    lib/signer/
      qr_signer.dart               ← QR 签名协议实现
      offline_sign_service.dart    ← 离线签名服务
      payload_decoder.dart         ← payload 解码器
      pallet_registry.dart         ← pallet 索引注册表
```

### 二、整体架构变更

```
旧流程：
  治理 → 钱包管理 Tab → 导入冷钱包(公钥) → 机构页匹配公钥 → 显示"我的钱包" → 显示提案按钮

新流程：
  治理 → 机构详情页 → 管理员列表(全部灰色+激活按钮) → 点击"激活" → 冷钱包扫码签名
  → 本地验证签名 → 管理员变绿 + 提案按钮可操作
```

### 三、删除项

| 位置 | 文件/代码 | 说明 |
|------|----------|------|
| 前端 | `frontend/governance/ColdWalletManager.tsx` | 整个组件删除 |
| 前端 | `frontend/governance/GovernanceSection.tsx` 第111行 | 删除"钱包管理" Tab 按钮 |
| 前端 | `frontend/governance/GovernanceSection.tsx` 第148行 | 删除 `ColdWalletManager` 渲染 |
| 前端 | `frontend/governance/GovernanceSection.tsx` 第9行 | 删除 `ColdWalletManager` import |
| 前端 | `frontend/governance/GovernanceSection.tsx` SubTab 类型 | 删除 `'wallets'` |
| 前端 | `frontend/api.ts` | 删除 `getColdWallets`、`addColdWallet`、`removeColdWallet`、`checkAdminWallets` |
| 前端 | `frontend/governance/governance-types.ts` | 删除 `ColdWallet`、`ColdWalletList`、`AdminWalletMatch` 类型 |
| 后端 | `node/src/ui/settings/cold-wallets/mod.rs` | 删除 `add_cold_wallet`、`remove_cold_wallet`、`get_cold_wallets` 命令 |
| 后端 | `node/src/ui/governance/mod.rs` | 删除 `check_admin_wallets` 命令和 `AdminWalletMatch` 结构体 |
| Flutter | `wuminapp/lib/governance/proposal_context.dart` | 删除冷钱包匹配逻辑（`coldWallets` 相关代码），改为读取激活状态 |

### 四、新增：管理员激活机制

#### 4.1 Rust 后端（`node/src/ui/governance/`）

**新文件 `activation.rs`** — 管理员激活相关命令：

**`build_activate_admin_request` (Tauri 命令)**
```
输入：pubkey_hex（管理员公钥）, shenfen_id（机构身份码）
输出：QR 签名请求 JSON + request_id + expected_payload_hash
逻辑：
  1. 查链上管理员列表 → 验证 pubkey 确实是该机构管理员
  2. 构造激活 payload（非链上交易）：
     - 固定前缀 "GMB_ACTIVATE" (12 bytes)
     - shenfen_id (48 bytes, 右补零)
     - timestamp (8 bytes, u64 LE)
     - random_nonce (16 bytes)
  3. 计算 payload_hex = hex(上述拼接)
  4. 复用 WUMIN_SIGN_V1.0.0 协议生成 QrSignRequest：
     {
       proto: "WUMIN_SIGN_V1.0.0",
       type: "sign_request",
       request_id: 随机生成,
       account: pubkey 对应的 SS58 地址,
       pubkey: "0x...",
       sig_alg: "sr25519",
       payload_hex: "0x...",
       issued_at: now,
       expires_at: now + 90,
       display: {
         action: "activate_admin",
         summary: "激活管理员 - XX省储备银行",
         fields: [
           { key: "institution", value: "XX省储备银行" },
           { key: "shenfen_id", value: "..." }
         ]
       }
     }
  5. 返回：{ request_json, request_id, expected_payload_hash }
```

**`verify_activate_admin` (Tauri 命令)**
```
输入：request_id, pubkey_hex, expected_payload_hash, response_json（冷钱包返回的签名回执）
输出：成功/失败
逻辑：
  1. 解析 response_json，校验 proto/request_id/pubkey/payload_hash 一致性
  2. 用 sr25519 验证签名：sr25519_verify(signature, payload_bytes, pubkey)
     注意：激活 payload 短于 256 字节，不需要 blake2_256 预哈希
  3. 签名验证成功 → 将激活凭证写入本地加密存储
  4. 激活凭证结构：
     {
       pubkey_hex: String,          // 管理员公钥（小写无0x）
       shenfen_id: String,          // 所属机构身份码
       activated_at_ms: u64,        // 激活时间戳
       signature_hex: String,       // 签名（用于凭证校验）
       payload_hash_hex: String,    // payload hash
     }
  5. 存储位置：本地加密 JSON 文件（与 cold-wallets 同级目录）
     路径：{base_path}/activated-admins.json（设备密码加密）
```

**`get_activated_admins` (Tauri 命令)**
```
输入：shenfen_id（机构身份码）
输出：Vec<ActivatedAdmin>
逻辑：
  1. 从加密存储读取该机构的激活记录
  2. 查链上当前管理员列表 → 交叉校验
  3. 链上已移除的管理员 → 自动删除本地激活记录
  4. 返回仍有效的已激活管理员
```

**`deactivate_admin` (Tauri 命令)**
```
输入：pubkey_hex, shenfen_id, device_password
输出：成功/失败
逻辑：
  1. 验证设备密码
  2. 删除该激活记录
  3. 如果该管理员同时是签名管理员(验证者)，一并清除
```

#### 4.2 签名协议扩展

**不需要新增协议版本**，仍用 `WUMIN_SIGN_V1.0.0`。

关键区别：激活不是链上交易，`payload_hex` 不包含 nonce/era/genesis_hash 等链上签名扩展，而是自定义的 "GMB_ACTIVATE" 前缀 payload。

**wumin 冷钱包端需要的改动：**

在 `payload_decoder.dart` 中新增对 `activate_admin` 动作的支持：

```dart
// 检测 payload 前 12 字节是否为 "GMB_ACTIVATE"
// 如果是 → 解码 shenfen_id + timestamp + nonce
// 返回 DecodedPayload(action: "activate_admin", summary: "激活管理员 - XX机构")
```

在 `offline_sign_service.dart` 中：
- 当前 `allowedHashedActions` 白名单不需要改（激活 payload 小，不会走 hash 路径）
- 激活 payload 不是 pallet+call 格式 → `PayloadDecoder.decode` 会返回 null
- 需要新增对 `display.action == "activate_admin"` 的处理：
  - 识别到 `activate_admin` 时，独立解码 payload 中的 "GMB_ACTIVATE" 前缀
  - 如果前缀匹配 → 允许签名（信任 display）
  - 如果前缀不匹配 → 拒绝签名

#### 4.3 React 前端（`citizenchain/node/frontend/governance/`）

**GovernanceSection.tsx 改造：**
- 删除"钱包管理" Tab 按钮和 `ColdWalletManager` 渲染
- 删除 `SubTab` 类型中的 `'wallets'`
- 删除 `import { ColdWalletManager }`
- `GovernanceView` 类型中涉及 `AdminWalletMatch[]` 的字段改为 `ActivatedAdmin[]`

**InstitutionDetailPage.tsx 改造：**

```
旧逻辑（第25-32行）：
  加载时 → api.checkAdminWallets(shenfenId) → 如果有匹配 → isAdmin=true → 显示提案按钮

新逻辑：
  加载时 → api.getActivatedAdmins(shenfenId) → activatedAdmins 列表
  isAdmin = activatedAdmins.length > 0

管理员列表（第164-184行）改造：
  旧：管理员公钥 + [我的钱包] 标签
  新：管理员公钥 + [激活按钮] 或 [已激活标签]

  每个管理员行：
  - 默认灰色文字 + [激活] 按钮（灰色图标）
  - 已激活 → 绿色文字 + [已激活] 标签（绿色）
  - 已激活 + 机构类型=PRB → 额外显示 [设为验证者] 按钮

提案按钮区（第97-122行）改造：
  旧：isAdmin ? 显示 : 不显示
  新：始终显示，但 disabled={!isAdmin}
  - 无激活管理员 → 全部灰色 disabled
  - 有激活管理员 → 已开通的按钮正常颜色可点击
  - 未开通功能 → 保持 disabled + "即将上线"

活跃提案列表（第124-161行）改造：
  旧：isAdmin 时提案卡片可点击
  新：始终可点击查看提案详情（查看不需要管理员权限）
  - isAdmin 时额外显示"可投票"标记
```

**新增激活交互流程组件：**

复用现有的 `QrScanner.tsx` + `VoteSigningFlow.tsx` 模式：

```
1. 用户点击管理员行的 [激活] 按钮
2. 弹出签名流程弹窗（类似 VoteSigningFlow）
3. 调用 api.buildActivateAdminRequest(pubkeyHex, shenfenId) → 获取 QR JSON
4. 显示签名请求二维码 → 用户用 wumin 扫码
5. wumin 显示"激活管理员 - XX机构" → 用户确认签名
6. wumin 生成签名回执二维码 → 用户在节点端扫描回执（通过 QrScanner）
7. 调用 api.verifyActivateAdmin(requestId, pubkeyHex, payloadHash, responseJson)
8. 验证成功 → 刷新页面状态 → 管理员变绿 + 提案按钮可操作
```

**"设为验证者"按钮（仅省储行）：**

```
条件：已激活 + detail.orgType === 2 (PRB)
点击 → 弹窗要求输入管理员私钥种子（64位hex）+ 设备密码
调用已有 api.setSigningAdmin(pubkeyHex, privateKey, password)
成功后 → 按钮变为 [已设为验证者] + 提示"重启节点后生效"
```

#### 4.4 wuminapp Flutter 联动

**`proposal_context.dart` 改造：**
- 旧逻辑：从本地冷钱包列表匹配链上管理员
- 新逻辑：通过 RPC 调用节点的 `get_activated_admins` 获取激活状态
- 注意：wuminapp 是轻节点连接，不能直接调 Tauri 命令
- 方案：wuminapp 的激活状态也需要独立存储在 Isar 本地数据库中

**`institution_detail_page.dart` 改造：**
- 同 nodeui：管理员列表行内激活按钮 + 提案按钮灰色/可操作
- 激活流程：生成激活 payload → 跳转 wumin 扫码 → 返回签名 → 本地验证 → 写入 Isar

**`admin_list_page.dart` 改造：**
- 旧：冷钱包匹配显示"我"
- 新：读取本地激活状态，已激活显示绿色 + "已激活"

**删除冷钱包导入相关入口**（如果 wuminapp 中有的话）

### 五、保留项

| 功能 | 原位置 | 新位置 | 说明 |
|------|--------|--------|------|
| `set_signing_admin` | cold-wallets/mod.rs + ColdWalletManager | cold-wallets/mod.rs + InstitutionDetailPage（PRB 管理员行内） | 后端逻辑不变，仅前端入口变 |
| `get_signing_admin` | 同上 | 同上 | 用于显示"已设为验证者"状态 |
| QR 签名提交流程 | signing.rs 全套 | 不变 | 提案/投票签名仍走 QR 扫码 |
| 开发升级页 | DeveloperUpgradePage | 不变 | 与钱包管理无关 |

### 六、状态管理与安全

| 场景 | 处理方式 |
|------|---------|
| 链上管理员被替换 | `get_activated_admins` 每次调用时与链上交叉校验，自动清除失效记录 |
| 用户卸载重装 | 激活状态存在本地加密存储（activated-admins.json），随节点数据目录保留 |
| 前端篡改绕过激活 | 提案操作调后端 API → 后端校验激活状态 → 最终提交仍需冷钱包签名（三层保护） |
| 多管理员同机 | 支持同一节点激活多个不同机构的管理员 |
| 跨机构 | 每条激活记录绑定 (pubkey, shenfen_id)，互不干扰 |
| wuminapp 与 node 激活状态 | 各自独立存储（node 在加密 JSON，wuminapp 在 Isar），互不依赖 |

### 七、执行顺序与调度

```
Step 1: [Blockchain Agent] Rust 后端
  - 新建 node/src/ui/governance/activation.rs
  - 实现 build_activate_admin_request / verify_activate_admin / get_activated_admins / deactivate_admin
  - 注册 Tauri 命令
  - 激活凭证加密存储

Step 2: [Blockchain Agent] wumin 冷钱包端
  - payload_decoder.dart 新增 "GMB_ACTIVATE" 前缀识别
  - offline_sign_service.dart 新增 activate_admin 白名单

Step 3: [Blockchain Agent] nodeui React 前端
  - 改造 InstitutionDetailPage.tsx（管理员行内激活 + 提案按钮灰色/可操作）
  - 改造 GovernanceSection.tsx（删除钱包管理 Tab）
  - 新增激活签名流程弹窗（复用 QrScanner + VoteSigningFlow 模式）
  - 更新 api.ts 和 governance-types.ts
  - 删除 ColdWalletManager.tsx

Step 4: [Mobile Agent] wuminapp Flutter
  - 改造 institution_detail_page.dart（同 nodeui 逻辑）
  - 改造 proposal_context.dart（读取激活状态替代冷钱包匹配）
  - 改造 admin_list_page.dart
  - 新增 Isar 激活状态存储

Step 5: 清理残留
  - 删除旧冷钱包相关代码（add/remove/get_cold_wallets）
  - 删除 check_admin_wallets
  - 更新文档
```

**前置依赖**：Step 1 + Step 2 必须先完成（冷钱包不支持 activate_admin 则激活流程走不通）。Step 3 和 Step 4 可并行。

---

## 输出物

- 代码
- 中文注释
- 文档更新（模块技术文档）
- 残留清理（冷钱包管理相关代码全部删除）

## 验收标准

- 所有用户安装后看到统一的机构页面（管理员列表含激活按钮 + 灰色提案按钮）
- 管理员可通过冷钱包扫码激活，激活后按钮变为可操作
- 省储行管理员激活后可设为验证者
- 非管理员看到提案按钮但不可点击
- 前端篡改无法绕过后端激活校验
- 链上管理员变更后本地激活自动失效
- 冷钱包管理 Tab 和页面已完全删除
- 治理主页 Tab 顺序：提案 / 国储会 / 省储会 / 省储行 / 开发升级
- wuminapp 同步适配
- wumin 冷钱包支持 activate_admin 签名
- 文档已更新
- 残留已清理
