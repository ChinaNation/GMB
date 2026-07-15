# chainspec 与创世状态冻结规则（铁律）

> 适用范围：全节点 `citizenchain.plain.json`、安装包内置 `genesis-state/`、CitizenApp
> `assets/chainspec.json`、`assets/light_sync_state.json`，以及任何分发给轻节点 / 钱包 / App 的创世锚点文件。

## 结论

主网创世那一刻必须同时冻结四件东西：

1. **全节点 plain SSOT**：`citizenchain/node/chainspecs/citizenchain.plain.json`。
   它只保存 runtime WASM、genesis patch、bootNodes、properties 和 protocolId。
2. **节点创世初始化**：安装包不携带物化创世数据库；节点依据同一份 plain chainspec
   本地初始化空数据库并通过 bootNodes 同步网络。烘焙产生的状态只用于验收。
3. **CitizenApp 轻形态 chainspec**：`citizenapp/assets/chainspec.json`。
   它只保存轻节点联网所需字段和 `genesis.stateRootHash`，不得携带 GB 级 raw state。
4. **CitizenApp light sync checkpoint**：`citizenapp/assets/light_sync_state.json`。
   它保存 smoldot 加入 `stateRootHash` 轻形态链所必需的 finalized header 和 GRANDPA authority set。
   正式链高度仍为 0 时允许保存创世头 checkpoint；这是 PoW + `stateRootHash` 轻形态的启动锚点。

创世后 runtime 升级一律走链上 `system.setCode` 交易。除非正式硬分叉，任何脚本、CI、
启动入口都不得重烤创世锚点。

## 为什么

1. genesis hash 决定 Substrate / libp2p 通知协议名。
2. 创世状态必须由同一份 CI WASM 和同一份 plain spec 冻结；raw chainspec 不作为仓库和
   App 资产。
3. 每台节点都从同一份 chainspec 初始化，确保 genesis hash 一致后通过网络同步。
4. CitizenApp 只需要轻节点创世头校验信息，公权机构目录另走“内置快照 + 链上投影增量”。

## 单一权威源

- 全节点创世配置唯一真源：`citizenchain/node/chainspecs/citizenchain.plain.json`。
- 创世状态包唯一来源：`citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>`，
  并必须提供 `--wasm-ci-run-id <RUN_ID> --wasm-ci-head-sha <HEAD_SHA>`；清单记录可追溯的 WASM CI run 和提交。
- CitizenApp 轻形态与 checkpoint 唯一来源：同一次 `bake-chainspec.sh` 读取块 0 header 并调用
  `sync_state_genLightSyncState` 后写出。
- 公权机构唯一真源：链上 `PublicManage::Institutions` /
  `PublicManage::InstitutionAccounts`；App 内置快照和 OnChina PostgreSQL 都只是缓存。

## 正式创世流程

1. GitHub `CitizenChain WASM` CI 成功后，下载同一提交的
   `citizenchain.compact.compressed.wasm`。
2. 执行：

   ```bash
   citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM> \
     --wasm-ci-run-id <RUN_ID> \
     --wasm-ci-head-sha <HEAD_SHA>
   ```

3. 脚本必须完成：
   - 导出 fresh plain spec。
   - 启动临时节点物化块 0。
   - 通过 `check-constitution-genesis.py --rpc --expect-code-file <CI_WASM>`。
   - 写回 `citizenchain/node/chainspecs/citizenchain.plain.json`。
   - 写回 `citizenapp/assets/chainspec.json` 的 `stateRootHash` 轻形态。
   - 写回 `citizenapp/assets/light_sync_state.json` 的 smoldot checkpoint。
   - 导出 `target/chainspec/genesis-state/manifest.json` 和
     `target/chainspec/genesis-state/chains/citizenchain/db`。
4. 打包前执行 `citizenchain/scripts/prepack.sh` 或 `prepack.ps1`，把
   `genesis-state/` 放进 Tauri resources。
5. 节点首启时优先复制内置创世状态包；没有该包时，只允许开发/排障场景回退到
   GenesisBuilder 本地物化。
6. `CitizenChain` CI 只构建四个平台软件 artifact；不得上传或注入物化创世数据库。
7. 无头服务器只下载 Linux amd 软件，停服务安装后保留节点身份密钥和 GRANDPA keystore，
   再由节点按 plain chainspec 初始化并联网同步。

