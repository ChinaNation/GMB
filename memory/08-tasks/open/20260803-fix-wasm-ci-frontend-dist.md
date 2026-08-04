# CitizenChain WASM CI 节点前端构建前置修复

## 任务需求

修复 GitHub `CitizenChain WASM` 工作流在候选 WASM 已成功生成后，执行 NodeGuard
安全行为探针时因 Tauri `frontendDist` 不存在而编译失败的问题。

## 所属模块

- GitHub Actions：CitizenChain WASM 独立产物工作流。
- CitizenChain node：NodeGuard 候选 WASM 行为探针的宿主编译边界。

## 输入文档

- `memory/00-vision/project-goal.md`
- `memory/00-vision/trust-boundary.md`
- `memory/01-architecture/citizenchain/CITIZENCHAIN_TECHNICAL.md`
- `memory/03-security/security-rules.md`
- `memory/05-modules/citizenchain/node/node-guard/NODE_GUARD_TECHNICAL.md`
- `memory/07-ai/module-checklists/citizenchain.md`
- `memory/07-ai/module-definition-of-done/citizenchain.md`

## 已确认根因

- WASM 编译、产物校验均已成功。
- 后续 `cargo test --locked -p node` 会编译完整 Node/Tauri 宿主。
- `tauri::generate_context!` 在编译期要求 `citizenchain/node/frontend/dist` 存在。
- 该目录是正确的 Git 忽略构建产物，但 WASM 工作流未先执行真实节点前端构建。

## 实施范围

1. WASM 工作流使用 Node.js 24，并在 NodeGuard 行为探针前执行节点前端的锁文件安装与真实构建。
2. 保留 `frontend/dist` 的 Git 忽略状态，不提交或伪造构建产物。
3. 不修改 NodeGuard 业务逻辑、Tauri 配置或 `citizenchain/runtime/`。
4. 更新架构和 NodeGuard 技术文档，清理与当前工作流不一致的旧口径。

## 必须遵守

- 不可绕过候选 WASM 行为探针。
- 不可用空目录或占位 `index.html` 代替真实前端构建。
- 不可提交 `frontend/dist`、`node_modules` 或 Rust 编译产物。
- 不可推送 GitHub 或触发远端 CI，除非用户另行明确授权。

## 输出物

- WASM 工作流修复与中文注释。
- NodeGuard、CitizenChain 架构文档更新。
- 本地前端构建、定向行为探针和残留检查结果。

## 验收标准

- 节点前端依赖按锁文件安装并成功生成真实 `frontend/dist/index.html`。
- 候选 WASM 行为探针不再因 `frontendDist` 缺失失败。
- Workflow YAML 和动作版本边界通过检查。
- 文档与当前实现一致，无构建产物进入 Git。
- 模块执行清单与完成标准已对照。

## 当前状态

- 本地修复与验证完成；等待用户另行授权推送 `main` 后，以 GitHub 干净 runner 完成最终
  WASM CI 验收。

## 实施结果

- `.github/workflows/citizenchain-wasm.yml` 已固定使用 `actions/setup-node` v5.0.0 的完整
  提交 SHA，并配置 Node.js 24 与节点前端锁文件缓存。
- 候选 WASM 行为探针前已增加 `npm ci` 和 `npm run build`，真实生成 Tauri 编译期要求的
  `frontend/dist`。
- `frontend/dist` 和 `node_modules` 继续由 Git 忽略，没有提交占位文件或构建产物。
- NodeGuard 与 CitizenChain 架构文档已补齐前端构建前置条件；AI 工作流和仓库映射中的
  “仅手动 dispatch”旧口径已改为当前固定提交消息的 `main` push 入口。

## 本地验证

- Ruby YAML 解析：通过。
- `actionlint .github/workflows/citizenchain-wasm.yml`：通过。
- `npm --prefix citizenchain/node/frontend ci`：通过，0 个依赖漏洞。
- `npm --prefix citizenchain/node/frontend run build`：通过，真实生成非空
  `frontend/dist/index.html`。
- `git check-ignore citizenchain/node/frontend/dist/index.html`：命中根 `.gitignore`，未进入
  Git 跟踪。
- 使用现有压缩候选 WASM 执行与 CI 相同的
  `current_wasm_passes_candidate_runtime_policy_behavior_probes`：1/1 通过，153.24 秒。
- 本次 Rust 测试产生的三个 `target/**/incremental` 目录已清理。

## 未执行

- 未提交、未推送 GitHub、未创建或更新 PR、未触发远端 workflow。
- GitHub 干净 runner 的最终成功状态仍需独立推送授权后验证。
