# CitizenChain 目标结构设计

## 1. 设计目标

CitizenChain 必须被视为一个完整的软件产品，而不是若干松散目录的拼接。

对外发布时：

- 节点程序与节点 UI 为同一个桌面安装包
- 用户只下载一个安装包
- 节点核心和节点界面版本必须保持一致

## 2. 顶层目标结构

```text
citizenchain/
  node/
  runtime/
    governance/
    issuance/
    otherpallet/
    transaction/
    primitives/
  packaging/
  docs/
```

## 3. 各目录职责

### `node/`

- 节点核心程序
- 链连接、RPC、节点服务能力
- 与 runtime 编译产物的集成
- 当前唯一桌面节点 UI 与桌面壳
- 负责节点启停、本地设置、状态展示与桌面打包
- 对外发布的桌面节点产品入口
- 桌面端 Rust 后端模块放在 `node/src/<功能名>`，不再保留 `node/src/ui` 目录层
- 历史 `nodeuitauri/` 与独立 `nodeui/` 能力已统一收口到此目录

### `runtime/`

`runtime/` 是链上规则和运行时能力的统一目录，内部固定为：

```text
runtime/
  governance/
  issuance/
  otherpallet/
  transaction/
  primitives/
```

其中：

- `governance/`：治理相关 pallet
- `issuance/`：发行相关 pallet
- `otherpallet/`：其他链上功能模块
- `transaction/`：交易相关 pallet
- `primitives/`：runtime 内部常量、基础类型、运行时组织层

## 4. 关于 `runtime/primitives/` 的说明

`citizenchain/runtime/primitives/` 已经承载原先仓库根目录 `primitives/` 的 Rust crate 与数据文件。

因此当前约束为：

- 运行时共享常量、基础类型、制度保留地址等统一放在 `citizenchain/runtime/primitives/`
- 其他 crate 如需依赖该能力，统一引用 `citizenchain/runtime/primitives`
- 后续新增 runtime 常量与基础类型，不再回到仓库根目录新增独立 `primitives/`

## 5. 迁移原则

- 先定目标结构，再迁移代码
- 先补文档和规则，再做目录移动
- 迁移时保持构建可验证
- 迁移时不得影响单安装包发布目标
- 旧桌面目录只有在功能、脚本和文档全部收口后才能删除
