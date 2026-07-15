# 公民钱包 CI birth_date 测试夹具修复

## 任务目标

修复公民钱包 CI 中 `PayloadDecoder` 的两个失败测试，使候选公民身份原始载荷与当前 runtime 的 `CandidateIdentity` 字段布局一致，并完成验证、文档更新、中文注释完善和残留清理。

## 已确认根因

runtime 的 `CandidateIdentity` 在 `citizen_sex` 后新增必填 `birth_date: u32`。公民钱包解码器已按四字节小端整数读取并校验该字段，但测试夹具仍使用旧布局，导致两个原始载荷测试解码返回 `null`。

## 实施范围

- 更新 `/Users/rhett/GMB/citizenwallet/test/signer/payload_decoder_test.dart` 的候选身份载荷夹具，补齐合法出生日期字段。
- 增加对 `birth_date` 解码结果的断言，防止夹具再次与载荷布局漂移。
- 更新相关 CitizenWallet 技术文档和中文注释，说明字段顺序及测试夹具边界。
- 搜索并清理同一测试范围内的旧载荷残留。
- 运行针对性测试、完整 Flutter 测试、分析和可行的本地构建验证。

## 明确不做

- 不修改 `citizenchain/runtime/`、链上载荷协议或解码器生产逻辑。
- 不新增旧格式兼容分支。
- 不推送 GitHub、不重跑远程工作流、不创建或更新 PR。

## 验收标准

- 两个原失败测试通过。
- 相关测试及完整 Flutter 测试通过。
- `flutter analyze --no-fatal-infos` 通过。
- 代码、文档、注释和残留清理完成，`git diff --check` 无错误。

## 执行记录

- 已确认 GitHub Actions 失败仅发生在两个 CandidateIdentity 原始载荷测试；索引同步、依赖安装、分析和其余测试均通过。
- 修复策略为同步测试夹具与当前 `CandidateIdentityPayload`，不放宽生产解码器校验。
- 已在候选身份测试夹具中补齐 `citizen_sex` 后的 `birth_date=20260630`，并为原始候选身份及升级交易增加字段和展示格式断言。
- 已同步更新 `memory/07-ai/unified-protocols.md` 的字段顺序说明。
- 已完成本地验收：针对性测试 71 项通过、完整 Flutter 测试 142 项通过、`flutter analyze --no-fatal-infos` 通过、`flutter build apk --debug` 成功。
- 已确认 `citizenchain/runtime/` 无本任务改动，`git diff --check` 通过；未触发远程 CI、未推送 GitHub。
