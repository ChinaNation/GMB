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

1. 把已审查的 `main` 提交固定为直接指向该提交的轻量候选 tag；只推送该 tag，不提前移动远端
   `main`，避免误触发其它软件 CI。以该 tag 手动执行 GitHub `CitizenChain WASM`
   普通源码构建，且运行必须成功。
2. 使用 `citizenchain/scripts/download-wasm.sh` 明确传入本次运行的 run ID、40 位
   head SHA 和候选 tag；脚本先确认远端 `refs/tags/<候选名>` 直接指向该提交，再核对
   workflow、`workflow_dispatch` 事件、完成状态、conclusion、head SHA、ref 和唯一未过期 artifact 后，下载同一运行的
   `citizenchain.compact.compressed.wasm`。禁止选择“最新成功”运行。
3. 执行：

   ```bash
   citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM> \
     --wasm-ci-run-id <RUN_ID> \
     --wasm-ci-head-sha <HEAD_SHA>
   ```

4. 脚本必须完成：
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
5. 打包前执行 `citizenchain/scripts/prepack.sh` 或 `prepack.ps1`，把
   `genesis-state/` 放进 Tauri resources。
6. 节点首启时优先复制内置创世状态包；没有该包时，只允许开发/排障场景回退到
   GenesisBuilder 本地物化。
7. `CitizenChain` 软件 CI 只能消费已冻结且 `artifact_stage=release` 的状态包；preview
   清单、缺失 CI provenance 或与仓库 plain spec hash 不一致时打包必须失败。
8. 无头服务器下载对应平台软件，停服务安装后保留节点身份密钥和 GRANDPA keystore；
   新数据目录由安装包内 release 状态包初始化，既有链数据库不得被覆盖。

## 当前唯一正式创世锚点（2026-07-26）

- runtime 源提交：`ac6de21b2432f52f45f1767f88f4e6833a2c79d0`；冻结资产提交：
  `a5204a39b90bf83daab8b91d83da6dd150269d9a`；GitHub `CitizenChain WASM`
  run：`30190068925`，artifact：`8628330093`。
- `genesis_hash`：
  `0xe8f4067de2323dc27b2a2c409fa4b3ab882e4e88dfa6f4a81355f51f8cf8eb45`。
- `state_root`：
  `0xbdc2593a538b7010717ac475b0b59973dd57c77d35683c4e7d9b8058b9ae18f9`。
- `runtime_wasm_hash`：
  `a838dd763c1c7003aca1edf177738d85b64936bbc1ba98dda7da348cc57d0d1a`。
- 全节点 `chainspec_hash`：
  `3e79942fabad332fee5e8692b503c393005730bc5b2d85b9d38694833fada652`。
- CitizenApp `chainspec_hash`：
  `6d5fce5d349d99c8521a5a8aa690cdb038d6594003cebf55ce01cc7825861310`。
- `light_sync_state_hash`：
  `95beb873cce95ca1744193c0aa0c7023a4b4070346b8ba68758d7a140d8a61c0`。
- `public_institution_root`：
  `c21f99f5bd40bc3c9fcee9439de9f6902c98212b2510dd7440c9630284ab939f`，
  43 省共 49,593 个公权机构。

同一次 release bake 生成的节点 plain SSOT、创世状态包、CitizenApp 轻形态 chainspec、
light-sync checkpoint、公权机构分片和 Cloudflare 链身份已经交叉校验通过。2026-07-26
本机正式节点与固定远端 RPC 均返回上述创世哈希和状态根；本机正式链审计时 best/finalized
均为 block #6、`isSyncing=false`。正式创世已经完成，不得再次执行 `--finalize` 覆盖上述
锚点；后续 runtime 升级只能通过正式链 `system.setCode`，除非用户另行明确批准正式硬分叉。

## S7A 重新创世源码 preview（2026-07-30，已被 CI-WASM preview 取代）

- 候选源码提交：
  `450e47af19851b9176a5e8bda128aba455bda482`。该提交已经完成创世前全仓扩展审查、
  格式清零、Rust 全 workspace Clippy/测试、CitizenApp/Worker/OnChina/官网门禁和
  production release 冷构建。