## 当前唯一冻结锚点（2026-07-14）

- Git commit：`40646f360f01fe362d38ada6085357c586848210`；GitHub `CitizenChain WASM` run：`29388014765`。
- `genesis_hash`：`0xbb993e8fb7aa6c06e44b96f4ba35179ef8644ade17c37529c1742e1fb261b095`。
- `state_root`：`0xd285f98522ca3bce15decd52e61a6d9e444a069a4544a8141eec0017d6e324ac`。
- `runtime_wasm_hash`：`e7c239c78e337fde6a5e107669f77a517de199dfced1f9b96c9ae49a297b1f79`。
- 全节点 `chainspec_hash`：`471effd6403fcce6bee84ab9bde7ed9aa6e2b3dbb54248a923d4d0d0688c4651`。
- CitizenApp `chainspec_hash`：`85a4e6276df86bfad1caca83a634ea684f23619df16c09dbfc74bb0e9b05cd3a`。
- `light_sync_state_hash`：`16b722fb297a358efb61b56c70883e0ff0fc1b9cb4af5c508f5d38d624a35c42`。
- `public_institution_root`：`b6f3927a831d940cf7037d68fcbc5fc62f9ebb4d2c2a8ce0ef4da15d59fa3855`，43 省共 49,593 个机构。

正式 bake 的创世物化耗时 50 秒；公民宪法 `law_id=0`、v1 生效版和不可变条款校验通过。正式包的隔离副本已使用默认内嵌链规范真实启动，RPC 返回同一 block#0/state root，`isSyncing=false`。`bake-chainspec.sh` 的 RPC 轮询必须让内嵌 Python 正常解析响应；不得抑制解析失败后把已就绪节点误判为超时。

## 防御措施

1. `citizenapp/scripts/check-chainspec-frozen.sh` 校验 node plain SSOT、CitizenApp
   轻形态 chainspec 与 `light_sync_state.json` 是否匹配；正式发布设置
   `CITIZENAPP_REQUIRE_STATE_ROOT=1`。
2. `citizenchain/node/src/home/process/mod.rs` 在启动节点前尝试安装内置
   `genesis-state/`，并在 RPC `chain_getBlockHash(0)` 成功前保持首次“初始化中”或普通“启动中”。
3. `citizenchain/node/src/onchina_proc/mod.rs` 启动 OnChina 前必须确认本机链 RPC 已就绪。
4. `citizenchain/scripts/prepack.sh` / `prepack.ps1` 只允许复制 `manifest.json` 与
   `chains/citizenchain/db/**`；任何符号链接、TLS、network、keystore 或其他路径都必须失败关闭。
   macOS 部署归档必须禁用 AppleDouble，禁止把系统扩展属性展开成 `._*` 成员。
5. CitizenApp Cloudflare bootstrap 的默认常量及各环境 `CHAIN_GENESIS_HASH` / `CHAIN_STATE_ROOT` 必须与本地冻结资产一致；Worker 只发布链身份和 bootnodes，不得成为远端 checkpoint 真源。

## 绝对不能做的事

- 在启动脚本里重新 `build-spec --raw` 覆盖主网创世。
- 把 raw chainspec 重新作为仓库 SSOT 或 App 资产。
- 把 `lightSyncState` 内嵌回 `chainspec.json`，或让 `light_sync_state.json` 保持空对象。
- 直接把正式 `genesis-state/` 当作节点 `--base-path` 启动；真实验收必须先复制到仓库外临时目录，
  否则会把 TLS 私钥、libp2p 身份和 keystore 运行残留写进正式包。
- 让 OnChina 或 CitizenApp 把链下公权机构目录当成真源。
- 因 runtime 升级重新烘焙 genesis；runtime 升级只能走链上 `system.setCode`。

## 正确的事

- runtime 升级：编译新 WASM，发链上升级交易，genesis 不动。
- 预上线重新创世：等 CI WASM 成功，再用 `bake-chainspec.sh` 同步 plain SSOT、
  CitizenApp 轻形态、light sync checkpoint 和 genesis-state。
- 正式节点打包：prepack 复制真实 `genesis-state/`，安装包首启直接复制链数据库。
- CitizenApp 公权机构目录：内置创世快照缓存，运行后只按链上投影版本拉增量。
