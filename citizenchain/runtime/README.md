# CitizenChain Runtime 目录说明

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

当前实现已经将治理、发行、交易、其他 pallet，以及共享 `primitives` crate 全部整合到本目录下。
