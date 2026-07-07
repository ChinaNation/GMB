# citizenchain clean-run.sh 二选一启动模式

## 任务需求

- `citizenchain/scripts/clean-run.sh` 开头加菜单,两个选项都删本机链数据 + 链上中国运行时 PG,区别在启动方式:
  - **[1] 烘焙创世 + 从网络同步**:删库后用冻结 SSOT(`node/chainspecs/citizenchain.plain.json`)启动,不重新创世,作为新节点从区块链网络同步区块。
  - **[2] 重新创世 + 新链**:删库后用当前源码 `genesis_build` 现造创世启动(= 现行 clean-run 行为),一条独立本地新链。
- 两选项都删:`chains/citizenchain/db`(区块+状态)+ `onchina-pgdata`(链上中国运行时 PG)。
- **保留**:`node-key`(PeerId)/`keystore`(矿工密钥)/`tls`(WSS 证书)= 与创世无关的节点身份(用户确认保留)。
- **不动**:`china.sqlite`(行政区只读源数据)。

## 建议模块

- `citizenchain/scripts/clean-run.sh`

## 影响范围

- 仅改本地开发启动脚本;不改 runtime/node 源码,不改 `run.sh`。
- 公共段(杀进程、删 db + PG、建 onchina/前端/内嵌 PG/TLS/dev 签名 env)两模式复用,只有末尾启动 env 分支。

## 主要风险点

- 模式 1 从网络同步依赖:冻结创世 = 现网创世、且 bootnode 可达;本机为唯一节点时同步不到对等数据(机制决定,脚本仅打印提示)。
- 删 PG 是链上中国运行时库(可从链重投影),非 `china.sqlite` 源数据,不可混淆。
- `set -euo pipefail` 下 `read` 交互;支持 `./clean-run.sh 1|2` 参数免交互;非法输入报错退出。

## 是否需要先沟通

- 否。菜单语义与"保留节点身份"用户已确认。

## 预计修改目录

- `citizenchain/scripts/`:改 `clean-run.sh`;脚本。

## 分步骤技术方案

1. 头部注释改为"清库后二选一:烘焙创世同步 / 重新创世新链",写清删/留清单。
2. `set -euo pipefail` 后加模式选择:`MODE="${1:-}"`,空则 `read`;`case` 校验 1/2,非法退出。
3. 公共段(杀进程 → 删 db + PG → 建 onchina/PG/env)保持不变。
4. 末尾按模式设启动 env:
   - 模式 2:`WASM_BUILD_FROM_SOURCE=1` + `CITIZENCHAIN_CHAIN_SPEC=citizenchain-fresh`。
   - 模式 1:不设二者(= run.sh 那套,默认冻结 SSOT),启动前打印"需可达 bootnode / 唯一节点同步不到"提示。
5. 验收:`bash -n` 语法检查;人工核对两分支 env 与 run.sh/现行 clean-run 一致。

## 当前执行状态

- [x] 头部注释重写为二选一(删/留/不动清单齐全)。
- [x] `set -euo pipefail` 后加步骤 0 菜单:`MODE="${1:-}"` 空则 `read`;`case` 校验 1/2,非法 `>&2` 报错 `exit 1`;菜单在 `trap`/删库之前，非法输入干净早退。
- [x] 公共段(杀进程、删 db + PG、建 onchina/PG/env）不变；PG 删除提示改中性（重投影，非"新创世"）。
- [x] 步骤 4 按 `MODE` 分支：模式 2 = `WASM_BUILD_FROM_SOURCE=1` + `CITIZENCHAIN_CHAIN_SPEC=citizenchain-fresh`；模式 1 = 不设二者（默认冻结 SSOT）+ 打印同步前提提示。
- [x] 验收：`bash -n` 通过；传参 `9` 在删任何数据前报错退出。
- [ ] 待用户真机验证：模式 2 出新链自挖矿；模式 1 有可达 bootnode 时从网络同步。
