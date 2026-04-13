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

## 防御措施

1. **文件**：`wuminapp/assets/chainspec.json.sha256` 记录冻结版本的 sha256。
   sha256 基于 `jq -cS 'del(.bootNodes, .lightSyncState)'` 后的内容计算，
   bootNodes 和 lightSyncState 变更不影响校验（两者都不参与 genesis hash）。
2. **脚本**：`scripts/check-chainspec-frozen.sh` 做完整性校验（已排除 bootNodes）。
3. **git hook**：`.githooks/pre-commit` 调用上述脚本校验创世内容。
   bootNodes 域名变更可正常提交，genesis 内容变更会被拦截。
   启用：`git config core.hooksPath .githooks`。
4. **CI**：`.github/workflows/wuminapp-ci.yml` 在 check 和 android job 开头
   都调用 `scripts/check-chainspec-frozen.sh`。
5. **启动脚本**：`wuminapp/scripts/wuminapp-run.sh` 启动前先校验哈希（已排除 bootNodes），
   不一致直接退出。

## 如果真的需要改（硬分叉流程）

1. 写 ADR 说明硬分叉理由和影响范围。
2. 所有全节点同步停机 → 清数据 → 换新 chainspec.json 重启。
3. 所有钱包 / 轻节点 / App 同步发版。
4. 更新 `wuminapp/assets/chainspec.json.sha256`。
5. `git commit --no-verify`（绕过 pre-commit 守卫），commit message 必须
   包含 `[HARDFORK]` 标签。

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
