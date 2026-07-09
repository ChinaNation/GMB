# 白皮书 runtime 模组/模块英文对齐 + 删除过往式描述

## 任务需求
1. 白皮书 section 5 模组/模块英文和仓库对齐（不补模块、不扩英文树）：
   - 模组：5.5 公权业务模组 Public-Business Modules→public；5.6 实体模组 Entity Modules→entity。
   - 模块（描述式英文→crate 名）：5.2.1~5.2.4、5.4.1~5.4.3、5.5.1~5.5.2、5.6.1~5.6.3 共 12 处。
   - bug：英文架构树 Official citizenweb→Official website（改名误伤）。
2. 删除「旧的/过往的」回顾式描述（如「私密聊天不再由区块链节点承载」）：白皮书 227、232 两行的该子句（中英）。

## 仓库真源（construct_runtime! 35 pallet）
- votingengine: internal-vote/joint-vote/legislation-vote/election-vote
- admins: personal-admins/private-admins/public-admins
- public: legislation-yuan/election-campaign
- entity: personal-manage/private-manage/public-manage

## 核心边界
- 只改英文标题（名称），中文标题与英文描述性正文（镜像中文）不动。
- 不补 address-registry/square-post（用户明确不补模块）。
- 只删回顾式子句，不删正当"历史"内容（国家历史/法律版本历史/历史数据）。
- 桌面端内置白皮书随真源重生成。
- 只在 /Users/rhett/GMB 操作。

## 验收
- build/lint 通过；生成文件同步；浏览器抽查 section 5 英文标题+节点段落。

## 执行记录
### 阶段 0：任务卡创建
- 已创建。

### 阶段 1：实现 + 验证（2026-07-08，均在 /Users/rhett/GMB）
- 任务1：sed 15 处英文标题/架构树替换——5.5 public、5.6 entity；5.2.1~5.2.4/5.4.1~5.4.3/5.5.1~5.5.2/5.6.1~5.6.3 全对齐 crate 名；Official citizenweb→website。中文标题与英文描述性正文(镜像中文)未动。
- 任务2：删 227/232 两行「私密聊天不再由区块链节点承载」中英子句(4 处 Edit)，句子其余保留。
- 重跑 generate-local-docs.mjs 同步桌面端；生成文件残留检查全 0。
- build/lint 通过；浏览器(main 5205)核对 section 5 英文标题全对齐、回顾式子句清零、控制台无 error。
### 结论
- 任务1(一/二/四)+任务2 完成并验证。未补模块、未扩英文树(按用户要求)。
