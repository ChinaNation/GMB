# chainspec 与创世状态冻结规则（铁律）

> 适用范围：全节点 `citizenchain.plain.json`、安装包内置 `genesis-state/`、CitizenApp
> `assets/chainspec.json`，以及任何分发给轻节点 / 钱包 / App 的创世锚点文件。

## 结论

主网创世那一刻必须同时冻结三件东西：

1. **全节点 plain SSOT**：`citizenchain/node/chainspecs/citizenchain.plain.json`。
   它只保存 runtime WASM、genesis patch、bootNodes、properties 和 protocolId。
2. **创世链状态包**：`genesis-state/` 安装包资源，来源于
   `citizenchain/scripts/bake-chainspec.sh` 临时节点物化块 0 后导出的
   `target/chainspec/genesis-state/`。
3. **CitizenApp 轻形态 chainspec**：`citizenapp/assets/chainspec.json`。
   它只保存轻节点联网所需字段和 `genesis.stateRootHash`，不得携带 GB 级 raw state。

创世后 runtime 升级一律走链上 `system.setCode` 交易。除非正式硬分叉，任何脚本、CI、
启动入口都不得重烤创世锚点。

## 为什么

1. genesis hash 决定 Substrate / libp2p 通知协议名。
2. 创世状态必须由同一份 CI WASM 和同一份 plain spec 冻结；raw chainspec 不作为仓库和
   App 资产。
3. 节点安装包内置已物化的 RocksDB 创世状态，可以避免每台用户电脑首启都跑一次全量
   GenesisBuilder。
4. CitizenApp 只需要轻节点创世头校验信息，公权机构目录另走“内置快照 + 链上投影增量”。

## 单一权威源

- 全节点创世配置唯一真源：`citizenchain/node/chainspecs/citizenchain.plain.json`。
- 创世状态包唯一来源：`citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>`。
- CitizenApp 轻形态唯一来源：同一次 `bake-chainspec.sh` 读取块 0 header 后写出。
- 公权机构唯一真源：链上 `PublicManage::Institutions` /
  `PublicManage::InstitutionAccounts`；App 内置快照和 OnChina PostgreSQL 都只是缓存。

## 正式创世流程

1. GitHub `CitizenChain WASM` CI 成功后，下载同一提交的
   `citizenchain.compact.compressed.wasm`。
2. 执行：

   ```bash
   citizenchain/scripts/bake-chainspec.sh --finalize --wasm <CI_WASM>
   ```

3. 脚本必须完成：
   - 导出 fresh plain spec。
   - 启动临时节点物化块 0。
   - 通过 `check-constitution-genesis.py --rpc --expect-code-file <CI_WASM>`。
   - 写回 `citizenchain/node/chainspecs/citizenchain.plain.json`。
   - 写回 `citizenapp/assets/chainspec.json` 的 `stateRootHash` 轻形态。
   - 导出 `target/chainspec/genesis-state/manifest.json` 和
     `target/chainspec/genesis-state/chains/citizenchain/db`。
4. 打包前执行 `citizenchain/scripts/prepack.sh` 或 `prepack.ps1`，把
   `genesis-state/` 放进 Tauri resources。
5. 节点首启时优先复制内置创世状态包；没有该包时，只允许开发/排障场景回退到
   GenesisBuilder 本地物化。

## 防御措施

1. `citizenapp/scripts/check-chainspec-frozen.sh` 校验 node plain SSOT 与 CitizenApp
   轻形态阶段是否匹配；正式发布设置 `CITIZENAPP_REQUIRE_STATE_ROOT=1`。
2. `citizenchain/node/src/home/process/mod.rs` 在启动节点前尝试安装内置
   `genesis-state/`，并在 RPC `chain_getBlockHash(0)` 成功前保持“创世准备中”。
3. `citizenchain/node/src/onchina_proc/mod.rs` 启动 OnChina 前必须确认本机链 RPC 已就绪。
4. `citizenchain/scripts/prepack.sh` / `prepack.ps1` 负责把正式创世状态包放入安装包资源。

## 绝对不能做的事

- 在启动脚本里重新 `build-spec --raw` 覆盖主网创世。
- 把 raw chainspec 重新作为仓库 SSOT 或 App 资产。
- 让 OnChina 或 CitizenApp 把链下公权机构目录当成真源。
- 因 runtime 升级重新烘焙 genesis；runtime 升级只能走链上 `system.setCode`。

## 正确的事

- runtime 升级：编译新 WASM，发链上升级交易，genesis 不动。
- 预上线重新创世：等 CI WASM 成功，再用 `bake-chainspec.sh` 同步 plain SSOT、
  CitizenApp 轻形态和 genesis-state。
- 正式节点打包：prepack 复制真实 `genesis-state/`，安装包首启直接复制链数据库。
- CitizenApp 公权机构目录：内置创世快照缓存，运行后只按链上投影版本拉增量。
