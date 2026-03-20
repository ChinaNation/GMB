# CitizenChain NodeUI

这是 `CitizenChain` 的新版 `Flutter Desktop` 节点 UI 工程。

## 当前定位

- 目录：`citizenchain/nodeui`
- 技术栈：Flutter Desktop
- 状态：新版正式实现入口，当前已完成工程初始化
- 参考实现：`citizenchain/nodeuitauri`

## 迁移原则

- 旧版 `nodeuitauri` 继续承担当前可运行桌面节点壳职责
- 新版 `nodeui` 以 Flutter Desktop 重建节点桌面体验
- 功能迁移完成、打包切换完成、回归验证完成后，再删除 `nodeuitauri`

## 近期目标

- 建立首页框架与导航结构
- 对齐旧版节点启停、状态、挖矿、网络、设置等功能模块
- 逐步替换旧版 Tauri UI
