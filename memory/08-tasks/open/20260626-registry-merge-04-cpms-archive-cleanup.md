# 任务卡：registry 并入 Step4 — 旧授信与旧二维码导入清理

## 任务需求

完成功能彻底切换与残留清理：

- 裁撤旧安装授信、旧二维码导入、旧状态导入和旧独立分类。
- 公民改为注册局直接录入并直接发护照；机构营业执照统一由注册局颁发，无跨机构授信。
- 旧公民护照备份目录只保留在 `docs/citizenpassport/`，本次不得修改该备份目录。
- 清理全仓相关旧口径文档、open 任务和代码残留。

## 所属模块

citizenchain/registry、CitizenApp、CitizenWallet、website、memory 文档

## 当前执行摘要

- registry 后端/前端改为注册局直接新增公民。
- 市公安局折叠为普通公权机构，不再保留独立分类、独立 tab、独立缓存和独立业务状态轴。
- CitizenApp 公权机构资产包中的旧公安局独立分类已改为普通公权机构。
- CitizenWallet 与 CitizenApp 已删除旧删档 action 解码和协议常量。
- 官网和有效 memory 文档已同步为三系统口径。
- `docs/citizenpassport/` 按要求未修改。

## 验收标准

- 非备份代码路径无旧授信、旧二维码导入、旧独立分类残留。
- 有效架构文档、模块文档、AI 规则和 open 任务队列不再描述旧流程。
- registry 前端、registry 后端、CitizenApp、CitizenWallet、website 可完成构建或静态检查。
- runtime 旧签名常量已按 runtime 二次确认规则完成清理。
