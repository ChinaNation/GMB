# CitizenChain Runtime 目录说明

本目录用于承载 CitizenChain 运行时能力的统一组织结构。

目标结构固定如下：

```text
runtime/
  entity/
  governance/
  issuance/
  otherpallet/
  transaction/
  primitives/
```

当前实现已经将实体生命周期、治理、发行、交易、其他 pallet，以及共享 `primitives` crate 全部整合到本目录下。

实体生命周期边界：

- `entity/genesis-manage`：创世机构本体、主账户/费用账户索引、不可注销封存表；不提供 extrinsic。
- `entity/public-manage`：创世后公权机构生命周期。
- `entity/private-manage`：创世后私权机构生命周期。
- `entity/personal-manage`：个人多签账户生命周期。
