# 全仓审计整改 · 第 4 轮：官网+基建(TC5,收尾)

任务需求：落地审计 TC5 项。承接 Round 1/2/3,审计整改收尾。
所属模块：citizenweb(官网) + .github(CI) + citizenapp/smoldotpow(fork 追踪)。

## 本轮条目与处置(用户确认)
| # | 处置 | 落点 |
|---|---|---|
| 24 | 官网下载 chevron 换实心三角(死规则 no-solid-triangle) | citizenweb DownloadButton.tsx `▾` |
| 25 | 宪法响应加最小形状校验(Array.isArray(data.chapters)) | citizenweb Constitution.tsx |
| 18 | CI 加 Dart pallet 注册表 vs 链上 construct_runtime diff 校验 | .github + 校验脚本 |
| 3 | smoldotpow 建独立 fork 追踪上游(可执行同步流程 + 基线记录) | UPSTREAM.md(创 fork 属用户 GitHub 动作) |

## 必须遵守
- 只在 /Users/rhett/GMB 主检出
- 死规则:展开指示器只用折线 chevron,复用 citizenconsole CHEVRON_SVG 视觉
- item 3 创 GitHub fork/推仓库属用户动作,我只落可执行流程+基线,不擅自建远端仓库

## 输出物 / 验收
- citizenweb 改动 + `tsc`/`vite build` 绿
- CI 校验脚本 + workflow 接线;脚本本地跑通(链改索引→红)
- UPSTREAM.md 同步流程可执行
- 残留清理干净

## 执行进度
| Step | 状态 | 说明 |
|---|---|---|
| 24 官网 chevron | ✅ | DownloadButton `▾` 实心三角 → polyline 折线 SVG(复用 citizenconsole CHEVRON 视觉,open 时 rotate-180);`npm run build`(tsc+vite)exit 0 |
| 25 宪法形状校验 | ✅ | Constitution.tsx `if (!Array.isArray(data.chapters)) throw` 最小形状校验,Worker 字段改名给可辨识错误而非不透明运行时崩;build exit 0 |
| 18 CI 注册表校验 | ✅ | 新建 `.github/scripts/check-pallet-registry-sync.mjs`:解析 construct_runtime + 两份 Dart pallet_registry,逐常量比对索引;**实测通过=33 常量一致,注入错误索引=exit 1**;接入 ai-guardrails.yml(每非草稿 PR 跑,ubuntu 预装 node) |
| 3 smoldotpow fork 追踪 | ✅ | UPSTREAM.md §4 由抽象规则改为**可执行流程**:4.0 建 fork(标注为用户 GitHub 动作,AI 不代建仓库)/4.1 无需 fork 即可 `git clone 上游@基线 + diff` 生成 local-vs-upstream.patch(追踪能力)/4.2 fork 上 rebase 新上游/4.3 回灌+更新基线/4.4 硬规则(§3 清单必须与补丁对齐,多出即漂移) |

## 本轮验证记录
- citizenweb:`npm run build`(tsc + vite)exit 0(items 24/25)
- item 18 脚本:当前代码 exit 0(33 常量一致);注入 squarePostPallet=99 → exit 1 + 明确报错;还原后 exit 0
- ai-guardrails.yml:新步骤缩进与既有步骤逐字节一致;`.github/scripts/` 为 git 跟踪目录
- smoldotpow 为 git 跟踪(UPSTREAM.md/pow.rs 在册);cargo 缓存已无 smoldot,4.1 追踪需联网 clone 上游(已在流程注明)
- item 3 创 GitHub fork 属用户一次性动作,未擅自建远端仓库
