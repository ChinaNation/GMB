# chainspec 与创世状态冻结规则（铁律）

> 适用范围：全节点 `citizenchain.plain.json`、安装包内置 `genesis-state/`、CitizenApp
> `assets/chainspec.json`、`assets/light_sync_state.json`，以及任何分发给轻节点 / 钱包 / App 的创世锚点文件。

## 结论

主网创世那一刻必须由同一次 release bake 同时冻结六组资产：

1. **全节点 plain SSOT**：`citizenchain/node/chainspecs/citizenchain.plain.json`。
   它只保存 runtime WASM、genesis patch、bootNodes、properties 和 protocolId。
2. **节点创世状态包**：`genesis-state/manifest.json` 与
   `genesis-state/chains/citizenchain/db/**`。正式安装包携带经 release 清单校验的块 0
   数据库，首启复制到独立数据目录；开发/排障才允许仅按 plain chainspec 本地物化。
3. **CitizenApp 轻形态 chainspec**：`citizenapp/assets/chainspec.json`。
   它只保存轻节点联网所需字段和 `genesis.stateRootHash`，不得携带 GB 级 raw state。
4. **CitizenApp light sync checkpoint**：`citizenapp/assets/light_sync_state.json`。
   它保存 smoldot 加入 `stateRootHash` 轻形态链所必需的 finalized header 和 GRANDPA authority set。
   正式链高度仍为 0 时允许保存创世头 checkpoint；这是 PoW + `stateRootHash` 轻形态的启动锚点。
5. **CitizenApp 公权机构分片**：`citizenapp/assets/public_institutions/manifest.json` 与
   43 个省级分片。它们必须直接读取同一临时节点的块 0 状态生成，不能外部传入根值。
6. **Cloudflare 链身份锚点**：`citizenapp/cloudflare/wrangler.toml` 各环境的
   `CHAIN_GENESIS_HASH/CHAIN_STATE_ROOT`。Worker 不提供默认值，缺失或格式错误必须失败关闭。

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
- 创世状态包、App 轻节点资产、公权机构分片与 Cloudflare 锚点唯一来源：
  `citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>`，
  并必须提供 `--wasm-ci-run-id <RUN_ID> --wasm-ci-head-sha <HEAD_SHA>`；清单记录可追溯的 WASM CI run 和提交。
- CitizenApp 轻形态、checkpoint 和公权机构分片唯一来源：同一次 `bake-chainspec.sh`
  读取同一块 0 header/storage 后写出；脚本不接受外部公权机构 root。
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
   - 从同一块 0 生成 43 个公权机构分片及唯一 Merkle 根。
   - 同步并校验 Cloudflare 各环境的 genesis/state root。
   - 导出 `target/chainspec/genesis-state/manifest.json` 和
     `target/chainspec/genesis-state/chains/citizenchain/db`。
4. 打包前执行 `citizenchain/scripts/prepack.sh` 或 `prepack.ps1`，把
   `genesis-state/` 放进 Tauri resources。
5. 节点首启时优先复制内置创世状态包；没有该包时，只允许开发/排障场景回退到
   GenesisBuilder 本地物化。
6. `CitizenChain` 软件 CI 只能消费已冻结且 `artifact_stage=release` 的状态包；preview
   清单、缺失 CI provenance 或与仓库 plain spec hash 不一致时打包必须失败。
7. 无头服务器下载对应平台软件，停服务安装后保留节点身份密钥和 GRANDPA keystore；
   新数据目录由安装包内 release 状态包初始化，既有链数据库不得被覆盖。

## 当前唯一冻结锚点（2026-07-16）

- runtime 源提交：`7abac7982a5c5ee25580583d456523ce2132743e`；冻结资产提交：`80f58aa5cfe19713edfba7331ea2896cacf09b62`；GitHub `CitizenChain WASM` run：`29530114067`。
- `genesis_hash`：`0x840d5b12c541a010783e54069c9168a13d102ba63cd8f3a00263440c1803aad9`。
- `state_root`：`0x99b4cb3031baa5e87536a22190dc81bf6bf49d3678c0abae86a312268506fe09`。
- `runtime_wasm_hash`：`be4585ce369e658e6799be667ed5be692fc050f9c6196ab14c53f7dfa5dc6e70`。
- 全节点 `chainspec_hash`：`5e609d166e8517d20ec0cd2095b88825146e34e64b3ebaba54152c7bde9d1f60`。
- CitizenApp `chainspec_hash`：`973beeae264a7d2510c27957f6b2abd6b68e01860b6d976029817da4043d58b9`。
- `light_sync_state_hash`：`4b05735ed59a8ef3756bf6445f1e4fa744730d2161ad14a62be1e16856bbfb9a`。
- `public_institution_root`：`ecff487ce7d2bac6cb89d064a456187b453acd27f4bee2b140f474a48d072682`，43 省共 49,593 个机构。

正式 bake 的创世物化耗时 51 秒；公民宪法 `law_id=0`、v1 生效版和不可变条款校验通过。临时节点使用同一 CI WASM 真实启动并经 RPC 返回上述 block#0/state root，`isSyncing=false`。`bake-chainspec.sh` 的 RPC 轮询必须让内嵌 Python 正常解析响应；不得抑制解析失败后把已就绪节点误判为超时。

## 第 5 步 preview 候选（2026-07-16，非冻结值）

- 本次只完成创世准备，没有执行 CI、`--finalize`、正式冻结或正式创世；上节 2026-07-14
  锚点仍是仓库当前唯一正式锚点。
- preview 候选：`genesis_hash=0x8347f61bd28c93c4ce6d6b98f4b5a70f185841e0ac87b0bab9eb8c6caf8375ed`，
  `state_root=0x467996c0094900833e30ff0a11e668aaf234abc35acdb4917f858702642ee707`，
  `runtime_wasm_hash=c5333afdf66c5d60f58d9101c2dc49a50885773c7708dace7d64fd5f7a1079b5`，
  `chainspec_hash=0cfe7fa42d4afc34987c69357f593748ee6f4fc9d388378744ad2fa32c67ea8b`，
  `light_sync_state_hash=7caa134d4af22be0d214b383c0d0c6b8df995f5da0fcf2e2e63a8c8284034c92`，
  `public_institution_root=ecff487ce7d2bac6cb89d064a456187b453acd27f4bee2b140f474a48d072682`。
- 候选状态仅保存在忽略目录 `citizenchain/target/chainspec/`，清单明确标记
  `artifact_stage=preview` 且不伪造 CI run/commit；`prepack.sh` 已验证会拒绝该包。
- 真实隔离 node 与 OnChina/PostgreSQL 验收使用候选包副本，结束后进程、端口和仓库外
  临时目录已清理；该验收不构成正式冻结。

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
5. CitizenApp Cloudflare 各环境 `CHAIN_GENESIS_HASH` / `CHAIN_STATE_ROOT` 必须与本地
   release 冻结资产一致；bootstrap 不得存在默认 hash 回落，Worker 只发布链身份和
   bootnodes，不得成为远端 checkpoint 真源。

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
