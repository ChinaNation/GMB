# OtherPallet 目录说明

本目录用于承载 CitizenChain runtime 下的其他链上模块与文档。
当前其他链上基础能力 crate 已统一放在本目录下，后续新增模块也必须直接落在这里。

- `citizen-identity/`：链上公民身份、人口统计与投票/参选资格真源模块。
- `pow-difficulty/`：PoW 动态难度调整模块。
- `address-registry/`：地址变更上链模块，只记录地址库版本、单条地址当前哈希和变更事件。
