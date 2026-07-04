# 创世链状态包与公民公权机构快照

## 任务目标

- 节点安装包内置创世链状态包,避免每台机器首次启动都重新物化创世状态。
- CitizenApp 内置创世区块公权机构快照包,用于首屏快速展示;后续只按链上变化增量更新本地 Isar 缓存。
- 链上仍然是公权机构唯一真源;节点本地数据库、OnChina 投影库、CitizenApp 内置快照和 Isar 缓存都只是链上状态副本或查询投影。
- 代码完成后准备本地提交,提交信息使用“创世”;不新建分支、不打 tag、不创建 PR。GitHub 推送必须在推送前再次列明远端、当前分支、CI 范围和风险,等用户明确允许后执行。

## 范围

- `citizenchain/scripts/`:生成、校验和同步创世链状态包 / CitizenApp 快照包的发布脚本。
- `citizenchain/node/src/`:节点启动时识别并加载内置创世链状态包;节点状态必须以 RPC ready 为准。
- `citizenchain/onchina/src/`:提供 CitizenApp 查询公权机构变更和详情的链上投影接口;接口不作为真源。
- `citizenapp/assets/public_institutions/`:将现有资源语义收敛为链上创世快照缓存。
- `citizenapp/lib/citizen/public/data/`:快照 manifest 校验、导入、增量同步和本地缓存。
- `memory/`:更新架构、模块文档、真源边界和验收记录。

## 明确不做

- 不创建 `创世` 分支。
- 不创建 tag。
- 不创建 PR。
- 不绕过推送确认规则。
- 不把 CitizenApp 内置快照当作真源。
- 不恢复 OnChina 本地派生公权机构作为运行态真源。
- `citizenchain/runtime/` 已在本轮按用户二次确认修改;后续再次涉及 runtime 仍须重新二次确认。

## 执行步骤

- [x] 读取模块文档和现有脚本 / 数据流。
- [x] 明确创世链状态包 manifest 与 CitizenApp 快照 manifest 字段。
- [x] 改造节点启动流程,区分进程启动、创世准备、RPC ready。
- [x] 改造发布脚本,从同一创世状态派生节点包和 CitizenApp 快照包。
- [x] 改造 CitizenApp 快照导入与增量同步语义。
- [x] 改造 OnChina 公权机构增量查询接口或补齐现有接口的链上投影边界。
- [x] 更新文档、补中文注释、清理旧口径残留。
- [x] 运行本地检查和可行的真实运行态验收。
- [x] 准备本地提交“创世”。
- [x] 推送前再次请求用户确认远端、当前分支、CI 范围和风险。
- [x] GitHub `CitizenChain WASM` 成功后下载 CI WASM artifact,执行正式 bake。
- [x] 用正式创世链启动临时节点、同步 OnChina 链上公权机构投影、生成 CitizenApp 公权机构快照。

## 验收标准

- 节点启动不再把“进程已创建”等同于“节点可用”,必须 `chain_getBlockHash(0)` 成功后才进入可用态。
- 创世链状态包 manifest 可校验 `genesis_hash`、`state_root`、`runtime_wasm_hash`、`chainspec_hash`、`public_institution_root`。
- CitizenApp 快照 manifest 记录 `snapshot_block_number`、`snapshot_block_hash`、`genesis_hash`、`state_root`、`public_institution_root`、`shard_hashes`。
- CitizenApp 只把内置快照作为本地缓存;链上变化通过增量接口更新本地持久化缓存。
- 文档明确:行政区真源仍为既有行政区真源,公权机构真源为链上状态。
- 无旧“链下公权机构目录真源”口径残留。

## 执行记录

- 节点启动流程:
  - 新增 `genesis_preparing` 生命周期状态。
  - 首启优先识别安装包资源 `genesis-state/` 或 `CITIZENCHAIN_GENESIS_STATE_DIR` 指向的创世链状态包。
  - 只复制 `chains/citizenchain/db`,不复制 keystore / network key。
  - 空资源占位目录会被忽略;出现 `manifest.json` 或 `chains/` 但包不完整时 fail-fast。
  - 首页和 OnChina 启动前都以 `chain_getBlockHash(0)` 成功作为 RPC ready 标准。
  - 真实临时节点验收发现:复制 `genesis-state` 后仍会触发 Substrate plain spec 创世存储校验,约 5 分钟 CPU;该过程不重新写库,但正式部署必须预留 RPC ready 等待窗口。
- 发布脚本:
  - `bake-chainspec.sh` 改为导出 plain spec、物化块 0、生成 CitizenApp `stateRootHash` 轻形态并输出 `target/chainspec/genesis-state/`。
  - `prepack.sh` / `prepack.ps1` 从 `CITIZENCHAIN_GENESIS_STATE_DIR` 或默认 `target/chainspec/genesis-state` 复制创世状态包到 Tauri resources。
  - Tauri 资源配置改为映射已有 `resources/` 根目录,不要求源码树存在 `resources/genesis-state/` 占位目录。