- preview 候选：
  - `genesis_hash`：
    `0x37a3913895b6b2eda0e9fe242639338cc55ee1023e3caf25e31a178af990fa90`。
  - `state_root`：
    `0xfdc23210da6d85e69eb2e107e291a1be2edf4c413516af0e12c35a40be4ed2f4`。
  - `runtime_wasm_hash`：
    `4f553d22433d1348a133e44468d128ad47803c86e0e3542e14b69ec58a39413e`。
  - 全节点 `chainspec_hash`：
    `ea951f6372facc3d9ad9b748a6a436f516b36a2a4b614c3688ae0525897e5cb8`。
  - `light_sync_state_hash`：
    `0f3e54e599cfe987596894278f35d64e7a9077ea93c260f2e0d2d20ed09afa16`。
  - `public_institution_root`：
    `c21f99f5bd40bc3c9fcee9439de9f6902c98212b2510dd7440c9630284ab939f`。
- `bake-chainspec.sh` 真实物化块 0 用时 131 秒；宪法 `law_id=0`、v1 生效版和不可变条款
  校验通过。同一次候选产物的 plain spec、App 轻形态、checkpoint、公权分片、
  Cloudflare 派生配置和 genesis-state manifest 交叉哈希校验通过。
- 隔离节点直接复制候选 genesis-state 启动后为 0 peers、`isSyncing=false`；RPC 逐页统计
  块 0 storage 为 49,593 个公权机构 / 99,232 个公权协议账户，公民链基金会另计
  1 个私权机构 / 2 个私权协议账户，全创世总计 49,594 个机构 / 99,234 个机构协议账户。
- 候选只保存在忽略目录 `citizenchain/target/chainspec/`，
  `artifact_stage=preview`，CI run/SHA 均为空；没有执行 `--finalize`，没有覆盖当前正式
  锚点，没有推送或触发 CI。隔离节点已停止，RPC 19945 已关闭；仓库外临时目录已移入
  macOS 废纸篓，可恢复。
- S7B-A 已把 WASM workflow 改为上传失败即整次运行失败；三个 WASM 文件上传前必须
  存在、非空并把大小与 Blake2-256 写入 CI Summary。历史 artifact 不再在新上传前
  删除，由 30 天 retention 自动过期，避免“旧证据已删、新上传失败但 CI 仍成功”。
- 下载入口必须钉死 run ID、head SHA 和远端轻量候选 tag，并确认 tag 直接指向预期提交，
  然后在系统临时目录完整核验三个 WASM
  后才替换本地忽略目录 `citizenchain/target/wasm-ci/`。下一步须另获远端操作许可，
  只推不可变候选 tag，不移动 `origin/main`；然后在该 tag 上手动执行唯一
  `CitizenChain WASM` CI。CI 成功后使用该次运行的
  `citizenchain.compact.compressed.wasm` 生成 CI-WASM preview。CI 与本机源码 WASM
  字节或创世结果不一致时必须先停止并核对提交、锁文件、工具链、runtime 版本和 API，
  不能直接执行 release `--finalize`。

## S7B-B GitHub CI-WASM preview（2026-07-30，当前候选、尚未正式冻结）

- 轻量候选 tag：
  `genesis-wasm-candidate-20260730-b77ca3c1`，直接指向提交
  `b77ca3c1ce7e12fe9df87e15a29444f7650bff7c`；只推送了该 tag，
  `origin/main` 未移动，未触发任何自动 workflow。
- 手动普通源码构建：
  - workflow：`CitizenChain WASM`
  - run ID：`30589266930`
  - job ID：`91027744279`
  - artifact ID：`8777906747`
  - `runtime_upgrade=false`，未携带升级链版本或创世哈希
  - run、编译、摘要校验和 artifact 上传全部成功，耗时 9 分 39 秒
- GitHub artifact 精确包含三个非空 WASM：
  - `citizenchain.wasm`：6,860,169 字节，
    SHA-256 `0d1c2fb5dc4c3f7d1486a10f6efb4ed84de03c4eca2b66a6b604df7fd666cf51`，
    Blake2-256 `ed3a7a448d6946c008ee6cafcfbaf3e66427e6741cfc97ab3db5e9909586b814`
  - `citizenchain.compact.wasm`：6,581,358 字节，
    SHA-256 `74c3553b5cbcbda3c7e055145b01e826f1d160ce62bc0e88441bf5b67e52d035`，
    Blake2-256 `e772daba5ee2280408a17be3460c9fd12fae421bb3b193e059590487826a21d2`
  - `citizenchain.compact.compressed.wasm`：1,162,535 字节，
    SHA-256 `eecd43eb87815e2fe7601ef02856717b3ba7a1204f59998321887a3388fa4e91`，
    Blake2-256 `8d92e92ccd52693bce9ae915bae74600d58f6581d8e800396ef9bcfbf0b5f93e`
