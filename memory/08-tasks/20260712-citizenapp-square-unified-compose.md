# CitizenApp 广场统一发布页:一页发全部内容 + 草稿箱 + 文章正文内联图

任务需求：把现有分离的动态/文章发布合并为一个发布页，支持图片/视频/文章及竞选变体，含草稿箱与文章正文图文混排。
所属模块：citizenapp / 8964（广场发布）

## 定稿设计（用户逐条确认）

- **顶栏**：`取消`（左）｜（中间留空，避开手机摄像头）｜`草稿` + `发布`（右）。
- **头像 + 类型下拉**（取代分段+档位 chip）：取消下方左侧圆形头像（当前默认热钱包用户头像），
  右侧下拉默认"动态"；普通用户 2 项（动态/文章），**竞选公民 4 项**（+竞选动态/竞选文章，认证门禁）。
- **动态**：图片/视频不手动选，**第一次选中的媒体类型锁定子类**（先图=图片动态只可加图；先视频=视频动态仅 1 个）。
  媒体计数（如 `2/9`）**右侧带框小加号**开选择器；下方只显示已选（九宫格图 / 单视频）。
  **发布页视频预览恒横屏 16:9**（仅发布页，不影响 feed 竖屏视频规则）。
- **文章**：`标题`（`x/50` 计数右侧带框小加号加**首图缩略✕**，不显示大封面）→ `正文`（`x/30000` 计数右侧
  `插入`按钮，往正文当前位置插图，图文混排）。**标题行 + 正文计数行固定顶部不滚**，下方图文正文可滚动；
  **正文内插图恒横屏**。
- **草稿箱**：全类型（图/视频/文章）覆盖，每条**缩略卡**，**新→旧**排列。
- **底部**：会员额度提示（发布门禁一致）。

## 目录/文件分隔（用户要求"用目录或文件区分动态/文章"）

```
lib/8964/compose/
  compose_page.dart           统一壳:顶栏 + 头像+类型下拉 + 按类型挂子编辑器 + 发布/存草稿/进草稿箱
  compose_type.dart           类型枚举:动态/文章 × 普通/竞选(下拉4项映射) + 门禁判据
  compose_controller.dart     共享:当前类型/档位、认证+会员门禁、额度、提交(复用 publish service)
  post/post_compose_body.dart 动态:正文 + 媒体计数[＋]首选锁图/视频 + 九宫格/单视频(恒横屏预览)
  article/article_compose_body.dart 文章:固定标题+首图缩略 + 可滚图文块编辑器(内联插图、内联图横屏)
  article/article_blocks.dart 图文块模型(text/image) + 与 manifest 序列化
  drafts/drafts_page.dart     草稿箱:全类型缩略卡,新→旧
```
复用/按需扩展现有：`services/square_compose_signers.dart`、`square_media_draft.dart`、
`square_publish_service.dart`、`storage/square_draft_store.dart`。落地后**删旧的**
`pages/square_compose_page.dart` + `pages/square_article_compose_page.dart`（无残留）。

## 关键架构决策（文章正文图文混排——最重）

现状：文章正文=纯文本 `text` + 扁平 `media_items`（首图=[0]、正文图=[1..]，**无图在文中的位置**）。
内联要求正文按顺序图文交替，需引入**有序图文块**：

- 提议模型：文章 manifest 新增有序 `content_blocks: [{t:'text', text} | {t:'image', media_index}]`；
  实际资源仍在 `media_items`（首图=[0] 保持不变、作 feed 封面；内联图=[1..]，块按 index 引用）。
- 三处联动：① Worker manifest 结构（`posts/confirm.ts` 的 article 分支 + 校验）；
  ② Flutter 编辑器（块列表编辑，光标处插图）；③ 阅读器（`square_article_detail_page` 按块渲染，内联图横屏）。
- 兼容：旧文章（无 content_blocks）= 单 text 块降级渲染。

## 分期（一次做完，但按阶段推进验证）

1. 壳 + 类型下拉 + 头像 + 顶栏；接会员+认证门禁与额度。
2. 动态模式：正文 + 媒体[＋]首选锁定 + 九宫格/单视频恒横屏预览。
3. 文章模式：固定标题+首图缩略 + 图文块编辑器（内联插图/顶部固定/内联图横屏）。
4. 图文块模型 + Worker manifest 扩展 + 文章阅读器渲染内联图。
5. 草稿箱全类型缩略卡（新→旧）+ 草稿模型扩展（覆盖文章图文块）。
6. 测试（门禁/首选锁定/块序列化/草稿往返）+ 文档 + 删旧两页清残留。

## 待确认（唯一大件）

- 文章正文图文块**数据模型**采用上述 `content_blocks + media_items index 引用`？此决定牵动 Worker/编辑器/阅读器，
  确认后开建。其余 6 点已定。

## 执行状态（2026-07-12）

