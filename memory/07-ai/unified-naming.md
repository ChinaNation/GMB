# GMB 统一命名文件

## 1. 定位

本文件是 GMB AI 编程系统的统一命名入口。

以后任何设计、编码、建文档、建任务卡、建目录、建文件、建字段之前，只要涉及新命名，都必须先查本文件。

本文件统一管理：

- 目录名
- 文件名
- 模块名
- 类名 / 结构体名 / 枚举名
- 函数名 / 方法名
- 变量名 / 常量名
- API 字段名
- storage 字段名
- QR display 字段名
- 任务卡文件名
- 文档文件名

协议名、载荷格式名、接口契约名归 `memory/07-ai/unified-protocols.md` 管；本文件只管命名规则和命名登记。

## 2. 强制规则

1. 所有命名尽量精简，不把需求描述塞进名称里。
2. 不确定的命名必须先报告用户确认，不得擅自新建。
3. 新命名必须说明中文名、英文名、使用位置和简介。
4. 同一概念只能有一个当前命名；旧名必须标为废弃或历史。
5. 文件名只表达主题，不表达完整需求；完整中文标题写入文件内容。
6. 目录名只表达边界，不表达流程步骤。
7. 字段名必须表达数据含义，不表达 UI 文案。
8. 跨端同一字段必须同名，除非有明确语言风格差异并在本文件登记。
9. 不允许为规避冲突随意加 `new`、`old`、`v2`、`temp`、`fix`、`final`。
10. 需要中英文名称的地方，中文名用于说明和 UI，英文名用于目录、代码、字段和接口。

## 3. 命名风格

| 对象 | 风格 | 示例 |
|---|---|---|
| 顶层目录 | lowercase | `memory` |
| Rust crate 目录 | kebab-case | `organization-manage` |
| Rust 模块 / 文件 | snake_case | `chain_duoqian_info.rs` |
| Dart / TS 文件 | snake_case 或既有框架风格 | `duoqian_manage_service.dart` |
| 前端功能目录 | snake_case | `sheng_admins` |
| Rust 类型 | PascalCase | `InstitutionAccountInfo` |
| Dart / TS 类型 | PascalCase | `InstitutionAccountEntry` |
| 函数 / 方法 | snake_case(Rust) / lowerCamelCase(Dart/TS) | `build_call_data` / `buildCallData` |
| 常量 | SCREAMING_SNAKE_CASE(Rust) / lowerCamelCase 或 static const(Dart) | `MODULE_TAG` / `actionCreate` |
| JSON / API 字段 | snake_case | `signer_admin_pubkey` |
| storage 字段 | PascalCase | `InstitutionAccounts` |
| QR display field key | snake_case | `institution_name` |
| 任务卡文件名 | 短日期 + 短 slug | `20260507-ai-unified-naming.md` |
| 技术文档文件名 | SCREAMING_SNAKE_CASE | `BACKEND_LAYOUT.md` |

## 4. 目录结构命名总表

| 路径 | 中文名称 | English name | 简介 |
|---|---|---|---|
| `memory/` | AI 系统永久记忆 | memory | 仓库文档、规则、任务卡和 AI 系统主目录 |
| `memory/00-vision/` | 愿景 | vision | 项目目标、信任边界和长期方向 |
| `memory/01-architecture/` | 架构 | architecture | 仓库级和产品级架构文档 |
| `memory/01-architecture/qr/` | QR 扫码协议 | qr-protocol | WUMIN_QR_V1 协议、签名识别、action registry 和 golden fixture 当前详细真源 |
| `memory/03-security/` | 安全 | security | 安全规则、边界和风险要求 |
| `memory/04-decisions/` | 架构决策 | decisions | ADR 和重要设计决策 |
| `memory/05-modules/` | 模块文档 | modules | 各产品、各模块技术文档 |
| `memory/06-quality/` | 质量 | quality | 测试、缺陷、变更记录模板 |
| `memory/06-quality/fixtures/` | 测试数据 | fixtures | 跨端共享测试 fixture，作为测试数据唯一真源 |
| `memory/07-ai/` | AI 系统规则 | ai-system | AI 编程系统规则、流程、统一入口 |
| `memory/08-tasks/` | 任务卡 | tasks | open / done / templates 任务记录 |
| `citizenchain/` | 公民链 | citizenchain | runtime、节点、桌面端和打包 |
| `citizenchain/runtime/` | 链上运行时 | runtime | pallet、runtime 配置和链上规则 |
| `citizenchain/node/` | 节点桌面端 | node | 原生节点、Tauri 后端和桌面前端 |
| `sfid/` | 在线身份系统 | sfid | SFID 后端、前端和部署配置 |
| `sfid/backend/sfid/` | SFID 核心规则 | sfid-core | SFID 后端核心身份号码、省市码和校验规则唯一源码目录 |
| `cpms/` | 离线实名系统 | cpms | CPMS 后端、部署配置和预留前端 |
| `wumin/` | 冷钱包 | wumin | 离线签名、扫码识别和冷钱包 UI |
| `wuminapp/` | 手机热钱包 | wuminapp | Flutter 移动端和轻节点能力 |
| `website/` | 官网 | website | GMB 官网前端工程 |
| `docs/` | 静态发布文档 | docs | 根目录静态页和展示资产，不承载系统权威记忆 |
| `tools/` | 工具 | tools | 仓库级脚本工具 |
| `scripts/` | 脚本 | scripts | 仓库级自动化脚本 |

