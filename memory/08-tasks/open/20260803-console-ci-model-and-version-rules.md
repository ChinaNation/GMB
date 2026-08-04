# 公民控制台 CI 模型整改：令牌、独立标签、版本规则、触发方式、WASM 归档

任务需求：把公民控制台的 CI 链路整改成「本地推送不触发任何 CI，只有按钮触发对应 CI」，
并让每个功能成为彻底独立、互不干涉的任务。

所属模块：citizenconsole（不在 Git 版本库内）、.github/workflows

必须遵守：
- 不可突破模块边界
- 不可绕过既有契约
- 不可擅自修改安全红线
- 不清楚逻辑时先沟通

## 已定口径

1. **`pull_request` 触发保留**，特别是 `guardrails`（文档更新 + 残留清理 + pallet 注册表一致性）
   和 `golden-vectors`（金标向量跨端一致）。
2. **版本规则统一**：`a.b.c`，起点 `1.0.0`，`c ≤ 99`、`b ≤ 99`、`a` 无上限。
   `ci` → `c+1`，满 99 进位到 `b`；`release` → `b+1` 且 `c=0`，`b` 满 99 进位到 `a`。
   适用公民链桌面端、CitizenApp、CitizenWallet（后两者 `+CODE` 仍每次 +1，单调不受进位影响）。
   **runtime 除外**——它走链上 `spec_version` 整数 +1，与语义化版本无关。
3. **禁止别人推 main**：GitHub 仓库层解决（Collaborators 清空 + Ruleset 锁 main），非代码问题。
   注意：直推 main 不存在「合并」环节，「不合并」只对 PR 成立。
4. **WASM 不需要 Release 才能上链**：CI 成功即产物就位，「开发升级」直接拉最近成功 run 的
   artifact。但建议另加按 `spec_version` 的不可变 Release 作永久档案——artifact
   `retention-days: 30`，超期后历史升级的原始 WASM 字节不可复核。
5. **每功能彻底独立**：一功能一标签，随时可关，互不阻塞；44 个节点部署各自独立。
   唯一限制是同一功能不允许同时开两个任务。
6. **CI 触发走「甲」**：`GH_TOKEN` + `workflow_dispatch`，不走「标记提交 + 零登录」方案。

## 分步

| 步 | 内容 | 状态 |
|---|---|---|
| 一 | `GH_TOKEN` 收进控制台 Keychain 并注入 CI 动作；修误导文案 | 完成 |
| 二 | 标签彻底独立：随时可关、去全局生产串行、去重键加 `nodeId`、运行中可停、部署标签带节点名；删净 `canDetach` + `CITIZENCONSOLE_REMOTE_RUN_STARTED` | 完成 |
| 三 | 版本号两个 bump 函数合并为单源 + 99 进位 | 完成 |
| 四 | 四个 workflow 去 `on.push`、WASM 加 `workflow_dispatch`、保留 `pull_request`；清掉提交名路由全部残桩；失败重跑不涨版本 | 完成 |
| 五 | WASM 按 `spec_version` 发不可变 Release 作永久档案 | 方案待确认 |
| 六 | GitHub Ruleset 锁 main（用户在网页操作，或第一步完成后用 API 核对） | 待办 |

每步固定收尾：更新文档 → 完善注释 → 完善测试 → 清理残留 → 输出下一步技术方案，
用户确认后才进入下一步。

## 第一步落地记录

| 文件 | 改动 |
|---|---|
| `security-app/.../OperationCatalog.swift` | `isAllowed` 的 `github` 白名单加 `GH_TOKEN`；改写原「不允许任意 GitHub 凭证」注释 |
| `server.mjs` | 两个模块 `localKeys` 各加 `GH_TOKEN`；`secretComments` 加条目；`usesProductPush` 分支校验并取用；注入 `childEnv.GH_TOKEN` 并入脱敏表；`continueRuntimeCiRun` 的预授权清单同步补 `GH_TOKEN` |
| `actions/common.sh` | 先校验 `GH_TOKEN` 已注入再调 `gh`；删掉误导文案「GitHub CLI 当前无法访问当前仓库 Actions」 |
| `test/production-security.test.mjs` | 新增两项：原生白名单逐环境 `deepEqual` 固定；CI 同时取用两把凭据并注入 |
| `test/biometric-security.test.mjs` | 更新白名单字面断言；补 WASM CI 预授权含 `GH_TOKEN` |

**关键坑**：`continueRuntimeCiRun`（WASM CI 异步路径）有独立的 `secretRefs`，其结果作为
`preauthorizedValues` 传给 `startRun`。只改 `usesProductPush` 而不同步这份清单，
WASM CI 会在取令牌时取空并当场抛错。

验收：`npm test` 62/62；`node --check`、`bash -n`、Swift `-typecheck -warnings-as-errors` 全过。

**未验证项**：真实 CI 触发未跑过——需用户先重新构建签名包、在密钥列表配置 `GH_TOKEN`
（`repo` + `workflow` 权限），再点「运行节点 CI」实测。涉及 Touch ID 与签名，AI 不代跑。