- OnChina / CitizenApp:
  - OnChina 公权机构版本接口下发 `chain_genesis_hash / chain_block_hash / chain_block_number / synced_at`。
  - `manifest_version` 改为链投影 finalized anchor + 投影数量,不再由本地 `synced_at` 单独推进。
  - `serve` 启动先比对本地投影锚点与当前链 finalized head;一致则跳过全量同步,链变化或本地无有效投影时才重新读链。
  - CitizenApp 增量同步缺少链投影版本时直接失败,不再用本机时间自造版本。
  - 公权机构快照生成器要求真实 `genesis_hash / snapshot_block_hash / state_root`,缺失则拒绝生成。
- 残留清理:
  - `run.sh` / `clean-run.sh` 移除 `ONCHINA_GOV_AUTO_RECONCILE` 开关,不再启动期本地生成公权机构。
  - 长期规则、ADR、模块文档和白皮书源码已更新到 plain SSOT + genesis-state + CitizenApp 快照增量口径。

## 验收记录

- `bash -n citizenchain/scripts/bake-chainspec.sh citizenchain/scripts/prepack.sh citizenchain/scripts/run.sh citizenchain/scripts/clean-run.sh citizenapp/scripts/citizenapp-run.sh citizenapp/scripts/check-chainspec-frozen.sh`:通过。
- `python3 -m py_compile citizenchain/scripts/check-constitution-genesis.py`:通过。
- `node --check citizenapp/tools/generate_public_institution_bundle.mjs`:通过。
- `npm --prefix citizenchain/node/frontend run build`:通过,并重新生成 `citizenchain/node/frontend/generated/local-docs.generated.ts`。
- `flutter test test/citizen/public/public_institution_bundle_loader_test.dart test/citizen/public/public_institution_sync_test.dart`:通过。
- `bash citizenapp/scripts/check-chainspec-frozen.sh`:通过;正式 `citizenapp/assets/chainspec.json` 已切换为 `stateRootHash` 轻形态。
- `cargo check --manifest-path citizenchain/Cargo.toml -p node -p onchina`:通过;输出 Polkadot SDK 既有循环提示,无编译错误。
- GitHub `CitizenChain WASM` run `28694378543`:成功;artifact `citizenchain.compact.compressed.wasm` 的 sha256 为 `70e6d1fd01b763628e8b595399487bdfe19191a44a4cfadd5255be0577b9310a`。
- `citizenchain/scripts/bake-chainspec.sh --finalize --wasm ... --public-institution-root 4923744ae6150717a2ea84be189f7842081197fe94ff7a3956cfac5a576d2318`:通过;`genesis_hash=0xc4f78c4fdec0a52bff5af160514cf447ed476a9f02eb24ba4c0df665a66cd1b7`,`state_root=0xb4a27c4c2ff18a17f1b561296cf51f72c00775f781aa826c70e1777daac32eb0`,`chainspec_hash=650c1ed8462a326e43394576eaa99f7533f9ee427cf1d80c58cd2922a82d7558`,创世物化 30 秒。
- `chain_getBlockHash(0)`:临时正式节点返回 `0xc4f78c4fdec0a52bff5af160514cf447ed476a9f02eb24ba4c0df665a66cd1b7`。
- `onchina sync-gov`:通过;链上投影 `chain_institutions=49581`,`chain_accounts=99162`,`local_institutions=49581`,`local_accounts=99162`,链上创世哈希等于正式锚点。
- 新 OnChina `serve` 启动验收:本地投影锚点等于当前 finalized head 时打印 `cid gov chain projection is current; skip startup full sync`,随后抽样对账通过并监听 `http://127.0.0.1:8975`。
- `GEN_DELAY_MS=0 ONCHINA_BASE_URL=http://127.0.0.1:8975 node citizenapp/tools/generate_public_institution_bundle.mjs ...`:通过;生成 43 个省级分片,合计 49,581 个创世公权机构,`public_institution_root=4923744ae6150717a2ea84be189f7842081197fe94ff7a3956cfac5a576d2318`。
- `npm --prefix citizenchain/node/frontend run build`:通过,同步重建本地文档索引。
- `git diff --check`:通过。
- `git diff --name-only -- citizenchain/runtime`:本轮已按用户二次确认修改 runtime,旧“不修改 runtime”记录不再适用。

## 发布边界

- 本地更新按用户当前确认推送 `origin main`,触发 GitHub CI;不创建分支、不打 tag、不创建 PR。
- 正式安装包打包前把 `target/chainspec/genesis-state/` 作为 `genesis-state/` 资源内置;该目录为生成物,不进 Git。
- 6 节点部署时逐台核对 `chain_getBlockHash(0)` 和 `stateRoot`,再启动 OnChina 服务。