- 本机与 CI 使用同一提交、同一 `Cargo.lock` 和 Rust 1.97.1；macOS ARM 与 Ubuntu
  runner 产出的 WASM 函数/类型排列和调试名称布局不同，但 `runtime_version`、
  `runtime_apis`、producer 和 target features 完全一致。已按门禁停止核对，最终确认
  GitHub CI compressed WASM 才是正式创世唯一权威输入，本机 WASM 不再参与冻结。
- 使用上述 CI compressed WASM 执行非 finalize preview：
  - `genesis_hash`：
    `0x278e68bced2dabf9690701188272da22d216fdaa2c617e7dcbe100df3e8bcbfa`
  - `state_root`：
    `0xa5386e7c0a0222fd030250b533bf73e78e947aec9f6a98dea7c1d5d64881c8c2`
  - `chainspec_hash`：
    `df2e5a28d99084ec5bcbed28db21ec3eecacbf364b421ce1fb47628c897387fe`
  - `light_sync_state_hash`：
    `a1a5d43046b379e8168a9651c41a7bbadf1299971252b4e9f99e7701056f8045`
  - `public_institution_root`：
    `c21f99f5bd40bc3c9fcee9439de9f6902c98212b2510dd7440c9630284ab939f`
- manifest 为 `artifact_stage=preview`，精确记录上述 run ID/head SHA；`:code`
  1,162,535 字节并与 CI compressed WASM 逐字节一致。块 0 物化耗时 131 秒，宪法、
  白名单、节点/App/checkpoint/43 个公权分片/Cloudflare 交叉检查全部通过。
- 隔离节点复制该 genesis-state 后真实启动；RPC 返回 0 peers、`isSyncing=false`、
  runtime `specVersion=0`，块 0 哈希和状态根与候选一致。逐页统计公权 49,593 个机构 /
  99,232 个账户、基金会 1 个机构 / 2 个账户，总计 49,594 个机构 / 99,234 个账户。
  节点已经停止，RPC 19945 已关闭，验收目录已移入 macOS 废纸篓。
- 本次 workflow 唯一注解指出 `actions/upload-artifact@v4` 仍声明 Node 20，GitHub
  runner 强制以 Node 24 运行。官方当前 `v7.0.1` 已原生使用 Node 24；正式冻结前必须
  先升级 action、重新生成候选 tag 并重跑唯一 WASM CI，不能直接拿本次有弃用注解的
  run 执行 `--finalize`。

## 历史冻结锚点（2026-07-16，已被正式创世替代）

- runtime 源提交：`7abac7982a5c5ee25580583d456523ce2132743e`；冻结资产提交：`80f58aa5cfe19713edfba7331ea2896cacf09b62`；GitHub `CitizenChain WASM` run：`29530114067`。
- `genesis_hash`：`0x840d5b12c541a010783e54069c9168a13d102ba63cd8f3a00263440c1803aad9`。
- `state_root`：`0x99b4cb3031baa5e87536a22190dc81bf6bf49d3678c0abae86a312268506fe09`。
- `runtime_wasm_hash`：`be4585ce369e658e6799be667ed5be692fc050f9c6196ab14c53f7dfa5dc6e70`。
- 全节点 `chainspec_hash`：`5e609d166e8517d20ec0cd2095b88825146e34e64b3ebaba54152c7bde9d1f60`。
- CitizenApp `chainspec_hash`：`973beeae264a7d2510c27957f6b2abd6b68e01860b6d976029817da4043d58b9`。
- `light_sync_state_hash`：`4b05735ed59a8ef3756bf6445f1e4fa744730d2161ad14a62be1e16856bbfb9a`。
- `public_institution_root`：`ecff487ce7d2bac6cb89d064a456187b453acd27f4bee2b140f474a48d072682`，43 省共 49,593 个机构。

正式 bake 的创世物化耗时 51 秒；公民宪法 `law_id=0`、v1 生效版和不可变条款校验通过。临时节点使用同一 CI WASM 真实启动并经 RPC 返回上述 block#0/state root，`isSyncing=false`。`bake-chainspec.sh` 的 RPC 轮询必须让内嵌 Python 正常解析响应；不得抑制解析失败后把已就绪节点误判为超时。

## 第 5 步 preview 候选（2026-07-16，历史非冻结值）

- 本次只完成创世准备，没有执行 CI、`--finalize`、正式冻结或正式创世；上节 2026-07-14
  锚点在当时仍是仓库发布锚点。该记录现已被 2026-07-26 正式创世锚点取代。
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
3. `citizenchain/node/src/onchina_proc.rs` 启动 OnChina 前必须确认本机链 RPC 已就绪。
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