## 第二步落地记录

| 文件 | 改动 |
|---|---|
| `server.mjs` | 删 `anyProductionRunActive` 及其两处调用；`duplicateRunActive` 键加 `nodeId`；`createRun` 存 `nodeId` 并把节点名拼进标题；`reserveRun` 接 options 解析节点；`runSummary` 输出 `nodeId`、`canStop` 改为「运行中即可停」；删 `canDetach` 字段与暗号过滤；所有动作子进程一律 `detached: true`；stop 端点去掉 `localOnly` 限制；编译动作标 `localOnly` 并 detached |
| `web/app.js` | 「停止」与「×」改为并存（旧写法二选一，运行中根本没有关闭入口）；`closeTab` 去掉全部前置条件；删净 `canDetach` |
| `actions/common.sh` | 删 `CITIZENCONSOLE_REMOTE_RUN_STARTED` 暗号与 `announced` 变量 |
| 两个测试文件 | 旧机制断言改为新契约；新增「任务彻底独立」四点断言 |

**两个执行中发现的连带问题**：

1. `process.kill(-pid)` 需要独立进程组，而原先只有 `localOnly` 动作是 detached。
   要让远端任务可停，必须把所有动作子进程都改成 detached，否则「停止」按钮静默失效。
2. 「编译」动作原本刻意不 detached（要与控制台同生共死）。改为 detached 后必须同时
   标上 `localOnly`，让 `killAllManagedRuns` 认领它，才能保持「控制台退出即收掉构建」。

验收：`npm test` 63/63；`node --check`、`bash -n` 全过。

## 第三步落地记录

| 文件 | 改动 |
|---|---|
| `actions/common.sh` | 新增 `next_semver`（版本推进唯一真源，纯 bash 算术，含 99 进位）；`bump_pubspec_version` 与 `bump_chain_version` 都改为调用它，各自只保留文件格式差异；两份重复的 Python 内嵌算术全部删除 |
| `test/production-security.test.mjs` | 新增 `runInCommon` 辅助（真实 bash source 后执行）；两项测试：7 组进位用例 + 非法输入 fail-closed；两个 bump 函数在临时目录上的真实文件改写，含 `versionCode` 不归零与桌面端两文件同步 |

版本推进过去零测试覆盖，现在按真实执行验证——源码文本断言证明不了进位对不对。

验收：`npm test` 65/65；`bash -n` 过；旧算术（`minor += 1` / `patch += 1`）全仓零残留。

## 第四步落地记录

口径补充（用户在本步中追加）：**每点一次 CI 都涨一版**，唯一例外是上一次运行失败——
那是重跑同一份代码，沿用当前版本并**删掉那条失败记录**后重跑。不抓失败日志（用户明确不要）。

| 文件 | 改动 |
|---|---|
| 四个 workflow | 全删 `on.push`；WASM 加 `workflow_dispatch`（无 inputs）；`citizenchain-ci` / `citizenapp-ci` 保留 `pull_request`；删提交名路由的 6 处 `if:` 守卫与 4 处 concurrency 表达式；`changes` / `guardrails` 里的 push 分支与 `github.event.before` 一并清理；`push_bundle_targets` → `ci_bundle_targets` 改名；陈旧注释同步 |
| `actions/common.sh` | 删 `wait_public_workflow` 整函数（87 行）；`push_all_code` 去掉 `skip_ci` / `allow_empty` 两个形参与空提交 hack；新增 `previous_failed_run` 与 `discard_failed_run` |
| 三个产品脚本 | 统一改为「先判失败、后 bump」；`wasm-ci` 从 `push_all_code + wait_public_workflow` 改为 `run_workflow citizenchain-wasm.yml` |
| 两个测试文件 | 旧 push 模型断言全部改为 dispatch 契约；`WASM CI 失败重试只创建同一代码树的新提交` 整条重写为「无改动不产生提交」；新增「CI 只由按钮 dispatch + 失败重跑不涨版本」契约测试 |

**两次自打脸**：我自己在 `citizenchain-wasm.yml` 和 `common.sh` 的注释里写了
`head_commit.message` 和 `[skip ci]` 字面量，被新写的 `doesNotMatch` 断言当场逮到——
注释里留旧机制的字面量同样是残留。已改写措辞而非放宽测试。

**一处方案自我更正**：第四步方案原说要删 `ensure_product_pushed` 的 else 分支（无改动 →
用远端 tip）。加入「失败重跑不涨版本」后，无改动成为常态，该分支正是重跑路径，**必须保留**。

验收：`npm test` 66/66；四个脚本 `bash -n` 全过；`wait_public_workflow` / `allow_empty` /
`skip_ci` / 提交名路由 全仓零残留。

**未验证项**：真实 dispatch 与 `gh run delete` 未跑过（需 `GH_TOKEN` 配好且重新构建控制台）。
