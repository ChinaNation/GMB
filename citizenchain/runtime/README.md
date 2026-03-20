# CitizenChain Runtime 目标结构说明

本目录用于承载 CitizenChain 运行时能力的统一组织结构。

目标结构固定如下：

```text
runtime/
  governance/
  issuance/
  otherpallet/
  transaction/
  primitives/
```

当前仓库中的 `citizenchain/governance`、`citizenchain/issuance`、`citizenchain/otherpallet`、`citizenchain/transaction` 仍为旧布局来源目录。

本阶段先建立目标结构和文档基线，不直接进行大规模物理迁移。