**已完成并验证（核心统一发布页 + 内联图文全链路）：**
- 目录 `lib/8964/compose/`：`compose_type.dart`（类型下拉4项+竞选门禁+降级+fromPost 映射）、
  `compose_payload.dart`（子编辑器→壳的统一载荷）、`compose_page.dart`（壳：顶栏取消/草稿/发布、
  头像+类型下拉、IndexedStack 双 body、底部额度、发布协调+编辑预填 replacePostId）、
  `post/post_compose_body.dart`（动态：正文+媒体计数[＋]**首选锁图/视频**+九宫格/单视频**恒横屏预览**）、
  `article/article_blocks.dart`（编辑侧拍平 buildArticleManifest + 校验/常量）、
  `article/article_compose_body.dart`（**顶部固定标题+紧凑首图缩略 + 可滚图文块编辑器、插入内联横屏图**）。
- 内联图数据全链路：manifest 加 `content_blocks`（`upload_service`+`publish_service` 穿参）→
  Worker `posts/confirm.ts` 存/回传 + `types.ts` 类型 → Flutter `SquarePost.contentBlocks` +
  `parseArticleContentBlocks`（阅读侧模型在 square_models，避免循环导入）→ 文章详情
  `square_article_detail_page` 按块渲染（内联图横屏），无块降级纯文本。
- 入口：home `_openCompose` 直进统一页（删底部分流+`_ComposeKind`）；两个详情页编辑入口改指新页
  （`SquareComposeType.fromPost` 映射+预填）；**删旧 `square_compose_page.dart`+`square_article_compose_page.dart`**，
  全仓无残留引用。
- 验证：`flutter analyze lib` 仅 2 条无关既有 info；Worker `tsc` 通过；测试全绿——
  `compose_type_test`(3)、`article_blocks_test`(6)、`square_article_test`(5)、
  `square_publish_service_test`(6，fake 补 contentBlocks)、widgets(12)/feed/home(3) 无回归。

**草稿箱（阶段5，已完成并验证）：**
- 落地 `lib/8964/compose/drafts/`：`compose_draft.dart`（模型+JSON，往返测试过）、`compose_draft_media.dart`
  （选中即复制到 `{appDocs}/square_drafts/{draftId}/`、删草稿删目录）、`compose_draft_store.dart`
  （AppKvEntity 前缀多条、新→旧、上限100淘汰最旧；**根因排查**：findAll/build 是 isar 包扩展，需本文件
  `import 'package:isar_community/isar.dart'` 才可用）、`drafts_page.dart`（缩略卡+右滑删除+点击恢复）。
- 壳集成：`_draftId` 开页即建；`persistMedia`/`onChanged` 注入两 body；**持续防抖自动保存**（800ms + 退出/取消
  flush、空内容不存/删）；发布成功删草稿；发布失败保留可重发；草稿按钮→DraftsPage→按类型 restore。
- 两 body 加 `snapshot()`（不校验、可空首图）/`restore()`（文章按块或降级重建首图+图文块）。
- 验证：`flutter analyze lib` 仅 2 条无关既有 info；compose 测试 13 全过（含 Isar store save/list、模型往返）；
  回归 home/publish/widgets/feed 25 全过；Worker `tsc` 通过。
- 旧 `storage/square_draft_store.dart`（每人一条失败恢复）**未退役**：仍被发布服务内部重试状态与 2 个发布服务
  测试使用；其 restore 已随旧页移除（新页由持续自动保存进箱），属可选后续退役。

**草稿箱（阶段5，技术方案，已确认）：**
- 目录 `lib/8964/compose/drafts/`：`compose_draft.dart`（SquareComposeDraft 全类型模型+JSON，含文章
  content_blocks+持久媒体路径）、`compose_draft_media.dart`（媒体复制到 appDocs 持久目录/清理）、
  `compose_draft_store.dart`（AppKvEntity 前缀 `square.compose.draft.{owner}.{draftId}`、intValue=updated_at
  新→旧、**上限 100 淘汰最旧**）、`drafts/drafts_page.dart`（全类型缩略卡、**右滑弹删除**、点击恢复）。
- 确认的行为：① **持续自动保存**（编辑中防抖存 + 退出/取消自动存，无对话框；空内容不存/删）；
  ② **本地持久化**（picker 临时文件在选中时即复制到草稿目录，路径入库）；③ **发布失败**保留草稿+可"继续发布"
  重试，发布成功删草稿；④ 上限 100；⑤ 右滑删除（删 KV+媒体目录）。
- 发布页集成：壳持 `_draftId`（开页即建）+ 向 body 提供 `persistMedia` 回调（选中即持久化）+ `onChanged`
  触发防抖存；两 body 加 `snapshot()/restore()`。
- 旧 `SquarePublishDraft` 失败恢复已随旧页退役（新页不再 restore）；失败内容由持续自动保存进箱，放弃 uploadId
  断点续传（范围外）。
- 头像现为默认用户昵称首字占位（真头像需 profile.json avatar + session，同 feed 管线，属小增强，非本次）。

## 验收标准

- 一页可发 图片/视频/文章 + 竞选四类；首选锁定、竞选认证门禁、会员额度门禁生效。
- 文章正文图文混排可编辑/发布/阅读，内联图横屏，标题+正文计数固定顶部。
- 草稿箱全类型缩略卡新→旧；草稿往返（含文章块）不丢。
- 旧两个 compose 页删除无残留；`flutter analyze` 0 + Worker `tsc`/`vitest` 全过；相关测试通过。
