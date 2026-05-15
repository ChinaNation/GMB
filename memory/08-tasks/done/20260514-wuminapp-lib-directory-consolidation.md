# wuminapp lib 零散目录整合

## 需求

- 将公民 Tab 相关入口收拢到 `wuminapp/lib/citizen/`。
- 将交易 Tab 入口与交易共享能力收拢到 `wuminapp/lib/transaction/`。
- 将治理共享能力从 `common` 收拢到 `wuminapp/lib/governance/shared/`。
- 在 `wuminapp/lib/votingengine/` 下新增 `joint-vote`、`citizen-vote` 占位目录。
- 其他目录保持现状。

## 边界

- 只做目录整合、import 更新、文档更新和残留清理。
- 不改 UI 显示、业务逻辑、页面结构。
- 不处理当前工作区可能已有的无关改动。

## 验收

- 旧 `common`、`public`、`vote`、`trade` 顶层目录清理完成。
- `ui` 不再承载 `transaction_tab_page.dart`。
- `dart analyze lib` 通过。
- 旧路径 import 无残留。

## 完成记录

- 已将公民入口、公共页面和投票页收拢到 `wuminapp/lib/citizen/`。
- 已将交易 Tab 与本地交易记录、pending 对账收拢到 `wuminapp/lib/transaction/`。
- 已将治理共享模型、机构信息和提案通用能力收拢到 `wuminapp/lib/governance/shared/`。
- 已在本地创建 `wuminapp/lib/votingengine/joint-vote/` 与 `wuminapp/lib/votingengine/citizen-vote/` 空占位目录；未额外创建 README 或 `.gitkeep`。
- 已更新 wuminapp 模块现有技术文档与源码注释，清理旧路径残留。
- 复查后已补充清理 `memory/01-architecture/wuminapp/`、`memory/07-ai/` 和 `memory/08-tasks/open/` 中仍会影响后续开发判断的旧路径残留；历史 `done` 任务卡保留历史记录不改。
- 验证：
  - `/Users/rhett/flutter/bin/cache/dart-sdk/bin/dart analyze lib test` 通过。
  - `git diff --check` 通过。
  - 旧路径 import 与当前模块文档残留扫描无结果。
  - `flutter test` 因沙箱不允许写入 `/Users/rhett/flutter/bin/cache/engine.stamp` 未执行。
