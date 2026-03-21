# ADR-001：GMB 仓库与 CitizenChain 结构决策

## 状态

已采纳

## 决策日期

2026-03-20

## 背景

GMB 需要统一以下目标：

- 唯一仓库
- 唯一中文主聊天窗口
- AI 持续开发能力
- 区块链作为一个完整桌面产品交付
- 所有项目知识长期沉淀

CitizenChain 历史上存在以下旧布局：

- `governance/`
- `issuance/`
- `otherpallet/`
- `transaction/`
- 仓库根目录 `primitives/`

这些目录已在本次结构收敛中统一并入 `citizenchain/runtime/`。

## 决策

1. GMB 采用唯一仓库模式。
2. `memory/` 作为 AI 永久记忆中心。
3. `citizenchain` 作为一个完整软件产品管理，不拆成独立发布的软件。
4. `citizenchain` 的目标结构固定为：

```text
citizenchain/
  node/
  nodeuitauri/
  nodeui/
  runtime/
    governance/
    issuance/
    otherpallet/
    transaction/
    primitives/
```

5. 旧版 Tauri 实现当前使用 `citizenchain/nodeuitauri`。
6. 新版 Flutter Desktop 节点 UI 使用 `citizenchain/nodeui`。
7. 运行时共享常量 crate 统一放在 `citizenchain/runtime/primitives/`。

## 后果

正面效果：

- 明确 CitizenChain 是一个完整产品
- 明确 runtime 内部的目标组织方式
- 为 NodeUI Flutter 化迁移提供稳定目标
- 便于 AI 在统一结构下工作

注意事项：

- 后续任何新增 runtime 模块都必须直接放在 `citizenchain/runtime/` 下
- 结构调整必须同时兼顾 Cargo 路径、workspace 配置、脚本与发布流程
