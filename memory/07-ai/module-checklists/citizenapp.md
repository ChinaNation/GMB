# CitizenApp 模块执行清单

- App 只是交互入口，不承担信任根职责
- Isar 结构、认证流程、关键交互变化前必须先沟通
- 所有 CitizenApp UI 设计、实现和评审必须先读取
  `memory/01-architecture/citizenapp/CITIZENAPP_TECHNICAL.md` §4.1.1，并核对目标页面代码、
  `lib/ui/app_theme.dart` 与实际图标资产
- 未经当前任务明确授权，不得修改底部“广场 / 公民 / 聊天 / 交易 / 我的”的顺序、
  标签或既有图标
- 关键 Flutter 交互与本地存储逻辑必须补中文注释
- 文档与残留必须一起收口
