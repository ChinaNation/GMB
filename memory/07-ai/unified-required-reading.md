# GMB 统一必读文件

## 1. 定位

本文件是 GMB AI 编程系统的统一必读入口。

以后每次设计、编程、改协议、改命名、改文档、改流程前，都必须先从本文件确认需要读取和遵守的文档。

本文件只管理“必须读什么、什么时候读、读完必须遵守什么”。具体规则仍保留在各自文件中。

## 2. 强制规则

1. 新线程首轮必须先读本文件，再按本文件读取首轮必读文档。
2. 进入执行阶段前，必须先读执行前必读文档。
3. 涉及协议、命名、安全、模块边界、代码修改、文档重排时，必须读取对应任务类型的必读文档。
4. 不允许把关键规则只留在聊天历史里；规则必须回写到 `memory/`。
5. 如果本文件和其他入口文件的必读清单冲突，以本文件为准，并同步修正其他入口文件。

## 3. 首轮必读

任何 GMB 新线程第一轮需求分析前，必须读取：

| 顺序 | 文件 | 中文名称 | English name | 必读原因 |
|---:|---|---|---|---|
| 1 | `memory/00-vision/project-goal.md` | 项目目标 | project goal | 确认系统长期目标 |
| 2 | `memory/00-vision/trust-boundary.md` | 信任边界 | trust boundary | 确认安全与信任假设 |
| 3 | `memory/01-architecture/repo-map.md` | 仓库映射 | repo map | 确认仓库结构和模块边界 |
| 4 | `memory/03-security/security-rules.md` | 安全规则 | security rules | 确认开发安全底线 |
| 5 | `memory/07-ai/agent-rules.md` | Agent 规则 | agent rules | 确认 AI 系统硬规则 |
| 6 | `memory/07-ai/chat-protocol.md` | 聊天协议 | chat protocol | 确认首轮响应格式 |
| 7 | `memory/07-ai/requirement-analysis-template.md` | 需求分析模板 | requirement analysis template | 确认需求分析输出结构 |
| 8 | `memory/07-ai/thread-model.md` | 线程模型 | thread model | 确认多线程协作规则 |

首轮只能做需求分析，不得直接进入实现。只读报错诊断例外按 `AGENTS.md` 和 `chat-protocol.md` 执行。

## 4. 执行前必读

用户确认继续执行后，进入任何写代码、改文档、清残留、改配置之前，必须读取：

| 顺序 | 文件 | 中文名称 | English name | 必读原因 |
|---:|---|---|---|---|
| 1 | `memory/07-ai/workflow.md` | 工作流 | workflow | 确认执行闭环 |
| 2 | `memory/07-ai/context-loading-order.md` | 上下文装载顺序 | context loading order | 确认上下文读取顺序 |
| 3 | `memory/07-ai/document-boundaries.md` | 文档边界 | document boundaries | 确认文档职责不漂移 |
| 4 | `memory/07-ai/definition-of-done.md` | 完成标准 | definition of done | 确认完成条件 |
| 5 | `memory/07-ai/pre-submit-checklist.md` | 提交前清单 | pre submit checklist | 确认提交前检查项 |
| 6 | 当前任务卡 | task card | task card | 确认任务目标、范围和进度 |
| 7 | 对应模块技术文档 | module technical docs | module technical docs | 确认模块真实边界 |
| 8 | 对应模块执行清单 | module checklist | module checklist | 确认模块特殊规则 |

真实开发任务必须先创建任务卡；只读报错诊断不创建任务卡。

新建目录或文件的强制限制：

- 未获得用户明确允许时，任何 AI 线程不得新建任何目录或文件。
- 这条限制覆盖代码文件、文档文件、任务卡、测试文件、配置文件、生成物和临时文件。
- 需要新建目录或文件时，必须先列出完整路径、用途和原因，等用户明确同意后才能创建。
- 如果既有流程要求创建任务卡，但用户未允许新建文件，必须先向用户说明冲突并请求授权，不能擅自创建任务卡。

## 5. 按任务类型必读

### 5.1 协议 / 载荷 / 接口 / 签名任务

必须读取：

- `memory/07-ai/unified-protocols.md`
- 对应模块技术文档
- 对应测试或 fixture 文档

适用范围：

- 扫码协议
- 交易载荷格式
- API 契约
- 签名 / 验签
- nonce / era
- pallet index / call index
- storage key / subject id
- fixture / golden data

### 5.2 命名 / 目录 / 字段任务

必须读取：

- `memory/07-ai/unified-naming.md`
- `memory/01-architecture/repo-map.md`
- 对应模块技术文档

适用范围：

- 新建或重命名目录
- 新建或重命名文件
- 新建字段名
- 新建变量、类、函数、常量
- 新建任务卡文件名
- 新建文档文件名
- 调整跨端命名

不确定命名必须先报告用户确认。

### 5.3 安全 / 权限 / 密钥任务

必须读取：

- `memory/03-security/security-rules.md`
- `memory/00-vision/trust-boundary.md`
- 对应模块技术文档

适用范围：

- 密钥
- 签名权限
- 管理员权限
- 支付、转账、投票、身份绑定
- 链上资金或状态修改

### 5.4 代码修改任务

必须读取：

- 当前任务卡
- `memory/07-ai/workflow.md`
- `memory/07-ai/definition-of-done.md`
- 对应模块技术文档
- 对应模块完成标准

代码修改后必须：

- 更新文档
- 完善中文注释
- 清理残留
- 跑对应测试或说明未跑原因

### 5.5 文档重排 / AI 系统改造任务

必须读取：

- `memory/07-ai/document-boundaries.md`
- `memory/07-ai/agent-rules.md`
- `memory/07-ai/context-loading-order.md`
- `memory/01-architecture/repo-map.md`
- `memory/07-ai/unified-naming.md`

适用范围：

- 新建 AI 系统规则文件
- 调整必读清单
- 调整任务卡规则
- 重排 `memory/` 文档结构
- 删除或合并文档

## 6. 禁止跳过

以下行为禁止：

- 不读任务卡直接改代码
- 不读统一协议文件直接改协议、载荷或接口字段
- 不读统一命名文件直接新建命名
- 不读安全规则直接改密钥、签名、权限、资金、身份相关逻辑
- 只改代码不更新文档
- 只更新聊天记录不回写仓库
- 创建第二套必读清单和本文件竞争

## 7. 文件关系

| 文件 | 中文名称 | English name | 职责 |
|---|---|---|---|
| `memory/07-ai/unified-required-reading.md` | 统一必读文件 | unified required reading | 管理必须读什么 |
| `memory/07-ai/unified-naming.md` | 统一命名文件 | unified naming | 管理怎么命名 |
| `memory/07-ai/unified-protocols.md` | 统一协议文件 | unified protocols | 管理协议和载荷契约 |
| `memory/07-ai/agent-rules.md` | Agent 规则 | agent rules | 管理开发硬规则 |
| `memory/07-ai/context-loading-order.md` | 上下文装载顺序 | context loading order | 管理读取顺序 |
| `memory/07-ai/document-boundaries.md` | 文档边界 | document boundaries | 管理文档职责 |

## 8. 后续维护

新增任何“必读文档”时，必须同步更新：

- 本文件
- `memory/07-ai/context-loading-order.md`
- `memory/07-ai/document-boundaries.md`
- 如涉及新线程首轮规则，同步更新 `AGENTS.md` 和 `memory/AGENTS.md`
