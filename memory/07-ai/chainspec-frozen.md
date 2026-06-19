# chainspec.json 冻结规则（铁律）

> 适用范围：`wuminapp/assets/chainspec.json` 及任何分发给轻节点 / 钱包 / App
> 的 chain specification 文件。

## 结论

**chainspec.json 在主网创世那一刻就必须冻结，之后任何人、任何脚本、任何 CI
都不许修改。** runtime 升级走链上 `system.setCode` 交易，全节点和轻节点自动
跟上，**chainspec.json 永远不动**。

唯一例外：真的硬分叉（改 genesis 配置、另起一条链）。这种情况下所有节点必须
清数据重启、所有客户端必须同步更新，属于重大事件，需要正式 ADR 决议。

## 为什么

1. `chainspec.json` 里的 raw genesis state 决定了链的 **genesis hash**。
2. Substrate / libp2p 的通知协议名是
   `/<genesis_hash>/block-announces/1`、`/<genesis_hash>/transactions/1`、
   `/<genesis_hash>/grandpa/1` 等等。
3. genesis hash 不同 → 协议名不同 → 节点之间 gossip 握手直接
   `Substream(Substream(ProtocolNotAvailable))` → 永远连不上。
4. **runtime wasm (`:code:`) 是 genesis state 的一部分**。每次 runtime 重编
   出来的 wasm 字节都不完全一样，所以只要跑一次 `build-spec --raw`，
   chainspec.json 的 genesis hash 就会变。这就是为什么不能"启动时自动同步
   chainspec"。
5. runtime 升级走链上 `system.setCode` 交易：交易进块后，所有全节点和轻节点
   从下一个区块开始自动用新 wasm 执行，**genesis hash 不变**，chainspec.json
   不需要任何动作。

## 踩过的坑（2026-04-07）

- `wuminapp/scripts/wuminapp-run.sh` 启动前会跑
  `citizenchain build-spec --raw > wuminapp/assets/chainspec.json`。
- `.github/workflows/wuminapp-ci.yml` 打包 APK 前会从另一个 CI 下载
  `citizenchain-chainspec` artifact 覆盖 chainspec.json。
- 两条路径都会在 runtime 重编后生成一份和线上全节点 genesis 对不上的新
  chainspec。手机端 smoldot 连 prczss / nrcgch，WSS 握手成功但
  gossip-open 报 `ProtocolNotAvailable`，runtime 下载报
  `StorageQuery { errors: [] }`（因为没有可用 peer），App 永远连不上链。
- 修复：删除两处自动重生成逻辑，chainspec.json 回退到 `97086e8`（创世）
  的版本，记录 sha256 到 `wuminapp/assets/chainspec.json.sha256`，加
  pre-commit hook + CI 校验 + 脚本启动前校验三道防线。

## 单一权威源(SSOT)— 2026-06-08 重构

**唯一权威创世 = `citizenchain/node/chainspecs/citizenchain.raw.json`,其 `:code` 永远 = CI WASM。**
所有消费者从它派生,谁都不再各自造创世:

- 桌面端(`run.sh` / `clean-run.sh`)`include_bytes!` 内嵌该 SSOT;clean-run 只清 db,不再本地现造创世(旧版用本地 WASM 会与线上 CI WASM 分叉)。
- `wuminapp/assets/chainspec.json` 是 SSOT 的派生副本,创世部分必须逐字节等价。
- `wumin`(冷签)零创世依赖,只签二维码 payload(genesis hash 在 payload 里)。

旧的 `wuminapp/assets/chainspec.json.sha256` 冻结常量已删除——守卫改为直接和 SSOT 比对,SSOT 即唯一基准。

## 防御措施

1. **SSOT 守卫脚本**:`scripts/check-chainspec-frozen.sh` 比对
   `wuminapp/assets/chainspec.json` 与 SSOT 的创世部分(`jq -cS 'del(.bootNodes,.lightSyncState)'` 后 sha256),不一致即拒绝。
2. **git hook**:`.githooks/pre-commit` 调用上述脚本。启用:`git config core.hooksPath .githooks`。
3. **CI**:`.github/workflows/wuminapp-ci.yml` 在 check 和 android job 开头都调用该脚本;
   其 paths 已加 `citizenchain/node/chainspecs/**`,SSOT 变更会触发 wuminapp 重建 + 守卫。
4. **启动脚本**:`wuminapp/scripts/wuminapp-run.sh` 启动前调用该脚本,不一致直接退出。
5. **重新创世唯一入口**:`citizenchain/scripts/bake-chainspec.sh`(仅预上线用)——
   下载 CI WASM → `export-chain-spec --chain citizenchain-fresh --raw` → 断言 `:code`==CI WASM →
   同时写 SSOT 与 wuminapp 副本,保证两者永远同步。

2026-06-19 预上线重新创世收口记录:

- 本次使用本地 release WASM 重新导出 fresh raw chainspec,流程沿用 `bake-chainspec.sh` 的断言口径。
- `citizenchain/node/chainspecs/citizenchain.raw.json` 与 `wuminapp/assets/chainspec.json` sha256 均为 `cdf74fd89148ab8d681b020c65f59ff8f93e238a1404da44a7b47fae8bb4757a`。
- `scripts/check-chainspec-frozen.sh` 通过;bootNodes 保持 44 个,伊犁省权威节点域名为 `prcyls.crcfrcn.com`。

## 如果真的需要改(预上线重新创世 / 硬分叉流程)

1. 写 ADR 说明理由和影响范围。
2. runtime 改动 → 推送(commit message 含「重新创世」让 wasm CI 跳过版本守卫,保 spec_version)→ wasm CI 出新 WASM。
3. 跑 `citizenchain/scripts/bake-chainspec.sh`:用 CI WASM 重新烘焙 SSOT 并同步 wuminapp 副本(脚本内置 `:code`==CI WASM 断言,并保留当前 SSOT 的权威节点 bootNodes)。
4. 提交两份 chainspec → 推送(触发 CitizenChain 节点 CI + WuMinApp CI;SSOT 守卫自动通过因两者已同步)。
5. 所有全节点 `fuwuqi.sh q <ip>` 清数据重部署;所有钱包 / 轻节点 / App 同步发版。
6. 守卫无需手改常量(已无 `.sha256`);如确有特殊绕过需求,`git commit --no-verify`。

## 绝对不能做的事

- ❌ 在启动脚本里 `build-spec --raw > chainspec.json`
- ❌ 在 CI 里从 artifact 下载 chainspec.json 覆盖仓库版本
- ❌ 每次 runtime 升级都"顺手"重新导出 chainspec
- ❌ 用 `--chain=local` / `--chain=dev` 导出的 chainspec 覆盖主网 chainspec
- ❌ 用 `build-spec --raw` 只为改 `bootNodes`（会连带改 `:code:`）

## 正确的事

- ✅ runtime 升级 = 编译新 wasm + 发 `system.setCode` 交易
- ✅ 全节点升级 client 代码 = 重编二进制 + 重启，chainspec 不动
- ✅ 轻节点升级 = 发新版 App，chainspec 不动
- ✅ 直接编辑 chainspec.json 中的 `bootNodes` 字段（域名变更、增删节点）— 不影响 genesis hash，校验已排除此字段
- ✅ 更新 chainspec.json 中的 `lightSyncState` 字段（轻节点 checkpoint）— 不影响 genesis hash，校验已排除此字段