## 5. 当前核心命名登记

| 中文名称 | English name | 使用位置 | 简介 |
|---|---|---|---|
| 统一命名文件 | `unified-naming.md` | `memory/07-ai/` | 管理目录、文件、字段等新命名 |
| 统一协议文件 | `unified-protocols.md` | `memory/07-ai/` | 管理协议、载荷格式和接口契约 |
| 统一必读文件 | `unified-required-reading.md` | `memory/07-ai/` | 管理每次设计和编程前必须读取的文档 |
| Step2D 凭证载荷 fixture | `step2d_credential_payload.json` | `memory/06-quality/fixtures/` | wumin / wuminapp 共享的 ADR-008 Step2D SCALE 字节一致性测试数据 |
| 机构管理 | `organization-manage` | runtime crate / pallet | 机构多签管理 pallet |
| 个人多签管理 | `personal-manage` | runtime crate / pallet | 个人多签管理 pallet |
| 管理员变更 | `admins-change` | runtime crate / pallet | 管理员主体、阈值和管理员变更 |
| 内部投票 | `internal-vote` | runtime crate / pallet | 机构内部治理投票 |
| 联合投票 | `joint-vote` | runtime crate / pallet | 联合治理投票 |
| 机构账户 | `InstitutionAccounts` | storage | 机构账户 storage |
| 个人多签 | `PersonalDuoqians` | storage | 个人多签 storage |
| 治理主体 | `Subjects` | storage | 管理员主体 storage |
| 机构身份号码 | `sfid_number` | API / call data / storage key | SFID 机构 ID |
| 机构名称 | `institution_name` | API / call data | 机构显示名称 |
| 账户名称 | `account_name` | API / call data | 机构账户名 |
| 签发省份 | `province` | credential / call data | SFID 省级签名来源 |
| 签发管理员公钥 | `signer_admin_pubkey` | credential / call data | 省级签发 admin 公钥 |
| 已签名交易构造器 | `SignedExtrinsicBuilder` / `signed_extrinsic_builder.dart` | `wuminapp/lib/rpc/` | 统一构造 wuminapp 在线 signed extrinsic，固定执行 immortal era 协议 |

## 6. 新命名登记模板

新增命名时，按这个模板登记：

```text
### 中文名称

- English name：
- 类型：目录 / 文件 / 字段 / 类 / 函数 / 常量 / storage / 任务卡 / 文档
- 使用位置：
- 简介：
- 命名理由：
- 是否确认：已确认 / 待确认
```

## 7. 不确定命名处理

以下情况必须先报告确认：

- 同一概念已有 2 个以上候选名
- 中文业务词难以直译
- 命名会影响跨端字段、storage、接口或协议
- 命名会导致目录移动或文件重命名
- 命名会影响用户可见 UI 文案
- 命名需要保留旧词但旧词已被标记为废弃

报告格式：

```text
命名待确认：

对象：
候选 1：
候选 2：
推荐：
原因：
影响范围：
```

## 8. 禁止命名

禁止新增以下命名形态：

- `old_*`
- `new_*`
- `tmp_*`
- `temp_*`
- `final_*`
- `v2_*`
- `fix_*`
- `xxx2`
- `copy`
- `backup`
- 无意义缩写
- 只有拼音且不能稳定表达业务含义的名称

禁止新增或恢复以下目录：

- `sfid/backend/src/`
- `sfid/backend/chain/`
- `sfid/frontend/chain/`
- `sfid/frontend/api/`

历史文件或外部工具生成物中已有的，不因此自动修改；新建命名禁止使用。

## 9. 待补充登记

以下命名还需要后续逐步纳入本文件：

- 全仓库现有顶层配置文件
- `memory/05-modules/` 下所有模块目录
- `sfid/backend/` 与 `sfid/frontend/` 功能目录
- `wuminapp/lib/` 功能目录
- `citizenchain/runtime/` pallet 目录
- `citizenchain/node/src/` 桌面后端功能目录
- API 字段名总表
- QR display field key 总表
