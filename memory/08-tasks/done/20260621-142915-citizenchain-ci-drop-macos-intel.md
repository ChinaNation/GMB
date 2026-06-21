# 任务卡：CitizenChain CI 删除 macOS Intel，macOS 仅保留 ARM

- 任务编号：20260621-142915
- 状态：done
- 所属模块：.github/workflows（citizenchain 桌面端 CI）
- 当前负责人：Blockchain Agent
- 创建时间：2026-06-21 14:29:15

## 任务需求

从 citizenchain 桌面端 CI 删除 macOS Intel 构建，macOS 仅保留 ARM（Apple Silicon）。背景：本次 push CI 中 macOS Intel job 在 `bundle_dmg.sh` 环节间歇性失败（已诊断为 Intel runner 的 DMG flake，非代码缺陷），用户决定直接砍掉 Intel。

## 必读上下文

- memory/07-ai/ci-path-routing.md
- memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md

## 必须遵守

- 不可突破模块边界（只动 CI + 同步描述该 CI 的文档，不碰链上 runtime/node 与桌面 App 代码）
- 零例外删除 macOS Intel 全部引用
- 改代码后必须更新文档和清理残留

## 输出物

- `.github/workflows/citizenchain.yml`
- 文档同步
- 残留清理

## 待确认问题

- 暂无

## 实施记录

- `.github/workflows/citizenchain.yml`：
  - 删 matrix `macOS Intel` 条目（platform/name/os=macos-15-intel/artifact/installer/targets/updater/arch/cache 共 10 行）
  - `publish-github-release` 必需安装包清单删 `公民链-macOS-Intel.dmg`
  - release updater 删 `addPlatform('darwin-x86_64', ...)` 通道
  - 计数文案 `下载五个安装包`→`四个`、`固定为 5 个`→`4 个`
- 文档同步：
  - memory/07-ai/ci-path-routing.md：5 个→4 个、matrix 列表去 Intel、补「macOS 仅保留 ARM，不再构建 Intel」
  - memory/05-modules/citizenchain/node/CROSS_PLATFORM_BUILD.md：安装包/ updater 清单去 Intel、5 个→4 个
- 不动：历史完成卡 20260524-citizenchain-ci-five-packages.md（快照）；桌面 App 侧无 darwin-x86_64 引用，无需改

## 验证

- ruby YAML 解析通过，matrix 现为 4 job：Linux amd / Linux arm / Windows / macOS Apple
- 全仓残留扫描（排除历史 done 卡）零命中：macos-intel / macOS-Intel / darwin-x86_64 / macos-15-intel / updater-macos-intel

## 影响范围与风险

- push 与手动发布的用户安装包由 5 个降为 4 个
- 存量 macOS Intel 用户不再收到自动更新（无 Intel 包可推）——删 Intel 的必然结果，符合用户意图

## 完成信息

- 完成时间：2026-06-21 14:29:15
- 完成摘要：见实施记录与验证；链上 runtime/node 与桌面 App 零改动
