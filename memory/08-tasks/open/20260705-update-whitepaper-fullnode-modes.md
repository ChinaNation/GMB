# 更新白皮书全节点模式与残留表述

## 任务需求

更新白皮书中全节点模式和可选节点能力的表述：全节点只分为归档全节点和普通全节点；链上中国与节能能力属于可打开或关闭的本机节点能力；CitizenApp 私密聊天不再属于区块链节点通信能力，而是 Cloudflare 密文 mailbox + 近场通信。删除白皮书中“旧版”“不再使用旧口径”等历史残留说明，只保留当前目标状态。

## 影响范围

- `website/src/whitepaper.md`
  - 白皮书唯一真源，更新中英文正文、节点模式、移除节点通信能力表述、链上中国启停能力和国家储委会创世账户表述。
- 本任务卡
  - 记录本次文档修改范围、执行步骤和验收结果。

## 实施步骤

- [x] 读取执行前必读文档和网站模块文档。
- [x] 更新白皮书全节点模式为归档全节点/普通全节点。
- [x] 移除“通信属于节点能力”的口径，只保留链上中国等本机节点能力表述。
- [x] 删除白皮书中的旧版口径残留表述。
- [x] 执行残留扫描和网站构建验证。
- [x] 更新任务卡验收记录。

## 验收标准

- 白皮书中不再把通信写成第三种全节点模式，也不再把 CitizenApp 私密聊天写成区块链节点能力。
- 白皮书中不再出现“通信全节点”“三种模式”“3种模式”“旧版”“不再使用旧版”等残留表述。
- 中文正文和英文对照同步更新。
- 网站构建通过，或明确说明无法构建原因。

## 验收记录

- `rg -n '通信全节点|三种模式|3种模式|旧版|以前|不再使用旧版|communication full node|communication full-node|communication full nodes|three modes|old NRC|no longer used|legacy' website/src/whitepaper.md`
  - 无输出，表示白皮书中已无本次目标残留关键词。
- `npm run build`
  - 通过，TypeScript 构建检查和 Vite 生产构建均成功。
- `git diff --check`
  - 通过，无空白错误。
- 本地预览服务：`npm run preview -- --host 127.0.0.1 --port 4175`
  - 已访问 `http://127.0.0.1:4175/whitepaper`。
  - 页面正文包含“全节点只分为归档全节点和普通全节点两种模式”“链上中国平台和节能能力均属于本机节点能力”“CitizenApp 私密聊天不再由区块链节点承载”。
  - 页面正文不包含“通信全节点”“三种模式”“3种模式”“旧版”“不再使用旧版”“communication full nodes”“communication full-node”“three modes”“old NRC”“no longer used”。
