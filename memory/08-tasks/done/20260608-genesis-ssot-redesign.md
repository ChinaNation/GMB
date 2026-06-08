# 任务卡:创世单一权威源(SSOT)重构 + 消除多端 chainspec 漂移

## 任务需求

重新创世暴露出 chainspec 被复制/再造成 3 份(node 内嵌、clean-run 现造、wuminapp 副本),靠人手同步 → wuminapp 漂成旧创世、clean-run 用本地 WASM 会与线上 CI-WASM 创世分叉。按"单一权威源"重构。

所属模块:citizenchain/scripts + wuminapp(Blockchain Agent + Mobile Agent)

## 设计(已与 user 确认)

SSOT = `citizenchain/node/chainspecs/citizenchain.raw.json`,`:code` 永远 = CI WASM。所有端从它派生,谁都不再造创世。

## 必须遵守

- [feedback_chainspec_frozen]:上线后冻结,bake 脚本只在预上线重新创世用;steady-state 走链上 setCode
- [feedback_no_compatibility]:无兼容层,旧创世彻底弃用
- wumin(冷签)零创世依赖,**不得引入**
- 不动链端 runtime 代码

## 落地清单

1. 新增 `citizenchain/scripts/bake-chainspec.sh`:下载 CI `citizenchain-wasm` → `WASM_FILE=<compact.compressed> export-chain-spec --chain citizenchain-fresh --raw` → 断言 `:code` blake2==CI wasm + bootNodes=44 + name/id/chainType/protocolId/properties 与现 SSOT 一致 → 写 SSOT。(固化本轮手动流程)
2. wuminapp 去重:`assets/chainspec.json` 改为**构建期从 SSOT 复制**(gitignore + 删除已 commit 的副本 + 删 `chainspec.json.sha256` 常量);`wuminapp-run.sh` 与 `wuminapp-ci.yml` 在 flutter build 前 copy SSOT;`wuminapp-ci.yml` paths 加 `citizenchain/node/chainspecs/**`(SSOT 变更触发 wuminapp 重建)
3. `clean-run.sh` 改为:清 db + 用默认 SSOT 启动(删掉本地现造 fresh genesis 的分叉来源);保留 node-key/keystore/tls
4. `run.sh` 不变(已用 SSOT)
5. wumin 不动

## 验收标准

- `bash -n` 所有改动脚本通过
- wuminapp `assets/chainspec.json`(派生后)genesis 部分(剔除 bootNodes/lightSyncState)== SSOT genesis 部分
- wuminapp `flutter analyze` 通过(若环境可跑)
- 全仓无第二份 commit 的 chainspec 副本;wumin 仍零 genesis 引用
- 文档/记忆更新(chainspec-frozen.md 补 SSOT 流程)

## 执行记录(2026-06-08 完成)

- [x] 新增 `citizenchain/scripts/bake-chainspec.sh`(force-add,citizenchain/scripts 被 gitignore)
- [x] `scripts/check-chainspec-frozen.sh` 改为 SSOT 比对(wuminapp 创世 == node SSOT 创世),删 `.sha256` 常量依赖
- [x] `wuminapp/assets/chainspec.json` 同步为新 SSOT(:code beaaadeb,创世 sha f4bfdaac== SSOT);删 `chainspec.json.sha256`
- [x] `wuminapp/scripts/wuminapp-run.sh` 改调 SSOT 守卫
- [x] `wuminapp-ci.yml` paths 加 `scripts/check-chainspec-frozen.sh` + `citizenchain/node/chainspecs/**`
- [x] `clean-run.sh` 改为清 db + 用内嵌 SSOT 启动(删本地现造创世的分叉源)
- [x] `run.sh` 不变(本就用 SSOT);`wumin` 不动(零 genesis 依赖)
- [x] `memory/07-ai/chainspec-frozen.md` 补 SSOT 模型 + bake 流程
- [x] 验证:4 脚本 `bash -n` 过;SSOT 守卫运行通过;wuminapp chainspec 合法 JSON;全仓仅 SSOT+wuminapp 两份且创世一致

决策:wuminapp chainspec 选择「committed + SSOT 守卫」而非 gitignore 派生(fail-closed,不会缺资产构建失败,复用既有 hook/CI 接线)。仍是单一权威:bake 同步两者 + 守卫防漂移。如需彻底去副本(gitignore+构建期生成)可另开卡。
不在本卡范围:`fuwuqi.sh` 的逐省密钥编辑(user 每次部署自行改),工作树保留不提交。
未跑 `flutter analyze`(本次零 Dart 改动,仅脚本+资产),CI 会跑。
