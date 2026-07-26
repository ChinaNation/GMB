# 任务卡:WASM CI 改读中枢省节点 + 只允许 WASM CI + 修钱包测试

> 状态：已关闭并被 `20260721-citizen-election-subject-snapshot-unify` 第 2 步取代。2026-07-21 最终确认正式创世前项目版本归零，GitHub WASM workflow 不再查询开发链或临时抬升版本；正式创世后的加一改由公民控制台读取明确目标链后写入源码，因此不再需要为本流程设置 `GMB_SSH_KEY`，也不得按本卡旧待办推送。

以下内容只保留为历史诊断和实施记录，不代表当前有效 CI 或版本规则。

## 需求(用户三点)
1. WASM CI 的 SSH 版本校验步改读**中枢省权威节点** `64.181.239.233`(原 `147.224.14.117` 认证被拒 `Permission denied (publickey)`);`GMB_SSH_KEY` 换成控制台里中枢省(node-02)保存的 SSH 私钥;`spec_version` 不足时抬到链上 +1(已有逻辑)。
2. 「运行 WASM CI」只推 runtime 相关代码、且**只允许 WASM CI** 触发(不连带钱包等其它 CI)。
3. 修复公民钱包(CI 里 1 个测试挂)。

## 根因(诊断结论)
- WASM CI 失败=编译前「SSH 读链上 spec_version」步 `ubuntu@147.224.14.117 Permission denied (publickey)`,`GMB_SSH_KEY` 对应公钥不在该主机 authorized_keys。非代码错。
- 连带钱包 CI=控制台 `git push` 的 commit 动了 `citizenchain/runtime/src/lib.rs`,而 `citizenwallet-ci.yml` push 路径盯着该文件。工作流路径触发,按配置该触发。
- 钱包测试挂=`propose_create_public_institution` 断言把 `default_role`/`protocol_accounts` **写死**('LR/法定代表人（空缺）' 等),与生成注册表 `fieldValueZhByKey`(法定代表人岗位空缺 / 链上按机构号自动建立)漂移。

## 落地(as-built)
1. **wasm.yml**(`.github/workflows/citizenchain-wasm.yml`):
   - `GMB_CHAIN_HOST: 147.224.14.117 → 64.181.239.233`(中枢省 node-02)。
   - 触发改为**仅 `workflow_dispatch`**(删 push 触发):runtime 推送不再自动跑 wasm,消除「push 版(不做版本校验)+ dispatch 版」双跑并发互相取消的竞态;控制台按钮走 dispatch 即单跑。
   - 清理随之失效的 2 处 `if: workflow_dispatch` 守卫 + 过时注释(no-remnants)。
   - `+1` 抬版逻辑保留(source ≤ 链上则临时 = 链上+1,不改源码不提交)。
2. **citizenwallet-ci.yml**:删 4 条 `citizenchain/runtime/...` push 路径,钱包 CI 只由钱包自身代码触发,与 citizenchain-ci 既有 ci-path-routing 一致(runtime 只走 wasm CI)。
3. **钱包测试**(`citizenwallet/test/signer/payload_decoder_test.dart`):断言改为对比 `GeneratedQrActionRegistry.fieldValueForKey(...)`(单源)。**该修复本就在工作区未提交**;本地 `flutter test payload_decoder_test.dart` = 90/90 通过。committed g.dart 的这两个值未变,故只提交此测试文件即可让 CI 绿。

## 历史待办（已取消）
- **GMB_SSH_KEY**(凭据边界,由用户执行):
  `cd /Users/rhett/GMB && bash citizenconsole/keychain.sh get-multiline node-02 SSH_KEY | gh secret set GMB_SSH_KEY`
  (中枢省私钥经管道直入 GitHub secret,不打印;keychain 账户 `node-02:SSH_KEY` / service `GMB Deploy`,均已确认存在)。
- **提交推送 3 文件**(非 runtime,须走普通推送,不走 WASM CI 按钮):两个工作流 yml + 钱包测试。推送会触发一次 citizenwallet-ci(其路径含 workflow 文件与 citizenwallet/**)验证转绿;dispatch-only 的 wasm 不会被 push 触发。

## 验收
控制台点「运行 WASM CI」→ 只 dispatch 一个 wasm 运行 → SSH 中枢省读 spec_version → 不足则 +1 → 编译产 WASM;全程不连带钱包/节点/App CI;钱包 CI 独立转绿。
