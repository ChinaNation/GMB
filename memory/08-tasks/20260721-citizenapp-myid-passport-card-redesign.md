# CitizenApp 我的·电子护照三身份卡 UI 重设计（访客轻节点压缩 + 匿名标签 + 标题改写）

任务需求：重设计公民App「我的 → 电子护照」页三张身份卡（访客 / 投票 / 竞选）的展示，
在保留投票、竞选身份必须上链字段的前提下，做文案改写、访客卡结构精简与高度压缩、新增「匿名」提示标签。
所属模块：citizenapp / my·myid（电子护照）

## 定稿设计（用户逐轮确认）

- 重构力度：**结构精修 + 细节升级**——沿用现有身份配色体系（访客金/投票蓝/竞选红）与
  `IdentityBadge`，不改字段集、不改链、不做大幅视觉重构。
- 标题改写（`_PassportIdentityCard._title`）：
  - `voting`：`公民 · 投票身份` → `公民身份 · 投票`
  - `candidate`：`公民 · 竞选身份` → `公民身份 · 竞选`
  - `visitor`：`匿名访客` → `访客轻节点`
- 访客卡新增「匿名」提示：标题右侧一个**小圆角标签(pill)**，内含隐私图标 + `匿名` 二字，用身份色。
- 删除访客卡「没有公民身份信息」空态文案（连同其 28px 上下内边距整块删除）。
- 访客卡高度缩小：删空态后仅剩「徽章 + 标题 + 匿名标签」单行，卡片高度自然压缩约一半。
- 投票 / 竞选卡上链字段（投票账户 / CID / 居住选区 / 身份状态 / 有效期 / 姓名 / 性别 / 出生日期 / 出生地）**保留不动**。

## 落点（唯一改动文件为纯 UI）

- `citizenapp/lib/my/myid/myid_page.dart`
  - `_title` getter（三档文案）
  - `_PassportIdentityCard.build`：标题行加匿名标签（仅 visitor）；删 visitor 空态块
  - 新增私有 `_AnonymousTag` 小组件
  - 类头注释（含「匿名访客」字样）同步更新
- `citizenapp/lib/my/myid/myid_service.dart`：`MyIdTier.visitor` 枚举注释「匿名访客」字样同步更新（仅注释）

## 必须遵守

- 不改 `myid_service.dart` 数据模型与链读逻辑；不改上链字段集与隐私控制（非当前公民卡只显字段名）。
- 不碰链端、不碰徽章绘制、不碰 `AppTheme` 配色常量。
- 「把 X 改成 Y」按字面替换，不附加子任务（费率/字段/数据链不在本次范围）。

## 输出物

- 代码（myid_page.dart 改动 + 注释同步）
- 中文注释
- widget 测试（三档标题、访客匿名标签、访客卡无空态文案、投票/竞选字段保留）
- 残留清理（旧文案、旧空态块零残留）

## 验收标准

- `flutter analyze lib/my/myid` 0 问题
- 三档标题为「访客轻节点 / 公民身份 · 投票 / 公民身份 · 竞选」
- 访客卡不再出现「没有公民身份信息」，高度显著缩小，右侧有「匿名」标签
- 投票/竞选卡上链字段与隐私控制不变
- 相关 widget 测试通过

## 执行结果（2026-07-21）

- **文案**：`_title` 三档改为「访客轻节点 / 公民身份 · 投票 / 公民身份 · 竞选」；标题加
  `maxLines:1 + ellipsis` 防窄屏溢出。
- **匿名标签**：新增私有 `_AnonymousTag`（`myid_page.dart`），小圆角药丸 + `visibility_off_outlined`
  隐私图标 + `匿名` 二字，用所在卡片身份色（访客金），带 key `passport-anonymous-tag`；仅 visitor
  卡标题行右侧渲染。
- **访客卡压缩**：删除「没有公民身份信息」空态整块（连同上下各 28px 内边距）；字段区渲染条件由
  `else`（visitor 走空态）改为 `if (tier != visitor)`，访客卡只剩标题行，高度由约 157px 降到约 76px。
- **上链字段/隐私控制**：`_fieldsFor` 与 `_PassportFieldRow` 未动，投票/竞选九项字段与「非当前公民卡
  只显字段名」不变。
- **注释同步（防漂移）**：`myid_page.dart` 类头注释、错误兜底注释；`myid_service.dart` `MyIdTier.visitor`
  枚举注释；旧文案在 lib 下零残留。
- **测试**：`test/myid_page_test.dart` 4 处断言随文案更新，新增匿名标签存在性 + 旧文案 `findsNothing`
  零残留断言；`myid_service_test.dart` 测试名同步。`flutter analyze lib/my/myid test/myid_page_test.dart`
  0 问题；`flutter test test/myid_page_test.dart` 10/10 通过（含窄屏 320 溢出用例）。
- **边界**：单文件纯 UI（+注释/测试同步），未碰链端、数据模型、徽章绘制、AppTheme 配色。

### 追加微调（2026-07-21 续，用户第二轮）

- **匿名标签贴标题右侧**：标题由 `Expanded` 改 `Flexible`，标签紧贴「访客轻节点」右侧（不再被撑到卡片
  最右端）；标题过长仍走 `ellipsis` 防溢出。
- **字段标签改名**（`_fieldsFor`，仅护照页，全仓唯一）：`公民身份 CID 号` → `公民CID号`；
  `投票身份有效期` → `身份有效期`。
- 验证：`flutter analyze` 0 问题；`flutter test test/myid_page_test.dart` 10/10 通过（字段值断言不受标签改名影响）。
