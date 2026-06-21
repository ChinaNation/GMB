# CPMS 编译期引用 CID 行政区唯一源并按安装码启用城市

## 任务需求

- 行政区数据唯一源只能是 CID 系统 `citizencode/backend/cid`。
- CPMS 源码树不得保存行政区第二份源码，也不得保留源码复制脚本。
- CPMS 后端编译期直接引用 CID 工具行政区源，发行包只内置编译后的只读数据。
- 一个 CID 安装码对应一个市公安局，一个 CPMS 实例运行时只启用安装码对应城市。
- 一并修复 CPMS 公民档案出生日期、性别、身高必填与前端日期输入问题。

## 影响范围

- `citizenpassport/backend/main.rs`: 声明编译期只读引用，指向 `citizencode/backend/citizencode/province.rs`。
- `citizenpassport/backend/address.rs`: 使用 CID 工具唯一源，按安装码所属市重建镇村地址表。
- `citizenpassport/backend/initialize/`: 初始化完成后触发当前市地址同步。
- `citizenpassport/scripts/`: 打包脚本只构建发行包，不写入行政区第二份源码。
- `citizenpassport/frontend/web/src/operator/`: 新建/编辑档案字段必填校验。
- `memory/05-modules/citizenpassport/`: 更新 CPMS 行政区来源与字段规则文档。

## 验收标准

- CPMS 后端能编译通过。
- CPMS 源码树不存在行政区第二份源码目录和源码复制脚本。
- 打包脚本不再执行 CID 工具行政区源码复制。
- 初始化 CPMS 后地址接口只返回安装码对应城市的镇村。
- 创建/编辑档案时出生日期、性别、身高均为必填。
